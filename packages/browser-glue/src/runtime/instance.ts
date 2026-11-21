// W4: handle state is PER INSTANCE, not per page.
//
// Before W4 every handle table (`__elementHandles`, `__eventHandles`, the
// `__next*` counters, …) lived on `globalThis` as a mutable Map/Set. Two
// components booted on one page then shared one table: handles crossed
// instances, a destroyed instance left its listeners/observers/timers behind,
// and nothing could be authorized per instance (§0 B1).
//
// The migration keeps the *call sites* unchanged: the glue (and the code
// `generateModuleCode` emits) still writes `globalThis.__elementHandles.set(…)`,
// but those names are now ACCESSOR PROPERTIES that forward to the current
// instance's own tables. Nothing mutable is stored on `globalThis` any more —
// it only holds the pointer to the current instance and the instance registry.
//
// Lifecycle: `createInstance()` → `setCurrentInstance()` → the guest runs →
// `disposeInstance()` clears the tables, the timers, the observers and the
// listeners, so any handle from that instance stops resolving (the existing
// lookups throw "…handle N not found", which is the contract W4 pins).

/** The mutable tables one instance owns. */
export interface InstanceTables {
  elementHandles: Map<bigint, unknown>;
  documentHandles: Map<bigint, unknown>;
  nodeHandles: Map<bigint, unknown>;
  textHandles: Map<bigint, unknown>;
  mutationRecordHandles: Map<bigint, unknown>;
  resizeObserverEntryHandles: Map<bigint, unknown>;
  resizeObserverSizeHandles: Map<bigint, unknown>;
  domRectHandles: Map<bigint, unknown>;
  nodeListHandles: Map<bigint, unknown>;
  cssStyleDeclarationHandles: Map<bigint, unknown>;
  listenerHandles: Map<bigint, unknown>;
  tairitsuScrollListeners: Map<bigint, unknown>;
  // Page-global indexes that hand handles between instances if they stay
  // shared: the element→handle reverse index and the (element,type)→listener
  // cache used by the dispatch bridge.
  elementReverseMap: Map<unknown, unknown> | undefined;
  listenerByElement: Map<unknown, unknown>;
  tairitsuResizeListeners: Map<bigint, unknown>;
  tairitsuWsCallbacks: Map<bigint, unknown>;
  subEventHandles: WeakMap<object, bigint>;
  // Timer / animation / websocket state bags: their ids are handed to the guest,
  // so a shared bag lets one instance cancel another's timer.
  timerState: { timeoutCallbacks: Map<number, number>; intervalCallbacks: Map<number, number>; nextTimeoutId: number };
  animState: { animationCallbacks: Map<number, unknown>; nextAnimationId: number };
  wsState: { nextCallbackId: number };
  eventHandles: Map<bigint, unknown>;
  dispatchingEvents: Set<string>;
}

/** Anything the instance owns that must be undone / cleared on dispose. */
interface Reclaimables {
  timers: Set<ReturnType<typeof globalThis.setTimeout>>;
  intervals: Set<ReturnType<typeof globalThis.setInterval>>;
  frames: Set<number>;
  sockets: Set<{ close(): void }>;
  observers: Set<{ disconnect(): void }>;
  listeners: Set<{
    target: EventTarget;
    type: string;
    handler: EventListener;
    options?: boolean | AddEventListenerOptions;
  }>;
}

export class TairitsuInstance {
  static nextId = 1;

  readonly id: number = TairitsuInstance.nextId++;
  readonly tables: InstanceTables = {
    elementHandles: new Map(),
    documentHandles: new Map(),
    nodeHandles: new Map(),
    textHandles: new Map(),
    mutationRecordHandles: new Map(),
    resizeObserverEntryHandles: new Map(),
    resizeObserverSizeHandles: new Map(),
    domRectHandles: new Map(),
    nodeListHandles: new Map(),
    cssStyleDeclarationHandles: new Map(),
    listenerHandles: new Map(),
    tairitsuScrollListeners: new Map(),
    elementReverseMap: undefined as unknown as Map<unknown, unknown>,
    listenerByElement: new Map(),
    tairitsuResizeListeners: new Map(),
    tairitsuWsCallbacks: new Map(),
    subEventHandles: new WeakMap(),
    timerState: { timeoutCallbacks: new Map(), intervalCallbacks: new Map(), nextTimeoutId: 1 },
    animState: { animationCallbacks: new Map(), nextAnimationId: 1 },
    wsState: { nextCallbackId: 1 },
    eventHandles: new Map(),
    dispatchingEvents: new Set(),
  };

