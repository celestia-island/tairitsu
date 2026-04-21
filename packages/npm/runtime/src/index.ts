export interface BootOptions {
  componentUrl: string;
  wrapperUrl?: string;
  appRootId?: string;
  glueModules?: string[];
  components?: Record<string, string>;
  glueImports?: Record<string, Record<string, unknown>>;
}

type WasmExports = Record<string, unknown>;
type WasmImports = Record<string, Record<string, unknown>>;

const componentCache = new Map<string, unknown>();

export async function boot(options: BootOptions): Promise<unknown> {
  const {
    componentUrl,
    wrapperUrl,
    appRootId = "app",
  } = options;

  document.getElementById(appRootId);

  if (options.components) {
    await preloadComponents(options.components);
  }

  let result: unknown;

  try {
    if (wrapperUrl) {
      const mod = await import(wrapperUrl);
      const instantiate = (mod as any).instantiate || (mod as any).default || (mod as any).init;
      if (typeof instantiate === "function") {
        result = await instantiate(buildImports(options.glueImports));
      } else {
        result = mod;
      }
    } else {
      const response = await fetch(componentUrl);
      const bytes = await response.arrayBuffer();
      const magic = new Uint8Array(bytes, 0, 8);
      const isComponent =
        magic[0] === 0x00 &&
        magic[1] === 0x61 &&
        magic[2] === 0x73 &&
        magic[3] === 0x6d &&
        magic[4] === 0x0d &&
        magic[5] === 0x00 &&
        magic[6] === 0x01 &&
        magic[7] === 0x00;

      if (isComponent && typeof (WebAssembly as any).Component === "function") {
        const Component = (WebAssembly as any).Component;
        const component = new Component(bytes);
        result = await (WebAssembly as any).instantiate(component, buildImports(options.glueImports));
      } else {
        const module = await WebAssembly.compile(bytes);
        const instantiated = await WebAssembly.instantiate(module, buildImports(options.glueImports) as WebAssembly.Imports);
        result = instantiated;
      }
    }
  } catch (err) {
    console.error("[tairitsu] Component loading failed:", err);
    throw err;
  }

  const g = globalThis as any;
  if (g.__setWasmExports && result) {
    g.__setWasmExports(result);
  }

  await tryInvokeBootExports(result);

  return result;
}

export async function loadComponent(name: string, wrapperUrl: string, glueImports?: Record<string, Record<string, unknown>>): Promise<WasmExports> {
  if (componentCache.has(name)) {
    return componentCache.get(name) as WasmExports;
  }

  const mod = await import(wrapperUrl);
  const instantiate = (mod as any).instantiate || (mod as any).default || (mod as any).init;

  if (typeof instantiate !== "function") {
    throw new Error(`[tairitsu] Component "${name}" has no instantiate function`);
  }

  const result = await instantiate(buildImports(glueImports));
  componentCache.set(name, result);
  return result as WasmExports;
}

export async function loadComponentFromCdn(name: string, version: string): Promise<WasmExports> {
  const scope = name.startsWith("@celestia/") ? name : `@celestia/${name}`;
  const wrapperUrl = `https://esm.sh/${scope}@${version}`;
  return loadComponent(name, wrapperUrl);
}

async function preloadComponents(components: Record<string, string>): Promise<void> {
  const entries = Object.entries(components);
  await Promise.all(
    entries.map(([name, url]) =>
      loadComponent(name, url).catch((err) => {
        console.warn(`[tairitsu] Failed to preload component "${name}":`, err);
      })
    )
  );
}

function buildImports(extraImports?: Record<string, Record<string, unknown>>): WasmImports {
  const imports: WasmImports = {};
  const g = globalThis as any;
  if (g.__TAIRITSU_GLUE && g.__TAIRITSU_GLUE.INTERFACES) {
    for (const [shortName, exp] of Object.entries(g.__TAIRITSU_GLUE.INTERFACES as Record<string, Record<string, unknown>>)) {
      const ifaceName = shortName.replace("@tairitsu-glue/", "");
      const fullName = `tairitsu-browser:full/${ifaceName}@0.2.0`;
      imports[fullName] = exp;
    }
  }

  if (extraImports) {
    for (const [key, value] of Object.entries(extraImports)) {
      if (!imports[key]) {
        imports[key] = value;
      } else {
        Object.assign(imports[key], value);
      }
    }
  }

  return imports;
}

async function tryInvokeBootExports(result: unknown): Promise<boolean> {
  const normalizeBootName = (name: string): string | null => {
    const lowered = String(name || "").toLowerCase();
    if (lowered === "run") return "run";
    if (lowered === "main") return "main";
    if (lowered === "init") return "init";
    if (lowered === "start") return "start";
    return null;
  };

  const seenObjects = new Set();
  const discovered: Array<{ kind: string; fn: () => unknown }> = [];

  const collect = (obj: unknown, depth = 0) => {
    if (!obj || typeof obj !== "object" || depth > 3) return;
    if (seenObjects.has(obj)) return;
    seenObjects.add(obj);

    for (const [name, value] of Object.entries(obj as Record<string, unknown>)) {
      if (typeof value !== "function") continue;
      const kind = normalizeBootName(name);
      if (!kind) continue;
      if (seenObjects.has(value)) continue;
      seenObjects.add(value);
      discovered.push({ kind, fn: value as () => unknown });
    }

    for (const [, value] of Object.entries(obj as Record<string, unknown>)) {
      if (value && typeof value === "object") {
        collect(value, depth + 1);
      }
    }
  };

  const r = result as any;
  const targets = [result, r?.instance, r?.exports, r?.instance?.exports];

  for (const target of targets) {
    collect(target);
    if (target && (target as any).exports) collect((target as any).exports);
  }

  for (const preferred of ["run", "main", "init"]) {
    for (const entry of discovered) {
      if (entry.kind === preferred) {
        await entry.fn();
        return true;
      }
    }
  }

  const fallbackStart = discovered.find((entry) => entry.kind === "start");
  if (fallbackStart) {
    await fallbackStart.fn();
    return true;
  }

  return false;
}
