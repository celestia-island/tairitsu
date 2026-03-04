use tairitsu_vdom::Signal;

pub fn use_signal<T: Clone + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_signal() {
        let signal = use_signal(0);

        assert_eq!(signal.get(), 0);

        signal.set(42);
        assert_eq!(signal.get(), 42);
    }
}
