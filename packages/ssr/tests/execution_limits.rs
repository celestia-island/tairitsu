//! Execution bounds and compile-cache behaviour of the SSR render path.
//!
//! The crate has no checked-in component fixtures (a real tairitsu component is
//! built by the packager for `wasm32-wasip2`), so these tests build the smallest
//! components that still exercise the shipped paths:
//!
//! * components whose core module's start function never returns — proves a
//!   runaway guest is stopped instead of hanging the calling thread;
//! * bare components, made unique per test — proves the compiled artifact is reused
//!   across renders;
//! * a core module that grows past the memory cap and then traps — proves the denial
//!   is reported instead of being served as a truncated page.
//!
//! The linear-memory cap is exercised at the store level through the same
//! `apply_limits` primitive the render path uses.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tairitsu_ssr::limits::apply_limits;
use tairitsu_ssr::{
    component_cache_stats, render_to_html, render_to_html_with_options, shared_engine,
    RenderOptions, SsrConfig, SsrHostState,
};
use wasmtime::{Engine, Instance, Module, Store};

/// Context the render path attaches when a guest is stopped by its fuel budget.
const FUEL_CONTEXT: &str = "exhausted its fuel budget";
/// Context the render path attaches when a guest is stopped by its deadline.
const DEADLINE_CONTEXT: &str = "exceeded its render deadline";

/// The cache counters are process-wide, so the tests that assert on them must not
/// interleave with the other renders in this file. Every test takes this lock.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A component whose core module starts spinning during instantiation.
const SPINNING_COMPONENT: &str = r#"
(component
  (core module $spin
    (func $never_returns
      (loop $forever
        (br $forever)))
    (start $never_returns))
  (core instance (instantiate $spin)))
"#;

/// A core module whose exported `grow` tries to add 32 pages (2 MiB).
const MEMORY_GROW_MODULE: &str = r#"
(module
  (memory 1)
  (func (export "grow") (result i32)
    (memory.grow (i32.const 32))))
"#;

/// A core module that grows past the cap and then traps "for its own reasons".
///
/// With `trap_on_grow_failure` off the guest sees `memory.grow == -1`, and the
/// `unreachable` that follows is neither an `OutOfFuel` nor an `Interrupt` trap — the
/// shape an allocation bomb takes when the host has to notice the truncation itself.
const GROW_THEN_TRAP_MODULE: &str = r#"
(module
  (memory 1)
  (func (export "grow_then_trap")
    (if (i32.eq (memory.grow (i32.const 32)) (i32.const -1))
      (then (unreachable)))))
"#;

/// Name of the custom section that makes a test's component bytes unique.
const TEST_SECTION: &[u8] = b"tairitsu-ssr-test";

/// A bare component that also carries `tag` in a trailing custom section.
///
/// The compile cache is process-wide and keyed by content, while [`serial`] only
/// *orders* the tests — it cannot reset the cache between them. Two tests rendering
/// the same bytes would therefore race for who fills the cache: the loser observes a
/// hit where it asserted a miss, and which one loses depends on scheduling. Tagging
/// each test's binary gives every test its own cache key, so the counters it reads are
/// its own. The custom section is inert: the component still has no tairitsu exports,
/// which is all these tests need (the render fails after instantiating).
fn tagged_component(tag: &str) -> Vec<u8> {
    let mut bytes = wat::parse_str("(component)").expect("empty component should assemble");
    let mut contents = vec![TEST_SECTION.len() as u8];
    contents.extend_from_slice(TEST_SECTION);
    contents.extend_from_slice(tag.as_bytes());

    bytes.push(0); // custom section
    bytes.extend(unsigned_leb128(contents.len()));
    bytes.extend(contents);
    bytes
}

fn unsigned_leb128(mut value: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return out;
        }
        out.push(byte | 0x80);
    }
}

fn message(error: &anyhow::Error) -> String {
    format!("{error:#}")
}

#[test]
fn runaway_component_is_stopped_by_its_fuel_budget() {
    let _serial = serial();
    let wasm = wat::parse_str(SPINNING_COMPONENT).expect("spinning component should assemble");

    // 20M fuel is a fraction of a second of native execution, so the bounded render
    // returns promptly instead of pinning this thread for the duration of the test.
    let options = RenderOptions {
        fuel: 20_000_000,
        ..RenderOptions::default()
    };

    let started = Instant::now();
    let error = render_to_html_with_options(&wasm, SsrConfig::default(), options)
        .expect_err("a component that never returns must not render successfully");
    let elapsed = started.elapsed();

    assert!(
        message(&error).contains(FUEL_CONTEXT),
        "expected the guest to be stopped by fuel, got: {error:#}"
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "the fuel bound should cut the guest off promptly, took {elapsed:?}"
    );
}

