use std::any::Any;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use tracing::trace;

thread_local! {
    static DEPENDENCIES: RefCell<Vec<Rc<RefCell<dyn Any>>>> = RefCell::new(Vec::new());
    static BATCHING: RefCell<bool> = const { RefCell::new(false) };
    static PENDING_UPDATES: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

struct SignalInner<T> {
    value: T,
    subscribers: Vec<Rc<dyn Fn()>>,
}

/// A mutable guard for Signal values (Dioxus compatibility)
pub struct SignalMut<'a, T> {
    guard: std::cell::RefMut<'a, SignalInner<T>>,
}

impl<'a, T> Deref for SignalMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard.value
    }
}

impl<'a, T> DerefMut for SignalMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.value
    }
}

impl<'a, T: Clone + 'static> Drop for SignalMut<'a, T> {
    fn drop(&mut self) {
        // Trigger subscribers when the mutable guard is dropped
        let subscribers = self.guard.subscribers.clone();
        if BATCHING.with(|b| *b.borrow()) {
            trace!("Signal update batched");
        } else {
            for subscriber in subscribers {
                subscriber();
            }
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                subscribers: Vec::new(),
            })),
        }
    }

    pub fn get(&self) -> T {
        DEPENDENCIES.with(|deps| {
            deps.borrow_mut()
                .push(Rc::clone(&self.inner) as Rc<RefCell<dyn Any>>);
        });

        self.inner.borrow().value.clone()
    }

    pub fn set(&self, value: T) {
        let subscribers = {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.subscribers.clone()
        };

        if BATCHING.with(|b| *b.borrow()) {
            trace!("Signal update batched");
        } else {
            for subscriber in subscribers {
                subscriber();
            }
        }
    }

    pub fn subscribe<F: Fn() + 'static>(&self, callback: F) {
        self.inner.borrow_mut().subscribers.push(Rc::new(callback));
    }

    /// Alias for get() - Dioxus compatibility
    pub fn read(&self) -> T {
        self.get()
    }

    /// Returns a mutable guard - Dioxus compatibility
    /// Usage: let mut guard = signal.write(); guard.field = new_value;
    pub fn write(&self) -> SignalMut<T> {
        DEPENDENCIES.with(|deps| {
            deps.borrow_mut()
                .push(Rc::clone(&self.inner) as Rc<RefCell<dyn Any>>);
        });
        SignalMut {
            guard: self.inner.borrow_mut(),
        }
    }

    /// Alias for set() - explicit set with value
    pub fn set_value(&self, value: T) {
        self.set(value)
    }
}

pub struct EffectHandle {
    _cleanup: Box<dyn Fn()>,
}

pub fn create_effect<F>(f: F) -> EffectHandle
where
    F: FnMut() + 'static,
{
    let callback = Rc::new(RefCell::new(f));
    let wrapped = callback.clone();

    DEPENDENCIES.with(|deps| {
        deps.borrow_mut().clear();
    });

    wrapped.borrow_mut()();

    let cleanup = Box::new(move || {
        trace!("Effect cleaned up");
    });

    EffectHandle { _cleanup: cleanup }
}

pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    BATCHING.with(|b| {
        *b.borrow_mut() = true;
    });

    let result = f();

    BATCHING.with(|b| {
        *b.borrow_mut() = false;
    });

    PENDING_UPDATES.with(|updates| {
        let pending: Vec<_> = updates.borrow_mut().drain(..).collect();
        for update in pending {
            update();
        }
    });

    result
}

impl<T: std::fmt::Debug> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("value", &self.inner.borrow().value)
            .finish()
    }
}
