use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

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

/// Provide a context value that is automatically removed when the returned
/// guard is dropped. Useful for scoped contexts in component trees and tests.
///
/// # Example
///
/// ```rust,ignore
/// let _guard = provide_context_scoped(my_value);
/// // context is available here
/// // _guard is dropped → context is removed
/// ```
pub fn provide_context_scoped<T: 'static>(value: T) -> ContextGuard {
    provide_context(value);
    ContextGuard::new::<T>()
}

/// Remove a specific context type from the registry.
/// Returns `true` if the context was present and removed.
pub fn drop_context<T: 'static>() -> bool {
    CONTEXT.with(|ctx| ctx.borrow_mut().remove(&TypeId::of::<T>()).is_some())
}

/// Clear all contexts from the registry. Useful in tests and during cleanup.
pub fn clear_all_contexts() {
    CONTEXT.with(|ctx| ctx.borrow_mut().clear());
}

pub fn use_context<T: 'static + Clone>() -> Option<Context<T>> {
    CONTEXT.with(|ctx| {
        ctx.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<Context<T>>().cloned())
    })
}

pub fn consume_context<T: 'static + Clone>() -> Option<T> {
    use_context::<T>().map(|ctx| ctx.get().clone())
}

/// Consume a context value, panicking if not found.
/// Prefer [`consume_context`] for fallible access.
pub fn consume_context_expect<T: 'static + Clone>() -> T {
    consume_context::<T>().unwrap_or_else(|| {
        panic!(
            "Context not found for type {}. Make sure to call provide_context first.",
            std::any::type_name::<T>()
        )
    })
}

/// A guard that removes a context type from the registry when dropped.
/// Created by [`provide_context_scoped`].
pub struct ContextGuard {
    type_id: Option<std::any::TypeId>,
}

impl ContextGuard {
    fn new<T: 'static>() -> Self {
        Self {
            type_id: Some(TypeId::of::<T>()),
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        if let Some(type_id) = self.type_id.take() {
            CONTEXT.with(|ctx| {
                ctx.borrow_mut().remove(&type_id);
            });
        }
    }
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

        let value = consume_context_expect::<String>();
        assert_eq!(value, "test value");
    }

    #[test]
    fn test_consume_context_missing() {
        clear_all_contexts();
        let value = consume_context::<i64>();
        assert!(value.is_none());
    }

    #[test]
    fn test_context_scoped_cleanup() {
        {
            let _guard = provide_context_scoped::<i32>(42);
            let ctx = use_context::<i32>();
            assert!(ctx.is_some());
        }
        // After guard is dropped, context should be removed
        let ctx = use_context::<i32>();
        assert!(ctx.is_none());
    }

    #[test]
    fn test_drop_context() {
        provide_context::<i32>(42);
        assert!(use_context::<i32>().is_some());

        let removed = drop_context::<i32>();
        assert!(removed);
        assert!(use_context::<i32>().is_none());
    }

    #[test]
    fn test_clear_all_contexts() {
        provide_context::<i32>(42);
        provide_context::<String>("hello".to_string());

        clear_all_contexts();

        assert!(use_context::<i32>().is_none());
        assert!(use_context::<String>().is_none());
    }
}