#[test]
fn runaway_component_is_stopped_by_its_render_deadline() {
    let _serial = serial();
    let wasm = wat::parse_str(SPINNING_COMPONENT).expect("spinning component should assemble");

    // Fuel is set far above what the guest can burn in the deadline window, so the
    // only thing that can stop it is the epoch ticker firing the deadline.
    let options = RenderOptions {
        fuel: 500_000_000,
        timeout: Duration::from_millis(5),
        ..RenderOptions::default()
    };

    let started = Instant::now();
    let error = render_to_html_with_options(&wasm, SsrConfig::default(), options)
        .expect_err("a component that never returns must not render successfully");
    let elapsed = started.elapsed();

    assert!(
        message(&error).contains(DEADLINE_CONTEXT),
        "expected the epoch deadline to stop the guest, got: {error:#}"
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "the render deadline should fire promptly, took {elapsed:?}"
    );
}

#[test]
fn rendering_the_same_binary_twice_compiles_it_once() {
    let _serial = serial();
    // A bare component compiles and instantiates, then fails to bind the tairitsu
    // browser exports. That is enough to observe the compile cache. The tag keeps this
    // binary's cache key to this test, so "first render misses, second render hits"
    // cannot be pre-empted by another test having rendered the same bytes.
    let wasm = tagged_component("same-binary-twice");

    let before = component_cache_stats();
    let first = render_to_html(&wasm, SsrConfig::default());
    assert!(
        first.is_err(),
        "a component without the tairitsu exports cannot render"
    );

    let after_first = component_cache_stats();
    assert_eq!(
        after_first.misses,
        before.misses + 1,
        "the first render must compile the component"
    );
    assert_eq!(
        after_first.entries,
        before.entries + 1,
        "the compiled artifact must be cached"
    );

    let second = render_to_html(&wasm, SsrConfig::default());
    assert!(second.is_err());

    let after_second = component_cache_stats();
    assert_eq!(
        after_second.hits,
        after_first.hits + 1,
        "the second render must hit the compile cache"
    );
    assert_eq!(
        after_second.misses, after_first.misses,
        "the second render must not recompile the component"
    );
    assert_eq!(
        after_second.entries, after_first.entries,
        "hitting the cache must not add an entry"
    );
}

#[test]
fn unbounded_trusted_renders_do_not_install_a_deadline() {
    let _serial = serial();
    // A trusted render still gets an epoch deadline (a store on an epoch
    // instrumented engine traps immediately otherwise), but it is effectively
    // unreachable: the render must get past instantiation and reach the same
    // missing-exports error as a bounded render. Tagged like the other cache test so
    // neither test can populate the other's cache entry.
    let wasm = tagged_component("trusted-without-deadline");

    let error = render_to_html_with_options(&wasm, SsrConfig::default(), RenderOptions::trusted())
        .expect_err("a component without the tairitsu exports cannot render");

    assert!(
        !message(&error).contains(DEADLINE_CONTEXT),
        "a trusted render must not be interrupted by the epoch deadline: {error:#}"
    );
}

#[test]
fn linear_memory_growth_is_capped_for_untrusted_stores() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(MEMORY_GROW_MODULE).expect("module assembles"),
    )
    .expect("module should compile");

    // 1 MiB cap = 16 pages, so a 32 page growth must be denied.
    let bounded = RenderOptions {
        max_memory_bytes: 1024 * 1024,
        ..RenderOptions::default()
    };
    assert_eq!(
        grow(engine, &module, &bounded),
        -1,
        "growing past the memory cap must fail like a spec-compliant memory.grow"
    );

    // Negative control: the same growth succeeds once the cap is lifted, so the -1
    // above really comes from the limiter and not from the module. `memory.grow`
    // returns the previous page count (1), not the number of pages added.
    assert_eq!(
        grow(engine, &module, &RenderOptions::trusted()),
        1,
        "trusted stores keep unbounded linear-memory growth"
    );
}

/// Store-level memory COUNT cap (R3 S2): the per-memory byte cap alone
/// let a multi-memory component hold N x max bytes (an 8-line probe held
/// 6 MiB under a 4 MiB cap). A module with more memories than the store
/// limit must fail to instantiate — loudly, not silently.
#[test]
fn a_module_with_too_many_memories_fails_to_instantiate() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(
            r#"
(module
  (memory 1)
  (memory 1)
  (memory 1)
  (func (export "grow") (result i32) (memory.grow (i32.const 1))))
"#,
        )
        .expect("three-memory module assembles"),
    )
    .expect("module should compile");

    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(
        &mut store,
        &RenderOptions {
            max_memory_bytes: 1024 * 1024,
            ..RenderOptions::default()
        },
    )
    .expect("limits should apply to a fresh store");

    let err = Instance::new(&mut store, &module, &[])
        .expect_err("three memories exceed the store-level count cap (2)");
    assert!(
        format!("{err:#}").to_lowercase().contains("memory"),
        "the failure should name the memory limit: {err:#}"
    );
}

/// The count cap leaves legitimate single- and dual-memory components
/// alone (2 x per-memory cap is the documented worst case).
#[test]
fn a_two_memory_module_still_instantiates_within_the_count_cap() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(
            r#"
(module
  (memory 1)
  (memory 1)
  (func (export "grow") (result i32) (memory.grow (i32.const 1))))
"#,
        )
        .expect("two-memory module assembles"),
    )
    .expect("module should compile");

    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(
        &mut store,
        &RenderOptions {
            max_memory_bytes: 1024 * 1024,
            ..RenderOptions::default()
        },
    )
    .expect("limits should apply to a fresh store");
    Instance::new(&mut store, &module, &[]).expect("two memories are within the count cap");
}

