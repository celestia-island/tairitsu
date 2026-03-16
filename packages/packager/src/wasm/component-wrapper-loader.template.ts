const CANDIDATES = [
  './component-wrapper/__WASM_STEM__.js',
  './component-wrapper/index.js',
];

export async function instantiateWithWrapper(imports = {}) {
  const existingCandidates = [];

  for (const path of CANDIDATES) {
    try {
      const probeUrl = new URL(path, import.meta.url);
      const probe = await fetch(probeUrl, { method: 'HEAD' });
      if (probe.ok) {
        existingCandidates.push(path);
      }
    } catch (_probeErr) {
      // Ignore probe failures and let runtime error message guide next steps.
    }
  }

  if (existingCandidates.length === 0) {
    throw new Error(
      'No transpiled component wrapper entry found under ./component-wrapper/. '
      + 'Expected one of: ' + CANDIDATES.join(', ')
    );
  }

  let lastError = null;

  for (const path of existingCandidates) {
    try {
      const mod = await import(path);
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
        return await instantiate(async (modulePath) => {
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
      lastError = error;
    }
  }

  throw new Error(
    'Component wrapper not found or could not be initialized. '
    + 'Expected a transpiled wrapper under ./component-wrapper/. '
    + (lastError ? `Last error: ${lastError}` : '')
  );
}