//! Reactive runtime for automatic component re-rendering.
//!
//! This module provides the runtime infrastructure for tracking signal
//! dependencies and scheduling re-renders when signals change.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use tracing::trace;

use crate::{patch::Patch, VNode};

/// Component ID - unique identifier for each component instance
pub type ComponentId = usize;

/// Render function type - produces a VNode tree
pub type RenderFn = Rc<RefCell<dyn FnMut() -> VNode>>;

/// Callback type for scheduling renders with requestAnimationFrame
type ScheduleCallback = Rc<RefCell<dyn FnMut(Box<dyn FnOnce()>)>>;

/// Callback type for applying patches to the DOM
type ApplyPatchesCallback = Rc<RefCell<dyn FnMut(ComponentId, Vec<Patch>)>>;

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
    /// Callback for scheduling renders via requestAnimationFrame
    schedule_callback: Option<ScheduleCallback>,
    /// Callback for applying patches to the DOM
    apply_patches_callback: Option<ApplyPatchesCallback>,
    /// Whether a re-render is scheduled
    scheduled: bool,
    /// Pending rAF callback ID
    raf_id: Option<u32>,
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
            schedule_callback: None,
            apply_patches_callback: None,
            scheduled: false,
            raf_id: None,
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<RuntimeInner> = RefCell::new(RuntimeInner::new());
}

/// Set the callback for scheduling renders via requestAnimationFrame.
///
/// This should be called once during initialization with a callback that
/// wraps the platform's request_animation_frame implementation.
pub fn set_schedule_callback<F: FnMut(Box<dyn FnOnce()>) + 'static>(callback: F) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().schedule_callback = Some(Rc::new(RefCell::new(callback)));
        trace!("Schedule callback registered");
    });
}

/// Set the callback for applying patches to the DOM.
///
/// This should be called once during initialization with a callback that
/// wraps the platform's apply_patches implementation.
pub fn set_apply_patches_callback<F: FnMut(ComponentId, Vec<Patch>) + 'static>(callback: F) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().apply_patches_callback = Some(Rc::new(RefCell::new(callback)));
        trace!("Apply patches callback registered");
    });
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

/// Update a component's render function.
///
/// This is useful when the initial render function is a placeholder
/// and needs to be replaced with the actual implementation.
pub fn update_render_function(id: ComponentId, render_fn: impl FnMut() -> VNode + 'static) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut()
            .render_functions
            .insert(id, Rc::new(RefCell::new(render_fn)));
        trace!("Updated render function for component {}", id);
    });
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

    if let Some(schedule_cb) = &rt.schedule_callback {
        let cb = Rc::clone(schedule_cb);
        let render_fn = Box::new(|| {
            flush_render();
        });
        (cb.borrow_mut())(render_fn);
        trace!("Scheduled render via callback");
    } else {
        trace!("Schedule callback not set, render will be manual");
    }
}

/// Flush pending renders and apply patches.
pub fn flush_render() {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let dirty = std::mem::take(&mut rt.dirty_components);
        rt.scheduled = false;
        rt.raf_id = None;

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
            let patches = crate::diff::diff(Some(&old), &new_vnode);

            if !patches.is_empty() {
                trace!("Component {} generated {} patches", id, patches.len());

                // Apply patches through the callback
                if let Some(apply_patches_cb) = &rt.apply_patches_callback {
                    let cb = Rc::clone(apply_patches_cb);
                    (cb.borrow_mut())(id, patches);
                    trace!("Applied patches for component {}", id);
                } else {
                    trace!("Apply patches callback not set, patches not applied");
                }
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

/// Get the current VNode for a component (useful for testing).
pub fn get_current_vnode(id: ComponentId) -> Option<VNode> {
    RUNTIME.with(|runtime| {
        runtime.borrow().component_vnodes.get(&id).cloned()
    })
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

    #[test]
    fn test_schedule_callback() {
        let called = Rc::new(RefCell::new(false));
        let called_clone = Rc::clone(&called);

        // Don't call the callback in the test to avoid RefCell borrow conflict
        set_schedule_callback(move |_callback| {
            *called_clone.borrow_mut() = true;
        });

        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        mark_dirty(id);

        // The schedule callback should have been called
        assert!(*called.borrow());
    }

    #[test]
    fn test_apply_patches_callback() {
        let applied_patches = Rc::new(RefCell::new(Vec::new()));
        let applied_clone = Rc::clone(&applied_patches);

        set_apply_patches_callback(move |component_id, patches| {
            applied_clone.borrow_mut().push((component_id, patches));
        });

        // Set a schedule callback that doesn't call flush_render to avoid RefCell conflict
        set_schedule_callback(move |_callback| {
            // Just mark that we were called, don't call the callback
        });

        // Create a component and render it once to establish the initial VNode
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        mark_dirty(id); // Mark as dirty so it gets rendered
        flush_render(); // Initial render to store the VNode

        // Now update the render function and mark dirty to generate patches
        update_render_function(id, || VNode::Text(crate::vnode::VText::new("updated")));
        mark_dirty(id);
        flush_render();

        // Check that patches were applied
        assert!(!applied_patches.borrow().is_empty());
    }
}
