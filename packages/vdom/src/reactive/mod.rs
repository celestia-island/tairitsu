use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tracing::trace;

type SubscribeFn = Box<dyn Fn(Rc<dyn Fn()>)>;

pub struct DependencyEntry {
    subscribe: SubscribeFn,
}

thread_local! {
    static DEPENDENCIES: RefCell<Vec<DependencyEntry>> = const { RefCell::new(Vec::new()) };
    static BATCHING: RefCell<bool> = const { RefCell::new(false) };
    static PENDING_UPDATES: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
}

fn refcell_ptr<T>(refcell: &Rc<RefCell<T>>) -> usize {
    refcell.as_ref() as *const RefCell<T> as usize
}

/// A reactive value container that tracks reads and notifies subscribers on writes.
///
/// Signals are the foundation of Tairitsu's reactivity system. When `.get()` is
/// called inside a [`create_effect`] closure, the signal is automatically tracked
/// as a dependency. When `.set()` is called later, the effect re-runs.
///
/// # Example
///
/// ```no_run
/// use tairitsu_vdom::Signal;
///
/// let count = Signal::new(0);
/// assert_eq!(count.get(), 0);
/// count.set(1);
/// assert_eq!(count.get(), 1);
/// ```
#[derive(Clone)]
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

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
    /// Create a new signal with the given initial value.
    ///
    /// ```no_run
    /// let name = tairitsu_vdom::Signal::new("Alice".to_string());
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                subscribers: Vec::new(),
            })),
        }
    }

    /// Read the current value. If called inside [`create_effect`], this signal
    /// is automatically tracked as a dependency.
    pub fn get(&self) -> T {
        let signal_ptr = refcell_ptr(&self.inner);
        crate::runtime::track_signal(signal_ptr);

        let signal = self.clone();
        DEPENDENCIES.with(|deps| {
            deps.borrow_mut().push(DependencyEntry {
                subscribe: Box::new(move |cb: Rc<dyn Fn()>| {
                    signal.inner.borrow_mut().subscribers.push(cb);
                }),
            });
        });

        self.inner.borrow().value.clone()
    }

    /// Write a new value and notify all subscribers. If not inside a [`batch`],
    /// subscribers are called synchronously.
    pub fn set(&self, value: T) {
        let signal_ptr = refcell_ptr(&self.inner);

        let subscribers = {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.subscribers.clone()
        };

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

    pub fn read(&self) -> T {
        self.get()
    }

    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.value)
    }

    pub fn notify(&self) {
        let subscribers = self.inner.borrow().subscribers.clone();
        for subscriber in subscribers {
            subscriber();
        }
    }
}

/// A handle to a reactive effect created by [`create_effect`].
///
/// Drop this handle to allow the effect to be cleaned up, or call [`stop()`](EffectHandle::stop)
/// to deactivate the effect without dropping it.
pub struct EffectHandle {
    stopped: Rc<Cell<bool>>,
}

impl Clone for EffectHandle {
    fn clone(&self) -> Self {
        Self {
            stopped: self.stopped.clone(),
        }
    }
}

impl EffectHandle {
    /// Stop the effect. It will no longer re-run when tracked signals change.
    pub fn stop(&self) {
        self.stopped.set(true);
    }

    /// Returns `true` if the effect has been stopped.
    pub fn is_stopped(&self) -> bool {
        self.stopped.get()
    }
}

/// Create a reactive effect that auto-tracks signal dependencies.
///
/// The closure runs immediately. Any [`Signal::get()`] calls inside the closure
/// register the signal as a dependency. When a dependency changes, the closure
/// re-runs and dependencies are re-tracked.
///
/// # Example
///
/// ```no_run
/// use tairitsu_vdom::{Signal, create_effect};
///
/// let count = Signal::new(0);
/// let count_clone = count.clone();
///
/// create_effect(move || {
///     println!("count = {}", count_clone.get());
/// });
///
/// count.set(1); // prints "count = 1"
/// ```
pub fn create_effect<F>(f: F) -> EffectHandle
where
    F: FnMut() + 'static,
{
    let callback: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new(f));
    let stopped = Rc::new(Cell::new(false));
    let generation = Rc::new(Cell::new(0u64));

    execute_effect(&callback, &stopped, &generation);

    EffectHandle { stopped }
}

fn execute_effect(
    callback: &Rc<RefCell<dyn FnMut()>>,
    stopped: &Rc<Cell<bool>>,
    generation: &Rc<Cell<u64>>,
) {
    if stopped.get() {
        return;
    }

    let gen = generation.get();
    generation.set(gen + 1);
    let my_gen = generation.get();

    DEPENDENCIES.with(|deps| deps.borrow_mut().clear());

    callback.borrow_mut()();

    let deps: Vec<DependencyEntry> =
        DEPENDENCIES.with(|deps| deps.borrow_mut().drain(..).collect());

    if deps.is_empty() {
        return;
    }

    let cb = callback.clone();
    let stopped_clone = stopped.clone();
    let gen_clone = generation.clone();
    let rerun: Rc<dyn Fn()> = Rc::new(move || {
        if stopped_clone.get() {
            return;
        }
        if gen_clone.get() != my_gen {
            return;
        }
        execute_effect(&cb, &stopped_clone, &gen_clone);
    });

    for dep in deps {
        (dep.subscribe)(rerun.clone());
    }
}

pub fn drain_dependencies() -> Vec<DependencyEntry> {
    DEPENDENCIES.with(|deps| {
        deps.borrow_mut().clear();
    });

    // Run nothing — caller should run their closure first, then call this
    DEPENDENCIES.with(|deps| deps.borrow_mut().drain(..).collect())
}

pub fn clear_dependencies() {
    DEPENDENCIES.with(|deps| deps.borrow_mut().clear());
}

pub fn take_dependencies() -> Vec<DependencyEntry> {
    DEPENDENCIES.with(|deps| deps.borrow_mut().drain(..).collect())
}

/// Batch multiple signal writes into a single update. Subscribers are deferred
/// until the closure returns, then all pending updates are flushed at once.
///
/// # Example
///
/// ```no_run
/// use tairitsu_vdom::{Signal, batch};
///
/// let a = Signal::new(1);
/// let b = Signal::new(2);
///
/// batch(|| {
///     a.set(10);
///     b.set(20);
///     // subscribers not yet called
/// });
/// // subscribers called once now
/// ```
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
