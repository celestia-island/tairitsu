//! Browser-specific platform implementation

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::AddAssign;

pub struct BrowserPlatform {
    timeout_callbacks: Rc<RefCell<HashMap<u32, u32>>>,
    next_timeout_id: Rc<RefCell<u32>>,
    animation_callbacks: Rc<RefCell<HashMap<u32, u32>>>,
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

    #[allow(unused_variables)]
    pub fn set_timeout<F>(&self, callback: F, delay_ms: u32) -> u32
    where
        F: Fn() + 'static,
    {
        let id = *self.next_timeout_id.borrow();
        self.next_timeout_id.borrow_mut().add_assign(1);

        // In a real implementation, this would interact with the browser APIs
        // For now, we'll just store the callback
        self.timeout_callbacks.borrow_mut().insert(id, 0);

        id
    }

    #[allow(unused_variables)]
    pub fn request_animation_frame<F>(&self, callback: F) -> u32
    where
        F: Fn() + 'static,
    {
        let id = *self.next_animation_id.borrow();
        self.next_animation_id.borrow_mut().add_assign(1);

        // In a real implementation, this would interact with the browser APIs
        // For now, we'll just store the callback
        self.animation_callbacks.borrow_mut().insert(id, 0);

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
    // Initialize browser-specific features
    println!("Initializing Tairitsu Web (Browser platform)");
}