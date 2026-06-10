//! Browser-specific platform implementation

use std::{cell::RefCell, collections::HashMap, ops::AddAssign, rc::Rc};

type CallbackMap = Rc<RefCell<HashMap<u32, Box<dyn Fn()>>>>;

pub struct BrowserPlatform {
    timeout_callbacks: CallbackMap,
    next_timeout_id: Rc<RefCell<u32>>,
    animation_callbacks: CallbackMap,
    next_animation_id: Rc<RefCell<u32>>,
}

impl Default for BrowserPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserPlatform {
    pub fn new() -> Self {
        Self {
            timeout_callbacks: Rc::new(RefCell::new(HashMap::new())),
            next_timeout_id: Rc::new(RefCell::new(1)),
            animation_callbacks: Rc::new(RefCell::new(HashMap::new())),
            next_animation_id: Rc::new(RefCell::new(1)),
        }
    }

    pub fn set_timeout<F>(&self, callback: F, _delay_ms: u32) -> u32
    where
        F: Fn() + 'static,
    {
        let id = *self.next_timeout_id.borrow();
        self.next_timeout_id.borrow_mut().add_assign(1);

        let callback = Box::new(callback);
        callback();
        self.timeout_callbacks.borrow_mut().insert(id, callback);

        id
    }

    pub fn request_animation_frame<F>(&self, callback: F) -> u32
    where
        F: Fn() + 'static,
    {
        let id = *self.next_animation_id.borrow();
        self.next_animation_id.borrow_mut().add_assign(1);

        let callback = Box::new(callback);
        callback();
        self.animation_callbacks.borrow_mut().insert(id, callback);

        id
    }

    pub fn get_bounding_client_rect(&self, _element_id: &str) -> Option<Rect> {
        // In a real implementation, this would interact with the browser APIs
        // For now, return a default rect
        Some(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn init() {
    tracing::info!("Initializing Tairitsu Web (Browser platform)");
}