  /**
   * The `__next*` counters, also per instance. `nextHandle` is seeded from the
   * instance id so handle NUMBERS never collide across instances: with a bare
   * per-instance `1n` counter, a handle minted by B would silently resolve to
   * A's object of the same number instead of failing as "not found". Handles
   * stay bigints; the seed is module state, not `globalThis` state.
   */
  readonly seed: bigint = BigInt(this.id) << 32n;
  counters = {
    nextHandle: this.seed,
    nextMutationRecord: this.seed,
    nextResizeObserverEntry: this.seed,
    nextResizeObserverSizeHandle: this.seed,
    nextDomRectHandle: this.seed,
    nextNodeList: this.seed,
    nextCssStyleDeclarationHandle: this.seed,
    nextListenerHandle: this.seed,
    nextEventHandle: this.seed,
  };

  readonly reclaimables: Reclaimables = {
    timers: new Set(),
    intervals: new Set(),
    frames: new Set(),
    sockets: new Set(),
    observers: new Set(),
    listeners: new Set(),
  };

  /** Set by `disposeInstance`; a disposed instance is never usable again. */
  disposed = false;

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;

    for (const table of Object.values(this.tables)) {
      // WeakMaps cannot be cleared and a lazy table may be absent yet; the
      // instance itself is dropped, so skipping both is the right reclaim.
      if (table && typeof (table as { clear?: () => void }).clear === "function") {
        (table as { clear: () => void }).clear();
      }
    }

    for (const timer of this.reclaimables.timers) globalThis.clearTimeout(timer);
    for (const interval of this.reclaimables.intervals) globalThis.clearInterval(interval);
    for (const frame of this.reclaimables.frames) globalThis.cancelAnimationFrame(frame);
    for (const socket of this.reclaimables.sockets) {
      try {
        socket.close();
      } catch {
        // A socket that already closed must not block the rest of the reclaim.
      }
    }
    for (const observer of this.reclaimables.observers) {
      try {
        observer.disconnect();
      } catch {
        // A disconnect that throws must not block the rest of the reclaim.
      }
    }
    for (const { target, type, handler, options } of this.reclaimables.listeners) {
      try {
        target.removeEventListener(type, handler, options);
      } catch {
        // Same: a stale target must not abort the reclaim.
      }
    }

    this.reclaimables.timers.clear();
    this.reclaimables.intervals.clear();
    this.reclaimables.frames.clear();
    this.reclaimables.sockets.clear();
    this.reclaimables.observers.clear();
    this.reclaimables.listeners.clear();

    // Every counter resets to the instance's own seed, so a disposed instance
    // cannot hand out a number another instance already used.
    for (const key of Object.keys(this.counters) as (keyof typeof this.counters)[]) {
      this.counters[key] = this.seed;
    }

    // The state bags hold ids and callbacks too; clear them with the tables.
    this.tables.timerState.timeoutCallbacks.clear();
    this.tables.timerState.intervalCallbacks.clear();
    this.tables.timerState.nextTimeoutId = 1;
    this.tables.animState.animationCallbacks.clear();
    this.tables.animState.nextAnimationId = 1;
    this.tables.wsState.nextCallbackId = 1;
  }
}

/** Every instance the page has created and not yet disposed. */
const instances = new Set<TairitsuInstance>();
let current: TairitsuInstance | null = null;
/**
 * The instance currently OWNING the page's single global glue surface. W4's
 * explicit contract: one active guest per page. A second `claimGuest` while
 * another instance holds the surface is REFUSED — silently displacing the
 * running component (which is what a bare `setCurrent` did) is a bug, not a
 * behaviour. Per-call instance binding, which would lift this, is W6's input.
 */
let activeGuest: TairitsuInstance | null = null;

export class GuestAlreadyActiveError extends Error {
  constructor(active: TairitsuInstance, requested: TairitsuInstance) {
    super(
      `another component already owns this page's glue surface (instance ${active.id}); ` +
        `refusing to boot instance ${requested.id} over it — dispose the running component first`,
    );
    this.name = "GuestAlreadyActiveError";
  }
}

/** Take the page's glue surface for `instance`; throws if another holds it. */
export function claimGuest(instance: TairitsuInstance): void {
  if (activeGuest && activeGuest !== instance && !activeGuest.disposed) {
    throw new GuestAlreadyActiveError(activeGuest, instance);
  }
  activeGuest = instance;
  current = instance;
}

/** Give the surface back (called by dispose; idempotent). */
export function releaseGuest(instance: TairitsuInstance): void {
  if (activeGuest === instance) {
    activeGuest = null;
    if (current === instance) current = null;
  }
}

export function createInstance(): TairitsuInstance {
  const instance = new TairitsuInstance();
  instances.add(instance);
  return instance;
}

export function currentInstance(): TairitsuInstance {
  if (!current) {
    // A glue call before any instance exists (e.g. a module imported for its
    // side effects) still needs tables; give it the page's default instance.
    current = createInstance();
  }
  return current;
}

