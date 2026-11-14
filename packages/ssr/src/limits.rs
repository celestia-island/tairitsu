//! Process-wide SSR engine and per-render guest execution bounds.
//!
//! SSR renders whatever component binary the caller hands over. That binary may
//! be a fresh build, but it may equally come from a user-supplied upload, a
//! third-party component, or a compromised build pipeline. Before this module
//! existed every [`render_to_html`](crate::render_to_html) call built its own
//! [`Engine`], re-JIT-compiled the whole component, and then ran the guest with
//! **no** execution bound at all: a component containing `loop {}` (or one that
//! grows a 4 GiB linear memory) pinned the host thread until the process died.
//!
//! This module fixes three separate problems:
//!
//! 1. **Compile once per binary** — [`cached_component`] compiles a component at
//!    most once per process and reuses the compiled artifact afterwards. The
//!    cache is a bounded LRU keyed by the SHA-256 of the wasm bytes, so a rebuilt
//!    component never hits a stale entry, and both the entry count
//!    ([`COMPONENT_CACHE_CAPACITY`]) and the input size ([`MAX_COMPONENT_BYTES`])
//!    are bounded, because the inputs can be uploads of unknown provenance.
//! 2. **CPU bound** — the shared engine enables
//!    [`Config::consume_fuel`](wasmtime::Config::consume_fuel) and
//!    [`Config::epoch_interruption`](wasmtime::Config::epoch_interruption), and
//!    [`apply_limits`] installs a fuel budget plus a wall-clock epoch deadline on
//!    each bounded store.
//! 3. **Memory bound** — [`apply_limits`] installs a
//!    [`StoreLimits`](wasmtime::StoreLimits) linear-memory cap through
//!    [`Store::limiter`], so an allocation bomb fails (`memory.grow` returns -1,
//!    or instantiation fails when the declared minimum already exceeds the cap)
//!    instead of pinning gigabytes of RSS. A `-1` alone would be invisible from
//!    outside the guest, so every LINEAR-MEMORY denial is also recorded on the host
//!    state (table denials are not — their caps bound arithmetic, and a denied
//!    `table.grow` traps the guest loudly) (see
//!    [`SsrHostState::memory_budget_error`]) and the render path turns it into an
//!    error instead of serving the truncated page the guest left behind.
//!
//! Bounds are **on by default** ([`RenderOptions::default`]). Removing them is an
//! explicit opt-in for callers rendering a component they build and trust:
//! [`RenderOptions::trusted`].

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard, Once, OnceLock},
    time::Duration,
};

use anyhow::Result;
use sha2::{Digest, Sha256};
use wasmtime::{
    component::Component, Config, Engine, OptLevel, Store, StoreLimitsBuilder, Strategy,
};

use crate::host_state::{SsrConfig, SsrHostState};

/// Fuel granted to one bounded render (one fuel unit ≈ one wasm instruction).
///
/// **Not calibrated against a real component.** This crate has no checked-in
/// component fixture — a real tairitsu component is built by the packager for
/// `wasm32-wasip2` — so this number is an estimate from the shape of the workload,
/// not a measurement. That shape is host-import dominated: DOM construction,
/// attribute writes and listener registration all happen in host code and are not
/// metered, so the metered guest work for `lifecycle::start` on a normal page is
/// roughly the component's bootstrap code, on the order of 10^6–10^7 instructions.
/// 1e8 therefore leaves about an order of magnitude of headroom for large pages
/// while still cutting a runaway guest off after well under a second of
/// JIT-compiled native execution. A host that knows its components better should
/// override it per render through [`SsrConfig::ssr_fuel`] instead of trusting this
/// default.
pub const DEFAULT_FUEL: u64 = 100_000_000;

/// Wall-clock deadline for one bounded render.
///
/// Fuel is the primary CPU bound, but it does not cover time spent inside a host
/// import (stdio, timers) nor pathological code where one instruction is expensive
/// (`memcpy`-shaped loops). 3 s is ~10x an *estimated* healthy SSR render (the same
/// caveat as [`DEFAULT_FUEL`]: no real component fixture exists to measure against),
/// so it only fires on genuinely stuck guests, and it keeps a single request's
/// latency bounded. Override per render through [`SsrConfig::ssr_timeout_ms`].
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

