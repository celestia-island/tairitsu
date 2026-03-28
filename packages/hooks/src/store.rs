//! Global state management store.
//!
//! This module provides a simple global state management solution inspired by
//! Redux and Zustand. It allows components to access and update shared state
//! without prop drilling.
//!
//! # Example
//!
//! ```ignore
//! use tairitsu_hooks::{store, use_store, Store};
//!
//! #[derive(Clone, PartialEq)]
//! struct AppState {
//!     count: i32,
//!     user: Option<String>,
//! }
//!
//! // Create a global store
//! store!(APP_STORE, AppState {
//!     count: 0,
//!     user: None,
//! });
//!
//! // In a component
//! fn my_component() {
//!     let state = use_store!(&APP_STORE);
//!     // state.count is now accessible
//! }
//! ```

use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Unique identifier for a store instance
pub type StoreId = usize;

/// Subscriber handle - can be used to unsubscribe
#[derive(Clone)]
pub struct SubscriptionHandle {
    id: usize,
    store_id: StoreId,
}

// Global store registry
thread_local! {
    static STORE_REGISTRY: RefCell<HashMap<StoreId, Box<dyn AnyStore>>> = RefCell::new(HashMap::new());
    static NEXT_STORE_ID: RefCell<StoreId> = const { RefCell::new(1) };
    static NEXT_SUBSCRIBER_ID: RefCell<usize> = const { RefCell::new(1) };
}

/// Trait for type-erased store operations
trait AnyStore {
    fn as_any(&self) -> &dyn std::any::Any;
    fn unsubscribe(&self, subscriber_id: usize);
    #[allow(dead_code)]
    fn subscriber_count(&self) -> usize;
}

/// A subscriber callback
type SubscriberFn<T> = Rc<dyn Fn(T)>;

/// A global store for managing application state
///
/// The store provides:
/// - State access via `RefCell` (single-threaded for WASM)
/// - Subscriber notifications on state changes
/// - Selector support for derived state
#[derive(Clone)]
pub struct Store<T: Clone + 'static> {
    id: StoreId,
    state: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<HashMap<usize, SubscriberFn<T>>>>,
}

impl<T: Clone + 'static> Store<T> {
    /// Create a new store with initial state
    pub fn new(initial: T) -> Self {
        Self::with_id(0, initial)
    }

    /// Create a new store with a specific ID (for internal use)
    fn with_id(id: StoreId, initial: T) -> Self {
        Self {
            id,
            state: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Get the current state
    pub fn get(&self) -> T {
        self.state.borrow().clone()
    }

    /// Update the state with a new value
    ///
    /// This will notify all subscribers after the state is updated.
    pub fn set(&self, value: T) {
        *self.state.borrow_mut() = value;
        self.notify();
    }

    /// Update the state using a mutation function
    ///
    /// This is useful for making partial updates to the state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// store.update(|state| {
    ///     state.count += 1;
    /// });
    /// ```
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut self.state.borrow_mut());
        self.notify();
    }

    /// Subscribe to state changes
    ///
    /// Returns a handle that can be used to unsubscribe.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handle = store.subscribe(|state| {
    ///     println!("State changed: {:?}", state);
    /// });
    ///
    /// // Later, to unsubscribe:
    /// handle.unsubscribe();
    /// ```
    pub fn subscribe<F: Fn(T) + 'static>(&self, callback: F) -> SubscriptionHandle {
        let subscriber_id = NEXT_SUBSCRIBER_ID.with(|n| {
            let id = *n.borrow();
            *n.borrow_mut() = id + 1;
            id
        });

        self.subscribers
            .borrow_mut()
            .insert(subscriber_id, Rc::new(callback));

        SubscriptionHandle {
            id: subscriber_id,
            store_id: self.id,
        }
    }

    /// Subscribe to state changes with a selector
    ///
    /// The selector function extracts a derived value from the state.
    /// The callback is only invoked when the derived value changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// store.subscribe_selector(
    ///     |s| s.count,
    ///     |count| println!("Count is now: {}", count),
    /// );
    /// ```
    pub fn subscribe_selector<
        S: PartialEq + Clone + 'static,
        F: Fn(&T) -> S + 'static,
        C: Fn(S) + 'static,
    >(
        &self,
        selector: F,
        callback: C,
    ) -> SubscriptionHandle {
        let last_value = Rc::new(RefCell::new(None::<S>));
        let selector = Rc::new(selector);

        self.subscribe(move |current_state| {
            let new_value = selector(&current_state);
            let mut last = last_value.borrow_mut();
            if last.as_ref() != Some(&new_value) {
                *last = Some(new_value.clone());
                drop(last);
                callback(new_value);
            }
        })
    }

    /// Unsubscribe a specific subscriber
    fn unsubscribe(&self, subscriber_id: usize) {
        self.subscribers.borrow_mut().remove(&subscriber_id);
    }

    /// Notify all subscribers of a state change
    fn notify(&self) {
        let state = self.get();
        let subscribers = self.subscribers.borrow();

        for subscriber in subscribers.values() {
            subscriber(state.clone());
        }
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.borrow().len()
    }
}

