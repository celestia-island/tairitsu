// W3 (fail-closed boot): the component's boot lifecycle, kept out of the HTML
// page so it can be driven by tests without a browser.
//
// Before W3 the page fished for an entry point by name — a depth-3 walk over
// `run` / `main` / `init` / `start` — invoked whatever it found, then reported
// `data-tairitsu-ready=hydrated` whether or not anything had actually run. A
// component with no entry, or whose entry threw, produced a "ready" page.
//
// The contract here is explicit:
//
//   * the entry is one of ENTRY_NAMES, resolved from the four documented
//     export surfaces only (no recursive probing);
//   * the resolved name is logged, so a build's entry is auditable;
//   * no entry = BootError('no-entry') — a hard failure, never a blank page;
//   * an entry that throws = BootError('entry-failed') with the cause kept;
//   * the caller's `surface.ready()` runs ONLY after the entry returned, and
//     `surface.fail()` runs instead on either failure.

/** Export names a guest may use as its boot entry, most specific first. */
export const ENTRY_NAMES = ['tairitsu_component_bootstrap', 'run', 'main', 'init'];

/** A boot failure with a machine-readable kind (`no-entry` / `entry-failed`). */
export class BootError extends Error {
  constructor(kind, message, cause) {
    super(message);
    this.name = 'BootError';
    this.kind = kind;
    this.cause = cause;
  }
}

/** The four documented shapes a jco wrapper / module instance exposes. */
const exportSurfaces = (result) => [
  result,
  result && result.exports,
  result && result.instance && result.instance.exports,
  result && result.instance,
];

/**
 * Resolve the guest entry from `result`: `{ name, fn }`, or `null` when the
 * component exports none of ENTRY_NAMES.
 */
export function resolveEntry(result) {
  for (const name of ENTRY_NAMES) {
    for (const surface of exportSurfaces(result)) {
      const fn = surface ? surface[name] : undefined;
      if (typeof fn === 'function') return { name, fn };
    }
  }
  return null;
}

/**
 * Run the component's boot lifecycle.
 *
 * `surface` is the page adapter: `ready(entryName)` and `fail(kind, error)`.
 * `log` defaults to the console (the page passes its own so the entry choice
 * lands in the browser console).
 *
 * Resolves `{ entry }` after `surface.ready(...)` ran; throws a BootError
 * after `surface.fail(...)` ran.
 */
export async function bootComponent({ result, surface, log = console }) {
  const entry = resolveEntry(result);
  if (!entry) {
    const error = new BootError(
      'no-entry',
      'component exports none of the boot entries: ' + ENTRY_NAMES.join(', '),
    );
    if (log && typeof log.error === 'function') log.error('[tairitsu] boot failed:', error.message);
    surface.fail('no-entry', error);
    throw error;
  }

  if (log && typeof log.info === 'function') {
    log.info('[tairitsu] boot entry: ' + entry.name);
  }

  try {
    await entry.fn();
  } catch (cause) {
    const error = new BootError(
      'entry-failed',
      'boot entry ' + entry.name + ' threw: ' + (cause && cause.message ? cause.message : cause),
      cause,
    );
    if (log && typeof log.error === 'function') log.error('[tairitsu] boot failed:', error.message);
    surface.fail('entry-failed', error);
    throw error;
  }

  surface.ready(entry.name);
  return { entry: entry.name };
}