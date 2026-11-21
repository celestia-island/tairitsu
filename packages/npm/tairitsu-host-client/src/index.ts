// Loading a tairitsu component from a LIBRARY (not a page).
//
// The page lane loads a component with `boot.js` + `component-wrapper-loader.js`,
// both emitted into a dist by the packager. A library — hikari's QR encoder is
// the first — has no dist of its own: it ships a component artifact and needs a
// callable surface. This module is that surface, and it keeps the four
// guarantees the page lane already established:
//
//   * fail-closed load (W3): a wrapper that cannot be found or instantiated is
//     an error, never a blank/partial surface;
//   * capability enforcement (W6): the imports handed to the component are built
//     from the declaration, and a declared-but-unprovided capability is an error
//     *before* instantiate; an import the wrapper uses but the declaration does
//     not name is refused by the glue module's own guard (W5/W6);
//   * the W4 lifecycle: the surface carries `stop()` and `whenIdle()` instead of
//     hiding them — a library consumer gets the same cooperative stop the page
//     does;
//   * a synchronous call surface: `loadComponent` is async, `component.call`
//     is sync. That is the contract the sync facade (hikari's `qrMatrix`) needs.

export interface CapabilityDeclaration {
  /** Glue interfaces the component may use (`document`, `element`, …). */
  interfaces?: string[];
  /** WASI shim modules the component may use (`io`, `clocks`, …). */
  wasiModules?: string[];
}

export interface LoadComponentOptions {
  /** The component's wrapper module (packaged next to the wasm). */
  wrapper?: string | URL;
  /** Capability declaration; defaults to "no glue, no wasi". */
  capabilities?: CapabilityDeclaration;
  /**
   * Glue interface implementations, keyed by bare interface name
   * (`document`, `element`, …). A declaration that names an interface with no
   * provider here is refused before instantiate.
   */
  glue?: Record<string, unknown>;
  /** WASI shim modules, keyed by bare module name (`io`, `clocks`, …). */
  wasi?: Record<string, unknown>;
  /** A wrapper the consumer imported itself (bundler-safe route). */
  module?: Record<string, unknown>;
  /** Override `fetch`/`import` for tests and non-browser hosts. */
  loaders?: {
    importModule?: (url: string) => Promise<Record<string, unknown>>;
  };
}

export interface HostComponent {
  /** The component's exports, exactly as the wrapper produced them. */
  readonly exports: Record<string, unknown>;
  /** Call one export synchronously. Throws once the component is stopped. */
  call<T = unknown>(name: string, ...args: unknown[]): T;
  /** Cooperative stop (W4 semantics): resolves once in-flight work has drained. */
  stop(): Promise<void>;
  /** Resolves when no call is in flight. */
  whenIdle(): Promise<void>;
  /** True once `stop()` ran. */
  readonly stopped: boolean;
}

/**
 * Publish the declaration this realm's glue modules read.
 *
 * Call this BEFORE a static `import` of the wrapper: the emitted glue modules
 * check the declaration when they are evaluated, and an absent declaration lets
 * them pass (`if (!capabilities) return true`). `loadComponent` does this for a
 * dynamic import, and restores the previous value once the module is evaluated.
 *
 * Returns the previous value so a caller can restore it.
 */
export function publishCapabilities(
  capabilities: CapabilityDeclaration,
): CapabilityDeclaration | undefined {
  const realm = globalThis as unknown as {
    __TAIRITSU_CAPABILITIES__?: CapabilityDeclaration;
  };
  const previous = realm.__TAIRITSU_CAPABILITIES__;
  realm.__TAIRITSU_CAPABILITIES__ = {
    interfaces: capabilities.interfaces ?? [],
    wasiModules: capabilities.wasiModules ?? [],
  };
  return previous;
}

declare global {
  // eslint-disable-next-line no-var
  var __TAIRITSU_CAPABILITIES__: CapabilityDeclaration | undefined;
  // eslint-disable-next-line no-var
  var __TAIRITSU_GLUE__: { INTERFACES: Record<string, unknown> } | undefined;
}

export class HostClientError extends Error {
  readonly kind: string;
  constructor(kind: string, message: string) {
    super(`${kind}: ${message}`);
    this.name = "HostClientError";
    this.kind = kind;
  }
}

