use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static CONTEXT: RefCell<HashMap<TypeId, Rc<dyn Any>>> = RefCell::new(HashMap::new());
}

pub struct Context<T> {
    value: Rc<RefCell<T>>,
}

impl<T: 'static> Context<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.value.borrow()
    }

    pub fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.value.borrow_mut()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
    }

    pub fn clone_value(&self) -> Rc<RefCell<T>> {
        Rc::clone(&self.value)
    }
}

impl<T: 'static> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

pub fn provide_context<T: 'static>(value: T) -> Context<T> {
    let context = Context::new(value);
    CONTEXT.with(|ctx| {
        ctx.borrow_mut()
            .insert(TypeId::of::<T>(), Rc::new(context.clone()));
    });
    context
}

pub fn use_context<T: 'static + Clone>() -> Option<Context<T>> {
    CONTEXT.with(|ctx| {
        ctx.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<Context<T>>().cloned())
    })
}

pub fn consume_context<T: 'static + Clone>() -> T {
    use_context::<T>()
        .expect("Context not found. Make sure to call provide_context first.")
        .get()
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let ctx = Context::new(42);
        assert_eq!(*ctx.get(), 42);

        ctx.set(100);
        assert_eq!(*ctx.get(), 100);
    }

    #[test]
    fn test_context_clone() {
        let ctx1 = Context::new(String::from("hello"));
        let ctx2 = ctx1.clone();

        ctx2.set(String::from("world"));

        assert_eq!(*ctx1.get(), "world");
        assert_eq!(*ctx2.get(), "world");
    }

    #[test]
    fn test_provide_and_use_context() {
        provide_context(42i32);

        let ctx = use_context::<i32>();
        assert!(ctx.is_some());
        assert_eq!(*ctx.unwrap().get(), 42);
    }

    #[test]
    fn test_consume_context() {
        provide_context(String::from("test value"));

        let value = consume_context::<String>();
        assert_eq!(value, "test value");
    }
}
