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
  wrapper: string | URL;
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
  /** True once `stop()` ran or the underlying instance was disposed. */
  readonly stopped: boolean;
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

const KNOWN_WASI = ["cli", "clocks", "filesystem", "io", "random", "sockets"];

/** Build the imports a component is allowed to see (W6, mirrored). */
export function buildImports(
  capabilities: CapabilityDeclaration,
  glue: Record<string, unknown> = {},
  wasi: Record<string, unknown> = {},
): Record<string, unknown> {
  const imports: Record<string, unknown> = {};

  const declaredInterfaces = (capabilities.interfaces ?? []).map(normalizeInterface);
  const missing = declaredInterfaces.filter((name) => !(name in glue));
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
  const missingWasi = declaredWasi.filter((name) => !(name in wasi));
  if (missingWasi.length > 0) {
    throw new HostClientError(
      "capability-denied",
      `the component declares WASI modules the host does not provide: ${missingWasi.sort().join(", ")}`,
    );
  }
  for (const name of declaredWasi) {
    imports[`wasi-${name}`] = wasi[name];
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
export async function loadComponent(options: LoadComponentOptions): Promise<HostComponent> {
  const { wrapper, capabilities = {}, glue = {}, wasi = {}, loaders } = options;
  const imports = buildImports(capabilities, glue, wasi);

  const url = typeof wrapper === "string" ? wrapper : wrapper.href;
  const importModule =
    loaders?.importModule ?? ((target: string) => import(/* @vite-ignore */ target));

  let module: Record<string, unknown>;
  try {
    module = await importModule(url);
  } catch (error) {
    throw new HostClientError(
      "wrapper-not-found",
      `could not load the component wrapper ${url}: ${(error as Error)?.message ?? error}`,
    );
  }

  const candidate =
    (module.instantiate as Instantiable["instantiate"]) ??
    (module.default as Instantiable["default"]) ??
    (module.init as Instantiable["init"]);

  let instance: Record<string, unknown>;
  if (typeof candidate === "function") {
    try {
      instance = (await candidate(imports)) as Record<string, unknown>;
    } catch (firstError) {
      // The jco shape: instantiate(resolveCoreModule, imports).
      try {
        instance = (await (
          candidate as unknown as (
            resolve: (path: string) => Promise<unknown>,
            imports: Record<string, unknown>,
          ) => unknown
        )(
          async (path: string) => {
            const resolved = new URL(path, url);
            const response = await fetch(resolved);
            if (!response.ok) {
              throw new Error(`failed to fetch core module ${path}: HTTP ${response.status}`);
            }
            return WebAssembly.compileStreaming(response);
          },
          imports,
        )) as Record<string, unknown>;
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
    instance = module;
  }

  let stopped = false;
  let inFlight = 0;
  let idleWaiters: Array<() => void> = [];

  const settleIdle = () => {
    const waiters = idleWaiters;
    idleWaiters = [];
    for (const resolve of waiters) resolve();
  };

  const surface: HostComponent = {
    exports: instance,
    get stopped() {
      return stopped;
    },
    call<T = unknown>(name: string, ...args: unknown[]): T {
      if (stopped) {
        throw new HostClientError("stopped", `this component was stopped; "${name}" is not callable`);
      }
      const candidate = instance[name];
      const resolved =
        typeof candidate === "function"
          ? candidate
          : (instance.lifecycle as Record<string, unknown> | undefined)?.[name];
      if (typeof resolved !== "function") {
        throw new HostClientError("no-export", `the component exports no "${name}"`);
      }
      inFlight += 1;
      try {
        return (resolved as (...callArgs: unknown[]) => T)(...args);
      } finally {
        inFlight -= 1;
        if (inFlight === 0) settleIdle();
      }
    },
    async stop(): Promise<void> {
      // Cooperative, like W4: mark first (no new calls), then drain.
      stopped = true;
      await surface.whenIdle();
    },
    whenIdle(): Promise<void> {
      if (inFlight === 0) return Promise.resolve();
      return new Promise<void>((resolve) => idleWaiters.push(resolve));
    },
  };
  return surface;
}