/// Linear-memory cap for one bounded render (per linear memory).
///
/// Real components keep single-digit MiB of linear memory (the DOM lives in host
/// memory, not in the guest). 64 MiB is ~16x headroom for large pages while making
/// the classic 4 GiB memory bomb impossible: a component whose *declared minimum*
/// already exceeds the cap fails to instantiate, and a `memory.grow` bomb is denied.
pub const DEFAULT_MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024;

/// Period of the process-wide epoch ticker.
///
/// The ticker bounds how late an epoch deadline can fire. 10 ms of extra runaway
/// execution is negligible, while a 100 Hz wakeup is cheap enough to run for the
/// lifetime of the process (one shared ticker, not one per render).
pub const EPOCH_TICK: Duration = Duration::from_millis(10);

/// Epoch deadline used when a store must never be interrupted by time.
///
/// `Store::set_epoch_deadline` adds `ticks` to the engine's current epoch, and a
/// store on an epoch-instrumented engine traps immediately unless a deadline is
/// installed. Trusted stores therefore get a deadline far enough out that it can
/// never be reached (u64::MAX/2 ticks at 100 Hz ≈ 2.9e9 years) without risking the
/// `current_epoch + delta` overflow inside wasmtime.
const NO_DEADLINE_TICKS: u64 = u64::MAX / 2;

/// Fuel granted to a trusted store.
///
/// A store on a fuel-consuming engine starts with **zero** fuel and traps on its
/// first fuel check, so "no fuel limit" has to be spelled as an explicit, effectively
/// unreachable budget (u64::MAX/2 leaves head-room for wasmtime's internal
/// arithmetic without ever running out on a real render).
const NO_FUEL_LIMIT: u64 = u64::MAX / 2;

/// Maximum number of compiled components kept in the process-wide cache.
///
/// Each entry holds the JIT-compiled artifact of one component (order of a few MiB
/// with Winch). 32 entries comfortably covers a site with up to 32 distinct
/// component builds; a larger working set evicts the least recently rendered
/// component instead of growing without bound. The entry count alone does not bound
/// the cache — each entry's input is capped by [`MAX_COMPONENT_BYTES`], and the byte
/// account lives there.
pub const COMPONENT_CACHE_CAPACITY: usize = 32;

/// Largest component binary [`cached_component`] accepts, in bytes.
///
/// This module handles binaries of unknown provenance — a user-supplied upload is
/// the documented worst case — so the *input* size needs a bound, not only the
/// number of cache entries: JIT compilation cost grows with the input, and so does
/// the artifact that then sits in the cache. An oversized input is rejected with an
/// error before it is hashed or compiled, so nothing unbounded is ever retained.
///
/// 8 MiB is generous next to a real component: a full tairitsu app including
/// hikari's UI code is expected to be single-digit MiB, though this crate has no
/// fixture that would let us measure it (the same gap [`DEFAULT_FUEL`] documents).
///
/// Byte account for the cache: 32 entries × 8 MiB of input = **256 MiB of input**
/// retained, and each entry's compiled artifact is a small multiple of its code
/// section under Winch baseline codegen (roughly 1–4×), so a fully populated cache
/// sits around 0.25–1 GiB. Reaching that requires the host to ingest 32 distinct
/// 8 MiB binaries — a deliberate act, not an accident.
pub const MAX_COMPONENT_BYTES: usize = 8 * 1024 * 1024;

/// Memories one untrusted store may define — the multiplier that turns
/// the per-memory byte cap into a total: the worst case per render is
/// `MAX_MEMORIES_PER_STORE x RenderOptions::max_memory_bytes`.
pub const MAX_MEMORIES_PER_STORE: usize = 2;

/// Core-module instances one untrusted store may define. Instances are
/// the only remaining wasmtime handle on the globals dimension, so this
/// is what keeps a small input from pinning gigabytes of RSS before any
/// fuel ticks (R4 finding; a measured 300 instances x 4k globals grew
/// RSS by 19 MiB). A packager-produced page component instantiates a
/// handful; 32 is a wide margin.
pub const MAX_INSTANCES_PER_STORE: usize = 32;

