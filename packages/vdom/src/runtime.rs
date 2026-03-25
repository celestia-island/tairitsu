//! Reactive runtime for automatic component re-rendering.
//!
//! This module provides the runtime infrastructure for tracking signal
//! dependencies and scheduling re-renders when signals change.

use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use tracing::trace;

use crate::VNode;

/// Component ID - unique identifier for each component instance
pub type ComponentId = usize;

/// Render function type - produces a VNode tree
pub type RenderFn = Rc<RefCell<dyn FnMut() -> VNode>>;

/// Inner state of the reactive runtime
struct RuntimeInner {
    /// Next available component ID
    next_id: ComponentId,
    /// Active component being rendered (for dependency tracking)
    active_component: Option<ComponentId>,
    /// Map of component ID to its current VNode
    component_vnodes: HashMap<ComponentId, VNode>,
    /// Map of component ID to its render function
    render_functions: HashMap<ComponentId, RenderFn>,
    /// Map of signal inner Rc to component IDs that depend on it
    signal_dependencies: HashMap<usize, Vec<ComponentId>>,
    /// Pending re-renders (dirty components)
    dirty_components: Vec<ComponentId>,
    /// Platform reference for applying patches
    platform: Option<Weak<dyn Any>>,
    /// Root element handle for patch application
    root_element: Option<Weak<dyn Any>>,
    /// Whether a re-render is scheduled
    scheduled: bool,
}

impl RuntimeInner {
    fn new() -> Self {
        Self {
            next_id: 1,
            active_component: None,
            component_vnodes: HashMap::new(),
            render_functions: HashMap::new(),
            signal_dependencies: HashMap::new(),
            dirty_components: Vec::new(),
            platform: None,
            root_element: None,
            scheduled: false,
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<RuntimeInner> = RefCell::new(RuntimeInner::new());
}

/// Register a component with the runtime.
///
/// This creates a new component instance and tracks its render function.
/// The component will be automatically re-rendered when its dependencies change.
pub fn use_component<F>(render_fn: F) -> ComponentId
where
    F: FnMut() -> VNode + 'static,
{
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        let id = rt.next_id;
        rt.next_id += 1;

        rt.render_functions
            .insert(id, Rc::new(RefCell::new(render_fn)));

        trace!("Registered component {}", id);

        id
    })
}

/// Mark a component as dirty and schedule a re-render.
pub fn mark_dirty(id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        if !rt.dirty_components.contains(&id) {
            rt.dirty_components.push(id);
            trace!("Marked component {} as dirty", id);
        }

        schedule_render(&mut rt);
    });
}

/// Set the active component for dependency tracking.
pub fn with_component<T>(id: ComponentId, f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let prev = rt.active_component;
        rt.active_component = Some(id);
        drop(rt);
        let result = f();
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().active_component = prev;
        });
        result
    })
}

/// Track a signal dependency for the current component.
pub fn track_signal(signal_ptr: usize) {
    RUNTIME.with(|runtime| {
        let rt = runtime.borrow_mut();

        if let Some(component_id) = rt.active_component {
            drop(rt);
            RUNTIME.with(|runtime| {
                let mut rt = runtime.borrow_mut();
                rt.signal_dependencies
                    .entry(signal_ptr)
                    .or_insert_with(Vec::new)
                    .push(component_id);
                trace!(
                    "Component {} now depends on signal {:?}",
                    component_id,
                    signal_ptr
                );
            });
        }
    });
}

/// Schedule a render using the platform's scheduling mechanism.
fn schedule_render(rt: &mut RuntimeInner) {
    if rt.scheduled {
        return;
    }

    rt.scheduled = true;

    // Use requestAnimationFrame if available, otherwise use microtask
    // For now, we'll use a simple timeout-based approach
    // In a real implementation, this would use the Platform trait
    trace!("Scheduling re-render");

    // The actual rendering will be triggered by flush_render()
}

/// Flush pending renders and apply patches.
pub fn flush_render() {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let dirty = std::mem::take(&mut rt.dirty_components);
        rt.scheduled = false;

        drop(rt);

        if !dirty.is_empty() {
            trace!("Flushing {} dirty components", dirty.len());

            for component_id in dirty {
                render_component(component_id);
            }
        }
    });
}

/// Render a single component and apply patches.
fn render_component(id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        // Get the render function
        let render_fn = if let Some(render_fn) = rt.render_functions.get(&id) {
            render_fn.clone()
        } else {
            trace!("No render function for component {}", id);
            return;
        };

        // Set as active component
        let prev = rt.active_component;
        rt.active_component = Some(id);

        // Get the old VNode
        let old_vnode = rt.component_vnodes.get(&id).cloned();

        // Call the render function
        let new_vnode = (render_fn.borrow_mut())();

        // Restore active component
        rt.active_component = prev;

        // Store the new VNode
        rt.component_vnodes.insert(id, new_vnode.clone());

        // If we had an old VNode, compute patches
        if let Some(old) = old_vnode {
            drop(rt);

            let patches = crate::diff::diff(Some(&old), &new_vnode);

            if !patches.is_empty() {
                trace!("Component {} generated {} patches", id, patches.len());

                // Apply patches through the platform
                // This requires the platform to be set
                RUNTIME.with(|runtime| {
                    let rt = runtime.borrow();
                    if let (Some(platform_weak), Some(root_weak)) = (&rt.platform, &rt.root_element)
                    {
                        if let (Some(_platform), Some(_root)) =
                            (platform_weak.upgrade(), root_weak.upgrade())
                        {
                            // Apply patches using the platform
                            trace!("Applying patches for component {}", id);
                            // Note: This is a simplified version
                            // In the full implementation, we'd call platform.apply_patches()
                        }
                    }
                });
            }
        } else {
            trace!("Initial render for component {}", id);
        }
    });
}

/// Subscribe a component to a signal's changes.
pub fn subscribe_component(signal_ptr: usize, component_id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.signal_dependencies
            .entry(signal_ptr)
            .or_insert_with(Vec::new)
            .push(component_id);
        trace!(
            "Component {} subscribed to signal {:?}",
            component_id,
            signal_ptr
        );
    });
}

/// Notify all dependent components that a signal has changed.
pub fn notify_signal(signal_ptr: usize) {
    RUNTIME.with(|runtime| {
        let rt = runtime.borrow();

        if let Some(components) = rt.signal_dependencies.get(&signal_ptr) {
            for &component_id in components {
                mark_dirty(component_id);
            }
        }
    });
}

/// Cleanup resources for a component.
pub fn cleanup_component(id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.render_functions.remove(&id);
        rt.component_vnodes.remove(&id);
        rt.dirty_components.retain(|&c| c != id);

        // Remove signal dependencies
        for deps in rt.signal_dependencies.values_mut() {
            deps.retain(|&c| c != id);
        }

        trace!("Cleaned up component {}", id);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_registration() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        assert!(id > 0);
    }

    #[test]
    fn test_mark_dirty() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert!(rt.dirty_components.is_empty());
        });

        mark_dirty(id);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.dirty_components, vec![id]);
        });
    }

    #[test]
    fn test_track_signal() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            track_signal(12345);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.signal_dependencies.get(&12345), Some(&vec![id]));
        });
    }
}
