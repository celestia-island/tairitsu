//! Render scheduler for efficient UI updates.
//!
//! This module provides the scheduler that batches re-render requests and
//! applies them using requestAnimationFrame for optimal performance.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::trace;

use crate::{platform::Platform, VNode};

/// Global scheduler ID counter
static NEXT_SCHEDULER_ID: AtomicUsize = AtomicUsize::new(1);

/// Scheduler ID type
pub type SchedulerId = usize;

/// Component state tracked by the scheduler
#[derive(Default)]
struct ComponentState {
    current_vnode: Option<VNode>,
    dirty: bool,
    root_element: Option<Rc<RefCell<dyn std::any::Any>>>,
    render_fn: Option<Box<dyn Fn() -> VNode>>,
}

/// Inner state of the scheduler
struct SchedulerInner<P: Platform> {
    /// Scheduler ID
    id: SchedulerId,
    /// Platform for DOM operations
    platform: Rc<RefCell<P>>,
    /// Component states
    components: Vec<ComponentState>,
    /// Whether a render is scheduled
    scheduled: bool,
    /// Pending rAF callback ID
    raf_id: Option<u32>,
}

impl<P: Platform> SchedulerInner<P> {
    fn new(platform: Rc<RefCell<P>>) -> Self {
        Self {
            id: NEXT_SCHEDULER_ID.fetch_add(1, Ordering::SeqCst),
            platform,
            components: Vec::new(),
            scheduled: false,
            raf_id: None,
        }
    }
}

/// Render scheduler for batching UI updates.
///
/// The scheduler tracks component states, batches re-render requests,
/// and applies patches efficiently using requestAnimationFrame.
pub struct Scheduler<P: Platform> {
    inner: Rc<RefCell<SchedulerInner<P>>>,
}

impl<P: Platform> Scheduler<P> {
    /// Create a new scheduler with the given platform.
    pub fn new(platform: Rc<RefCell<P>>) -> Self {
        let inner = Rc::new(RefCell::new(SchedulerInner::new(platform)));
        Self { inner }
    }

    /// Get the scheduler ID.
    pub fn id(&self) -> SchedulerId {
        self.inner.borrow().id
    }

    /// Register a component with the scheduler.
    ///
    /// Returns the component ID that can be used to update this component.
    pub fn register_component(&self, render_fn: Option<Box<dyn Fn() -> VNode>>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let id = inner.components.len();
        inner.components.push(ComponentState {
            render_fn,
            ..Default::default()
        });
        trace!("Scheduler {}: Registered component {}", inner.id, id);
        id
    }

    /// Set the root element for a component.
    ///
    /// This is where patches will be applied when the component re-renders.
    pub fn set_root_element(&self, component_id: usize, element: Rc<RefCell<dyn std::any::Any>>) {
        let mut inner = self.inner.borrow_mut();
        if let Some(component) = inner.components.get_mut(component_id) {
            component.root_element = Some(element);
            trace!(
                "Scheduler {}: Set root element for component {}",
                inner.id,
                component_id
            );
        }
    }

    /// Set the render function for a component.
    pub fn set_render_fn(&self, component_id: usize, render_fn: Box<dyn Fn() -> VNode>) {
        let mut inner = self.inner.borrow_mut();
        if let Some(component) = inner.components.get_mut(component_id) {
            component.render_fn = Some(render_fn);
            trace!(
                "Scheduler {}: Set render function for component {}",
                inner.id,
                component_id
            );
        }
    }

    /// Mark a component as dirty and schedule a re-render.
    pub fn mark_dirty(&self, component_id: usize) {
        let mut inner = self.inner.borrow_mut();
        if let Some(component) = inner.components.get_mut(component_id) {
            component.dirty = true;
            trace!(
                "Scheduler {}: Marked component {} as dirty",
                inner.id,
                component_id
            );
        }
        drop(inner);

        self.schedule_render();
    }

    /// Schedule a render using requestAnimationFrame.
    fn schedule_render(&self) {
        let mut inner = self.inner.borrow_mut();

        if inner.scheduled {
            return;
        }

        inner.scheduled = true;

        // Clone the inner Rc for the callback
        let inner_clone = self.inner.clone();

        // Clone the platform before releasing the borrow
        let platform_clone = Rc::clone(&inner.platform);

        drop(inner);

        // Schedule using the platform's request_animation_frame
        let raf_id = platform_clone
            .borrow_mut()
            .request_animation_frame(Box::new(move |_timestamp| {
                trace!("Render callback triggered");
                {
                    let mut inner = inner_clone.borrow_mut();
                    inner.scheduled = false;
                    inner.raf_id = None;
                }

                // Flush all dirty components
                let dirty_components: Vec<_> = inner_clone
                    .borrow()
                    .components
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.dirty)
                    .map(|(id, _)| id)
                    .collect();

                for component_id in dirty_components {
                    Self::render_component_inner(inner_clone.clone(), component_id);
                }
            }));

