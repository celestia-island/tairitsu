//! Global DOM operations for use outside the VDOM rendering cycle.
//!
//! Provides a [`DomHandle`] opaque wrapper around raw host handles and a set
//! of functions for direct DOM manipulation. These are intended for
//! imperative code (e.g. custom scrollbar setup) that runs after the VDOM
//! tree has been mounted.
//!
//! # Initialization
//!
//! The WIT binding function pointers **must** be registered once during
//! component bootstrap via [`register_wit_functions`] and
//! [`register_dom_functions`].

use std::sync::Mutex;

use crate::platform::DomRect;

// ---------------------------------------------------------------------------
// DomHandle
// ---------------------------------------------------------------------------

/// Opaque handle to a DOM node managed by the browser-glue host.
///
/// Wraps the raw `u64` handle that the WIT host assigns to each DOM node.
/// Use [`DomHandle::get_inner_id`] only when you need to pass the handle to
/// low-level WIT binding functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DomHandle(u64);

impl DomHandle {
    /// Construct a handle from a raw host id.
    ///
    /// This should only be called by platform implementation code that
    /// receives a handle from the WIT host.
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// The null handle (equivalent to an absent element).
    pub const fn null() -> Self {
        Self(0)
    }

    /// Whether this handle is non-null.
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }

    /// Return the raw `u64` host id.
    ///
    /// Only use this when interfacing with low-level WIT binding functions
    /// that require the raw id.
    pub const fn get_inner_id(&self) -> u64 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Internal: function pointer tables
// ---------------------------------------------------------------------------

static WIT_FUNCS: Mutex<Option<WitFuncs>> = Mutex::new(None);

struct WitFuncs {
    set_style: unsafe fn(u64, &str, &str) -> Result<(), String>,
    get_bounding_client_rect: unsafe fn(u64) -> DomRect,
    set_attribute: unsafe fn(u64, &str, &str),
}

static DOM_FUNCS: Mutex<Option<DomFuncs>> = Mutex::new(None);

/// Function pointers for extended DOM operations.
///
/// Filled in by the platform layer during bootstrap.
pub struct DomFuncs {
    pub get_scroll_top: unsafe fn(u64) -> f64,
    pub set_scroll_top: unsafe fn(u64, f64),
    pub get_scroll_height: unsafe fn(u64) -> i32,
    pub get_client_height: unsafe fn(u64) -> i32,
    pub get_class_list: unsafe fn(u64) -> u64,
    pub class_list_add: unsafe fn(u64, &[String]),
    pub class_list_remove: unsafe fn(u64, &[String]),
    pub class_list_contains: unsafe fn(u64, &str) -> bool,
    pub first_child: unsafe fn(u64) -> Option<u64>,
    pub query_selector_on: unsafe fn(u64, &str) -> Option<u64>,
    pub create_element: unsafe fn(&str) -> u64,
    pub append_child: unsafe fn(u64, u64) -> u64,
    pub remove_child: unsafe fn(u64, u64) -> u64,
    pub get_computed_style_value: unsafe fn(u64, &str) -> String,
    pub set_timeout_fn: unsafe fn(Box<dyn FnOnce()>, i32) -> i32,
    pub clear_timeout_fn: unsafe fn(i32),
    pub request_animation_frame_fn: unsafe fn(Box<dyn FnMut(f64)>) -> u32,
    pub cancel_animation_frame_fn: unsafe fn(u32),
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register the core WIT binding function pointers.
///
/// # Safety
/// Caller must ensure the pointers remain valid for the program lifetime.
pub unsafe fn register_wit_functions(
    set_style: unsafe fn(u64, &str, &str) -> Result<(), String>,
    get_bounding_client_rect: unsafe fn(u64) -> DomRect,
    set_attribute: unsafe fn(u64, &str, &str),
) {
    *WIT_FUNCS.lock().unwrap() = Some(WitFuncs {
        set_style,
        get_bounding_client_rect,
        set_attribute,
    });
}

/// Register extended DOM operation function pointers.
///
/// # Safety
/// Caller must ensure the pointers remain valid for the program lifetime.
pub unsafe fn register_dom_functions(funcs: DomFuncs) {
    *DOM_FUNCS.lock().unwrap() = Some(funcs);
}

// ---------------------------------------------------------------------------
// Core operations (backed by WIT_FUNCS)
// ---------------------------------------------------------------------------

/// Set a CSS property on an element.
pub fn set_style(el: DomHandle, property: &str, value: &str) {
    if let Some(f) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe { let _ = (f.set_style)(el.get_inner_id(), property, value); }
    }
}

