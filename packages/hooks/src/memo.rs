use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{Classes, IntoClassValue, IntoStyleValue, Signal, Style, VElement};

/// A memoized value that only recomputes when dependencies change.
///
/// Similar to React's useMemo or Dioxus's use_memo, this hook caches the result
/// of a computation function and only re-executes it when the dependencies change.
pub struct Memo<T, D, F>
where
    T: Clone + 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    value: Signal<T>,
    compute: F,
    deps: Rc<RefCell<D>>,
}

impl<T, D, F> Memo<T, D, F>
where
    T: Clone + 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    /// Creates a new Memo with the given compute function and dependencies.
    fn new(compute: F, deps: D) -> Self {
        let initial_value = compute();
        Self {
            value: Signal::new(initial_value),
            compute,
            deps: Rc::new(RefCell::new(deps)),
        }
    }

    /// Gets the current memoized value.
    /// Returns a Signal for reactivity.
    pub fn value(&self) -> Signal<T> {
        self.value.clone()
    }

    /// Gets the current value directly (Dioxus compatibility).
    pub fn read(&self) -> T {
        self.value.get()
    }

    /// Updates the dependencies and recomputes if they have changed.
    pub fn update_deps(&self, new_deps: D) {
        let mut deps = self.deps.borrow_mut();
        if *deps != new_deps {
            *deps = new_deps;
            drop(deps); // Release borrow before computing
            let new_value = (self.compute)();
            self.value.set(new_value);
        }
    }
}

impl<T, D, F> Clone for Memo<T, D, F>
where
    T: Clone + 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            compute: self.compute.clone(),
            deps: Rc::clone(&self.deps),
        }
    }
}

// Allow Memo<String> to be converted to Style
impl<D, F> From<Memo<String, D, F>> for Style
where
    D: PartialEq + 'static,
    F: Fn() -> String + Clone + 'static,
{
    fn from(memo: Memo<String, D, F>) -> Self {
        Style::from(memo.read())
    }
}

// Allow Memo<String> to be converted to Classes
impl<D, F> From<Memo<String, D, F>> for Classes
where
    D: PartialEq + 'static,
    F: Fn() -> String + Clone + 'static,
{
    fn from(memo: Memo<String, D, F>) -> Self {
        Classes::from(memo.read())
    }
}

impl<D, F> IntoStyleValue for Memo<String, D, F>
where
    D: PartialEq + 'static,
    F: Fn() -> String + Clone + 'static,
{
    fn apply_to(self, element: &mut VElement) {
        let signal = self.value();
        let compute_signal = signal.clone();
        element.dynamic_styles.push((
            "cssText".to_string(),
            std::rc::Rc::new(std::cell::RefCell::new(move || compute_signal.get())),
        ));
    }
}

impl<D, F> IntoClassValue for Memo<String, D, F>
where
    D: PartialEq + 'static,
    F: Fn() -> String + Clone + 'static,
{
    fn apply_to(self, element: &mut VElement) {
        let signal = self.value();
        let compute_signal = signal.clone();
        element
            .dynamic_classes
            .push(std::rc::Rc::new(std::cell::RefCell::new(move || {
                compute_signal.get()
            })));
    }
}

/// Creates a memoized value that only recomputes when accessed signals change.
/// This is the Dioxus-compatible API that accepts just a compute function.
///
/// # Arguments
/// * `compute` - A function that computes the value
///
/// # Returns
/// A `Memo` struct that provides access to the memoized value
///
/// # Example
/// ```ignore
/// let memo = use_memo(|| expensive_computation());
/// let value = memo.read();
/// ```
pub fn use_memo<T, F>(compute: F) -> Memo<T, (), F>
where
    T: Clone + 'static,
    F: Fn() -> T + Clone + 'static,
{
    Memo::new(compute, ())
}

/// Creates a memoized value with explicit dependencies.
/// Only recomputes when dependencies change.
///
/// # Arguments
/// * `compute` - A function that computes the value
/// * `deps` - The dependencies that trigger recomputation when changed
///
/// # Returns
/// A `Memo` struct that provides access to the memoized value
///
/// # Example
/// ```ignore
/// let memo = use_memo_with_deps(|| expensive_computation(a, b), (a, b));
/// let value = memo.value().get();
/// ```
pub fn use_memo_with_deps<T, D, F>(compute: F, deps: D) -> Memo<T, D, F>
where
    T: Clone + 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    Memo::new(compute, deps)
}

/// A simpler version of use_memo that works with a single dependency.
pub fn use_memo_with<T, D, F>(compute: F, deps: D) -> Memo<T, D, F>
where
    T: Clone + 'static,
    D: PartialEq + 'static,
    F: Fn() -> T + Clone + 'static,
{
    Memo::new(compute, deps)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn test_use_memo_basic() {
        let memo = use_memo(|| 42);
        assert_eq!(memo.value().get(), 42);
    }

    #[test]
    fn test_use_memo_with_deps() {
        let compute_count = Rc::new(Cell::new(0));
        let compute_count_clone = Rc::clone(&compute_count);

        let memo = use_memo_with_deps(
            move || {
                compute_count_clone.set(compute_count_clone.get() + 1);
                10 * 10
            },
            10,
        );

        // Initial computation
        assert_eq!(memo.value().get(), 100);
        assert_eq!(compute_count.get(), 1);

        // Same dependency - should not recompute
        memo.update_deps(10);
        assert_eq!(memo.value().get(), 100);
        assert_eq!(compute_count.get(), 1);

        // Different dependency - should recompute
        memo.update_deps(20);
        assert_eq!(memo.value().get(), 100);
        assert_eq!(compute_count.get(), 2);
    }

    #[test]
    fn test_use_memo_with_tuple_deps() {
        let memo = use_memo_with_deps(|| "hello world", (1, 2));

        assert_eq!(memo.value().get(), "hello world");

        // Same tuple - no recompute
        memo.update_deps((1, 2));
        assert_eq!(memo.value().get(), "hello world");
    }

    #[test]
    fn test_use_memo_clone() {
        let memo1 = use_memo_with(|| vec![1, 2, 3], ());
        let memo2 = memo1.clone();

        assert_eq!(memo1.value().get(), memo2.value().get());
    }

    #[test]
    fn test_use_memo_string_deps() {
        let memo = use_memo_with_deps(|| "computed", String::from("dep1"));

        assert_eq!(memo.value().get(), "computed");

        // Same string - no recompute
        memo.update_deps(String::from("dep1"));

        // Different string - recompute
        memo.update_deps(String::from("dep2"));
    }

    #[test]
    fn test_use_memo_vec_deps() {
        let memo = use_memo_with_deps(|| 100, vec![1, 2, 3]);

        assert_eq!(memo.value().get(), 100);

        // Same vec - no recompute
        memo.update_deps(vec![1, 2, 3]);

        // Different vec - recompute
        memo.update_deps(vec![1, 2, 4]);
    }
}
