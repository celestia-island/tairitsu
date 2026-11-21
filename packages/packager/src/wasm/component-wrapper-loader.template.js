// W3 (fail-closed boot): the wrapper loader NEVER swallows an instantiation
// failure. Two instantiate ABI shapes are still tried — the wrapper's own
// `instantiate(imports)` and the jco-style `instantiate(resolveCoreModule,
// imports)` — because both ship in the wild, but the moment a wrapper module
// is found and BOTH shapes fail, the collected failure is thrown. Before W3
// each shape's `catch` discarded the error and the loader continued, so a
// broken component surfaced as a blank page instead of an error.

const CANDIDATES = [
  './component-wrapper/__WASM_STEM__.js',
  './component-wrapper/index.js',
];

/** One line per failed attempt: which file, which ABI shape, what happened. */
const describeAttempts = (attempts) =>
  attempts
    .map(
      (attempt) =>
        `${attempt.path} [${attempt.form}]: ${
          attempt.error && attempt.error.message ? attempt.error.message : attempt.error
        }`,
    )
    .join(' | ');

export async function instantiateWithWrapper(imports = {}) {
  const attempts = [];

  for (const path of CANDIDATES) {
    const probeUrl = new URL(path, import.meta.url);

    // Candidate selection, not error swallowing: a wrapper that is not there
    // (or answers non-2xx) is simply not a candidate. The failure is still
    // recorded so the terminal error can explain every path that was tried.
    let probe;
    try {
      probe = await fetch(probeUrl, { method: 'HEAD' });
    } catch (error) {
      attempts.push({ path, form: 'probe', error });
      continue;
    }
    if (!probe.ok) {
      attempts.push({ path, form: 'probe', error: new Error(`HTTP ${probe.status}`) });
      continue;
    }

    // A module that exists but cannot be imported is a hard failure: it is
    // the wrapper we were told to load.
    const mod = await import(path);
    const instantiate = mod.instantiate || mod.default || mod.init;
    if (typeof instantiate !== 'function') {
      return mod;
    }

    try {
      return await instantiate(imports);
    } catch (firstError) {
      attempts.push({ path, form: 'instantiate(imports)', error: firstError });
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
    } catch (secondError) {
      attempts.push({ path, form: 'instantiate(resolveCoreModule, imports)', error: secondError });
      // Fail closed: this wrapper exists, was imported, and could not be
      // instantiated by either ABI shape. Continuing to the next candidate
      // (or reporting success) is exactly the behaviour W3 removes.
      throw new Error(
        'Component wrapper failed to instantiate — ' + describeAttempts(attempts),
      );
    }
  }

  throw new Error(
    'Component wrapper not found: expected a transpiled wrapper under '
      + './component-wrapper/. '
      + (attempts.length ? `Attempts: ${describeAttempts(attempts)}` : 'No candidate answered.'),
  );
}