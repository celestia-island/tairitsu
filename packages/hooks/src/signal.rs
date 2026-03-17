use tairitsu_vdom::Signal;

/// Creates a new Signal with the given initial value.
/// Takes a closure that returns the initial value (Dioxus-compatible API).
pub fn use_signal<T: Clone + 'static, F: FnOnce() -> T>(initial: F) -> Signal<T> {
    Signal::new(initial())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_signal() {
        let signal = use_signal(|| 0);

        assert_eq!(signal.get(), 0);

        signal.set(42);
        assert_eq!(signal.get(), 42);
    }
}
