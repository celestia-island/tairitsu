use std::{cell::Cell, cell::RefCell, rc::Rc};

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
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                subscribers: Vec::new(),
            })),
        }
    }

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

pub struct EffectHandle {
    stopped: Rc<Cell<bool>>,
}

impl EffectHandle {
    pub fn stop(&self) {
        self.stopped.set(true);
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.get()
    }
}

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