export function setCurrentInstance(instance: TairitsuInstance | null): TairitsuInstance | null {
  const previous = current;
  current = instance;
  return previous;
}

/** Run `fn` with `instance` as the current one (restores the previous). */
export function withInstance<T>(instance: TairitsuInstance, fn: () => T): T {
  const previous = setCurrentInstance(instance);
  try {
    return fn();
  } finally {
    setCurrentInstance(previous);
  }
}

/**
 * Dispose one instance. Returns false when it was already gone, so callers can
 * report a double-stop instead of silently succeeding.
 */
export function disposeInstance(instance: TairitsuInstance): boolean {
  if (!instances.delete(instance)) return false;
  if (current === instance) current = null;
  releaseGuest(instance);
  instance.dispose();
  return true;
}

/** The live instances (diagnostics / the isolation test). */
export function listInstances(): TairitsuInstance[] {
  return [...instances];
}

// ── compat surface ─────────────────────────────────────────────────────────
//
// Names the glue and the generated module code keep using. They are defined as
// accessors so a read or a write lands in the CURRENT instance's tables: the
// call sites stay untouched, and `globalThis` holds no mutable handle state.

const TABLE_NAMES = [
  "__elementHandles",
  "__documentHandles",
  "__nodeHandles",
  "__textHandles",
  "__mutationRecordHandles",
  "__resizeObserverEntryHandles",
  "__resizeObserverSizeHandles",
  "__domRectHandles",
  "__nodeListHandles",
  "__cssStyleDeclarationHandles",
  "__listenerHandles",
  "__tairitsuScrollListeners",
  "__elementReverseMap",
  "__listenerByElement",
  "__tairitsuResizeListeners",
  "__tairitsuWsCallbacks",
  "__subEventHandles",
  "__eventHandles",
  "__dispatchingEvents",
] as const;

const COUNTER_NAMES = [
  "__nextHandle",
  "__nextMutationRecord",
  "__nextResizeObserverEntry",
  "__nextResizeObserverSizeHandle",
  "__nextDomRectHandle",
  "__nextNodeList",
  "__nextCssStyleDeclarationHandle",
  "__nextListenerHandle",
  "__nextEventHandle",
] as const;

function defineDelegate(name: string, read: () => unknown, write: (value: unknown) => void): void {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
  if (descriptor && descriptor.get && descriptor.set) return; // already installed
  Object.defineProperty(globalThis, name, {
    configurable: true,
    enumerable: false,
    get: read,
    set: write,
  });
}

/**
 * Install the compat accessors. Idempotent; called by the modules that used to
 * create the tables (`handles.ts`, `eventTarget.ts`).
 */
/**
 * The current instance, reachable from `globalThis`.
 *
 * W5 removed the stringified module code, so the glue's own functions could now
 * import this directly; the global accessor stays because the build-time ESM
 * modules (`dist/glue/*.js`) delegate through `globalThis.__TAIRITSU_GLUE__` and
 * the registration sites are reachable from both, and because a host that swaps
 * the runtime for one in another realm still needs the indirection.
 */
export function installCurrentAccessor(): void {
  (globalThis as any).__TAIRITSU_CURRENT__ = () => currentInstance(); // ts-escape-ok: retires with L40
}

export function installGlobalDelegates(): void {
  installCurrentAccessor();
  for (const name of TABLE_NAMES) {
    defineDelegate(
      name,
      () => (currentInstance().tables as unknown as Record<string, unknown>)[name.slice(2)],
      // Assigning a table replaces it on the CURRENT instance only, so code
      // that swaps a table wholesale (tests do) keeps its meaning without
      // putting state back on `globalThis`.
      (value) => {
        (currentInstance().tables as unknown as Record<string, unknown>)[name.slice(2)] = value;
      },
    );
  }
  for (const name of COUNTER_NAMES) {
    const key = name.slice("__".length);
    defineDelegate(
      name,
      () => (currentInstance().counters as unknown as Record<string, unknown>)[key],
      (value) => {
        (currentInstance().counters as unknown as Record<string, unknown>)[key] = value;
      },
    );
  }
}

/**
 * Public API for the page / the browser e2e: create, inspect and dispose
 * instances. The W4 isolation test drives exactly these.
 */
export function installInstanceApi(): void {
  installCurrentAccessor();
  (globalThis as any).__TAIRITSU_INSTANCES__ = { // ts-escape-ok: retires with L40
    create: () => createInstance(),
    dispose: (instance: TairitsuInstance) => disposeInstance(instance),
    current: () => currentInstance(),
    setCurrent: (instance: TairitsuInstance | null) => setCurrentInstance(instance),
    withInstance,
    claim: (instance: TairitsuInstance) => claimGuest(instance),
    release: (instance: TairitsuInstance) => releaseGuest(instance),
    active: () => activeGuest,
    list: () => listInstances(),
  };
}