/// Per-render guest execution policy.
///
/// The default bounds the guest (CPU fuel, wall-clock deadline, linear memory).
/// Render components you built and trust with [`RenderOptions::trusted`] to opt
/// out of all three bounds.
///
/// ```
/// use tairitsu_ssr::RenderOptions;
///
/// // Bounded (default): safe for components of unknown provenance.
/// let bounded = RenderOptions::default();
/// assert!(bounded.untrusted);
///
/// // Unbounded: opt-in only, for self-built components.
/// let trusted = RenderOptions::trusted();
/// assert!(!trusted.untrusted);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions {
    /// Whether the guest is treated as untrusted, i.e. whether the bounds below
    /// are enforced. Defaults to `true`.
    pub untrusted: bool,
    /// Fuel granted to the guest store when [`untrusted`](Self::untrusted).
    pub fuel: u64,
    /// Wall-clock deadline enforced through epoch interruption when
    /// [`untrusted`](Self::untrusted).
    pub timeout: Duration,
    /// Maximum linear memory in bytes, per memory, when
    /// [`untrusted`](Self::untrusted).
    pub max_memory_bytes: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            untrusted: true,
            fuel: DEFAULT_FUEL,
            timeout: DEFAULT_TIMEOUT,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
        }
    }
}

impl RenderOptions {
    /// Render without any guest execution bound.
    ///
    /// Only use this for components whose binary you build and control: an
    /// unbounded guest can hang the calling thread (`loop {}`) or exhaust host
    /// memory (4 GiB linear memory) with no way for the host to intervene.
    pub fn trusted() -> Self {
        Self {
            untrusted: false,
            ..Self::default()
        }
    }

    /// Bounded policy that honours the per-render knobs on [`SsrConfig`].
    ///
    /// [`SsrConfig::ssr_fuel`] and [`SsrConfig::ssr_timeout_ms`] override
    /// [`DEFAULT_FUEL`] and [`DEFAULT_TIMEOUT`]; leaving them `None` keeps the
    /// defaults. The bounds themselves always stay in force — a config knob can
    /// adjust a budget but never remove one; dropping the bounds remains the
    /// explicit [`RenderOptions::trusted`] opt-in in code.
    pub fn for_config(config: &SsrConfig) -> Self {
        let mut options = Self::default();
        if let Some(fuel) = config.ssr_fuel {
            options.fuel = fuel;
        }
        if let Some(timeout_ms) = config.ssr_timeout_ms {
            options.timeout = Duration::from_millis(timeout_ms);
        }
        options
    }
}

/// Compile-cache counters, for server metrics and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentCacheStats {
    /// Compiles that were served from the cache.
    pub hits: u64,
    /// Compiles that had to JIT a component.
    pub misses: u64,
    /// Number of components currently cached.
    pub entries: usize,
    /// Cache capacity in components ([`COMPONENT_CACHE_CAPACITY`]).
    pub capacity: usize,
}

/// The process-wide engine used by every SSR render.
///
/// One engine per process means one JIT/code cache and one epoch ticker, instead
/// of a fresh `Engine` (and a fresh set of compiler threads) per request.
pub fn shared_engine() -> Result<&'static Engine> {
    if let Some(engine) = ENGINE.get() {
        // A thread that races the initialiser can see the engine before that
        // initialiser spawned the ticker. Without a ticker here, an epoch deadline
        // installed by this thread would only fire once some *other* render started
        // it — i.e. a bounded render could run to its fuel budget with a deadline
        // that never fires. `spawn_epoch_ticker` is `Once`-guarded, so calling it on
        // the fast path too closes that window and is free afterwards.
        spawn_epoch_ticker(engine);
        return Ok(engine);
    }

    // Build outside the `OnceLock` so a failure (unsupported strategy, no codegen
    // backend, ...) is reported to the caller instead of being cached as a panic.
    // Two threads racing here may both build an engine; the loser drops its copy,
    // which is harmless: no engine state is observable before the ticker starts.
    let candidate = build_engine()?;
    let engine = ENGINE.get_or_init(|| candidate);

    // Exactly one ticker drives the shared engine's epoch counter.
    spawn_epoch_ticker(engine);

    Ok(engine)
}

