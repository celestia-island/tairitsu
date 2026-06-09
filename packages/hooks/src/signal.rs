use std::ops::{Deref, DerefMut};

use tairitsu_vdom::{runtime, Classes, Signal, Style, VNode};

/// Creates a new Signal with the given initial value.
/// Takes a closure that returns the initial value (Dioxus-compatible API).
///
/// The signal automatically integrates with the reactive runtime to trigger
/// re-renders when its value changes.
pub fn use_signal<T: Clone + 'static, F: FnOnce() -> T>(initial: F) -> ReactiveSignal<T> {
    let signal = Signal::new(initial());
    let component_id = runtime::active_component_id()
        .unwrap_or_else(|| runtime::use_component(tairitsu_vdom::VNode::empty));

    ReactiveSignal {
        signal,
        component_id,
    }
}

pub fn use_standalone_signal<T: Clone + 'static>(initial: T) -> StandaloneSignal<T> {
    let signal = Signal::new(initial);
    let component_id =
        runtime::active_component_id().unwrap_or_else(|| runtime::use_component(VNode::empty));

    StandaloneSignal {
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
    ///
    /// Returns a guard that automatically marks the component dirty on drop,
    /// ensuring the UI re-renders after the value is mutated.
    pub fn write(&self) -> SignalWriteGuard<'_, T> {
        SignalWriteGuard::new(self.signal.write(), self.component_id)
    }
}

/// A wrapper around `RefMut` that marks the component dirty on drop.
///
/// This ensures that any mutation made through `ReactiveSignal::write()` or
/// `StandaloneSignal::write()` automatically triggers a re-render.
pub struct SignalWriteGuard<'a, T: Clone + 'static> {
    inner: std::cell::RefMut<'a, T>,
    component_id: runtime::ComponentId,
}

impl<'a, T: Clone + 'static> SignalWriteGuard<'a, T> {
    fn new(inner: std::cell::RefMut<'a, T>, component_id: runtime::ComponentId) -> Self {
        Self {
            inner,
            component_id,
        }
    }
}

impl<'a, T: Clone + 'static> Deref for SignalWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Clone + 'static> DerefMut for SignalWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T: Clone + 'static> Drop for SignalWriteGuard<'a, T> {
    fn drop(&mut self) {
        runtime::mark_dirty_deferred(self.component_id);
    }
}

pub struct StandaloneSignal<T> {
    signal: Signal<T>,
    component_id: runtime::ComponentId,
}

impl<T: Clone + 'static> StandaloneSignal<T> {
    pub fn get(&self) -> T {
        self.signal.get()
    }

    pub fn set(&self, value: T) {
        self.signal.set(value);
        runtime::request_rerender(Some(self.component_id));
    }

    pub fn inner(&self) -> &Signal<T> {
        &self.signal
    }

    pub fn read(&self) -> T {
        self.get()
    }

    pub fn write(&self) -> SignalWriteGuard<'_, T> {
        SignalWriteGuard::new(self.signal.write(), self.component_id)
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

    #[test]
    fn test_reactive_signal_write_marks_dirty() {
        let signal = use_signal(|| 0);

        // write() should return a guard that marks dirty on drop
        {
            let mut guard = signal.write();
            *guard = 42;
        }

        assert_eq!(signal.get(), 42);
    }

    #[test]
    fn test_standalone_signal_write_marks_dirty() {
        let signal = use_standalone_signal(0);

        {
            let mut guard = signal.write();
            *guard = 99;
        }

        assert_eq!(signal.get(), 99);
    }
}
