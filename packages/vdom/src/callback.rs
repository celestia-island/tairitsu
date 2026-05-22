use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// A smart pointer wrapping `Fn(T) -> R` for event handling.
///
/// `Callback<T, R>` is a cloneable wrapper around a closure that can be shared
/// across components. It's commonly used for event handlers in UI frameworks.
///
/// # Type Parameters
///
/// * `T` - The argument type passed to the callback
/// * `R` - The return type of the callback (defaults to `()`)
///
/// # Examples
///
/// ```
/// use tairitsu_vdom::Callback;
///
/// // Create a callback with no arguments (use |_| to accept unit tuple)
/// let callback = Callback::<(), ()>::new(|_| {
///     println!("Clicked!");
/// });
/// callback.call(());
///
/// // Create a callback with an argument
/// let callback = Callback::new(|x: i32| x * 2);
/// assert_eq!(callback.call(5), 10);
/// ```
#[derive(Clone)]
pub struct Callback<T, R = ()> {
    inner: Rc<dyn Fn(T) -> R>,
}

impl<T, R> Callback<T, R> {
    /// Creates a new `Callback` from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use tairitsu_vdom::Callback;
    ///
    /// let callback = Callback::new(|x: i32| x + 1);
    /// assert_eq!(callback.call(1), 2);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        Self { inner: Rc::new(f) }
    }

    /// Alias for `new()`. Creates a callback from a function.
    ///
    /// This method is provided for API consistency and explicit naming.
    ///
    /// # Examples
    ///
    /// ```
    /// use tairitsu_vdom::Callback;
    ///
    /// let callback = Callback::from_fn(|s: &str| s.to_uppercase());
    /// assert_eq!(callback.call("hello"), "HELLO");
    /// ```
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(T) -> R + 'static,
    {
        Self::new(f)
    }

    /// Invokes the callback with the given argument.
    ///
    /// # Examples
    ///
    /// ```
    /// use tairitsu_vdom::Callback;
    ///
    /// let callback = Callback::new(|x: i32| x * x);
    /// assert_eq!(callback.call(4), 16);
    /// ```
    pub fn call(&self, arg: T) -> R {
        (self.inner)(arg)
    }
}

impl<T, R> Deref for Callback<T, R> {
    type Target = dyn Fn(T) -> R;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T, R> DerefMut for Callback<T, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We cannot provide a mutable reference to the inner dyn Fn
        // since it's behind an Rc. This implementation exists for
        // completeness but panics at runtime if called.
        // In practice, this is rarely needed since callbacks are typically
        // invoked via call() or deref().
        panic!("Cannot mutate a Callback's inner function");
    }
}

impl<T, R, F> From<F> for Callback<T, R>
where
    F: Fn(T) -> R + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

impl<T, R> std::fmt::Debug for Callback<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callback")
            .field("inner", &"Rc<dyn Fn(T) -> R>")
            .finish()
    }
}

impl<T, R> PartialEq for Callback<T, R> {
    fn eq(&self, other: &Self) -> bool {
        // Two callbacks are equal if they point to the same Rc
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

// Specialization for unit argument callbacks
impl<R> Callback<(), R> {
    /// Creates a no-argument callback.
    ///
    /// This is a convenience method for callbacks that don't need arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use tairitsu_vdom::Callback;
    ///
    /// let callback: Callback<(), i32> = Callback::no_arg(|| 42);
    /// assert_eq!(callback.call(()), 42);
    /// ```
    pub fn no_arg<F>(f: F) -> Self
    where
        F: Fn() -> R + 'static,
    {
        Self::new(move |()| f())
    }
}

impl Callback<(), ()> {
    /// Creates a simple no-argument, no-return callback.
    ///
    /// This is the most common form for event handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// use tairitsu_vdom::Callback;
    ///
    /// let clicked: Callback<(), ()> = Callback::simple(|| {
    ///     println!("Button clicked!");
    /// });
    /// clicked.call(());
    /// ```
    pub fn simple<F>(f: F) -> Self
    where
        F: Fn() + 'static,
    {
        Self::no_arg(f)
    }
}

/// Type alias for Dioxus compatibility.
///
/// `EventHandler<T>` is an alias for a `Callback` that returns unit `()`.
/// This matches the naming convention used in Dioxus for event handlers.
pub type EventHandler<T> = Callback<T, ()>;

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_callback_new() {
        let callback = Callback::new(|x: i32| x * 2);
        assert_eq!(callback.call(5), 10);
    }

    #[test]
    fn test_callback_from_fn() {
        let callback = Callback::from_fn(|s: String| s.len());
        assert_eq!(callback.call("hello".to_string()), 5);
    }

    #[test]
    fn test_callback_clone() {
        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = call_count.clone();

        let callback = Callback::new(move |_: ()| {
            call_count_clone.set(call_count_clone.get() + 1);
        });

        let cloned = callback.clone();
        callback.call(());
        cloned.call(());

        assert_eq!(call_count.get(), 2);
    }

    #[test]
    fn test_callback_deref() {
        let callback = Callback::new(|x: i32| x + 1);
        // Can use the callback via deref coercion
        let result = callback(3);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_callback_from_impl() {
        let callback: Callback<i32, i32> = Callback::from(|x| x * x);
        assert_eq!(callback.call(4), 16);
    }

    #[test]
    fn test_callback_unit_arg() {
        let callback: Callback<(), i32> = Callback::no_arg(|| 42);
        assert_eq!(callback.call(()), 42);
    }

    #[test]
    fn test_callback_simple() {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let callback: Callback<(), ()> = Callback::simple(move || {
            called_clone.set(true);
        });

        callback.call(());
        assert!(called.get());
    }

    #[test]
    fn test_event_handler_alias() {
        let callback: EventHandler<i32> = Callback::new(|_| {});
        callback.call(42); // Should compile and run
    }

    #[test]
    fn test_callback_with_complex_type() {
        #[derive(Debug, Clone)]
        struct MyEvent {
            value: String,
        }

        let callback = Callback::new(|e: MyEvent| e.value.len());
        let event = MyEvent {
            value: "test".to_string(),
        };
        assert_eq!(callback.call(event), 4);
    }

    #[test]
    fn test_callback_debug() {
        let callback: Callback<i32, i32> = Callback::new(|x| x);
        let debug_output = format!("{:?}", callback);
        assert!(debug_output.contains("Callback"));
    }

    #[test]
    fn test_callback_shared_state() {
        let state = Rc::new(Cell::new(0));
        let state_clone = state.clone();

        let callback = Callback::new(move |x: i32| {
            state_clone.set(x);
            x
        });

        let callback2 = callback.clone();
        callback.call(10);
        assert_eq!(state.get(), 10);

        callback2.call(20);
        assert_eq!(state.get(), 20);
    }

    #[test]
    #[should_panic(expected = "Cannot mutate a Callback")]
    fn test_callback_deref_mut_panics() {
        let mut callback = Callback::new(|x: i32| x);
        let _ = callback.deref_mut();
    }
}