/// Compile `wasm_bytes` once per process and return the shared compiled component.
///
/// The key is the SHA-256 of the bytes, so a rebuilt component never reuses the
/// artifact compiled from an older binary. The compiled component belongs to the
/// process-wide engine returned by [`shared_engine`] and must only be instantiated
/// in stores created from that engine.
///
/// Inputs larger than [`MAX_COMPONENT_BYTES`] are rejected before hashing or
/// compiling: the input can only be the wrong file, so this is a loud error rather
/// than a silent skip (or a multi-hundred-MiB compile someone has to wait for).
pub fn cached_component(wasm_bytes: &[u8]) -> Result<Arc<Component>> {
    if wasm_bytes.len() > MAX_COMPONENT_BYTES {
        anyhow::bail!(
            "Refusing to compile a {} byte SSR component: it exceeds the {} byte limit \
             for components of unknown provenance (MAX_COMPONENT_BYTES)",
            wasm_bytes.len(),
            MAX_COMPONENT_BYTES
        );
    }

    let key = component_key(wasm_bytes);

    {
        let mut cache = lock_cache();
        if let Some(component) = cache.get(&key) {
            cache.hits += 1;
            return Ok(component);
        }
        cache.misses += 1;
    }

    // Compile outside the cache lock: JIT compilation takes milliseconds to
    // seconds, and holding the lock across it would serialise every concurrent
    // render (including cache hits for unrelated components).
    let engine = shared_engine()?;
    let component =
        Arc::new(Component::from_binary(engine, wasm_bytes).map_err(|error| {
            anyhow::Error::from(error).context("Failed to compile WASM component")
        })?);

    // A concurrent render may have compiled the same binary in the meantime; the
    // duplicate is dropped and the cache keeps a single artifact per key. A binary
    // that fails to compile is never cached, so a broken component cannot evict
    // healthy entries.
    lock_cache().insert(key, component.clone());

    Ok(component)
}

/// Current compile-cache counters.
pub fn component_cache_stats() -> ComponentCacheStats {
    let cache = lock_cache();
    ComponentCacheStats {
        hits: cache.hits,
        misses: cache.misses,
        entries: cache.entries.len(),
        capacity: COMPONENT_CACHE_CAPACITY,
    }
}

