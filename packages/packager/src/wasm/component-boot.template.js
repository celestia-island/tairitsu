// W3 (fail-closed boot): the component's boot lifecycle, kept out of the HTML
// page so it can be driven by tests without a browser.
//
// Before W3 the page fished for an entry point by name — a depth-3 walk over
// `run` / `main` / `init` / `start` — invoked whatever it found, then reported
// `data-tairitsu-ready=hydrated` whether or not anything had actually run.
//
// The contract here is explicit, and it matches what a REAL build produces: a
// tairitsu component's WIT world exports the `lifecycle` interface, so a jco
// wrapper namespace carries the entry at `lifecycle.start` (the transpiled
// form may also namespace it as `tairitsu-browser:full/lifecycle@0.2.0`).
// The legacy top-level names stay supported for hand-written wrappers and for
// parity with the SSR host, which accepts `tairitsu_component_bootstrap`,
// `run`, `_start` and `start` (`packages/ssr/src/lib.rs`).
//
//   * the entry is resolved from ENTRY_MANIFEST only — no recursive probing;
//   * the resolved name (and its kind) is logged, so a build's entry is
//     auditable;
//   * no entry = BootError('no-entry') — a hard failure, never a blank page;
//   * an entry that throws = BootError('entry-failed') with the cause kept;
//   * the caller's `surface.ready()` runs ONLY after the entry returned, and
//     `surface.fail()` runs instead on either failure.

/**
 * The documented boot entries, most specific first.
 *
 * `top-level` entries live on the instance/exports; `wit` entries are the
 * component's own WIT exports (one nesting level, exactly as jco emits them).
 * `run` / `main` / `init` are the names the browser page has always accepted;
 * `_start` / `start` are the WASI-style entries the SSR host also accepts, so
 * both hosts agree on what a component's boot entry is.
 */
export const ENTRY_MANIFEST = [
  { path: ["tairitsu_component_bootstrap"], kind: "top-level" },
  { path: ["run"], kind: "top-level" },
  { path: ["main"], kind: "top-level" },
  { path: ["init"], kind: "top-level" },
  { path: ["_start"], kind: "top-level" },
  { path: ["start"], kind: "top-level" },
  { path: ["lifecycle", "start"], kind: "wit" },
  { path: ["tairitsu-browser:full/lifecycle@0.2.0", "start"], kind: "wit" },
];

/** Human-readable entry list for the "no entry" error. */
export const ENTRY_NAMES = ENTRY_MANIFEST.map((entry) => entry.path.join("."));

/** A boot failure with a machine-readable kind (`no-entry` / `entry-failed`). */
export class BootError extends Error {
  constructor(kind, message, cause) {
    super(message);
    this.name = "BootError";
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

/** Follow `path` from one surface; a non-function leaf is not an entry. */
function follow(surface, path) {
  let cursor = surface;
  for (const key of path) {
    // Own properties only: the pre-W3 walker used Object.entries, and a
    // polluted Object.prototype must not be able to mint a boot entry.
    if (!cursor || typeof cursor !== "object" || !Object.hasOwn(cursor, key)) return null;
    cursor = cursor[key];
  }
  return typeof cursor === "function" ? cursor : null;
}

/**
 * Resolve the guest entry from `result`: `{ name, kind, fn }`, or `null` when
 * the component matches none of ENTRY_MANIFEST.
 */
export function resolveEntry(result) {
  for (const entry of ENTRY_MANIFEST) {
    for (const surface of exportSurfaces(result)) {
      const fn = follow(surface, entry.path);
      if (fn) return { name: entry.path.join("."), kind: entry.kind, fn };
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
 * Resolves `{ entry, entryKind }` after `surface.ready(...)` ran; throws a
 * BootError after `surface.fail(...)` ran.
 */
export async function bootComponent({ result, surface, log = console }) {
  const entry = resolveEntry(result);
  if (!entry) {
    const error = new BootError(
      "no-entry",
      "component exports none of the boot entries: " + ENTRY_NAMES.join(", "),
    );
    if (log && typeof log.error === "function") log.error("[tairitsu] boot failed:", error.message);
    surface.fail("no-entry", error);
    throw error;
  }

  if (log && typeof log.info === "function") {
    log.info("[tairitsu] boot entry: " + entry.name + " (" + entry.kind + ")");
  }

  try {
    await entry.fn();
  } catch (cause) {
    const error = new BootError(
      "entry-failed",
      "boot entry " + entry.name + " threw: " + (cause && cause.message ? cause.message : cause),
      cause,
    );
    if (log && typeof log.error === "function") log.error("[tairitsu] boot failed:", error.message);
    surface.fail("entry-failed", error);
    throw error;
  }

  surface.ready(entry.name);
  return { entry: entry.name, entryKind: entry.kind };
}