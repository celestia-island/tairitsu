use tairitsu_vdom::{Classes, Signal, Style, runtime};

/// Creates a new Signal with the given initial value.
/// Takes a closure that returns the initial value (Dioxus-compatible API).
///
/// The signal automatically integrates with the reactive runtime to trigger
/// re-renders when its value changes.
pub fn use_signal<T: Clone + 'static, F: FnOnce() -> T>(initial: F) -> ReactiveSignal<T> {
    let signal = Signal::new(initial());
    let component_id = runtime::use_component(|| {
        // Placeholder - the actual render function will be provided by the component
        tairitsu_vdom::VNode::empty()
    });

    ReactiveSignal {
        signal,
        component_id,
    }
}

/// A reactive signal that automatically triggers component re-renders when modified.
#[derive(Clone)]
pub struct ReactiveSignal<T> {
    signal: Signal<T>,
    component_id: runtime::ComponentId,
}

impl<T: Clone + 'static> ReactiveSignal<T> {
    /// Get the current value of the signal.
    pub fn get(&self) -> T {
        self.signal.get()
    }

    /// Set a new value and trigger re-render.
    pub fn set(&self, value: T) {
        self.signal.set(value);
        runtime::mark_dirty(self.component_id);
        runtime::flush_render();
    }

    /// Access the underlying signal for advanced operations.
    pub fn inner(&self) -> &Signal<T> {
        &self.signal
    }

    /// Dioxus compatibility alias for get()
    pub fn read(&self) -> T {
        self.get()
    }

    /// Dioxus compatibility alias for write()
    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        self.signal.write()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ReactiveSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactiveSignal")
            .field("signal", &self.signal)
            .field("component_id", &self.component_id)
            .finish()
    }
}

impl std::fmt::Display for ReactiveSignal<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl From<ReactiveSignal<String>> for Style {
    fn from(signal: ReactiveSignal<String>) -> Self {
        Style::from(signal.get())
    }
}

impl From<ReactiveSignal<String>> for Classes {
    fn from(signal: ReactiveSignal<String>) -> Self {
        Classes::from(signal.get())
    }
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

    #[test]
    fn test_reactive_signal_read() {
        let signal = use_signal(|| "hello");

        assert_eq!(signal.read(), "hello");
    }

    #[test]
    fn test_reactive_signal_inner() {
        let signal = use_signal(|| 123);

        // Inner signal should work independently
        let inner = signal.inner();
        assert_eq!(inner.get(), 123);
    }
}
