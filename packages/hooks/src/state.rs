use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{runtime, VNode};

pub struct StateSetter<T> {
    inner: Rc<RefCell<T>>,
    component_id: usize,
}

impl<T> StateSetter<T> {
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
        runtime::mark_dirty_deferred(self.component_id);
    }
}

impl<T> std::fmt::Debug for StateSetter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateSetter")
            .field("component_id", &self.component_id)
            .finish()
    }
}

pub fn use_state<T: Clone + Default + 'static>(initial: T) -> (Rc<RefCell<T>>, StateSetter<T>) {
    let component_id =
        runtime::active_component_id().unwrap_or_else(|| runtime::use_component(VNode::empty));
    let state = runtime::hook_slot(component_id, "use_state", || Rc::new(RefCell::new(initial)));
    let setter = StateSetter {
        inner: Rc::clone(&state),
        component_id,
    };

    (state, setter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_state() {
        let (state, set_state) = use_state(0);

        assert_eq!(*state.borrow(), 0);

        set_state.set(42);
        assert_eq!(*state.borrow(), 42);
    }

    #[test]
    fn test_use_state_marks_dirty() {
        let (_state, set_state) = use_state(0);

        runtime::flush_render();

        set_state.set(42);
        assert_eq!(*_state.borrow(), 42);
    }
}
