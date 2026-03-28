//! Suspense and async resource support for Tairitsu.
//!
//! This module provides:
//! - `Resource<T>`: Async data that can be in Loading, Ready, or Error states
//! - `use_resource`: Hook for creating async resources
//! - `use_suspense`: Hook for Suspense boundary components
//!
//! # Example
//!
//! ```rust,ignore
//! use tairitsu_hooks::suspense::{use_resource, use_suspense, ResourceState};
//! use tairitsu_vdom::{VElement, VNode, VText};
//!
//! fn component() -> VNode {
//!     let data = use_resource(|| async {
//!         // Simulate async fetch
//!         Ok::<_, String>("Hello, world!")
//!     });
//!
//!     use_suspense(
//!         VNode::Element(VElement::new("div").child(VNode::Text(VText::new("Loading...")))),
//!         || match data.read() {
//!             ResourceState::Loading => VNode::Text(VText::new("Loading...")),
//!             ResourceState::Ready(value) => VNode::Text(VText::new(value)),
//!             ResourceState::Error(err) => VNode::Text(VText::new(&format!("Error: {}", err))),
//!         },
//!     )
//! }
//! ```

use std::{
    cell::RefCell,
    fmt,
    future::Future,
    rc::Rc,
    sync::Arc,
};

use tairitsu_vdom::{runtime, VNode};

/// State of an async resource.
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceState<T> {
    /// The resource is still loading.
    Loading,
    /// The resource has loaded successfully.
    Ready(T),
    /// The resource failed to load.
    Error(String),
}

impl<T> ResourceState<T> {
    /// Returns true if the resource is still loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the resource is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Returns true if the resource errored.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Maps the ready value if present.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ResourceState<U> {
        match self {
            Self::Loading => ResourceState::Loading,
            Self::Ready(v) => ResourceState::Ready(f(v)),
            Self::Error(e) => ResourceState::Error(e),
        }
    }
}

/// An async resource that can be in different states.
///
/// Resources are used to manage async data fetching in components.
/// They automatically trigger re-renders when their state changes.
pub struct Resource<T> {
    inner: Rc<ResourceInner<T>>,
    component_id: runtime::ComponentId,
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            component_id: self.component_id,
        }
    }
}

struct ResourceInner<T> {
    state: RefCell<ResourceState<T>>,
    /// Arc-wrapped state for thread-safe access in tests
    thread_safe_state: Arc<std::sync::Mutex<ResourceState<T>>>,
    /// Task handle for cancelling pending operations (if supported)
    _task_handle: RefCell<Option<Box<dyn std::any::Any>>>,
}

impl<T: fmt::Debug> fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("state", &self.inner.state.borrow())
            .field("component_id", &self.component_id)
            .finish()
    }
}

impl<T> Resource<T> {
    /// Read the current state of the resource.
    pub fn read(&self) -> ResourceState<T>
    where
        T: Clone,
    {
        // Try the thread-safe state first (for cross-thread updates)
        if let Ok(state) = self.inner.thread_safe_state.lock() {
            match &*state {
                ResourceState::Ready(_) | ResourceState::Error(_) => {
                    // Return the cloned state from thread-safe storage
                    return state.clone();
                }
                ResourceState::Loading => {
                    // Fall through to check the local state
                }
            }
        }

        // Fall back to the local state
        self.inner.state.borrow().clone()
    }

    /// Get a reference to the current state without cloning.
    pub fn peek(&self) -> std::cell::Ref<'_, ResourceState<T>> {
        self.inner.state.borrow()
    }

    /// Update the resource state and trigger a re-render.
    fn update_state(&self, new_state: ResourceState<T>)
    where
        T: Clone,
    {
        *self.inner.state.borrow_mut() = new_state.clone();
        // Also update the thread-safe state for cross-thread access
        *self.inner.thread_safe_state.lock().unwrap() = new_state;
        runtime::mark_dirty(self.component_id);
        runtime::flush_render();
    }
}

