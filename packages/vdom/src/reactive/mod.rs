use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::trace;

type SubscribeFn = Box<dyn Fn(Rc<dyn Fn()>)>;

pub struct DependencyEntry {
    subscribe: SubscribeFn,
}

thread_local! {
    static DEPENDENCIES: RefCell<Vec<DependencyEntry>> = const { RefCell::new(Vec::new()) };
    static BATCH_DEPTH: RefCell<i32> = const { RefCell::new(0) };
    static PENDING_UPDATES: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

static NEXT_SIGNAL_ID: AtomicUsize = AtomicUsize::new(1);

pub type SignalId = usize;

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
    signal_id: SignalId,
    value: T,
    subscribers: Vec<Rc<dyn Fn()>>,
}

impl<T> Drop for SignalInner<T> {
    fn drop(&mut self) {
        crate::runtime::unregister_signal(self.signal_id);
    }
}

impl<T: Clone + 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    ///
    /// ```no_run
    /// let name = tairitsu_vdom::Signal::new("Alice".to_string());
    /// ```
    /// Return the unique identifier for this signal.
    pub fn id(&self) -> SignalId {
        self.inner.borrow().signal_id
    }

    pub fn new(value: T) -> Self {
        let id = NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                signal_id: id,
                value,
                subscribers: Vec::new(),
            })),
        }
    }

    /// Read the current value. If called inside [`create_effect`], this signal
    /// is automatically tracked as a dependency.
    pub fn get(&self) -> T {
        let id = self.inner.borrow().signal_id;
        crate::runtime::track_signal(id);

        let signal = self.clone();
        DEPENDENCIES.with(|deps| {
            let borrowed = deps.borrow();
            let tracking = TRACKING_ACTIVE.with(|t| t.get());
            drop(borrowed);
            if tracking {
                deps.borrow_mut().push(DependencyEntry {
                    subscribe: Box::new(move |cb: Rc<dyn Fn()>| {
                        signal.inner.borrow_mut().subscribers.push(cb);
                    }),
                });
            }
        });

        self.inner.borrow().value.clone()
    }

    /// Write a new value and notify all subscribers. If not inside a [`batch`],
    /// subscribers are called synchronously.
    pub fn set(&self, value: T) {
        let id = self.inner.borrow().signal_id;

        let subscribers = {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.subscribers.clone()
        };

        crate::runtime::notify_signal(id);

        let batched = BATCH_DEPTH.with(|d| *d.borrow() > 0);
        if batched {
            trace!(
                "Signal update batched (depth={})",
                BATCH_DEPTH.with(|d| *d.borrow())
            );
            PENDING_UPDATES.with(|updates| {
                for subscriber in subscribers {
                    updates.borrow_mut().push(Box::new(move || subscriber()));
                }
            });
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

    /// Mutate the value through a closure and automatically notify subscribers.
    ///
    /// This is the safe alternative to [`write`] which returns a raw `RefMut`
    /// requiring a manual [`notify`] call.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tairitsu_vdom::Signal;
    ///
    /// let count = Signal::new(0);
    /// count.update(|n| *n += 1);
    /// assert_eq!(count.get(), 1);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.inner.borrow_mut().value);
        self.notify();
    }

    /// Return a mutable reference to the inner value.
    ///
    /// **Note:** mutating through the returned `RefMut` does NOT automatically
    /// notify subscribers. Call [`notify`] afterwards, or use [`update`] instead.
    pub fn write(&self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.value)
    }

    pub fn notify(&self) {
        let _id = self.inner.borrow().signal_id;
        let subscribers = self.inner.borrow().subscribers.clone();
        let batched = BATCH_DEPTH.with(|d| *d.borrow() > 0);
        if batched {
            PENDING_UPDATES.with(|updates| {
                for subscriber in subscribers {
                    updates.borrow_mut().push(Box::new(move || subscriber()));
                }
            });
        } else {
            for subscriber in subscribers {
                subscriber();
            }
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

    let _prev_deps: Vec<DependencyEntry> =
        DEPENDENCIES.with(|deps| deps.borrow_mut().drain(..).collect());

    TRACKING_ACTIVE.with(|t| t.set(true));
    callback.borrow_mut()();
    TRACKING_ACTIVE.with(|t| t.set(false));

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

/// Drain all tracked dependency entries from the current context.
///
/// This is useful for testing and for manually managing dependency tracking.
pub fn take_dependencies() -> Vec<DependencyEntry> {
    let result = DEPENDENCIES.with(|deps| deps.borrow_mut().drain(..).collect());
    TRACKING_ACTIVE.with(|t| t.set(true));
    result
}

/// Stop tracking signal dependencies. Further `signal.get()` calls will
/// not record dependencies until `take_dependencies()` or `create_effect`
/// re-enables tracking.
pub fn stop_tracking() {
    TRACKING_ACTIVE.with(|t| t.set(false));
    DEPENDENCIES.with(|deps| deps.borrow_mut().clear());
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
    BATCH_DEPTH.with(|d| {
        *d.borrow_mut() += 1;
    });

    let result = f();

    let should_flush = BATCH_DEPTH.with(|d| {
        *d.borrow_mut() -= 1;
        *d.borrow() == 0
    });

    if should_flush {
        PENDING_UPDATES.with(|updates| {
            let pending: Vec<_> = updates.borrow_mut().drain(..).collect();
            for update in pending {
                update();
            }
        });
    }

    result
}

impl<T: std::fmt::Debug> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("value", &self.inner.borrow().value)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_new_assigns_unique_ids() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn test_signal_get_set() {
        let s = Signal::new(42i32);
        assert_eq!(s.get(), 42);
        s.set(100);
        assert_eq!(s.get(), 100);
    }

    #[test]
    fn test_signal_effect_tracks_dependency() {
        let s = Signal::new(1i32);
        let observed = Rc::new(RefCell::new(0i32));

        let effect_observed = observed.clone();
        let effect_s = s.clone();
        let _handle = create_effect(move || {
            *effect_observed.borrow_mut() = effect_s.get();
        });

        assert_eq!(*observed.borrow(), 1);

        s.set(5);
        assert_eq!(*observed.borrow(), 5);
    }

    #[test]
    fn test_signal_effect_multiple_dependencies() {
        let a = Signal::new(1i32);
        let b = Signal::new(10i32);
        let sum = Rc::new(RefCell::new(0i32));

        let effect_sum = sum.clone();
        let effect_a = a.clone();
        let effect_b = b.clone();
        let _handle = create_effect(move || {
            *effect_sum.borrow_mut() = effect_a.get() + effect_b.get();
        });

        assert_eq!(*sum.borrow(), 11);

        a.set(5);
        assert_eq!(*sum.borrow(), 15);

        b.set(20);
        assert_eq!(*sum.borrow(), 25);
    }

    #[test]
    fn test_signal_drop_cleans_up_runtime() {
        let signal_id;
        {
            let s = Signal::new(99i32);
            signal_id = s.id();

            // Track the signal to register it in the runtime
            crate::runtime::track_signal(signal_id);
            crate::runtime::with_component(
                crate::runtime::use_component(|| crate::VNode::Text(crate::vnode::VText::new(""))),
                || {
                    crate::runtime::track_signal(signal_id);
                },
            );

            // Verify it's tracked
            let tracked = crate::runtime::signal_is_tracked(signal_id);
            assert!(tracked, "Signal should be tracked before drop");
        }
        // Signal dropped here — dependencies should be cleaned up

        let tracked = crate::runtime::signal_is_tracked(signal_id);
        assert!(
            !tracked,
            "Signal dependencies should be cleaned up after drop"
        );
    }

    #[test]
    fn test_signal_write_bypasses_notify() {
        let s = Signal::new(0i32);
        let observed = Rc::new(RefCell::new(0i32));

        let effect_observed = observed.clone();
        let effect_s = s.clone();
        let _handle = create_effect(move || {
            *effect_observed.borrow_mut() = effect_s.get();
        });

        // Effect should have run once during creation
        assert_eq!(*observed.borrow(), 0);

        // write() returns RefMut that can modify value without notification
        {
            let mut guard = s.write();
            *guard = 42;
        }

        // Without explicit notify(), the effect should NOT have re-run
        // because write() bypasses the subscriber notification path
        assert_eq!(
            *observed.borrow(),
            0,
            "write() without notify() should not trigger effect"
        );

        // Now call notify(), which triggers subscribers directly
        s.notify();
        assert_eq!(
            *observed.borrow(),
            42,
            "After notify(), effect should re-run"
        );
    }

    #[test]
    fn test_signal_batch_deferred_updates() {
        let s = Signal::new(0i32);
        let observed = Rc::new(RefCell::new(0i32));

        let effect_observed = observed.clone();
        let effect_s = s.clone();
        let _handle = create_effect(move || {
            *effect_observed.borrow_mut() = effect_s.get();
        });

        batch(|| {
            s.set(1);
            s.set(2);
            s.set(3);
            // Effect should NOT have re-run yet
            assert_eq!(*observed.borrow(), 0);
        });

        // After batch, effect should re-run once with the final value
        assert_eq!(*observed.borrow(), 3);
    }

    #[test]
    fn test_signal_clone_shares_state() {
        let a = Signal::new("hello".to_string());
        let b = a.clone();

        assert_eq!(a.get(), "hello");
        assert_eq!(b.get(), "hello");

        a.set("world".to_string());
        assert_eq!(b.get(), "world");
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn test_signal_update_notifies() {
        let s = Signal::new(0i32);
        let observed = Rc::new(RefCell::new(0i32));
        let effect_observed = observed.clone();
        let effect_s = s.clone();
        let _handle = create_effect(move || {
            *effect_observed.borrow_mut() = effect_s.get();
        });

        s.update(|n| *n = 42);
        assert_eq!(*observed.borrow(), 42, "update() should trigger effect");
    }

    #[test]
    fn test_signal_subscribe_direct() {
        let s = Signal::new(0i32);
        let called = Rc::new(RefCell::new(0i32));

        let cb_called = called.clone();
        s.subscribe(move || {
            *cb_called.borrow_mut() += 1;
        });

        s.set(1);
        assert_eq!(*called.borrow(), 1);

        s.set(2);
        assert_eq!(*called.borrow(), 2);
    }

    #[test]
    fn test_multiple_effects_same_signal() {
        let s = Signal::new(0i32);
        let a = Rc::new(RefCell::new(0i32));
        let b = Rc::new(RefCell::new(0i32));

        let ca = a.clone();
        let cs = s.clone();
        let _h1 = create_effect(move || {
            *ca.borrow_mut() = cs.get();
        });
        let cb = b.clone();
        let cs2 = s.clone();
        let _h2 = create_effect(move || {
            *cb.borrow_mut() = cs2.get();
        });

        assert_eq!(*a.borrow(), 0);
        assert_eq!(*b.borrow(), 0);

        s.set(42);
        assert_eq!(*a.borrow(), 42, "first effect should re-run");
        assert_eq!(*b.borrow(), 42, "second effect should re-run");
    }

    #[test]
    fn test_effect_no_dependencies() {
        let run_count = Rc::new(RefCell::new(0u32));
        let rc = run_count.clone();
        let _handle = create_effect(move || {
            // No signal.get() call — no dependencies
            *rc.borrow_mut() += 1;
        });
        assert_eq!(*run_count.borrow(), 1, "effect should run once on creation");
    }

    #[test]
    fn test_effect_reattaches_dependencies() {
        // An effect that conditionally reads a signal should re-track dependencies
        // each time it runs.
        let toggle = Signal::new(false);
        let a = Signal::new(1i32);
        let b = Signal::new(10i32);
        let last = Rc::new(RefCell::new(0i32));

        let l = last.clone();
        let t = toggle.clone();
        let sa = a.clone();
        let sb = b.clone();
        let _handle = create_effect(move || {
            if t.get() {
                *l.borrow_mut() = sa.get(); // depends on 'a'
            } else {
                *l.borrow_mut() = sb.get(); // depends on 'b'
            }
        });

        assert_eq!(*last.borrow(), 10); // reads b

        b.set(20);
        assert_eq!(*last.borrow(), 20); // still reads b

        toggle.set(true); // now reads a
        assert_eq!(*last.borrow(), 1); // reads a

        a.set(42);
        assert_eq!(*last.borrow(), 42); // only a triggers (b no longer tracked)
    }

    #[test]
    fn test_signal_notify_inside_effect() {
        // Test that notify() during effect execution doesn't cause double re-run
        let s = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0u32));

        let c = count.clone();
        let cs = s.clone();
        let _handle = create_effect(move || {
            let _ = cs.get();
            *c.borrow_mut() += 1;
        });

        assert_eq!(*count.borrow(), 1);

        s.set(1);
        assert_eq!(*count.borrow(), 2, "effect should re-run once per set");
    }
}
