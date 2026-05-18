use std::cell::RefCell;
use std::rc::Rc;

pub fn use_state<T: Clone + Default + 'static>(initial: T) -> (Rc<RefCell<T>>, impl Fn(T)) {
    let state = Rc::new(RefCell::new(initial));
    let state_clone = Rc::clone(&state);

    let setter = move |value: T| {
        *state_clone.borrow_mut() = value;
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
}