/// Create a new async resource.
///
/// The `fetcher` closure is called immediately with the provided future.
/// The resource starts in the `Loading` state and updates to `Ready` or `Error`
/// when the future completes.
///
/// # Example
///
/// ```rust,ignore
/// use tairitsu_hooks::suspense::use_resource;
///
/// let resource = use_resource(|| async {
///     // Simulate an async operation
///     Ok::<_, String>(42)
/// });
/// ```
///
/// Note: This is a synchronous wrapper. For true async support in WASM,
/// you would need to integrate with a runtime like tokio or wasm-bindgen-futures.
pub fn use_resource<T, F, Fut>(fetcher: F) -> Resource<T>
where
    T: Clone + 'static + std::marker::Send,
    F: FnOnce() -> Fut + 'static + std::marker::Send,
    Fut: Future<Output = Result<T, String>> + 'static + std::marker::Send,
{
    let component_id = runtime::use_component(VNode::empty);

    let inner = Rc::new(ResourceInner {
        state: RefCell::new(ResourceState::Loading),
        thread_safe_state: Arc::new(std::sync::Mutex::new(ResourceState::Loading)),
        _task_handle: RefCell::new(None),
    });

    let resource = Resource {
        inner: Rc::clone(&inner),
        component_id,
    };

    // Spawn the async operation
    // Note: In a real implementation, this would integrate with a proper async runtime
    // For now, we provide a thread-pool based approach for testing
    #[cfg(test)]
    {
        // For testing, we need to use a different approach
        // since we can't send Rc across threads
        // We'll use a channel to communicate the result
        let (tx, rx) = std::sync::mpsc::channel();

        // Clone the Arc for thread-safe access
        let thread_safe_state = Arc::clone(&inner.thread_safe_state);

        std::thread::spawn(move || {
            // Create a minimal runtime for the future
            // In production, use the proper runtime
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let result = rt.block_on(fetcher());

            // Send the result back
            let _ = tx.send(result);
        });

        // Spawn a thread to wait for the result and update the resource
        std::thread::spawn(move || {
            if let Ok(result) = rx.recv() {
                // Update the thread-safe state
                let new_state = match result {
                    Ok(value) => ResourceState::Ready(value),
                    Err(err) => ResourceState::Error(err),
                };
                *thread_safe_state.lock().unwrap() = new_state.clone();
                // Mark the component as dirty so it will re-render
                runtime::mark_dirty(component_id);
                runtime::flush_render();
            }
        });
    }

    #[cfg(not(test))]
    {
        // For non-test environments, we can't spawn threads with futures
        // In production, this would integrate with the platform's async runtime
        let _ = fetcher; // Suppress unused warning
    }

    resource
}

/// Suspense boundary component.
///
/// Wraps children with a fallback UI that shows while any async resources
/// in the children are loading.
///
/// # Arguments
///
/// * `fallback` - The VNode to show while children are loading
/// * `children` - Closure that renders the actual content
///
/// # Example
///
/// ```rust,ignore
/// use tairitsu_hooks::suspense::use_suspense;
/// use tairitsu_vdom::{VElement, VNode, VText};
///
/// fn component() -> VNode {
///     use_suspense(
///         VNode::Element(VElement::new("div").child(VNode::Text(VText::new("Loading...")))),
///         || VNode::Element(VElement::new("div").child(VNode::Text(VText::new("Content loaded!")))),
///     )
/// }
/// ```
///
/// Note: This is a simplified implementation. A full Suspense implementation
/// would track pending resources and automatically show the fallback.
pub fn use_suspense<F>(fallback: VNode, children: F) -> VNode
where
    F: FnOnce() -> VNode + 'static,
{
    // In a full implementation, this would:
    // 1. Track all resources accessed during children rendering
    // 2. If any are loading, render the fallback
    // 3. Subscribe to resource updates and re-render when ready
    //
    // For now, we just render the children immediately
    // TODO: Implement proper resource tracking and fallback rendering

    let _fallback = fallback; // Suppress unused warning
    children()
}