/// Apply `options` to a freshly created SSR store.
///
/// This is the primitive [`render_to_html_with_options`] uses; it is public so a
/// host that builds its own store (for example a long-lived worker) can install the
/// same bounds. Bounded stores get a fuel budget, an epoch deadline and a
/// linear-memory limit. Trusted stores still need a fuel budget and an epoch
/// deadline — a store on a fuel- or epoch-instrumented engine traps immediately
/// without them — so they get budgets that no real render can reach.
///
/// [`render_to_html_with_options`]: crate::render_to_html_with_options
pub fn apply_limits(store: &mut Store<SsrHostState>, options: &RenderOptions) -> Result<()> {
    let fuel = if options.untrusted {
        options.fuel
    } else {
        NO_FUEL_LIMIT
    };
    store.set_fuel(fuel).map_err(|error| {
        anyhow::Error::from(error)
            .context("Failed to install the guest fuel budget on the SSR store")
    })?;

    if !options.untrusted {
        // Still install a deadline: a store on an epoch-instrumented engine traps
        // immediately otherwise, so `NO_DEADLINE_TICKS` is what "no deadline" means.
        store.set_epoch_deadline(NO_DEADLINE_TICKS);
        return Ok(());
    }

    // `set_epoch_deadline` wants ticks; round up so a sub-tick timeout still fires.
    let tick_millis = EPOCH_TICK.as_millis().max(1) as u64;
    let timeout_millis = options.timeout.as_millis() as u64;
    let ticks = timeout_millis.div_ceil(tick_millis).max(1);
    store.set_epoch_deadline(ticks);

    // `trap_on_grow_failure` stays off on purpose: a guest probing for available
    // memory (dlmalloc-style) must see `memory.grow == -1` rather than be killed by
    // a host trap, so the cap denies growth without breaking well-behaved guests.
    // The cost of spec behaviour is that a denial is invisible outside the guest — it
    // then either carries on with a truncated page, returns a typed error, or traps
    // for its own reasons (`unreachable` after `-1`), none of which is an
    // `OutOfFuel`/`Interrupt` trap. `SsrHostState` therefore implements
    // `ResourceLimiter` itself and counts denials; the render path refuses to report
    // success once that counter is non-zero (see `SsrHostState::memory_budget_error`).
    // The memory cap is PER LINEAR MEMORY (wasmtime semantics), so the
    // real total for one untrusted store is `MAX_MEMORIES_PER_STORE x
    // max_memory_bytes` (128 MiB with the default 64 MiB cap) — the
    // count limit below is what makes that an arithmetic bound instead
    // of an unbounded N (R3 S2: an 8-line two-memory probe held 6 MiB
    // under a 4 MiB per-memory cap).
    store.data_mut().store_limits = StoreLimitsBuilder::new()
        .memory_size(options.max_memory_bytes)
        .memories(MAX_MEMORIES_PER_STORE)
        .tables(8)
        .table_elements(10_000)
        // Instance count bounds the GLOBALS dimension wasmtime does not
        // meter elsewhere: ~4k globals x 16 B per instance, so the
        // default 10_000 would let a ~7 MiB input pin tens of GiB of
        // RSS before fuel ever ticks (R4 finding — a measured 300
        // instances x 4k globals grew RSS by 19 MiB). 32 is far above
        // any real page component.
        .instances(MAX_INSTANCES_PER_STORE)
        .build();
    store.limiter(SsrHostState::limiter_mut);

    Ok(())
}

/// The process-wide engine. Initialised once, then shared by every render.
static ENGINE: OnceLock<Engine> = OnceLock::new();

/// Guards the single epoch ticker thread.
static EPOCH_TICKER: Once = Once::new();

/// The process-wide compiled-component cache.
static CACHE: OnceLock<Mutex<ComponentCache>> = OnceLock::new();

fn build_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    // Winch is a baseline (single-pass) compiler that compiles WASM much faster
    // than Cranelift's optimizing pipeline. This is critical on QEMU TCG aarch64,
    // where Cranelift's codegen runs under emulation and takes minutes to
    // JIT-compile a typical tairitsu component. The generated code is less
    // optimized, but for SSR the compile-time savings dominate — and now that the
    // artifact is cached per binary, only the first render pays it.
    config.strategy(Strategy::Winch);
    config.cranelift_opt_level(OptLevel::None);
    // Enable the metering facilities; they stay inert for stores that never install
    // a budget (trusted renders), and per-store budgets are set in `apply_limits`.
    config.consume_fuel(true);
    config.epoch_interruption(true);

    Engine::new(&config).map_err(|error| {
        anyhow::Error::from(error).context("Failed to create the SSR wasmtime engine")
    })
}

/// Start the process-wide epoch ticker unless it is already running.
fn spawn_epoch_ticker(engine: &Engine) {
    EPOCH_TICKER.call_once(|| {
        let engine = engine.clone();
        let spawned = std::thread::Builder::new()
            .name("tairitsu-ssr-epoch".to_string())
            .spawn(move || loop {
                std::thread::sleep(EPOCH_TICK);
                engine.increment_epoch();
            });

        if let Err(error) = spawned {
            // Without a ticker the epoch deadline never fires; the fuel budget (which
            // any bounded render also installs) remains the CPU bound.
            //
            // `eprintln!` rather than `tracing::warn!`: this crate is a library and
            // most hosts install no tracing subscriber, so a `warn!` here would be
            // dropped silently — and this is exactly the degraded mode an operator
            // has to know about.
            eprintln!(
                "tairitsu-ssr: could not spawn the epoch ticker ({error}); \
                 wall-clock deadlines are disabled, fuel limits still apply"
            );
        }
    });
}

fn component_key(wasm_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    hasher.finalize().into()
}

