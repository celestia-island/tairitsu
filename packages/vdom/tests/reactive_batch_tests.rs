use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{batch, create_effect, take_dependencies, Signal};

#[test]
fn test_batch_defers_signal_notifications() {
    let signal = Signal::new(0);
    let called = Rc::new(RefCell::new(false));
    let called_clone = called.clone();

    signal.subscribe(move || {
        *called_clone.borrow_mut() = true;
    });

    batch(|| {
        signal.set(1);
        assert!(!*called.borrow(), "subscriber should NOT fire inside batch");
    });

    assert!(
        *called.borrow(),
        "subscriber should fire after batch flushes"
    );
}

#[test]
fn test_batch_multiple_signals_flush_at_end() {
    let a = Signal::new(0);
    let b = Signal::new(0);
    let counter = Rc::new(RefCell::new(0u32));
    let c1 = counter.clone();
    let c2 = counter.clone();

    a.subscribe(move || {
        *c1.borrow_mut() += 1;
    });
    b.subscribe(move || {
        *c2.borrow_mut() += 1;
    });

    batch(|| {
        a.set(1);
        b.set(2);
        assert_eq!(
            *counter.borrow(),
            0,
            "no subscriber should fire inside batch"
        );
    });

    assert_eq!(
        *counter.borrow(),
        2,
        "both subscribers should fire after batch"
    );
}

#[test]
fn test_nested_batch_flushes_at_outermost() {
    let signal = Signal::new(0);
    let called = Rc::new(RefCell::new(false));
    let c = called.clone();

    signal.subscribe(move || {
        *c.borrow_mut() = true;
    });

    batch(|| {
        signal.set(1);
        assert!(!*called.borrow(), "inner: not yet");

        batch(|| {
            signal.set(2);
            assert!(!*called.borrow(), "nested: should still not fire");
        });

        assert!(!*called.borrow(), "after nested: still not fired");
    });

    assert!(*called.borrow(), "after outermost: should fire");
}

#[test]
fn test_batch_no_signals_does_not_panic() {
    let result = batch(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn test_take_dependencies_returns_tracked_entries() {
    let signal = Signal::new(42);
    take_dependencies();
    let _val = signal.get();
    let deps = take_dependencies();
    assert!(
        !deps.is_empty(),
        "take_dependencies should return entries after signal.get()"
    );
}

#[test]
fn test_take_dependencies_clears() {
    let signal = Signal::new(10);
    take_dependencies();
    let _ = signal.get();
    let first = take_dependencies();
    assert!(!first.is_empty());

    let second = take_dependencies();
    assert!(
        second.is_empty(),
        "second take should be empty after first take"
    );
}

#[test]
fn test_notify_inside_batch_defers() {
    let signal = Signal::new(0);
    let called = Rc::new(RefCell::new(false));
    let c = called.clone();

    signal.subscribe(move || {
        *c.borrow_mut() = true;
    });

    batch(|| {
        signal.notify();
        assert!(!*called.borrow(), "notify should defer inside batch");
    });

    assert!(*called.borrow(), "notify should fire after batch flushes");
}

#[test]
fn test_notify_outside_batch_is_immediate() {
    let signal = Signal::new(0);
    let called = Rc::new(RefCell::new(false));
    let c = called.clone();

    signal.subscribe(move || {
        *c.borrow_mut() = true;
    });

    signal.notify();
    assert!(
        *called.borrow(),
        "notify should fire immediately outside batch"
    );
}

#[test]
fn test_create_effect_stop_prevents_re_run() {
    let signal = Signal::new(0);
    let count = Rc::new(RefCell::new(0u32));
    let c = count.clone();
    let s_clone = signal.clone();

    let handle = create_effect(move || {
        let _ = s_clone.get();
        *c.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 1, "effect runs immediately");

    handle.stop();

    signal.set(1);
    assert_eq!(*count.borrow(), 1, "effect should NOT re-run after stop");
}

#[test]
fn test_effect_handle_is_stopped() {
    let handle = create_effect(|| {});
    assert!(!handle.is_stopped());
    handle.stop();
    assert!(handle.is_stopped());
}

#[test]
fn test_effect_handle_clone_shares_stopped_state() {
    let handle = create_effect(|| {});
    let cloned = handle.clone();
    cloned.stop();
    assert!(handle.is_stopped());
}

#[test]
fn test_batch_preserves_result() {
    let result = batch(|| {
        let s = Signal::new(0);
        s.set(5);
        s.get()
    });
    assert_eq!(result, 5);
}

#[test]
fn test_multiple_effects_on_same_signal() {
    let signal = Signal::new(0);
    let a_count = Rc::new(RefCell::new(0u32));
    let b_count = Rc::new(RefCell::new(0u32));
    let ac = a_count.clone();
    let bc = b_count.clone();

    let s1 = signal.clone();
    let s2 = signal.clone();

    create_effect(move || {
        let _ = s1.get();
        *ac.borrow_mut() += 1;
    });
    create_effect(move || {
        let _ = s2.get();
        *bc.borrow_mut() += 1;
    });

    assert_eq!(*a_count.borrow(), 1);
    assert_eq!(*b_count.borrow(), 1);

    signal.set(42);

    assert_eq!(*a_count.borrow(), 2, "first effect should re-run");
    assert_eq!(*b_count.borrow(), 2, "second effect should re-run");
}