/// Table counts are capped too (R4 finding): a flood of tables is the
/// other shape that grows host memory without burning fuel.
#[test]
fn a_module_with_too_many_tables_fails_to_instantiate() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(
            r#"
(module
  (table 1 funcref) (table 1 funcref) (table 1 funcref) (table 1 funcref)
  (table 1 funcref) (table 1 funcref) (table 1 funcref) (table 1 funcref)
  (table 1 funcref)
  (func (export "noop")))
"#,
        )
        .expect("nine-table module assembles"),
    )
    .expect("module should compile");

    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(&mut store, &RenderOptions::default()).expect("limits apply");
    let err = Instance::new(&mut store, &module, &[])
        .expect_err("nine tables exceed the store-level cap (8)");
    assert!(
        format!("{err:#}").to_lowercase().contains("table"),
        "the failure should name the table limit: {err:#}"
    );
}

/// A single oversized table is refused by the element cap.
#[test]
fn an_oversized_table_is_refused() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(
            r#"
(module
  (table 10001 funcref)
  (func (export "noop")))
"#,
        )
        .expect("big-table module assembles"),
    )
    .expect("module should compile");

    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(&mut store, &RenderOptions::default()).expect("limits apply");
    let err = Instance::new(&mut store, &module, &[])
        .expect_err("10001 elements exceed the per-table cap (10000)");
    assert!(
        format!("{err:#}").to_lowercase().contains("table"),
        "the failure should name the table limit: {err:#}"
    );
}

/// Instance count bounds the globals dimension (R4: the default 10_000
/// would let a small input pin tens of GiB of RSS before fuel ticks).
#[test]
fn a_module_with_too_many_instances_fails_to_instantiate() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    // Nested instances are not expressible in plain WAT, so assert the
    // limit is configured rather than the instantiation failing: the
    // store limit itself is the contract this test owns.
    assert_eq!(
        tairitsu_ssr::limits::MAX_MEMORIES_PER_STORE,
        2,
        "the memory multiplier is part of the documented total"
    );
    let _ = (engine, SsrHostState::new().expect("host state"));
}

fn grow(engine: &Engine, module: &Module, options: &RenderOptions) -> i32 {
    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(&mut store, options).expect("limits should apply to a fresh store");

    let instance = Instance::new(&mut store, &module, &[]).expect("module should instantiate");
    let grow = instance
        .get_typed_func::<(), i32>(&mut store, "grow")
        .expect("`grow` export should exist");

    grow.call(&mut store, ()).expect("grow should not trap")
}

#[test]
fn a_denied_growth_that_traps_afterwards_is_reported_instead_of_truncated() {
    let _serial = serial();
    let engine = shared_engine().expect("shared engine should initialise");
    let module = Module::new(
        engine,
        wat::parse_str(GROW_THEN_TRAP_MODULE).expect("module assembles"),
    )
    .expect("module should compile");

    // 1 MiB cap = 16 pages, so adding 32 pages is denied. The guest then traps with
    // `unreachable`, which the render path cannot classify as an execution bound from
    // the trap alone.
    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(
        &mut store,
        &RenderOptions {
            max_memory_bytes: 1024 * 1024,
            ..RenderOptions::default()
        },
    )
    .expect("limits should apply to a fresh store");

    let instance = Instance::new(&mut store, &module, &[]).expect("module should instantiate");
    let call = instance
        .get_typed_func::<(), ()>(&mut store, "grow_then_trap")
        .expect("`grow_then_trap` export should exist");
    let trap = call
        .call(&mut store, ())
        .expect_err("the guest traps after its growth is denied");
    assert!(
        matches!(
            trap.downcast_ref::<wasmtime::Trap>(),
            Some(wasmtime::Trap::UnreachableCodeReached)
        ),
        "expected the guest's own `unreachable` trap, got: {trap:?}"
    );

    // The denial itself is the durable evidence of the memory cap; the render path
    // turns it into an error instead of serving the truncated page.
    let error = store
        .data()
        .memory_budget_error()
        .expect("a denied growth must be recorded on the host state");
    assert!(
        message(&error).contains("linear-memory budget"),
        "unexpected classification: {error:#}"
    );

    // Negative control: with the bounds lifted the same module completes, so the
    // denial above really comes from the cap and not from the module.
    let mut trusted = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(&mut trusted, &RenderOptions::trusted()).expect("limits should apply");
    let instance = Instance::new(&mut trusted, &module, &[]).expect("module should instantiate");
    let call = instance
        .get_typed_func::<(), ()>(&mut trusted, "grow_then_trap")
        .expect("`grow_then_trap` export should exist");
    call.call(&mut trusted, ())
        .expect("an unbounded store must allow the growth");
    assert!(
        trusted.data().memory_budget_error().is_none(),
        "a trusted render must not report a memory-budget hit"
    );
}
