use std::{any::Any, cell::RefCell, rc::Rc};

use tracing::trace;

thread_local! {
    static DEPENDENCIES: RefCell<Vec<Rc<RefCell<dyn Any>>>> = RefCell::new(Vec::new());
    static BATCHING: RefCell<bool> = const { RefCell::new(false) };
    static PENDING_UPDATES: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
}

// Get the memory address of a RefCell for use as a hash key
fn refcell_ptr<T>(refcell: &Rc<RefCell<T>>) -> usize {
    refcell.as_ref() as *const RefCell<T> as usize
}

#[derive(Clone)]
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

// Implement PartialEq by comparing the inner Rc pointer (identity comparison)
impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T> Eq for Signal<T> {}

struct SignalInner<T> {
    value: T,
    subscribers: Vec<Rc<dyn Fn()>>,
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
        // Track this signal access in the current component context
        let signal_ptr = refcell_ptr(&self.inner);

        // Notify the runtime that this signal was accessed
        // This will be used to establish dependencies
        crate::runtime::track_signal(signal_ptr);

        DEPENDENCIES.with(|deps| {
            deps.borrow_mut()
                .push(Rc::clone(&self.inner) as Rc<RefCell<dyn Any>>);
        });

        self.inner.borrow().value.clone()
    }

    pub fn set(&self, value: T) {
        let signal_ptr = refcell_ptr(&self.inner);

        let subscribers = {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.subscribers.clone()
        };

        // Notify runtime that this signal changed
        crate::runtime::notify_signal(signal_ptr);

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

    /// Returns a mutable reference to the value - Dioxus compatibility
    /// Usage: let mut guard = signal.write(); guard.push(item);
    /// Note: Changes made through this guard will NOT automatically trigger subscribers.
    /// Use signal.set() or manually call signal.notify() if reactivity is needed.
    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.value)
    }

    /// Manually trigger all subscribers (for use after write() modifications)
    pub fn notify(&self) {
        let subscribers = self.inner.borrow().subscribers.clone();
        for subscriber in subscribers {
            subscriber();
        }
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