/// Get the bounding client rect of an element.
pub fn get_bounding_client_rect(el: DomHandle) -> DomRect {
    if let Some(f) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe { (f.get_bounding_client_rect)(el.get_inner_id()) }
    } else {
        DomRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }
}

/// Set an attribute on an element.
pub fn set_attribute(el: DomHandle, name: &str, value: &str) {
    if let Some(f) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe { (f.set_attribute)(el.get_inner_id(), name, value) };
    }
}

// ---------------------------------------------------------------------------
// Extended DOM operations (backed by DOM_FUNCS)
// ---------------------------------------------------------------------------

pub fn get_scroll_top(el: DomHandle) -> f64 {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(0.0, |f| unsafe { (f.get_scroll_top)(el.get_inner_id()) })
}

pub fn set_scroll_top(el: DomHandle, value: f64) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        unsafe { (f.set_scroll_top)(el.get_inner_id(), value) };
    }
}

pub fn get_scroll_height(el: DomHandle) -> i32 {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(0, |f| unsafe { (f.get_scroll_height)(el.get_inner_id()) })
}

pub fn get_client_height(el: DomHandle) -> i32 {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(0, |f| unsafe { (f.get_client_height)(el.get_inner_id()) })
}

pub fn class_list_add(el: DomHandle, tokens: &[&str]) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        let list = unsafe { (f.get_class_list)(el.get_inner_id()) };
        let ts: Vec<String> = tokens.iter().map(|s| s.to_string()).collect();
        unsafe { (f.class_list_add)(list, &ts) };
    }
}

pub fn class_list_remove(el: DomHandle, tokens: &[&str]) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        let list = unsafe { (f.get_class_list)(el.get_inner_id()) };
        let ts: Vec<String> = tokens.iter().map(|s| s.to_string()).collect();
        unsafe { (f.class_list_remove)(list, &ts) };
    }
}

pub fn class_list_contains(el: DomHandle, token: &str) -> bool {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(false, |f| {
        let list = unsafe { (f.get_class_list)(el.get_inner_id()) };
        unsafe { (f.class_list_contains)(list, token) }
    })
}

pub fn first_child(el: DomHandle) -> Option<DomHandle> {
    DOM_FUNCS.lock().unwrap().as_ref().and_then(|f| {
        unsafe { (f.first_child)(el.get_inner_id()) }.map(DomHandle::from_raw)
    })
}

pub fn query_selector_on(el: DomHandle, selector: &str) -> Option<DomHandle> {
    DOM_FUNCS.lock().unwrap().as_ref().and_then(|f| {
        unsafe { (f.query_selector_on)(el.get_inner_id(), selector) }.map(DomHandle::from_raw)
    })
}

pub fn create_element(tag: &str) -> DomHandle {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(DomHandle::null(), |f| {
        DomHandle::from_raw(unsafe { (f.create_element)(tag) })
    })
}

pub fn append_child(parent: DomHandle, child: DomHandle) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        let _ = unsafe { (f.append_child)(parent.get_inner_id(), child.get_inner_id()) };
    }
}

pub fn remove_child(parent: DomHandle, child: DomHandle) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        let _ = unsafe { (f.remove_child)(parent.get_inner_id(), child.get_inner_id()) };
    }
}

pub fn get_computed_style_value(el: DomHandle, property: &str) -> String {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(String::new(), |f| unsafe {
        (f.get_computed_style_value)(el.get_inner_id(), property)
    })
}

pub fn set_timeout(callback: Box<dyn FnOnce()>, ms: i32) -> i32 {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(0, |f| unsafe {
        (f.set_timeout_fn)(callback, ms)
    })
}

pub fn clear_timeout(id: i32) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        unsafe { (f.clear_timeout_fn)(id) };
    }
}

pub fn request_animation_frame(callback: Box<dyn FnMut(f64)>) -> u32 {
    DOM_FUNCS.lock().unwrap().as_ref().map_or(0, |f| unsafe {
        (f.request_animation_frame_fn)(callback)
    })
}

pub fn cancel_animation_frame(id: u32) {
    if let Some(f) = DOM_FUNCS.lock().unwrap().as_ref() {
        unsafe { (f.cancel_animation_frame_fn)(id) };
    }
}
