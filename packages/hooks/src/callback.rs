use std::cell::RefCell;
use std::rc::Rc;

/// A cached callback that only recreates when dependencies change.
///
/// Similar to React's useCallback, this hook caches a callback function
/// and only recreates it when the dependencies change, providing a stable
/// reference for child components or effects.
pub struct Callback<F, D>
where
    F: ?Sized + 'static,
    D: PartialEq + 'static,
{
    callback: Rc<RefCell<Option<Rc<F>>>>,
    factory: Rc<dyn Fn() -> Rc<F>>,
    deps: Rc<RefCell<D>>,
}

impl<F, D> Callback<F, D>
where
    F: ?Sized + 'static,
    D: PartialEq + 'static,
{
    /// Creates a new Callback with the given factory function and dependencies.
    fn new<C>(factory: C, deps: D) -> Self
    where
        C: Fn() -> Rc<F> + 'static,
    {
        let initial_callback = factory();
        Self {
            callback: Rc::new(RefCell::new(Some(initial_callback))),
            factory: Rc::new(factory),
            deps: Rc::new(RefCell::new(deps)),
        }
    }

    /// Gets the current cached callback.
    pub fn get(&self) -> Rc<F> {
        self.callback.borrow().as_ref().unwrap().clone()
    }

    /// Updates the dependencies and recreates the callback if they have changed.
    pub fn update_deps(&self, new_deps: D) {
        let mut deps = self.deps.borrow_mut();
        if *deps != new_deps {
            *deps = new_deps;
            drop(deps); // Release borrow before recreating
            let new_callback = (self.factory)();
            *self.callback.borrow_mut() = Some(new_callback);
        }
    }
}

impl<F, D> Clone for Callback<F, D>
where
    F: ?Sized + 'static,
    D: PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            callback: Rc::clone(&self.callback),
            factory: Rc::clone(&self.factory),
            deps: Rc::clone(&self.deps),
        }
    }
}

/// Creates a cached callback that only recreates when dependencies change.
///
/// # Arguments
/// * `factory` - A function that creates the callback
/// * `deps` - The dependencies that trigger recreation when changed
///
/// # Returns
/// A `Callback` struct that provides access to the cached callback
///
/// # Example
/// ```ignore
/// let callback = use_callback(|| Rc::new(move || println!("Hello")), ());
/// let cb = callback.get();
/// cb();
/// ```
pub fn use_callback<F, D, C>(factory: C, deps: D) -> Callback<F, D>
where
    F: ?Sized + 'static,
    D: PartialEq + 'static,
    C: Fn() -> Rc<F> + 'static,
{
    Callback::new(factory, deps)
}

/// Type alias for a callback that takes no arguments and returns nothing.
pub type VoidCallback = dyn Fn();

/// Type alias for a callback that takes no arguments and returns a value.
pub type ReturnCallback<T> = dyn Fn() -> T;

/// Creates a cached void callback (no arguments, no return value).
///
/// This is a convenience function for the common case of callbacks
/// that take no arguments and return nothing.
///
/// # Example
/// ```ignore
/// let callback = use_void_callback(|| {
///     println!("Button clicked!");
/// }, count);
/// ```
pub fn use_void_callback<D, F>(callback: F, deps: D) -> Callback<VoidCallback, D>
where
    D: PartialEq + 'static,
    F: Fn() + Clone + 'static,
{
    use_callback(
        move || {
            let cb = callback.clone();
            Rc::new(cb) as Rc<VoidCallback>
        },
        deps,
    )
}

/// Creates a cached callback that returns a value.
///
/// # Example
/// ```ignore
/// let callback = use_return_callback(|| 42, ());
/// let result = callback.get()();
/// ```
pub fn use_return_callback<T, D, F>(callback: F, deps: D) -> Callback<ReturnCallback<T>, D>
where
    T: 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    use_callback(
        move || {
            let cb = callback.clone();
            Rc::new(cb) as Rc<ReturnCallback<T>>
        },
        deps,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_use_callback_basic() {
        let create_count = Rc::new(Cell::new(0));
        let create_count_clone = Rc::clone(&create_count);

        let callback = use_callback(
            move || {
                create_count_clone.set(create_count_clone.get() + 1);
                Rc::new(|| 42) as Rc<dyn Fn() -> i32>
            },
            (),
        );

        // Initial creation
        let cb1 = callback.get();
        assert_eq!(cb1(), 42);
        assert_eq!(create_count.get(), 1);

        // Same deps - should return cached callback
        let cb2 = callback.get();
        assert_eq!(cb2(), 42);
        // Callback factory not called again
        assert_eq!(create_count.get(), 1);
    }

    #[test]
    fn test_use_callback_with_deps() {
        let create_count = Rc::new(Cell::new(0));
        let create_count_clone = Rc::clone(&create_count);

        let callback = use_callback(
            move || {
                create_count_clone.set(create_count_clone.get() + 1);
                Rc::new(|| "hello") as Rc<dyn Fn() -> &'static str>
            },
            1,
        );

        // Initial creation
        assert_eq!(create_count.get(), 1);

        // Same deps - no recreate
        callback.update_deps(1);
        assert_eq!(create_count.get(), 1);

        // Different deps - recreate
        callback.update_deps(2);
        assert_eq!(create_count.get(), 2);
    }

    #[test]
    fn test_use_callback_clone() {
        let callback1 = use_callback(|| Rc::new(|| 123) as Rc<dyn Fn() -> i32>, ());
        let callback2 = callback1.clone();

        let cb1 = callback1.get();
        let cb2 = callback2.get();

        assert_eq!(cb1(), 123);
        assert_eq!(cb2(), 123);
    }

    #[test]
    fn test_use_void_callback() {
        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let callback = use_void_callback(
            move || {
                call_count_clone.set(call_count_clone.get() + 1);
            },
            (),
        );

        let cb = callback.get();
        cb();
        cb();
        assert_eq!(call_count.get(), 2);
    }

    #[test]
    fn test_use_return_callback() {
        let callback = use_return_callback(|| 42, ());

        let cb = callback.get();
        assert_eq!(cb(), 42);
    }

    #[test]
    fn test_use_callback_with_string_deps() {
        let callback =
            use_callback(|| Rc::new(|| "result") as Rc<dyn Fn() -> &'static str>, String::from("a"));

        // Same string - no recreate
        callback.update_deps(String::from("a"));

        // Different string - recreate
        callback.update_deps(String::from("b"));
    }

    #[test]
    fn test_use_callback_with_tuple_deps() {
        let callback = use_callback(|| Rc::new(|| 1) as Rc<dyn Fn() -> i32>, (1, 2));

        // Same tuple - no recreate
        callback.update_deps((1, 2));

        // Different tuple - recreate
        callback.update_deps((1, 3));
    }

    #[test]
    fn test_callback_stable_reference() {
        let callback = use_callback(|| Rc::new(|| 42) as Rc<dyn Fn() -> i32>, ());

        let ref1 = callback.get();
        let ref2 = callback.get();

        // Both references should point to the same callback
        assert!(Rc::ptr_eq(&ref1, &ref2));
    }

    #[test]
    fn test_callback_updates_reference_on_deps_change() {
        let callback = use_callback(|| Rc::new(|| 42) as Rc<dyn Fn() -> i32>, 1);

        let ref1 = callback.get();

        // Change deps
        callback.update_deps(2);

        let ref2 = callback.get();

        // References should be different after deps change
        assert!(!Rc::ptr_eq(&ref1, &ref2));
    }
}
