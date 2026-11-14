//! Execution bounds and compile-cache behaviour of the SSR render path.
//!
//! The crate has no checked-in component fixtures (a real tairitsu component is
//! built by the packager for `wasm32-wasip2`), so these tests build the smallest
//! components that still exercise the shipped paths:
//!
//! * a component whose core module's start function never returns — proves a
//!   runaway guest is stopped instead of hanging the calling thread;
//! * a bare component — proves the compiled artifact is reused across renders.
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
    // browser exports. That is enough to observe the compile cache.
    let wasm = wat::parse_str("(component)").expect("empty component should assemble");

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
    // missing-exports error as a bounded render.
    let wasm = wat::parse_str("(component)").expect("empty component should assemble");

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

fn grow(engine: &Engine, module: &Module, options: &RenderOptions) -> i32 {
    let mut store = Store::new(engine, SsrHostState::new().expect("host state"));
    apply_limits(&mut store, options).expect("limits should apply to a fresh store");

    let instance = Instance::new(&mut store, module, &[]).expect("module should instantiate");
    let grow = instance
        .get_typed_func::<(), i32>(&mut store, "grow")
        .expect("`grow` export should exist");

    grow.call(&mut store, ()).expect("grow should not trap")
}