impl<T: Clone + 'static> AnyStore for Store<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn unsubscribe(&self, subscriber_id: usize) {
        self.unsubscribe(subscriber_id);
    }

    fn subscriber_count(&self) -> usize {
        self.subscriber_count()
    }
}

impl SubscriptionHandle {
    /// Unsubscribe from the store
    pub fn unsubscribe(self) {
        STORE_REGISTRY.with(|registry| {
            if let Some(store) = registry.borrow().get(&self.store_id) {
                store.unsubscribe(self.id);
            }
        });
    }
}

/// Register a store in the global registry
///
/// # Example
///
/// ```ignore
/// static MY_STORE: OnceLock<Store<MyState>> = OnceLock::new();
///
/// fn get_store() -> Store<MyState> {
///     MY_STORE.get_or_init(|| {
///         register_store(Store::new(MyState::default()))
///     }).clone()
/// }
/// ```
pub fn register_store<T: Clone + 'static>(store: Store<T>) -> Store<T> {
    let id = NEXT_STORE_ID.with(|n| {
        let id = *n.borrow();
        *n.borrow_mut() = id + 1;
        id
    });

    let store = Store::with_id(id, store.get());

    STORE_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(id, Box::new(store.clone()));
    });

    store
}

/// Get a store by ID from the global registry
pub fn get_store<T: Clone + 'static>(id: StoreId) -> Option<Store<T>> {
    STORE_REGISTRY.with(|registry| {
        registry
            .borrow()
            .get(&id)?
            .as_any()
            .downcast_ref::<Store<T>>()
            .cloned()
    })
}

/// Macro to create and register a global store
///
/// # Example
///
/// ```ignore
/// use tairitsu_hooks::store;
///
/// store!(COUNTER_STORE, i32, 0);
/// ```
#[macro_export]
macro_rules! store {
    ($name:ident, $ty:ty, $initial:expr) => {
        static $name: std::sync::OnceLock<$crate::Store<$ty>> = std::sync::OnceLock::new();

        #[allow(dead_code)]
        fn $name() -> $crate::Store<$ty> {
            $name
                .get_or_init(|| $crate::register_store($crate::Store::new($initial)))
                .clone()
        }
    };
}

/// Macro to access a store in a component
///
/// # Example
///
/// ```ignore
/// use tairitsu_hooks::use_store;
///
/// let state = use_store!(COUNTER_STORE());
/// ```
#[macro_export]
macro_rules! use_store {
    ($store:expr) => {{ $store.get() }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct TestState {
        count: i32,
        name: String,
    }

    #[test]
    fn test_store_creation() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        assert_eq!(store.get().count, 0);
        assert_eq!(store.get().name, "test");
    }

    #[test]
    fn test_store_update() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        store.update(|state| {
            state.count = 42;
        });

        assert_eq!(store.get().count, 42);
    }

    #[test]
    fn test_store_set() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        store.set(TestState {
            count: 100,
            name: "updated".to_string(),
        });

        assert_eq!(store.get().count, 100);
        assert_eq!(store.get().name, "updated");
    }

    #[test]
    fn test_store_subscribe() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        let called = Rc::new(RefCell::new(false));
        let called_clone = Rc::clone(&called);

        store.subscribe(move |_| {
            *called_clone.borrow_mut() = true;
        });

        store.set(TestState {
            count: 1,
            name: "test".to_string(),
        });

        assert!(*called.borrow());
    }

    #[test]
    fn test_store_subscribe_selector() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        let results = Rc::new(RefCell::new(Vec::new()));
        let results_clone = Rc::clone(&results);

        store.subscribe_selector(
            |s| s.count,
            move |count| {
                results_clone.borrow_mut().push(count);
            },
        );

        // First update - should trigger
        store.update(|s| {
            s.count = 1;
        });

        // Second update with same count - should not trigger
        store.update(|s| {
            s.name = "updated".to_string();
        });

        // Third update - should trigger
        store.update(|s| {
            s.count = 2;
        });

        assert_eq!(*results.borrow(), vec![1, 2]);
    }

    #[test]
    fn test_store_unsubscribe() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        let called = Rc::new(RefCell::new(0));
        let called_clone = Rc::clone(&called);

        let handle = store.subscribe(move |_| {
            *called_clone.borrow_mut() += 1;
        });

        store.set(TestState {
            count: 1,
            name: "test".to_string(),
        });
        assert_eq!(*called.borrow(), 1);

        // Unsubscribe directly (since we're not in the registry)
        store.unsubscribe(handle.id);

        store.set(TestState {
            count: 2,
            name: "test".to_string(),
        });
        // Should still be 1 since we unsubscribed
        assert_eq!(*called.borrow(), 1);
    }

    #[test]
    fn test_store_registry() {
        let store = Store::new(42i32);
        let registered = register_store(store);

        assert_eq!(registered.get(), 42);

        let retrieved = get_store::<i32>(registered.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get(), 42);
    }

    #[test]
    fn test_subscriber_count() {
        let store = Store::new(42i32);
        assert_eq!(store.subscriber_count(), 0);

        let _sub1 = store.subscribe(|_| {});
        assert_eq!(store.subscriber_count(), 1);

        let _sub2 = store.subscribe(|_| {});
        assert_eq!(store.subscriber_count(), 2);
    }
}
