use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{runtime, VNode};

pub fn use_state<T: Clone + Default + 'static>(initial: T) -> (Rc<RefCell<T>>, impl Fn(T)) {
    let component_id =
        runtime::active_component_id().unwrap_or_else(|| runtime::use_component(VNode::empty));
    let state = Rc::new(RefCell::new(initial));
    let state_clone = Rc::clone(&state);

    let setter = move |value: T| {
        *state_clone.borrow_mut() = value;
        runtime::mark_dirty_deferred(component_id);
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

        set_state(42);
        assert_eq!(*state.borrow(), 42);
    }

    #[test]
    fn test_use_state_marks_dirty() {
        let (_state, set_state) = use_state(0);

        // Initially no dirty components
        runtime::flush_render();

        set_state(42);
        assert_eq!(*_state.borrow(), 42);
    }
}
