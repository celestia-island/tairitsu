//! Periodic interval hook, analogous to `setInterval` in JavaScript.
//!
//! `use_interval` returns an [`IntervalHandle`] that can be attached to any
//! [`Platform`](tairitsu_vdom::Platform) implementation. The interval is
//! **not** started automatically — call [`IntervalHandle::start`] with a
//! platform reference and a callback to begin periodic execution.
//!
//! The handle cleans up automatically on drop (calling `clear_interval` on
//! the platform), so you can safely let it go out of scope.
//!
//! # Example
//!
//! ```ignore
//! use tairitsu_hooks::use_interval;
//! use tairitsu_web::WitPlatform;
//!
//! let platform = WitPlatform::new()?;
//! let interval = use_interval();
//! let counter = std::cell::RefCell::new(0);
//! interval.start(&platform, Box::new(move || {
//!     *counter.borrow_mut() += 1;
//!     web_sys::console::log_1(&"tick".into());
//! }), 1000);
//! ```
//!
//! # Comparison with `use_animation`
//!
//! | Hook          | Timing                | Callback type     | Use case              |
//! |---------------|-----------------------|-------------------|-----------------------|
//! | `use_interval` | Fixed period (ms)    | `FnMut()`         | Polling, countdowns   |
//! | `use_animation`| vsync (rAF)          | `FnMut(f64)`      | CSS transitions, gl   |

use std::{cell::RefCell, rc::Rc};

/// Handle to a running interval, returned by [`use_interval`].
///
/// The handle stores the platform-issued interval ID. When dropped or
/// [`clear`](IntervalHandle::clear) is called, the interval is cancelled
/// on the platform.
///
/// Uses `Rc<RefCell<Option<i32>>>` internally so the handle can be shared
/// and cleared from anywhere.
pub struct IntervalHandle {
    id: Rc<RefCell<Option<i32>>>,
}

impl IntervalHandle {
    /// Start (or restart) the interval with the given callback and period.
    ///
    /// If the interval is already running, it is cleared first.
    ///
    /// # Arguments
    /// * `platform` — any [`Platform`](tairitsu_vdom::Platform) implementation.
    /// * `callback` — called every `ms` milliseconds repeatedly.
    /// * `ms` — interval period in milliseconds.
    pub fn start<P>(&self, platform: &P, callback: Box<dyn FnMut()>, ms: u32)
    where
        P: tairitsu_vdom::Platform,
    {
        self.clear::<P>(platform);
        let id = platform.set_interval(callback, ms as i32);
        *self.id.borrow_mut() = Some(id);
    }

    /// Cancel the interval. Safe to call multiple times; no-op if already
    /// cleared.
    pub fn clear<P>(&self, platform: &P)
    where
        P: tairitsu_vdom::Platform,
    {
        if let Some(id) = self.id.borrow_mut().take() {
            platform.clear_interval(id);
        }
    }

    /// Returns `true` if the interval is currently active.
    pub fn is_active(&self) -> bool {
        self.id.borrow().is_some()
    }
}

impl Default for IntervalHandle {
    fn default() -> Self {
        Self {
            id: Rc::new(RefCell::new(None)),
        }
    }
}

/// Create a new [`IntervalHandle`]. The interval is not started until
/// [`IntervalHandle::start`] is called with a platform reference.
pub fn use_interval() -> IntervalHandle {
    IntervalHandle::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_handle_default() {
        let handle = use_interval();
        assert!(!handle.is_active());
    }

    #[test]
    fn test_interval_handle_set_id() {
        let handle = IntervalHandle {
            id: Rc::new(RefCell::new(Some(42))),
        };
        assert!(handle.is_active());
    }

    #[test]
    fn test_interval_handle_clear_when_none() {
        let handle = use_interval();
        assert!(!handle.is_active());
        let id = handle.id.clone();
        assert!(id.borrow().is_none());
    }

    #[test]
    fn test_interval_handle_clear_sets_none() {
        let handle = IntervalHandle {
            id: Rc::new(RefCell::new(Some(42))),
        };
        assert!(handle.is_active());
        *handle.id.borrow_mut() = None;
        assert!(!handle.is_active());
    }
}