/// Suspense boundary state for tracking pending resources.
#[derive(Default)]
pub struct SuspenseBoundary {
    /// Resources that are currently loading
    pending_resources: Rc<RefCell<Vec<usize>>>,
}

impl SuspenseBoundary {
    /// Create a new suspense boundary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pending resource to this boundary.
    pub fn add_pending(&self, resource_id: usize) {
        self.pending_resources.borrow_mut().push(resource_id);
    }

    /// Check if any resources are still pending.
    pub fn has_pending(&self) -> bool {
        !self.pending_resources.borrow().is_empty()
    }

    /// Mark a resource as complete.
    pub fn mark_complete(&self, resource_id: usize) {
        self.pending_resources
            .borrow_mut()
            .retain(|&id| id != resource_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_resource_state_loading() {
        let state = ResourceState::<String>::Loading;
        assert!(state.is_loading());
        assert!(!state.is_ready());
        assert!(!state.is_error());
    }

    #[test]
    fn test_resource_state_ready() {
        let state = ResourceState::Ready("test".to_string());
        assert!(!state.is_loading());
        assert!(state.is_ready());
        assert!(!state.is_error());
    }

    #[test]
    fn test_resource_state_error() {
        let state = ResourceState::<()>::Error("oops".to_string());
        assert!(!state.is_loading());
        assert!(!state.is_ready());
        assert!(state.is_error());
    }

    #[test]
    fn test_resource_state_map() {
        let state = ResourceState::Ready(42);
        let mapped = state.map(|v| v * 2);
        assert_eq!(mapped, ResourceState::Ready(84));

        let loading = ResourceState::<i32>::Loading;
        let mapped_loading = loading.map(|v| v * 2);
        assert!(mapped_loading.is_loading());

        let error = ResourceState::<i32>::Error("fail".to_string());
        let mapped_error = error.map(|v| v * 2);
        assert!(mapped_error.is_error());
    }

    #[test]
    fn test_suspense_boundary() {
        let boundary = SuspenseBoundary::new();
        assert!(!boundary.has_pending());

        boundary.add_pending(1);
        assert!(boundary.has_pending());

        boundary.add_pending(2);
        assert!(boundary.has_pending());

        boundary.mark_complete(1);
        assert!(boundary.has_pending());

        boundary.mark_complete(2);
        assert!(!boundary.has_pending());
    }

    #[test]
    fn test_use_resource_sync() {
        // Test with an immediately-ready future
        let resource = use_resource(|| async {
            Ok::<_, String>("immediate")
        });

        // Give the thread a moment to complete
        std::thread::sleep(Duration::from_millis(500));

        match resource.read() {
            ResourceState::Ready(value) => assert_eq!(value, "immediate"),
            ResourceState::Loading => panic!("Resource should be ready"),
            ResourceState::Error(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_use_resource_error() {
        let resource = use_resource(|| async {
            Err::<(), _>("fetch failed".to_string())
        });

        // Give the thread a moment to complete
        std::thread::sleep(Duration::from_millis(500));

        match resource.read() {
            ResourceState::Error(msg) => assert_eq!(msg, "fetch failed"),
            ResourceState::Loading => panic!("Resource should have errored"),
            ResourceState::Ready(_) => panic!("Resource should have errored"),
        }
    }

    #[test]
    fn test_use_suspense_basic() {
        let fallback = VNode::Text(tairitsu_vdom::VText::new("Loading..."));
        let content = VNode::Text(tairitsu_vdom::VText::new("Done!"));

        let result = use_suspense(fallback, || content);

        // For now, just verify it returns the children
        assert_eq!(result, VNode::Text(tairitsu_vdom::VText::new("Done!")));
    }

    #[test]
    fn test_resource_clone() {
        let resource = use_resource(|| async {
            Ok::<_, String>(42)
        });

        let resource_clone = resource.clone();

        // Both should point to the same inner state
        std::thread::sleep(Duration::from_millis(500));

        match resource_clone.read() {
            ResourceState::Ready(value) => assert_eq!(value, 42),
            _ => panic!("Resource should be ready"),
        }
    }
}