        // Re-borrow to set the raf_id
        let mut inner = self.inner.borrow_mut();
        inner.raf_id = Some(raf_id);

        trace!(
            "Scheduler {}: Scheduled render with rAF id {}",
            inner.id,
            raf_id
        );
    }

    /// Render a single component and apply patches.
    fn render_component_inner(inner: Rc<RefCell<SchedulerInner<P>>>, component_id: usize) {
        let new_vnode = {
            let inner_ref = inner.borrow();
            match inner_ref.components.get(component_id) {
                Some(component) => match &component.render_fn {
                    Some(render_fn) => render_fn(),
                    None => {
                        trace!(
                            "Scheduler {}: Component {} has no render function, skipping",
                            inner_ref.id,
                            component_id
                        );
                        drop(inner_ref);
                        let mut inner_mut = inner.borrow_mut();
                        if let Some(component) = inner_mut.components.get_mut(component_id) {
                            component.dirty = false;
                        }
                        return;
                    }
                },
                None => return,
            }
        };

        let mut inner_ref = inner.borrow_mut();
        let scheduler_id = inner_ref.id;
        if let Some(component) = inner_ref.components.get_mut(component_id) {
            let old_vnode = component.current_vnode.take();
            component.current_vnode = Some(new_vnode.clone());

            if let Some(old) = &old_vnode {
                let patches = crate::diff::diff(Some(old), &new_vnode);
                if !patches.is_empty() {
                    trace!(
                        "Scheduler {}: Component {} has {} patches",
                        scheduler_id,
                        component_id,
                        patches.len()
                    );
                    for patch in &patches {
                        trace!("  Patch: {:?}", patch);
                    }
                }
            } else {
                trace!(
                    "Scheduler {}: Initial render for component {}",
                    scheduler_id,
                    component_id
                );
            }

            component.dirty = false;
            trace!(
                "Scheduler {}: Rendered component {}",
                scheduler_id,
                component_id
            );
        }
    }

    /// Update a component with a new VNode and apply patches.
    pub fn update_component(&self, component_id: usize, new_vnode: VNode) {
        let mut inner = self.inner.borrow_mut();

        if let Some(component) = inner.components.get_mut(component_id) {
            let old_vnode = component.current_vnode.take();
            component.current_vnode = Some(new_vnode.clone());

            // If we have a root element, apply patches
            if let Some(_root_element) = &component.root_element {
                if let Some(old) = old_vnode {
                    let patches = crate::diff::diff(Some(&old), &new_vnode);

                    if !patches.is_empty() {
                        trace!(
                            "Scheduler {}: Component {} has {} patches",
                            inner.id,
                            component_id,
                            patches.len()
                        );

                        // Apply patches to the DOM
                        // This requires downcasting the element to the platform's element type
                        // For now, we'll just log the patches
                        for patch in &patches {
                            trace!("  Patch: {:?}", patch);
                        }
                    }
                } else {
                    // Initial render - mount the VNode
                    trace!(
                        "Scheduler {}: Initial render for component {}",
                        inner.id,
                        component_id
                    );
                }
            }
        }
    }

    /// Cancel any pending render.
    pub fn cancel_render(&self) {
        let mut inner = self.inner.borrow_mut();

        if let Some(raf_id) = inner.raf_id.take() {
            inner.platform.borrow_mut().cancel_animation_frame(raf_id);
            inner.scheduled = false;
            trace!("Scheduler {}: Cancelled render", inner.id);
        }
    }

    /// Force an immediate render (bypasses scheduling).
    pub fn render_now(&self) {
        self.cancel_render();

        let dirty_components: Vec<_> = self
            .inner
            .borrow()
            .components
            .iter()
            .enumerate()
            .filter(|(_, c)| c.dirty)
            .map(|(id, _)| id)
            .collect();

        for component_id in dirty_components {
            Self::render_component_inner(self.inner.clone(), component_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{element::ElementHandle, event::EventHandle};
    use std::any::Any;

    // Mock platform for testing
    struct MockPlatform;

    #[derive(Clone)]
    struct MockElement;

    impl ElementHandle for MockElement {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Clone)]
    struct MockEvent;

    impl EventHandle for MockEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl Platform for MockPlatform {
        type Element = MockElement;
        type Event = MockEvent;

        fn create_element(&self, _tag: &str) -> Self::Element {
            MockElement
        }
        fn create_text_node(&self, _text: &str) -> Self::Element {
            MockElement
        }
        fn append_child(&self, _parent: &Self::Element, _child: &Self::Element) {}
        fn remove_child(&self, _parent: &Self::Element, _child: &Self::Element) {}
        fn set_attribute(&self, _element: &Self::Element, _name: &str, _value: &str) {}
        fn remove_attribute(&self, _element: &Self::Element, _name: &str) {}
        fn set_style(&self, _element: &Self::Element, _name: &str, _value: &str) {}
        fn set_class(&self, _element: &Self::Element, _class: &str) {}
        fn add_event_listener(
            &self,
            _element: &Self::Element,
            _event: &str,
            _handler: Box<dyn FnMut(Box<dyn crate::EventData>)>,
        ) {
        }
        fn remove_event_listener(&self, _element: &Self::Element, _event: &str) {}
        fn get_bounding_client_rect(&self, _element: &Self::Element) -> crate::DomRect {
            crate::DomRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }
        }
        fn inner_width(&self) -> i32 {
            0
        }
        fn inner_height(&self) -> i32 {
            0
        }
        fn set_timeout(&self, _callback: Box<dyn FnOnce()>, _ms: i32) -> i32 {
            0
        }
        fn clear_timeout(&self, _id: i32) {}
        fn request_animation_frame(&self, _callback: Box<dyn FnOnce(f64)>) -> u32 {
            0
        }
        fn cancel_animation_frame(&self, _id: u32) {}
        fn get_canvas_context(
            &self,
            _element: &Self::Element,
            _context_type: &str,
        ) -> Option<crate::CanvasContext> {
            None
        }
        fn canvas_set_fill_style(&self, _ctx: crate::CanvasContext, _color: &str) {}
        fn canvas_fill_rect(&self, _ctx: crate::CanvasContext, _x: f64, _y: f64, _w: f64, _h: f64) {
        }
        fn canvas_clear_rect(
            &self,
            _ctx: crate::CanvasContext,
            _x: f64,
            _y: f64,
            _w: f64,
            _h: f64,
        ) {
        }
        fn create_resize_observer(
            &self,
            _callback: Box<dyn FnMut(Vec<crate::ResizeObserverEntry>)>,
        ) -> u64 {
            0
        }
        fn observe_resize(&self, _observer: u64, _element: &Self::Element) {}
        fn unobserve_resize(&self, _observer: u64, _element: &Self::Element) {}
        fn disconnect_resize(&self, _observer: u64) {}
        fn create_mutation_observer(
            &self,
            _callback: Box<dyn FnMut(Vec<crate::MutationRecord>)>,
        ) -> u64 {
            0
        }
        fn observe_mutations(
            &self,
            _observer: u64,
            _element: &Self::Element,
            _options: Option<crate::MutationObserverInit>,
        ) {
        }
        fn disconnect_mutation(&self, _observer: u64) {}
        fn match_media(&self, _query: &str) -> u64 {
            0
        }
        fn media_query_list_get_media(&self, _list: u64) -> String {
            String::new()
        }
        fn media_query_list_get_matches(&self, _list: u64) -> bool {
            false
        }
        fn media_query_list_add_listener(
            &self,
            _list: u64,
            _callback: Box<dyn FnMut(bool)>,
        ) -> u64 {
            0
        }
        fn media_query_list_remove_listener(&self, _list: u64, _listener_id: u64) {}
        fn get_element_by_id(&self, _id: &str) -> Option<Self::Element> {
            None
        }
        fn query_selector(&self, _selector: &str) -> Option<Self::Element> {
            None
        }
        fn query_selector_all(&self, _selector: &str) -> Vec<Self::Element> {
            vec![]
        }
        fn element_from_point(&self, _x: i32, _y: i32) -> Option<Self::Element> {
            None
        }
        fn element_closest(
            &self,
            _element: &Self::Element,
            _selector: &str,
        ) -> Option<Self::Element> {
            None
        }
        fn get_scroll_y(&self) -> f64 {
            0.0
        }
        fn scroll_to(&self, _top: f64, _behavior: &str) {}
        fn on_scroll(&self, _callback: Box<dyn FnMut(f64, f64)>) {}
        fn on_resize(&self, _callback: Box<dyn FnMut(i32, i32)>) {}
        fn copy_to_clipboard(&self, _text: &str) -> bool {
            false
        }
        fn read_clipboard(&self) -> Option<String> {
            None
        }
        fn clipboard_write_text_async(
            &self,
            _text: &str,
            on_complete: Box<dyn FnOnce(Result<(), String>)>,
        ) {
            on_complete(Ok(()));
        }
        fn clipboard_read_text_async(&self, on_complete: Box<dyn FnOnce(Result<String, String>)>) {
            on_complete(Err("clipboard not available in mock".to_string()));
        }
        fn prefers_dark_mode(&self) -> bool {
            false
        }
        fn get_element_rect_by_id(&self, _id: &str) -> Option<crate::DomRect> {
            None
        }
        fn get_bounding_rect_by_class(
            &self,
            _class_name: &str,
            _element: &Self::Element,
        ) -> Option<crate::DomRect> {
            None
        }
        fn request_fullscreen(&self, _element: &Self::Element) {}

        fn get_contenteditable_state(
            &self,
            _element: &Self::Element,
        ) -> Option<crate::ContentEditableState> {
            None
        }

        fn exec_command(&self, _command: &str, _value: Option<&str>) -> bool {
            false
        }

        fn get_selection_start(&self, _element: &Self::Element) -> Option<u32> {
            None
        }

        fn get_selection_end(&self, _element: &Self::Element) -> Option<u32> {
            None
        }

        fn set_content_editable(&self, _element: &Self::Element, _editable: bool) {}

        fn get_inner_html(&self, _element: &Self::Element) -> String {
            String::new()
        }

        fn set_inner_html(&self, _element: &Self::Element, _html: String) {}

        fn get_element_scroll_top(&self, _element: &Self::Element) -> f64 {
            0.0
        }

        fn set_element_scroll_top(&self, _element: &Self::Element, _value: f64) {}

        fn video_play(&self, _element: &Self::Element) {}

        fn video_pause(&self, _element: &Self::Element) {}

        fn video_get_current_time(&self, _element: &Self::Element) -> f64 {
            0.0
        }

        fn video_get_duration(&self, _element: &Self::Element) -> f64 {
            0.0
        }

        fn video_seek(&self, _element: &Self::Element, _time: f64) {}

        fn video_set_muted(&self, _element: &Self::Element, _muted: bool) {}

        fn video_set_volume(&self, _element: &Self::Element, _volume: f64) {}

        fn create_audio_context(&self) -> u64 {
            0
        }

        fn create_analyser_node(&self, _audio_context: u64) -> u64 {
            0
        }

        fn create_media_element_source(&self, _audio_context: u64, _element: u64) -> u64 {
            0
        }

        fn analyser_node_get_frequency_data(&self, _analyser: u64) -> Vec<f32> {
            vec![]
        }

        fn analyser_node_get_time_domain_data(&self, _analyser: u64) -> Vec<f32> {
            vec![]
        }

        fn draw_qrcode_on_canvas_by_id(
            &self,
            _canvas_id: &str,
            _matrix: &[Vec<bool>],
            _modules: u64,
            _color: &str,
            _background: &str,
        ) -> bool {
            false
        }

        fn get_scroll_top_from_point(&self, _x: i32, _y: i32) -> f64 {
            0.0
        }

        fn get_scroll_top_by_selector(&self, _selector: &str) -> f64 {
            0.0
        }

        fn get_target_element_from_event(
            &self,
            _client_x: i32,
            _client_y: i32,
        ) -> Option<Self::Element> {
            None
        }

        fn get_current_position(
            &self,
            _on_success: Box<dyn FnOnce(crate::GeoPosition)>,
            on_error: Box<dyn FnOnce(crate::GeoPositionError)>,
            _enable_high_accuracy: bool,
            _timeout: u32,
            _maximum_age: u32,
        ) {
            on_error(crate::GeoPositionError {
                code: 1,
                message: "geolocation not available in mock".to_string(),
            });
        }
    }

    #[test]
    fn test_scheduler_creation() {
        let platform = Rc::new(RefCell::new(MockPlatform));
        let scheduler = Scheduler::new(platform);
        assert!(scheduler.id() > 0);
    }

    #[test]
    fn test_component_registration() {
        let platform = Rc::new(RefCell::new(MockPlatform));
        let scheduler = Scheduler::new(platform);

        let id = scheduler.register_component(None);
        assert_eq!(id, 0);

        let id2 = scheduler.register_component(None);
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_mark_dirty() {
        let platform = Rc::new(RefCell::new(MockPlatform));
        let scheduler = Scheduler::new(platform);

        let id = scheduler.register_component(None);
        scheduler.mark_dirty(id);

        // Check that the component is marked dirty
        let inner = scheduler.inner.borrow();
        assert!(inner.components[id].dirty);
    }
}
