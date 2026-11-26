//! `store!` / `use_store!` must compile and round-trip as their docs describe
//! (`packages/hooks/src/store.rs`, module doc and macro docs).
//!
//! Before the fix `store!` expanded to a `static` and a `fn` with the same
//! name (E0428), and that `static` held a `Store`, which is `!Send`/`!Sync`
//! (E0277) — the documented usage could not compile at all.

#[derive(Clone, PartialEq, Debug)]
struct Counter {
    count: i32,
}

tairitsu_hooks::store!(COUNTER, Counter, Counter { count: 0 });

#[test]
fn store_macro_round_trips() {
    let store = COUNTER();
    assert_eq!(store.get().count, 0);
    store.update(|s| s.count += 2);
    assert_eq!(tairitsu_hooks::use_store!(COUNTER()).count, 2);
}

/// `use_store!` only reads: an update on its own does not notify anyone.
/// Subscribers are told, and unsubscribing works, which proves the store the
/// macro hands out is the one in the registry.
#[test]
fn store_macro_store_is_registered() {
    let observed = std::rc::Rc::new(std::cell::Cell::new(0));
    let store = COUNTER();
    let sink = std::rc::Rc::clone(&observed);
    let handle = store.subscribe(move |_| sink.set(sink.get() + 1));
    assert_eq!(store.subscriber_count(), 1);

    store.update(|_| {});
    assert_eq!(observed.get(), 1);

    handle.unsubscribe();
    assert_eq!(store.subscriber_count(), 0);
    store.update(|_| {});
    assert_eq!(observed.get(), 1, "an unsubscribed callback is not called");
}
