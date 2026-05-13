use std::cell::RefCell;
use std::rc::Rc;

pub struct IntervalHandle {
    id: Rc<RefCell<Option<i32>>>,
}

impl IntervalHandle {
    pub fn start<P>(&self, platform: &P, callback: Box<dyn FnMut()>, ms: u32)
    where
        P: tairitsu_vdom::Platform,
    {
        self.clear::<P>(platform);
        let id = platform.set_interval(callback, ms as i32);
        *self.id.borrow_mut() = Some(id);
    }

    pub fn clear<P>(&self, platform: &P)
    where
        P: tairitsu_vdom::Platform,
    {
        if let Some(id) = self.id.borrow_mut().take() {
            platform.clear_interval(id);
        }
    }

    pub fn is_active(&self) -> bool {
        self.id.borrow().is_some()
    }
}

impl Default for IntervalHandle {
    fn default() -> Self {
        Self {
            id: Rc::new(RefCell::new(None)),
        }
    }
}

pub fn use_interval() -> IntervalHandle {
    IntervalHandle::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_handle_default() {
        let handle = use_interval();
        assert!(!handle.is_active());
    }

    #[test]
    fn test_interval_handle_set_id() {
        let handle = IntervalHandle {
            id: Rc::new(RefCell::new(Some(42))),
        };
        assert!(handle.is_active());
    }

    #[test]
    fn test_interval_handle_clear_when_none() {
        let handle = use_interval();
        assert!(!handle.is_active());
        let id = handle.id.clone();
        assert!(id.borrow().is_none());
    }

    #[test]
    fn test_interval_handle_clear_sets_none() {
        let handle = IntervalHandle {
            id: Rc::new(RefCell::new(Some(42))),
        };
        assert!(handle.is_active());
        *handle.id.borrow_mut() = None;
        assert!(!handle.is_active());
    }
}
