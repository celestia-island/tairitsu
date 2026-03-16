type HostImportResult =
  | object
  | string
  | number
  | boolean
  | bigint
  | null
  | undefined
  | void
  | Promise<object | string | number | boolean | bigint | null | undefined | void>;

type WrapperImports = Record<string, Record<string, (...args: never[]) => HostImportResult>>;

type WrapperInstantiate = {
  (imports: WrapperImports): Promise<object> | object;
  (
    loadCoreModule: (modulePath: string) => Promise<WebAssembly.Module>,
    imports: WrapperImports,
  ): Promise<object> | object;
};

type WrapperModule = {
  instantiate?: WrapperInstantiate;
  default?: WrapperInstantiate;
  init?: WrapperInstantiate;
};

const CANDIDATES = [
  './component-wrapper/__WASM_STEM__.js',
  './component-wrapper/index.js',
];

export async function instantiateWithWrapper(imports: WrapperImports = {}) {
  let lastError: Error | DOMException | TypeError | string | null = null;

  // Probe and load candidates in order, stopping at the first working entry.
  // This avoids noisy 404s when the preferred named wrapper file exists.
  for (const path of CANDIDATES) {
    try {
      const probeUrl = new URL(path, import.meta.url);
      const probe = await fetch(probeUrl, { method: 'HEAD' });
      if (!probe.ok) {
        continue;
      }

      const mod = await import(path) as WrapperModule;
      const instantiate = mod.instantiate || mod.default || mod.init;
      if (typeof instantiate !== 'function') {
        // Some transpilers emit self-initializing modules (top-level await)
        // with no explicit instantiate export. Import success means ready.
        return mod;
      }

      try {
        return await instantiate(imports);
      } catch (_e1) {
        // Try the next common signature below.
      }

      try {
        return await instantiate(async (modulePath: string) => {
          const resolved = new URL(modulePath, import.meta.url);
          const response = await fetch(resolved);
          if (!response.ok) {
            throw new Error(`Failed to fetch core module: ${modulePath}`);
          }
          return WebAssembly.compileStreaming(response);
        }, imports);
      } catch (_e2) {
        // Keep the outer import/instantiate error for diagnostics.
      }
    } catch (error) {
      lastError = error instanceof Error ? error : String(error);
    }
  }

  throw new Error(
    'Component wrapper not found or could not be initialized. '
    + 'Expected a transpiled wrapper under ./component-wrapper/. '
    + (lastError ? `Last error: ${lastError}` : '')
  );
}