fn lock_cache() -> MutexGuard<'static, ComponentCache> {
    // Recover from poisoning instead of propagating: a panic in one render must not
    // disable caching for the rest of the process, and the critical sections only
    // touch this map and deque.
    CACHE
        .get_or_init(|| Mutex::new(ComponentCache::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Default)]
struct ComponentCache {
    entries: HashMap<[u8; 32], Arc<Component>>,
    /// Least recently rendered key first; `COMPONENT_CACHE_CAPACITY` is enforced
    /// against this deque's length.
    recency: VecDeque<[u8; 32]>,
    hits: u64,
    misses: u64,
}

impl ComponentCache {
    fn get(&mut self, key: &[u8; 32]) -> Option<Arc<Component>> {
        let component = self.entries.get(key).cloned();
        if component.is_some() {
            self.touch(*key);
        }
        component
    }

    fn insert(&mut self, key: [u8; 32], component: Arc<Component>) {
        self.entries.insert(key, component);
        self.touch(key);

        while self.recency.len() > COMPONENT_CACHE_CAPACITY {
            if let Some(evicted) = self.recency.pop_front() {
                // A key can only be absent here if it was overwritten, in which case
                // the map already holds the newest artifact.
                self.entries.remove(&evicted);
            }
        }
    }

    fn touch(&mut self, key: [u8; 32]) {
        if let Some(position) = self.recency.iter().position(|known| *known == key) {
            self.recency.remove(position);
        }
        self.recency.push_back(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_render_options_are_bounded() {
        let options = RenderOptions::default();
        assert!(options.untrusted);
        assert_eq!(options.fuel, DEFAULT_FUEL);
        assert_eq!(options.timeout, DEFAULT_TIMEOUT);
        assert_eq!(options.max_memory_bytes, DEFAULT_MAX_MEMORY_BYTES);
    }

    #[test]
    fn trusted_render_options_drop_bounds() {
        assert!(!RenderOptions::trusted().untrusted);
    }

    #[test]
    fn render_options_honour_the_config_knobs() {
        let mut config = SsrConfig::default();
        assert_eq!(RenderOptions::for_config(&config), RenderOptions::default());

        config.ssr_fuel = Some(1_234);
        config.ssr_timeout_ms = Some(250);
        let options = RenderOptions::for_config(&config);
        assert!(
            options.untrusted,
            "a config knob must not disable the bounds"
        );
        assert_eq!(options.fuel, 1_234);
        assert_eq!(options.timeout, Duration::from_millis(250));
        assert_eq!(options.max_memory_bytes, DEFAULT_MAX_MEMORY_BYTES);
    }

    #[test]
    fn cache_key_tracks_content() {
        assert_eq!(component_key(b"a"), component_key(b"a"));
        assert_ne!(component_key(b"a"), component_key(b"b"));
    }

    #[test]
    fn oversized_component_inputs_are_rejected() {
        // The cap is checked before hashing/compiling, so this never touches the
        // engine: the only observable effect is the error.
        let oversized = vec![0u8; MAX_COMPONENT_BYTES + 1];
        let error = cached_component(&oversized)
            .expect_err("an input past MAX_COMPONENT_BYTES must be rejected");
        let message = format!("{error:#}");
        assert!(
            message.contains("exceeds") && message.contains("MAX_COMPONENT_BYTES"),
            "the rejection must name the limit, got: {message}"
        );
    }

    #[test]
    fn lru_evicts_oldest_entry_past_capacity() {
        let engine = shared_engine().expect("shared engine");
        let mut cache = ComponentCache::default();
        let component = Arc::new(
            Component::from_binary(engine, wat::parse_str("(component)").unwrap().as_slice())
                .unwrap(),
        );

        for index in 0..(COMPONENT_CACHE_CAPACITY + 1) {
            let mut key = [0u8; 32];
            key[0] = index as u8;
            cache.insert(key, component.clone());
        }

        assert_eq!(cache.entries.len(), COMPONENT_CACHE_CAPACITY);
        assert_eq!(cache.recency.len(), COMPONENT_CACHE_CAPACITY);
        assert!(!cache.entries.contains_key(&[0u8; 32]));
    }
}
