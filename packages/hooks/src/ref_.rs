use std::{cell::RefCell, rc::Rc};

pub struct UseRef<T> {
    current: Rc<RefCell<T>>,
}

impl<T> UseRef<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Rc::new(RefCell::new(initial)),
        }
    }

    pub fn current(&self) -> std::cell::Ref<'_, T> {
        self.current.borrow()
    }

    pub fn current_mut(&self) -> std::cell::RefMut<'_, T> {
        self.current.borrow_mut()
    }

    pub fn set(&self, value: T) {
        *self.current.borrow_mut() = value;
    }
}

impl<T> Clone for UseRef<T> {
    fn clone(&self) -> Self {
        Self {
            current: Rc::clone(&self.current),
        }
    }
}

pub fn use_ref<T: 'static>(initial: T) -> UseRef<T> {
    UseRef::new(initial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_ref() {
        let counter = use_ref(0);

        assert_eq!(*counter.current(), 0);

        *counter.current_mut() += 1;
        assert_eq!(*counter.current(), 1);

        counter.set(10);
        assert_eq!(*counter.current(), 10);
    }

    #[test]
    fn test_use_ref_clone() {
        let ref1 = use_ref(String::from("hello"));
        let ref2 = ref1.clone();

        ref2.set(String::from("world"));

        assert_eq!(*ref1.current(), "world");
        assert_eq!(*ref2.current(), "world");
    }
}
