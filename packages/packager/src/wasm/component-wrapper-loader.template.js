const CANDIDATES = [
  './component-wrapper/__WASM_STEM__.js',
  './component-wrapper/index.js',
];

export async function instantiateWithWrapper(imports = {}) {
  let lastError = null;

  for (const path of CANDIDATES) {
    try {
      const probeUrl = new URL(path, import.meta.url);
      const probe = await fetch(probeUrl, { method: 'HEAD' });
      if (!probe.ok) {
        continue;
      }

      const mod = await import(path);
      const instantiate = mod.instantiate || mod.default || mod.init;
      if (typeof instantiate !== 'function') {
        return mod;
      }

      try {
        return await instantiate(imports);
      } catch (_e1) {
      }

      try {
        return await instantiate(async (modulePath) => {
          const resolved = new URL(modulePath, import.meta.url);
          const response = await fetch(resolved);
          if (!response.ok) {
            throw new Error(`Failed to fetch core module: ${modulePath}`);
          }
          return WebAssembly.compileStreaming(response);
        }, imports);
      } catch (_e2) {
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