const normalizeInterface = (name: string): string =>
  String(name || "")
    .replace(/^@tairitsu-glue\//, "")
    .replace(/^tairitsu-browser:full\//, "")
    .replace(/@\d+\.\d+\.\d+$/, "");

const normalizeWasi = (name: string): string => String(name || "").trim();

// The import keys a component actually asks for — the page lane hardcodes the
// same names (`boot.template.js`), so this mirrors it rather than inventing.
const WASI_IMPORT_KEYS: Record<string, string[]> = {
  io: ["wasi:io/error@0.2.0", "wasi:io/poll@0.2.0", "wasi:io/streams@0.2.0"],
  clocks: ["wasi:clocks/monotonic-clock@0.2.0", "wasi:clocks/wall-clock@0.2.0"],
  random: ["wasi:random/random@0.2.0"],
  cli: [
    "wasi:cli/environment@0.2.0",
    "wasi:cli/exit@0.2.0",
    "wasi:cli/stdin@0.2.0",
    "wasi:cli/stdout@0.2.0",
    "wasi:cli/stderr@0.2.0",
  ],
  filesystem: ["wasi:filesystem/preopens@0.2.0"],
  sockets: [
    "wasi:sockets/tcp@0.2.0",
    "wasi:sockets/udp@0.2.0",
    "wasi:sockets/network@0.2.0",
    "wasi:sockets/instance-network@0.2.0",
  ],
};

const hasOwn = (target: object, key: string): boolean =>
  Object.prototype.hasOwnProperty.call(target, key);

/** The shapes a wrapper may carry its exports in (page lane parity). */
const pickExport = (instance: Record<string, unknown>, name: string): unknown => {
  const candidates = [
    instance,
    instance.exports,
    instance.instance,
    (instance.instance as Record<string, unknown> | undefined)?.exports,
    instance.lifecycle,
    instance["tairitsu-browser:full/lifecycle@0.2.0"],
  ];
  for (const holder of candidates) {
    if (holder && typeof holder === "object" && hasOwn(holder, name)) {
      return (holder as Record<string, unknown>)[name];
    }
  }
  return undefined;
};

const KNOWN_WASI = ["cli", "clocks", "filesystem", "io", "random", "sockets"];

/** Build the imports a component is allowed to see (W6, mirrored). */
export function buildImports(
  capabilities: CapabilityDeclaration,
  glue: Record<string, unknown> = {},
  wasi: Record<string, unknown> = {},
): Record<string, unknown> {
  const imports: Record<string, unknown> = {};

  const declaredInterfaces = (capabilities.interfaces ?? []).map(normalizeInterface);
  const missing = declaredInterfaces.filter((name) => !hasOwn(glue, name));
  if (missing.length > 0) {
    // Declared but the host has no provider: refuse before instantiate, exactly
    // like the page lane does.
    throw new HostClientError(
      "capability-denied",
      `the component declares capabilities the host does not provide: ${missing.sort().join(", ")}`,
    );
  }
  for (const name of declaredInterfaces) {
    imports[`tairitsu-browser:full/${name}@0.2.0`] = glue[name];
  }

  const declaredWasi = (capabilities.wasiModules ?? []).map(normalizeWasi);
  const unknownWasi = declaredWasi.filter((name) => !KNOWN_WASI.includes(name));
  if (unknownWasi.length > 0) {
    throw new HostClientError(
      "capability-denied",
      `the component declares WASI modules the host does not provide: ${unknownWasi.sort().join(", ")}`,
    );
  }
  const missingWasi = declaredWasi.filter((name) => !hasOwn(wasi, name));
  if (missingWasi.length > 0) {
    throw new HostClientError(
      "capability-denied",
      `the component declares WASI modules the host does not provide: ${missingWasi.sort().join(", ")}`,
    );
  }
  for (const name of declaredWasi) {
    for (const key of WASI_IMPORT_KEYS[name] ?? []) imports[key] = wasi[name];
  }

  return imports;
}

interface Instantiable {
  instantiate?: (imports: Record<string, unknown>) => unknown;
  default?: (imports: Record<string, unknown>) => unknown;
  init?: (imports: Record<string, unknown>) => unknown;
}

/**
 * Load a component and return its synchronous call surface.
 *
 * Both instantiate ABI shapes are accepted (the wrapper's own
 * `instantiate(imports)` and the jco-style `instantiate(resolveCoreModule,
 * imports)`), because both ship in the wild and the page loader supports both.
 */

/**
 * Build the surface from a module the consumer has ALREADY imported.
 *
 * This is the synchronous door: a wrapper in the consumer's static module graph
 * has its exports callable immediately, which is what a sync facade needs.
 * Capability enforcement is the same as `loadComponent` (the declaration is
 * checked before anything is handed over); the lifecycle is identical.
 *
 * ```ts
 * import * as wrapper from "./component/wrapper.js";
 * publishCapabilities({ interfaces: [], wasiModules: [] });
 * const component = createSurface(wrapper, { interfaces: [], wasiModules: [] });
 * ```
 *
 * A module that needs instantiating (an `instantiate`/`default`/`init` export)
 * must go through `loadComponent` instead — this door is for namespaces that are
 * already a component.
 */
export function createSurface(
  module: Record<string, unknown>,
  capabilities: CapabilityDeclaration = {},
  glue: Record<string, unknown> = {},
  wasi: Record<string, unknown> = {},
): HostComponent {
  const instance = ensureSurface(module, "the wrapper namespace");
  if (
    typeof instance.instantiate === "function" ||
    typeof instance.default === "function" ||
    typeof instance.init === "function"
  ) {
    throw new HostClientError(
      "needs-loading",
      "this wrapper needs instantiation; use loadComponent (or loadComponent({ module })) instead",
    );
  }
  // Same gate as the async door: refuse a declaration the host cannot satisfy
  // before handing out any call surface.
  buildImports(capabilities, glue, wasi);
  return makeSurface(instance);
}

/**
 * Build the call surface for an already-instantiated component.
 *
 * Synchronous by construction — this is what a sync facade over a component
 * needs (the wrapper is in the consumer's module graph, so its exports are
 * callable immediately). `loadComponent` uses it after instantiating.
 */
function ensureSurface(value: unknown, shape: string): Record<string, unknown> {
  if (value === null || typeof value !== "object") {
    throw new HostClientError(
      "instantiate-failed",
      `${shape} resolved to ${value === null ? "null" : typeof value} ` +
        "a component must resolve to an object of exports",
    );
  }
  return value as Record<string, unknown>;
};

function makeSurface(instance: Record<string, unknown>): HostComponent {
let stopped = false;
let inFlight = 0;
let idleWaiters: Array<() => void> = [];

const settleIdle = () => {
  const waiters = idleWaiters;
  idleWaiters = [];
  for (const resolve of waiters) resolve();
};

// Every holder `pickExport` can resolve must be behind the same gate, or
// `exports.<holder>.<fn>()` walks around `stop()`.
const guardedView = (holder: Record<string, unknown>): Record<string, unknown> =>
  new Proxy(holder, {
    get(target, property) {
      if (typeof property !== "string") return Reflect.get(target, property);
      // Resolve WITHIN this holder first: `exports.instance.encode` must call
      // the instance-shaped export, not the first holder that happens to carry
      // the same name.
      const own = hasOwn(target, property) ? Reflect.get(target, property) : undefined;
      if (typeof own === "function") {
        return (...args: unknown[]) => surface.invoke(property, own as (...a: unknown[]) => unknown, args);
      }
      if (own && typeof own === "object" && !Array.isArray(own)) {
        return guardedView(own as Record<string, unknown>);
      }
      const exported = pickExport(instance, property);
      if (typeof exported === "function") {
        return (...args: unknown[]) => surface.call(property, ...args);
      }
      return own;
    },
  });
const exportsView = guardedView(instance);

const invoke = <T = unknown>(
  name: string,
  resolved: (...args: unknown[]) => unknown,
  args: unknown[],
): T => {
  if (stopped) {
    throw new HostClientError("stopped", `this component was stopped; "${name}" is not callable`);
  }
  inFlight += 1;
  try {
    const value = resolved(...args);
    if (value !== null && typeof (value as { then?: unknown })?.then === "function") {
      void Promise.resolve(value as PromiseLike<unknown>).catch(() => {});
      throw new HostClientError(
        "async-export",
        `"${name}" returned a thenable; this client exposes a synchronous surface only`,
      );
    }
    return value as T;
  } finally {
    inFlight -= 1;
    if (inFlight === 0) settleIdle();
  }
};

const surface: HostComponent & {
  invoke: (name: string, resolved: (...a: unknown[]) => unknown, args: unknown[]) => unknown;
} = {
  exports: exportsView,
  invoke,
  get stopped() {
    return stopped;
  },
  call<T = unknown>(name: string, ...args: unknown[]): T {
    if (stopped) {
      throw new HostClientError("stopped", `this component was stopped; "${name}" is not callable`);
    }
    const resolved = pickExport(instance, name);
    if (typeof resolved !== "function") {
      throw new HostClientError("no-export", `the component exports no "${name}"`);
    }
    return invoke<T>(name, resolved as (...callArgs: unknown[]) => unknown, args);
  },
  async stop(): Promise<void> {
    // Mark first (no new calls), let the guest wind down its own state if it
    // exposes the cooperative hook, then drain.
    stopped = true;
    const guestStop = pickExport(instance, "stop");
    if (typeof guestStop === "function") {
      await (guestStop as () => Promise<void> | void)();
    }
    await surface.whenIdle();
  },
  whenIdle(): Promise<void> {
    if (inFlight === 0) return Promise.resolve();
    return new Promise<void>((resolve) => idleWaiters.push(resolve));
  },
};
return surface;
}

export async function loadComponent(options: LoadComponentOptions): Promise<HostComponent> {
  const { wrapper, capabilities = {}, glue = {}, wasi = {}, loaders } = options;
  const imports = buildImports(capabilities, glue, wasi);
  // W6's guard inside the emitted glue modules reads the declaration from the
  // realm that imports them — that is this one. A consumer that imports the
  // wrapper statically must call `declareCapabilities` BEFORE that import; a
  // dynamic import through this function is ordered for it.
  const previousDeclaration = publishCapabilities(capabilities);
  // The glue plane the W5 modules read. Providers passed here are visible to a
  // statically imported glue module for the duration of the import.
  const previousGlue = globalThis.__TAIRITSU_GLUE__;
  globalThis.__TAIRITSU_GLUE__ = { INTERFACES: glue };

  const url = wrapper === undefined ? "<module>" : typeof wrapper === "string" ? wrapper : wrapper.href;
  const importModule =
    options.module !== undefined
      ? async () => options.module as Record<string, unknown>
      : (loaders?.importModule ?? ((target: string) => import(/* @vite-ignore */ target)));

  let module: Record<string, unknown>;
  try {
    module = await importModule(url);
  } catch (error) {
    if (previousDeclaration === undefined) delete globalThis.__TAIRITSU_CAPABILITIES__;
    else globalThis.__TAIRITSU_CAPABILITIES__ = previousDeclaration;
    globalThis.__TAIRITSU_GLUE__ = previousGlue;
    // A glue module refusing an undeclared interface is a capability failure, not
    // a missing wrapper — keep the two diagnosable.
    const message = String((error as Error)?.message ?? error);
    const kind = /capability denied|is not declared/.test(message)
      ? "capability-denied"
      : "wrapper-not-found";
    throw new HostClientError(kind, `could not load the component wrapper ${url}: ${message}`);
  }

  // The modules a wrapper statically imports have been evaluated by now.
  if (previousDeclaration === undefined) delete globalThis.__TAIRITSU_CAPABILITIES__;
  else globalThis.__TAIRITSU_CAPABILITIES__ = previousDeclaration;
  globalThis.__TAIRITSU_GLUE__ = previousGlue;

  const candidate =
    (module.instantiate as Instantiable["instantiate"]) ??
    (module.default as Instantiable["default"]) ??
    (module.init as Instantiable["init"]);


  let instance: Record<string, unknown>;
  if (typeof candidate === "function") {
    try {
      instance = ensureSurface(await candidate(imports), "instantiate(imports)");
    } catch (firstError) {
      // The jco shape: instantiate(resolveCoreModule, imports).
      try {
        instance = ensureSurface(await (
          candidate as unknown as (
            resolve: (path: string) => Promise<unknown>,
            imports: Record<string, unknown>,
          ) => unknown
        )(
          async (path: string) => {
            const resolved = new URL(path, url === "<module>" ? import.meta.url : url);
            const response = await fetch(resolved);
            if (!response.ok) {
              throw new Error(`failed to fetch core module ${path}: HTTP ${response.status}`);
            }
            return WebAssembly.compileStreaming(response);
          },
          imports,
        ), "instantiate(resolveCoreModule, imports)");
      } catch (secondError) {
        throw new HostClientError(
          "instantiate-failed",
          `the component wrapper ${url} could not be instantiated ` +
            `(instantiate(imports): ${(firstError as Error)?.message ?? firstError}; ` +
            `instantiate(resolveCoreModule, imports): ${(secondError as Error)?.message ?? secondError})`,
        );
      }
    }
  } else {
    // A wrapper that is a plain namespace of exports needs no instantiation.
    instance = ensureSurface(module, "the wrapper namespace");
  }

  return makeSurface(instance);
}