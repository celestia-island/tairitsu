/**
 * `window.match-media` + `media-query-list`: the host side of a preference the
 * guest reads to decide whether to animate at all.
 *
 * Why this file exists
 * --------------------
 * `packages/web/wit/composed/_domain-css.wit` declares
 * `window.match-media: func(query: string) -> u64` and the whole
 * `media-query-list` interface, and `tairitsu-web`'s `wit_platform.rs` imports
 * both — but the browser-glue runtime exported neither `matchMedia` nor the
 * list ops. A component reading `(prefers-reduced-motion: reduce)` through the
 * WIT then got no implementation for the import, so hikari's AnimationProvider
 * (`packages/animation/src/prefers_reduced_motion.rs` →
 * `enabled: props.enabled && !reduced_motion`) saw `false` on every page and
 * ran the full animation set for users who asked it not to.
 *
 * `scripts/check_wit_surface.py` could not see it: it counts interfaces, and
 * `window` was wired. That is why the op-level gate
 * (`scripts/check_wit_ops.py`, `just wit-ops-check`) exists and why the last
 * block below drives it against fixtures instead of trusting it.
 *
 * The WIT declaration is the contract; nothing here invents a name the WIT does
 * not declare (`_domain-css.wit` declares get-media, get-matches, add-listener,
 * remove-listener, get-onchange, set-onchange — and nothing else).
 *
 * Run with: npm run test
 */

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { register } from "node:module";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

// The runtime sources use bundler-style extensionless imports; teach the ESM
// resolver the same trick so the tests can import the real modules.
register("./ts-resolve-hook.mjs", import.meta.url);

const HERE = dirname(fileURLToPath(import.meta.url));
const PKG = join(HERE, "..");
const REPO = join(PKG, "..", "..");
const WIT_DIR = join(REPO, "packages", "browser-worlds", "wit", "composed");
const WIT_MIRROR_DIR = join(REPO, "packages", "web", "wit", "composed");
const GATE = join(REPO, "scripts", "check_wit_ops.py");

// ---------------------------------------------------------------------------
// A MediaQueryList faithful to the DOM: `matches` recomputes from `evaluate()`
// on every read (so a test can flip the simulated preference the way CDP's
// Emulation.setEmulatedMedia does), and `change` listeners fire only on a flip.
// ---------------------------------------------------------------------------
class FakeMediaQueryList extends EventTarget {
  constructor(query, evaluate) {
    super();
    this.media = query;
    this._evaluate = evaluate;
    this._last = !!evaluate(query);
  }
  get matches() {
    this._last = !!this._evaluate(this.media);
    return this._last;
  }
  /** Simulate the browser noticing the preference changed. */
  flip() {
    const next = this.matches;
    this.dispatchEvent(Object.assign(new Event("change"), { matches: next, media: this.media }));
    return next;
  }
}

/**
 * Install the browser surface the runtime reads and return the controls.
 *
 * `media` is the object the fake `matchMedia` closes over: a test mutates
 * `media.reduce` to flip the simulated preference, exactly the way CDP's
 * `Emulation.setEmulatedMedia` flips the real one.
 *
 * The shared handle tables are reset too — these tests run against the page's
 * default instance, and a stale `onchange` binding from an earlier test would
 * make the next `set-onchange` assertion read the previous value.
 */
function installDom({ reduce = false } = {}) {
  const media = { reduce };
  const created = [];
  globalThis.window = {
    innerWidth: 1280,
    innerHeight: 720,
    matchMedia(query) {
      const list = new FakeMediaQueryList(
        query,
        (q) => q.includes("prefers-reduced-motion: reduce") && media.reduce,
      );
      created.push(list);
      return list;
    },
    getComputedStyle: () => ({ cssText: "" }),
  };
  globalThis.document = { createElement: () => ({}), body: {} };
  globalThis.__mediaQueryListHandles.clear();
  globalThis.__mediaQueryListListeners.clear();
  globalThis.__mediaQueryListOnchange.clear();
  return { media, created };
}

// The runtime reads its handle tables through the per-instance compat accessors.
// `handles.ts` is what installs them for real (`runtime/index.ts` imports it for
// side effects); production code never calls `installGlobalDelegates` directly.
// Importing it here — rather than re-installing by hand — keeps this suite on
// the same bootstrap the page runs, and it must happen BEFORE the first
// installDom(), because the accessors are what make
// `globalThis.__mediaQueryListHandles` resolve at all.
// MUT: bootstrap-manual-install (re-installing by hand instead of importing
// `handles.ts` leaves the tables undefined and every installDom() call throws,
// which is how this ordering was found).
await import(join(PKG, "src", "runtime", "handles.ts"));
const { installInstanceApi } = await import(join(PKG, "src", "runtime", "instance.ts"));
installInstanceApi();

installDom();

const { INTERFACES } = await import(join(PKG, "src", "runtime", "registry.ts"));
const window_exports = INTERFACES["@tairitsu-glue/window"];
const mediaQueryList_exports = INTERFACES["@tairitsu-glue/media-query-list"];
// The bridge's dispatch entry point, exported so the "does not escape" tests can
// drive it directly (`dispatchEvent` swallows listener throws asynchronously).
const { dispatchChange } = await import(join(PKG, "src", "runtime", "mediaQueryList.ts"));

/** The func names a WIT interface declares (single source = browser-worlds). */
function declaredFunctions(file, interfaceName) {
  const text = readFileSync(join(WIT_DIR, file), "utf8");
  const start = text.indexOf(`interface ${interfaceName} {`);
  assert.notEqual(start, -1, `${interfaceName} not found in ${file}`);
  const body = text.slice(start, text.indexOf("\n}", start));
  return [...body.matchAll(/^\s+([a-z][a-z0-9-]*): func\(/gm)].map((m) => m[1]);
}

const kebab = (name) => name.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

test("window: match-media is exported and declared (the op that went missing)", () => {
  assert.deepEqual(
    declaredFunctions("_domain-css.wit", "window").includes("match-media"),
    true,
    "the composed WIT must declare window.match-media — this suite pins the wiring to it",
  );
  assert.equal(
    typeof window_exports.matchMedia,
    "function",
    "window.matchMedia is required: a guest reading (prefers-reduced-motion: reduce) " +
      "has no other route to it, and a missing implementation made every such read false",
  );
});

test("window: every declared op of a wired interface maps to an export (no invention)", () => {
  // The interface-level gate cannot see this; the op-level one (driven at the
  // bottom of this file) can. Here the two interfaces this change touches are
  // checked against their own declaration.
  for (const [iface, witFile, exports] of [
    ["window", "_domain-css.wit", window_exports],
    ["media-query-list", "_domain-css.wit", mediaQueryList_exports],
  ]) {
    const declared = new Set(declaredFunctions(witFile, iface));
    for (const name of Object.keys(exports)) {
      assert.ok(
        declared.has(kebab(name)),
        `${iface}: exported ${name} is not declared by WIT as ${kebab(name)}`,
      );
    }
  }
  // The list's whole declared surface is wired (nothing silently dropped).
  const declared = declaredFunctions("_domain-css.wit", "media-query-list").map((n) => {
    const [head, ...rest] = n.split("-");
    return head + rest.map((p) => p[0].toUpperCase() + p.slice(1)).join("");
  });
  assert.deepEqual(
    declared.filter((n) => typeof mediaQueryList_exports[n] !== "function"),
    [],
    "media-query-list: declared in WIT but not wired in mediaQueryList.ts",
  );
});

test("match-media: handle names a live list, and both declared getters read it", () => {
  const { media, created } = installDom({ reduce: true });
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  assert.equal(typeof handle, "bigint", "the WIT result is a u64 handle");
  assert.equal(created.length, 1, "the host must build one MediaQueryList");
  assert.equal(created[0].media, "(prefers-reduced-motion: reduce)");

  assert.equal(mediaQueryList_exports.getMedia(handle), "(prefers-reduced-motion: reduce)");
  // The bug's user-visible signature: this used to be absent entirely.
  assert.equal(mediaQueryList_exports.getMatches(handle), true);
  assert.equal(typeof mediaQueryList_exports.getMatches(handle), "boolean");

  media.reduce = false;
  assert.equal(mediaQueryList_exports.getMatches(handle), false);
  media.reduce = true;
  assert.equal(mediaQueryList_exports.getMatches(handle), true, "matches must track the preference");
});

test("match-media: each call mints its own handle, and an unknown handle throws", () => {
  installDom();
  const a = window_exports.matchMedia("(min-width: 600px)");
  const b = window_exports.matchMedia("(min-width: 600px)");
  assert.notEqual(a, b, "two lists are two handles even for the same query");
  assert.equal(mediaQueryList_exports.getMedia(a), "(min-width: 600px)");

  assert.throws(
    () => mediaQueryList_exports.getMatches(999_999n),
    /MediaQueryList handle/,
    "a stale handle must fail loudly, not read as false",
  );
});

test("media-query-list: add-listener answers through the guest callback export", () => {
  const { media } = installDom({ reduce: false });
  const calls = [];
  globalThis.__wasmExports = {
    // The name jco actually emits. `jco types` on
    // `on-change: func(callback-id: u64, matches: bool)` yields
    // `export function onChange(callbackId: bigint, matches: boolean): void;`
    // — camelCase, because the kebab name has two segments. Stubbing
    // `on_change` here would have validated the provider's own misspelling
    // instead of the contract.
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      onChange(callbackId, matches) {
        calls.push([callbackId, matches]);
      },
    },
  };
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  const callbackId = 7n;
  mediaQueryList_exports.addListener(handle, callbackId);
  const list = globalThis.__mediaQueryListHandles.get(handle);

  // No change yet: nothing fires.
  assert.deepEqual(calls, []);

  media.reduce = true;
  assert.equal(mediaQueryList_exports.getMatches(handle), true);

  // Fire the browser's change event the way the DOM would.
  list.flip();
  assert.deepEqual(
    calls,
    [[7n, true]],
    "the guest's on-change must receive the callback id and the new matches value",
  );

  mediaQueryList_exports.removeListener(handle, callbackId);
  media.reduce = false;
  list.flip();
  assert.equal(calls.length, 1, "a removed listener must stop firing");
  delete globalThis.__wasmExports;
});

test("media-query-list: onchange round-trips through the declared u64 setter", () => {
  installDom({ reduce: false });
  const calls = [];
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      onChange(callbackId, matches) {
        calls.push([callbackId, matches]);
      },
    },
  };
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  assert.equal(mediaQueryList_exports.getOnchange(handle), 0n, "unset onchange reads as 0");

  mediaQueryList_exports.setOnchange(handle, 11n);
  assert.equal(mediaQueryList_exports.getOnchange(handle), 11n);

  const list = globalThis.__mediaQueryListHandles.get(handle);
  list.flip();
  assert.deepEqual(calls, [[11n, false]], "onchange must fire like a listener");

  // Replacing the slot must detach the previous callback, not stack it.
  mediaQueryList_exports.setOnchange(handle, 12n);
  list.flip();
  assert.deepEqual(calls, [[11n, false], [12n, false]], "the old onchange must stop firing");

  mediaQueryList_exports.setOnchange(handle, 0n);
  assert.equal(mediaQueryList_exports.getOnchange(handle), 0n);
  list.flip();
  assert.equal(calls.length, 2, "set-onchange 0 clears the slot");
  delete globalThis.__wasmExports;
});

test("media-query-list: the bridge binds the name jco emits (onChange), not on_change", () => {
  // The defect this pins: the bridge originally read `guest.on_change`, which
  // the transpiled wrapper never defines (jco camelCases multi-segment WIT
  // names). The result was a silent no-op — every change notification dropped
  // with no error — which is the same failure class as the missing
  // `match-media` op this PR fixes. A stub written to the provider's own
  // spelling would have gone green on the bug.
  // MUT: bridge-name-only-snake (reading only `on_change` reds the first arm).
  const { media } = installDom({ reduce: false });
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  const list = globalThis.__mediaQueryListHandles.get(handle);

  // The contract, in order of importance:
  // (a) the jco name MUST work — this is the one a real wrapper provides;
  const camel = [];
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      onChange: (id, m) => camel.push([id, m]),
    },
  };
  mediaQueryList_exports.addListener(handle, 1n);
  media.reduce = true;
  list.flip();
  assert.deepEqual(
    camel,
    [[1n, true]],
    "the jco camelCase export must receive the change — reading only on_change " +
      "silently drops every notification",
  );
  mediaQueryList_exports.removeListener(handle, 1n);

  // (b) there is NO snake fallback: jco emits `onChange` and nothing else, so a
  //     stub exposing `on_change` must receive nothing. (This used to assert the
  //     opposite — that the fallback delivers — which was only true in a world
  //     where a wrapper speaks snake, and none does.)
  const snake = [];
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      on_change: (id, m) => snake.push([id, m]),
    },
  };
  mediaQueryList_exports.addListener(handle, 2n);
  media.reduce = false;
  list.flip();
  assert.deepEqual(
    snake,
    [],
    "`on_change` is not a name the wrapper emits; dispatching to it would be a fallback that cannot run",
  );
  mediaQueryList_exports.removeListener(handle, 2n);

  // (c) neither name present: no throw, no delivery — the bridge is inert
  //     rather than crashing the DOM event loop.
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {},
  };
  mediaQueryList_exports.addListener(handle, 3n);
  media.reduce = true;
    // Drive the DISPATCH, not the DOM event: an `EventTarget` listener that
    // throws reports asynchronously (`uncaughtException`) and never reaches the
    // caller of `dispatchEvent`, so `doesNotThrow(() => list.flip())` is
    // vacuously true — a `catch` body changed to `throw` would still read green.
    // MUT: media-bridge-rethrows
  assert.doesNotThrow(() => dispatchChange(1n, true));
  delete globalThis.__wasmExports;
});

test("media-query-list: re-adding a callback id does not stack DOM listeners", () => {
  // `set-onchange` registers through `addListener`; a repeated id must REPLACE
  // its handler, not add a second one — a stacked listener would fire the guest
  // twice per change and leave one attachment untracked, so `removeListener`
  // could never detach it.
  // MUT: listener-stacking (removing the remove-then-add in `addListener`
  // makes the first assertion see two calls for one change).
  const { media } = installDom({ reduce: false });
  const calls = [];
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      onChange: (id, m) => calls.push([id, m]),
    },
  };
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  const list = globalThis.__mediaQueryListHandles.get(handle);
  mediaQueryList_exports.addListener(handle, 5n);
  mediaQueryList_exports.addListener(handle, 5n);

  media.reduce = true;
  list.flip();
  assert.deepEqual(calls, [[5n, true]], "the same id must fire exactly once per change");

  mediaQueryList_exports.removeListener(handle, 5n);
  media.reduce = false;
  list.flip();
  assert.equal(calls.length, 1, "removing the id must detach every attachment it made");
  delete globalThis.__wasmExports;
});

test("media-query-list: reclaim entries are one per (list, callback id)", () => {
  // Both failure modes at once, because a fix for one caused the other:
  //   * an entry PER CALL grows the set without bound (walked on every dispose);
  //   * an entry per (target, type) COLLAPSES distinct ids, so the second
  //     listener is never detached and a disposed instance keeps receiving
  //     change callbacks.
  // The identity is (list, callback id): 3 distinct ids -> 3 entries, and
  // re-registering one of them leaves the count unchanged.
  // MUT: reclaim-per-add | MUT: reclaim-collapsed-by-list
  installDom({ reduce: false });
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  const list = globalThis.__mediaQueryListHandles.get(handle);
  const before = countChangeReclaimables();

  mediaQueryList_exports.addListener(handle, 1n);
  mediaQueryList_exports.addListener(handle, 2n);
  mediaQueryList_exports.addListener(handle, 3n);
  assert.equal(
    countChangeReclaimables() - before,
    3,
    "each distinct callback id must have its own reclaim entry",
  );

  // Re-registering an existing id replaces its entry (no growth, no collapse).
  mediaQueryList_exports.addListener(handle, 2n);
  mediaQueryList_exports.setOnchange(handle, 3n);
  assert.equal(
    countChangeReclaimables() - before,
    3,
    "re-registering an id must replace its entry, not add or collapse one",
  );

  // Every id on this list must be individually reclaimable, or dispose leaves
  // a live handler behind (the test cannot dispose the page's default instance,
  // so it inspects the reclaim set directly; `R5` verified the detach count
  // independently with a throwaway instance: three ids -> three
  // removeEventListener calls, zero callbacks after dispose).
  // MUT: reclaim-collapsed-by-list (de-duplicating on (target, type) makes the
  // first assertion see one entry instead of three).
  const current = globalThis.__TAIRITSU_CURRENT__();
  const attachedForList = [...current.reclaimables.listeners].filter(
    (entry) => entry.target === list && entry.type === "change",
  );
  assert.equal(
    attachedForList.length,
    3,
    "every id attached to this list must be reclaimable, or dispose leaves a live handler",
  );
  assert.equal(
    new Set(attachedForList.map((entry) => entry.callbackId)).size,
    3,
    "the entries must be distinguishable by callback id",
  );
});
/** How many `change` listeners of a MediaQueryList are waiting on dispose. */
function countChangeReclaimables() {
  const reclaimables = globalThis.__TAIRITSU_CURRENT__().reclaimables.listeners;
  let count = 0;
  for (const entry of reclaimables) {
    if (entry.type === "change") count += 1;
  }
  return count;
}

test("media-query-list: a throwing guest callback does not escape into the event loop", () => {
  installDom({ reduce: false });
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      onChange() {
        throw new Error("guest bug");
      },
    },
  };
  const handle = window_exports.matchMedia("(prefers-reduced-motion: reduce)");
  mediaQueryList_exports.addListener(handle, 1n);
  const list = globalThis.__mediaQueryListHandles.get(handle);
  const originalError = console.error;
  console.error = () => {};
  try {
    // One component bug must not become an unhandled page error on every flip.
      // Drive the DISPATCH, not the DOM event: an `EventTarget` listener that
    // throws reports asynchronously (`uncaughtException`) and never reaches the
    // caller of `dispatchEvent`, so `doesNotThrow(() => list.flip())` is
    // vacuously true — a `catch` body changed to `throw` would still read green.
    // MUT: media-bridge-rethrows
  assert.doesNotThrow(() => dispatchChange(1n, true));
  } finally {
    console.error = originalError;
    delete globalThis.__wasmExports;
  }
});

test("match-media: handles are per instance and die with it (W4)", async () => {
  installDom();
  const { createInstance, disposeInstance, withInstance, currentInstance } = await import(
    join(PKG, "src", "runtime", "instance.ts")
  );
  const owner = currentInstance();
  const other = createInstance();

  const handle = withInstance(other, () => window_exports.matchMedia("(min-width: 1px)"));
  assert.equal(
    withInstance(other, () => mediaQueryList_exports.getMedia(handle)),
    "(min-width: 1px)",
  );
  assert.throws(
    () => withInstance(owner, () => mediaQueryList_exports.getMedia(handle)),
    /MediaQueryList handle/,
    "another instance must not resolve this list",
  );

  disposeInstance(other);
  assert.throws(
    () => mediaQueryList_exports.getMedia(handle),
    /MediaQueryList handle/,
    "a disposed instance's list handle must stop resolving",
  );
});

// ---------------------------------------------------------------------------
// The gate itself: driven against fixtures, so it is pinned to be able to FAIL.
// ---------------------------------------------------------------------------
test("check_wit_ops.py: the per-file binding SET is pinned, not just its size", () => {
  // The per-file floor asked "> 0 bindings", so `platformHelpers.ts` — 45 of them
  // — could lose 37 to a refactor the scan does not understand (`Reflect.get`,
  // a renamed local) and still read green. (The `88 (46)` of that round is
  // historical — the count is 47 bound now.) The dual with the base count is a
  // size; this is the identity: which files carry bridges,
  // and how many each. A bridge that stops being recognised must be re-declared
  // here, on purpose.
  // MUT: per-file-binding-set-drift
  // The pin lives IN THE GATE (`EXPECTED_BINDINGS_BY_FILE`), because `just ci`
  // runs `wit-ops-check` alone; this test proves the gate actually holds it by
  // losing one binding and requiring the gate to say so.
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = `        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];`;
    assert.ok(source.includes(needle), "the fixture must find a binding to lose");
    writeFileSync(
      path,
      source.replace(
        needle,
        `        const guest = Reflect.get(globalThis.__wasmExports ?? {}, "tairitsu-browser:full/idb-callbacks@0.2.0");`,
      ),
    );
  });
  assert.equal(rc, 1, "losing one binding must fail the GATE, not only the suite");
  assert.match(out, /platformHelpers\.ts: expected 45 guest-object bindings, the scan found 44/, `got: ${out}`);
});

test("check_wit_ops.py: the scan's own base count is pinned", () => {
  // The floor guards "the scan found nothing"; it does NOT guard "the scan found
  // a plausible number one digit short". A character class that admits nothing —
  // `[^"`$.]` excluded the `.` every callback key ends with (`@0.2.0`) — makes
  // every file report 0 bindings, which the floor tests describe as a PASSING
  // scenario (they assert that red). So the base count needs its own pin:
  // a scanner regression that quietly shrinks the report cannot hide behind a
  // test whose expectation is "0 bindings is an error".
  // MUT: callback-key-character-class
  const out = execFileSync("python3", [GATE, "--check"], { cwd: REPO, encoding: "utf8" });
  assert.match(
    out,
    /88 guest-export lookups \(47 bound\) named correctly/,
    `the scan base moved — update this pin only with a reason; got: ${out}`,
  );
});

test("check_wit_ops.py: green on the tree as it stands", () => {
  const out = execFileSync("python3", [GATE, "--check"], { cwd: REPO, encoding: "utf8" });
  assert.match(out, /WIT host surface ops OK/);
});

/** Run the gate against a throwaway copy with a source edit applied. */
function runGateAgainst(mutate) {
  const root = mkdtempSync(join(tmpdir(), "tairitsu-ops-gate-"));
  try {
    const copy = (from, to) => {
      mkdirSync(dirname(to), { recursive: true });
      writeFileSync(to, readFileSync(from));
    };
    copy(GATE, join(root, "scripts", "check_wit_ops.py"));
    copy(
      join(PKG, "src", "runtime", "registry.ts"),
      join(root, "packages", "browser-glue", "src", "runtime", "registry.ts"),
    );
    copy(
      join(PKG, "scripts", "emit-glue-esm.mjs"),
      join(root, "packages", "browser-glue", "scripts", "emit-glue-esm.mjs"),
    );
    // The runtime modules are discovered from the registry, so copy them all.
    const runtimeDir = join(PKG, "src", "runtime");
    for (const name of readFileSync(join(runtimeDir, "registry.ts"), "utf8").matchAll(
      /from "\.\/([A-Za-z0-9_$]+)"/g,
    )) {
      copy(join(runtimeDir, `${name[1]}.ts`), join(root, "packages", "browser-glue", "src", "runtime", `${name[1]}.ts`));
    }
    // ALL composed WIT files: the gate resolves a wired interface's declaration
    // by name across the whole world, and the 26 wired interfaces are spread
    // over ten domain files (`document` -> _domain-svg.wit, `node` ->
    // _domain-dom.wit, …). Copying only the two files this change edits left
    // 170 unrelated findings, so the gate exited 1 before any mutation was
    // applied and the `rc === 1` assertions below passed without proving
    // anything. The guard in the first test makes that impossible now.
    // Both composed copies: the gate reads declarations from the single source
    // and cross-checks the generated mirror byte-for-byte (a drifted mirror is
    // its own finding). A fixture with only one copy would trip that check.
    for (const copyRoot of [WIT_DIR, WIT_MIRROR_DIR]) {
      const witDir = join(root, copyRoot.slice(REPO.length + 1));
      mkdirSync(witDir, { recursive: true });
      for (const file of readdirSync(copyRoot)) {
        if (file.endsWith(".wit")) copy(join(copyRoot, file), join(witDir, file));
      }
    }
    // Allowances: everything the real tree allows, so only the injected defect
    // is left for the gate to report.
    // `wit-callback-allow.tsv` joined them with the guest-export direction: a
    // fixture missing it turns the unmutated baseline RED (the recorded
    // `resize-callbacks` line would look undeclared), which is exactly the
    // vacuous-pass failure the first test below exists to catch.
    for (const file of ["wit-op-allow.tsv", "wit-interface-allow.tsv", "wit-callback-allow.tsv"]) {
      copy(join(PKG, file), join(root, "packages", "browser-glue", file));
    }
    mutate(root);
    try {
      const out = execFileSync("python3", [join(root, "scripts", "check_wit_ops.py"), "--check"], {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      });
      return { rc: 0, out };
    } catch (error) {
      return { rc: error.status, out: String(error.stderr || "") + String(error.stdout || "") };
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("check_wit_ops.py: the fixture is GREEN before any mutation (no vacuous pass)", () => {
  // The two mutation tests below assert `rc === 1`. That assertion is worthless
  // unless the same fixture is rc === 0 with NO mutation applied — otherwise a
  // missing file, a broken scan or a stale allowance list makes the gate red for
  // an unrelated reason and both tests pass while proving nothing. (This is
  // exactly what happened with a 2-file WIT fixture: 170 findings, unmoved by
  // the mutation.) MUT: fixture-baseline-green
  const baseline = runGateAgainst(() => {});
  assert.equal(
    baseline.rc,
    0,
    `the unmutated fixture must pass, or the mutation tests below are vacuous; got: ${baseline.out}`,
  );
  assert.match(baseline.out, /WIT host surface ops OK/);
});

test("check_wit_ops.py: deleting the match-media export turns the gate RED", () => {
  // The negative control for this whole change: restore the exact defect the
  // PR fixes and the op gate must name it. If this passes, the gate is a no-op.
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "window.ts");
    const source = readFileSync(path, "utf8");
    const start = source.indexOf("  matchMedia(");
    assert.notEqual(start, -1, "the fixture must find matchMedia to remove it");
    // Remove the whole method (balanced to the closing `  },` at object depth).
    const end = source.indexOf("\n  },\n", start);
    const without = source.slice(0, start) + source.slice(end + "\n  },\n".length);
    assert.ok(
      !without.includes("matchMedia(query: string)"),
      "the mutation must actually delete the method",
    );
    writeFileSync(path, without);
  });
  assert.equal(rc, 1, "a missing match-media must fail the gate");
  assert.match(out, /window\.match-media/, `the gate must name the op; got: ${out}`);
});

test("check_wit_ops.py: an INVENTED export (runtime key the WIT lacks) turns the gate RED", () => {
  // The gate has two directions and both must be proven. This is the negative
  // control for the second one: a runtime op that the WIT does not declare is
  // dead weight at best and a capability the guest never asked for at worst.
  // Without this test, deleting the `invented` branch is a silent survivor —
  // the gate would then green-light exactly the class of drift it exists to
  // name. MUT: invented-export-injected
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "mediaQueryList.ts");
    const source = readFileSync(path, "utf8");
    const marker = "export const mediaQueryList_exports = {";
    assert.ok(source.includes(marker), "the fixture must find the exports object");
    writeFileSync(
      path,
      source.replace(
        marker,
        `${marker}\n  getBogusOp(self: bigint): boolean {\n    return false;\n  },`,
      ),
    );
  });
  assert.equal(rc, 1, "an export the WIT does not declare must fail the gate");
  assert.match(
    out,
    /media-query-list\.getBogusOp/,
    `the gate must name the invented op; got: ${out}`,
  );
});

test("check_wit_ops.py: a drifted WIT mirror turns the gate RED", () => {
  // The gate reads declarations from the single source and cross-checks the
  // generated copy. A union over both copies would let an op that exists in
  // only one satisfy the check, so a forgotten `sync_wit.py` would read green.
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "web", "wit", "composed", "_domain-css.wit");
    writeFileSync(path, readFileSync(path, "utf8") + "\n// drifted\n");
  });
  assert.equal(rc, 1, "a mirror that differs from the single source must fail the gate");
  assert.match(out, /composed WIT copies differ/, `got: ${out}`);
});

test("check_wit_ops.py: wiring an op back into the allow-list turns the gate RED", () => {
  // The inverse direction: an allowance for an op that IS implemented is stale
  // and must fail, so the inventory can only shrink.
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "wit-op-allow.tsv");
    writeFileSync(
      path,
      readFileSync(path, "utf8") +
        "media-query-list.get-matches\tstale by construction\n",
    );
  });
  assert.equal(rc, 1, "an allowance for an implemented op must fail the gate");
  // Name the EXACT branch. A bare `/stale allowance: <key>/` also matches the
  // "interface is not wired at all" and "the WIT declares no such func"
  // branches, so a mutation that broke the real path could still pass on an
  // unreachable sibling. The implemented-op branch is the only one whose text
  // says the op IS implemented. MUT: stale-allowance-branch
  assert.match(
    out,
    /stale allowance: media-query-list\.get-matches is allowed but the op IS implemented/,
    `the implemented-op branch must be the one that fires; got: ${out}`,
  );
});

test("check_wit_ops.py: a callback bridge reading a property jco never emits turns the gate RED", () => {
  // Direction 3 (host→guest). Restore the exact defect this change fixes:
  // `mutation-observer-callbacks` declares `on-mutate`, jco emits `onMutate`,
  // and the bridge used to read `on_mutation`. The gate must name it.
  // MUT: mutation-bridge-reads-on_mutation
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onMutate;";
    assert.ok(source.includes(needle), "the fixture must find the dispatch it is about to break");
    writeFileSync(path, source.replace(needle, "const handler = guest?.on_mutation;"));
  });
  assert.equal(rc, 1, "a callback property the guest does not export must fail the gate");
  // File + line + the offending read: a reworded finding must not turn this
  // into a silent pass (the failure mode this anchoring exists for).
  assert.match(
    out,
    /platformHelpers\.ts:\d+.*`(?:guest\.)?on_mutation`/,
    `the gate must name the file, a line, and the wrong read; got: ${out}`,
  );
});

test("check_wit_ops.py: an unexplained dead-interface bridge turns the gate RED", () => {
  // The allow file only says "known, not fixed" — it never makes a dead bridge
  // pass silently. Inject one (a `resize-callbacks` dispatch, the interface the
  // composed WIT never declared) and require the gate to name it; then add the
  // allowance line and require green, which is what proves the allowance is the
  // sanctioned channel rather than decoration.
  // MUT: callback-allow-file-ignored
  const injected = `        g.__wasmExports["tairitsu-browser:full/resize-callbacks@0.2.0"]?.on_resize?.(0n);\n`;
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    writeFileSync(path, readFileSync(path, "utf8") + injected);
  });
  assert.equal(rc, 1, "an undeclared interface with no allowance must fail the gate");
  assert.match(out, /dispatches into `resize-callbacks@0\.2\.0`/, `got: ${out}`);
  const allowed = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    writeFileSync(path, readFileSync(path, "utf8") + injected);
    const allow = join(root, "packages", "browser-glue", "wit-callback-allow.tsv");
    writeFileSync(
      allow,
      readFileSync(allow, "utf8") + "resize-callbacks\ttest injection\n",
    );
  });
  assert.equal(allowed.rc, 0, `an allowance must make the same site acceptable; got: ${allowed.out}`);
});

test("check_wit_ops.py: a BOUND lookup into an undeclared interface turns the gate RED", () => {
  const root0 = REPO;
  // The inline pass only sees `<key>"]?.<prop>`; a bound lookup
  // (`const guest = __wasmExports[...]`) into an interface the composed WIT does
  // not declare has to be reported by the BOUND pass, or the site is lost — that
  // is how `resize-callbacks` survived being rewired away.
  // This control exists because the branch was previously unpinned: reverting it
  // to `continue` left the gate green and only the behavioural suite noticed.
  // MUT: bound-pass-undeclared-continue
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = `      const guest = g.__wasmExports?.["tairitsu-browser:full/window-resize-callbacks@0.2.0"];`;
    assert.ok(source.includes(needle), "the fixture must find the bound lookup");
    writeFileSync(
      path,
      source.replace(
        needle,
        `      const guest = g.__wasmExports?.["tairitsu-browser:full/resize-callbacks@0.2.0"];`,
      ),
    );
  });
  assert.equal(rc, 1, "a bound lookup into an undeclared interface must fail the gate");
  assert.match(
    out,
    /binds a guest export of `resize-callbacks@0\.2\.0`/,
    `the BOUND branch must be the one that fires; got: ${out}`,
  );
  // …and it must point at THE binding. A `line_index` read before it was
  // assigned quoted the previous bridge's line instead (measured: 381 for a
  // binding on 473), which sends the reader to the wrong code.
  // MUT: bound-pass-stale-line-index
  const reported = /platformHelpers\.ts:(\d+) binds a guest export of `resize-callbacks/.exec(out);
  assert.ok(reported, `no line was reported; got: ${out}`);
  const declaration = `const guest = g.__wasmExports?.["tairitsu-browser:full/window-resize-callbacks@0.2.0"];`;
  const liveSource = readFileSync(
    join(root0, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts"),
    "utf8",
  );
  const trueLine = liveSource.slice(0, liveSource.indexOf(declaration)).split("\n").length;
  assert.equal(
    Number(reported[1]),
    trueLine,
    `the finding must name the bridge's own line (${trueLine}), not another bridge's`,
  );
  // …and the same site is acceptable once an allowance line explains it, which
  // proves the finding is the allowance channel and not a stray error.
  const allowed = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(
        `      const guest = g.__wasmExports?.["tairitsu-browser:full/window-resize-callbacks@0.2.0"];`,
        `      const guest = g.__wasmExports?.["tairitsu-browser:full/resize-callbacks@0.2.0"];`,
      ),
    );
    const allow = join(root, "packages", "browser-glue", "wit-callback-allow.tsv");
    writeFileSync(allow, readFileSync(allow, "utf8") + "resize-callbacks\ttest injection\n");
  });
  assert.equal(allowed.rc, 0, `an allowance must accept it; got: ${allowed.out}`);
});

test("check_wit_ops.py: a callback scan that stops matching fails on the floor", () => {
  // The scan's own negative control. Deleting every bridge would empty the site
  // list, and an empty list would read green — the floor is what turns "the scan
  // broke" into a red gate rather than a smaller report. This is the failure
  // that actually happened while writing this check: a single-pattern version
  // found 1 site of 42 and reported nothing.
  // MUT: callback-lookups-removed
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replaceAll("__wasmExports", "__neverSet"));
  });
  assert.equal(rc, 1, "a scan that finds nothing must fail, not pass");
  assert.match(out, /callback scan found only \d+ guest-export lookups/, `got: ${out}`);
});

test("check_wit_ops.py: any non-exported read is RED regardless of its neighbours", () => {
  // The rule is now "one candidate": jco emits exactly one identifier per func,
  // so a `??` chain offering a second spelling is not repaired by the presence of
  // a good name beside it. This control asserts the wrong operand is named even
  // when a correct one sits in the same chain.
  // MUT: callback-partner-anywhere-in-statement
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const onTimeout = exp?.onTimeout;";
    assert.ok(source.includes(needle), "the fixture must find the dispatch it is about to break");
    writeFileSync(
      path,
      source.replace(needle, "const onTimeout = exp?.on_time_out ?? exp?.on_timeout ?? exp?.onTimeout;"),
    );
  });
  assert.equal(rc, 1, "a chain whose wrong halves come first must fail the gate");
  assert.match(
    out,
    /platformHelpers\.ts:\d+.*`(?:exp\.)?on_time_out`/,
    `the gate must name the file, a line, and the wrong read; got: ${out}`,
  );
});

test("check_wit_ops.py: a second candidate spelling something else is RED", () => {
  // There is no alias table any more: `onResize ?? on_windows_resize` and
  // `onResize ?? on_resize` are BOTH findings, because jco emits only
  // `onResize`. A declared name from another interface does not rescue the read.
  // MUT: callback-second-candidate-accepted
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onResize;";
    assert.ok(source.includes(needle), "the fixture must find the dispatch");
    writeFileSync(path, source.replace(needle, "const handler = guest?.onResize ?? guest?.on_windows_resize;"));
  });
  assert.equal(rc, 1, "a fallback that is not a spelling of a declared func must fail the gate");
  assert.match(out, /platformHelpers\.ts:\d+.*`(?:guest\.)?on_windows_resize`/, `got: ${out}`);
});

test("check_wit_ops.py: a misspelled second candidate is RED", () => {
  // `onTimeout ?? on_timout`: the near-miss spelling must be named, not absorbed
  // by the correct name in front of it.
  // MUT: callback-fallback-spelling-unchecked
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const onTimeout = exp?.onTimeout;";
    assert.ok(source.includes(needle), "the fixture must find the dispatch");
    writeFileSync(path, source.replace(needle, "const onTimeout = exp?.onTimeout ?? exp?.on_timout;"));
  });
  assert.equal(rc, 1, "a fallback that is not a spelling of a declared func must fail the gate");
  assert.match(
    out,
    /platformHelpers\.ts:\d+.*`(?:exp\.)?on_timout`/,
    `got: ${out}`,
  );
});

test("check_wit_ops.py: an INDIRECT bridge (`const guest = callbacks()`) is in range", () => {
  // `mediaQueryList.ts` hands the guest object back from a helper, so a scan
  // anchored on `const x = __wasmExports…` saw ZERO bindings there and the file
  // was outside direction 3 entirely — `onChange ?? on_chang` read green.
  // MUT: helper-indirection-invisible
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "mediaQueryList.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onChange;";
    assert.ok(source.includes(needle), "the fixture must find the dispatch");
    writeFileSync(path, source.replace(needle, "const handler = guest?.onChange ?? guest?.on_chang;"));
  });
  assert.equal(rc, 1, "a wrong read behind a helper must fail the gate");
  assert.match(out, /mediaQueryList\.ts:\d+.*`(?:guest\.)?on_chang`/, `got: ${out}`);
});

test("check_wit_ops.py: a call to one helper is not satisfied by another's declaration", () => {
  // The first version of the helper detection asked "does this file declare ANY
  // helper?" and "does this file make ANY `const x = helper()` call?" — two
  // questions that can be answered by different helpers. Renaming the call while
  // the declaration keeps its old name must not count as a binding.
  // MUT: helper-pairing-loose
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "mediaQueryList.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const guest = callbacks();";
    assert.ok(source.includes(needle), "the fixture must find the helper call");
    writeFileSync(path, source.replace(needle, "const guest = otherCallbacks();"));
  });
  assert.equal(rc, 1, "an unpaired helper call must leave the file with 0 bindings and red");
  assert.match(out, /mediaQueryList\.ts references `__wasmExports` but the scan produced 0 bindings/, `got: ${out}`);
});

test("check_wit_ops.py: a bracket read is checked like any other read", () => {
  // `(guest as any)?.["on_idb_success"]` is the shape a hurried fix produces and
  // the exact name R8 removed. It read green: the read scanner only knew member
  // access, the per-file floor is saturated on that file, and a bracket read does
  // not move the base count. Both notations are now checked, and the cast form
  // (`local as T`) is recognised — without it the bracket branch matches only the
  // bare `guest["…"]` shape.
  // MUT: bracket-read-unchecked
  for (const [label, replacement] of [
    ["with a cast", '(guest as any)?.["on_idb_success"]'],
    ["without a cast", 'guest["on_idb_success"]'],
    // `satisfies` is the same kind of type-level operator as `as`: reducing only
    // `as` left a type name where the receiver belongs, and the read vanished
    // while the binding still existed — which the binding pin cannot see.
    // MUT: satisfies-not-reduced
    ["with satisfies", '(guest satisfies any)?.["on_idb_success"]'],
    ["with satisfies (dot)", "(guest satisfies any)?.on_idb_success"],
  ]) {
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      const needle = "const handler = guest?.onIdbRequestSuccess;";
      assert.ok(source.includes(needle), `the fixture must find the dispatch (${label})`);
      writeFileSync(path, source.replace(needle, `const handler = ${replacement};`));
    });
    assert.equal(rc, 1, `a wrong read (${label}) must fail the gate`);
    assert.match(
      out,
      /platformHelpers\.ts:\d+.*`guest(?:\["on_idb_success"\]|\.on_idb_success)`/,
      `got: ${out}`,
    );
  }
  // …and the SAME name written as a bracket read of the jco identifier stays green.
  const ok = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(
        "const handler = guest?.onIdbRequestSuccess;",
        'const handler = (guest as any)?.["onIdbRequestSuccess"];',
      ),
    );
  });
  assert.equal(ok.rc, 0, `a correct bracket read must stay green; got: ${ok.out}`);
});

test("check_wit_ops.py: an excluded file is out of EVERY callback scan, inline included", () => {
  // Round-17 fixed two of the three readers; the inline scan still walked every
  // file, so a bound-shaped wrong read injected into an excluded file was
  // COUNTED (88 → 89) and never checked — the bypass had simply moved. All three
  // readers plus the allowance-staleness walk now share `callback_scan_files`.
  // MUT: inline-scan-ignores-exclusions
  const injected = [
    '',
    'const injected = globalThis.__wasmExports?.["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"];',
    'const bad = injected?.on_mutation;',
    '',
  ].join("\n");
  const { rc, out } = runGateAgainst((root) => {
    // The fixture only copies the registry's runtime modules, and `wasmExports.ts`
    // is not one of them — start from the same content the real file has.
    const path = join(root, "packages", "browser-glue", "src", "runtime", "wasmExports.ts");
    writeFileSync(path, readFileSync(join(PKG, "src", "runtime", "wasmExports.ts"), "utf8") + injected);
  });
  // An excluded file is out of scope, so this is GREEN — but the count must not
  // move either, which is what "out of the scan" means.
  assert.equal(rc, 0, `an excluded file must not be scanned; got: ${out}`);
  assert.match(out, /88 guest-export lookups \(47 bound\)/, `the excluded file must not be counted; got: ${out}`);
});

test("check_wit_ops.py: an exclusion really takes a file out of the scan", () => {
  // The exclusion list used to gate only the per-file floor: `platformHelpers.ts`
  // added to it left `88 (47 bound)` untouched and the gate green, while the
  // docstring promised "a silent exclusion list is a place a bridge can be
  // hidden". Both scans now read ONE file list, so an exclusion lowers the bound
  // count and the global floor catches it.
  // (Reverting one reader to a bare glob WITHOUT injecting anything is green:
  // the discriminating case is exclusion × injection, which is what this runs.)
  // MUT: exclusion-not-applied-to-bound-scan
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "scripts", "check_wit_ops.py");
    const source = readFileSync(path, "utf8");
    const needle = 'CALLBACK_SCAN_EXCLUDED = ("global.d.ts", "wasmExports.ts")';
    assert.ok(source.includes(needle), "the fixture must find the exclusion list");
    writeFileSync(
      path,
      source.replace(needle, 'CALLBACK_SCAN_EXCLUDED = ("global.d.ts", "wasmExports.ts", "platformHelpers.ts")'),
    );
  });
  assert.equal(rc, 1, "excluding a bridge file must fail the gate, not merely print");
  assert.match(out, /bound-callback scan found only \d+ bound guest exports/, `got: ${out}`);
});

test("check_wit_ops.py: a refactor that hides a bridge turns RED (per-file floor)", () => {
  // The scan recognises three shapes, each by regex. A plain refactor — turn the
  // helper into an arrow function, or guard its body — makes `mediaQueryList.ts`
  // produce ZERO bindings while the global floor stays green at 47, and a wrong
  // read inside it reads clean. The per-file floor notices that the SHAPE
  // CHANGED without trying to parse it.
  // MUT: per-file-floor-missing
  for (const [label, from, to] of [
    [
      "arrow function",
      "function callbacks(): MediaQueryListCallbacks | undefined {",
      "const callbacks = (): MediaQueryListCallbacks | undefined => {",
    ],
    [
      "guarded body",
      'return globalThis.__wasmExports?.["tairitsu-browser:full/media-query-list-callbacks@0.2.0"];',
      'if (!globalThis.__wasmExports) {\n    return undefined;\n  }\n  return globalThis.__wasmExports["tairitsu-browser:full/media-query-list-callbacks@0.2.0"];',
    ],
  ]) {
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "mediaQueryList.ts");
      const source = readFileSync(path, "utf8");
      assert.ok(source.includes(from), `the fixture must find the helper (${label})`);
      writeFileSync(
        path,
        source.replace(from, to).replace("const handler = guest?.onChange;", "const handler = guest?.on_change;"),
      );
    });
    assert.equal(rc, 1, `a ${label} refactor that hides the bridge must fail the gate`);
    assert.match(out, /mediaQueryList\.ts references `__wasmExports` but the scan produced 0 bindings/, `got: ${out}`);
  }
});

test("check_wit_ops.py: the WRAPPED inline shape (the form the defect shipped in) turns RED", () => {
  // The original defect was written across lines:
  //   __wasmExports[\n  "…@0.2.0"\n]?.on_mutation?.(…)
  // A same-line-only scanner is blind to exactly that, which is how it read
  // green before this test existed.
  // MUT: callback-inline-scan-line-anchored
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = `      const guest =
        globalThis.__wasmExports?.["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"];`;
    assert.ok(source.includes(needle), "the fixture must find the dispatch it is about to break");
    const wrapped = `      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"]
      ) {
        globalThis.__wasmExports[
          "tairitsu-browser:full/mutation-observer-callbacks@0.2.0"
        ]?.on_mutation?.(callbackId, []);`;
    writeFileSync(path, source.replace(needle, wrapped));
  });
  assert.equal(rc, 1, "a wrapped inline dispatch with a wrong name must fail the gate");
  // Anchored on the DEFECT LINE, not on the message wording: the line number is
  // a `\d+` (the line moves with the file), but the file name and the offending
  // read must both be there, or a reworded message could let this go quiet.
  assert.match(
    out,
    /platformHelpers\.ts:\d+.*`(?:guest\.)?on_mutation`/,
    `the gate must name the file, a line, and the wrong read; got: ${out}`,
  );
});

test("check_wit_ops.py: a DESTRUCTURED guest read is reported, not skipped", () => {
  // `const { onMutate } = guest` keeps the binding (so `EXPECTED_BINDINGS_BY_FILE`
  // is satisfied) and hides the read (`_ACCESS_RE` wants a member access; a
  // destructuring binds a property KEY). Measured before direction 3c: rc 0 — the
  // wrong spelling read green, which is the exact defect class this gate exists
  // for. The declared set cannot supply a verdict about a name the source does
  // not spell as a name, so the SHAPE is the finding.
  // MUT: destructured-read-invisible
  const needle = "const handler = guest?.onMutate;";
  for (const [label, initialiser] of [
    ["plain", "guest"],
    // Ordinary TS: a type annotation on the pattern, an `as`/`satisfies` assertion
    // and a non-null assertion all sit between the pattern and the object, and the
    // first draft required `}` to be followed directly by `=`, so `: any` read as
    // "no destructuring here" and the wrong name went green (measured).
    // MUT: destructure-typed-pattern-missed
    ["typed pattern", "guest as any"],
    // …and the annotation may itself contain `=` (a function type), which the
    // first draft's `[^=]*` stopped at: `const { onMutato }: Record<string, () =>
    // void> = guest` read as "no destructuring" again.
    // MUT: destructure-arrow-type-annotation-missed
    ["arrow-type annotation", "guest"],
    ["satisfies", "guest satisfies Record<string, unknown>"],
    ["non-null assertion", "guest!"],
  ].map(([label, rhs]) => [
    label,
    `const { onMutato }${
      label === "typed pattern"
        ? ": any"
        : label === "arrow-type annotation"
          ? ": Record<string, () => void>"
          : ""
    } = ${rhs};`,
  ])) {
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      assert.ok(source.includes(needle), "the fixture must find the dispatch it is about to extend");
      writeFileSync(path, source.replace(needle, `${needle}\n      ${initialiser}`));
    });
    assert.equal(rc, 1, `a destructured guest read (${label}) must fail the gate`);
    assert.match(
      out,
      /platformHelpers\.ts:\d+ destructures `onMutato` out of `guest`/,
      `the gate must name the file, a line and the destructured key (${label}); got: ${out}`,
    );
  }
});

test("check_wit_ops.py: a destructured read is RED even when the KEY is correct", () => {
  // The rule reports the shape, not the spelling. A `{ onMutate }` that happens to
  // be right today is still a read the scan cannot check, and the next rename
  // flips it silently — which is why the finding is not "this name is wrong" but
  // "this shape is not judged". A pin that only covered the wrong name would let
  // a "fix" that keeps the destructuring and repairs the spelling read green.
  // MUT: destructured-read-judged-by-name
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onMutate;";
    writeFileSync(path, source.replace(needle, `${needle}\n      const { onMutate } = guest;`));
  });
  assert.equal(rc, 1, "a destructured read must fail the gate even with the right key");
  assert.match(
    out,
    /platformHelpers\.ts:\d+ destructures `onMutate` out of `guest`/,
    `got: ${out}`,
  );
});

test("check_wit_ops.py: a guest export taken as a DESTRUCTURED PARAMETER is reported", () => {
  // `({ onMutate }) => …` has no initialiser to anchor on, so the declared set is
  // the only handle: a parameter pattern that names a declared op is reported.
  // (A misspelled parameter pattern — `({ onMutato }) => …` — names no declared
  // op and is therefore NOT reported; that residual is stated in the module
  // docstring rather than guessed at with an edit-distance rule.)
  // MUT: arrow-param-destructure-invisible
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onMutate;";
    writeFileSync(
      path,
      source.replace(needle, `${needle}\n      const rename = ({ onMutate: fn }) => fn;`),
    );
  });
  assert.equal(rc, 1, "a destructured guest parameter must fail the gate");
  assert.match(
    out,
    /platformHelpers\.ts:\d+ destructures `onMutate` in a parameter list/,
    `got: ${out}`,
  );
});

test("check_wit_ops.py: a COMPUTED-KEY guest read is reported, cast or not", () => {
  // `` guest?.[`${name}`] `` and `guest[key]` both compute the property at run
  // time; the cast form is the "quick fix" shape and needs the same cast
  // reduction the member scan uses, or the local and its subscript are separated
  // by the cast and the read is invisible again. A QUOTED key is not this shape
  // (the member scan judges it), which is why the rule keys on the quote.
  // MUT: computed-key-read-invisible
  for (const [label, read] of [
    ["template", "const bad = guest?.[`${name}`];"],
    ["template behind a cast", "const bad = (guest as any)?.[`onMutato`];"],
    ["template, static text", "const bad = guest[`onMutato`];"],
    ["variable key", "const bad = guest[key];"],
    ["variable key behind a cast", "const bad = (guest as any)?.[key];"],
    // The key is "anything that is not a quote": a call or an index is the same
    // shape, and the first version's identifier-only key let both read green.
    // MUT: computed-key-identifier-only
    ["method-call key", "const bad = guest[keys.pop()];"],
    ["indexed key", "const bad = guest[k[0]];"],
  ]) {
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      const needle = "const handler = guest?.onMutate;";
      writeFileSync(path, source.replace(needle, `${needle}\n      ${read}`));
    });
    assert.equal(rc, 1, `a computed-key read (${label}) must fail the gate`);
    // Two stable halves rather than one pattern: the reported KEY is echoed as
    // written (`key`, `keys.pop()`, `k[0]`), so pinning the exact bracket text
    // would pin the spelling instead of the rule.
    assert.match(out, /platformHelpers\.ts:\d+ reads `guest\[/, `got: ${out}`);
    assert.match(
      out,
      /with a (?:template-literal|computed) key/,
      `the finding must say the key is computed; got: ${out}`,
    );
  }
});

test("check_wit_ops.py: the fail-closed read detector has a GREEN side too", () => {
  // The three tests above assert rc === 1. That assertion proves nothing unless
  // the same fixture is green without the shape, and unless the shapes it is NOT
  // about stay green: a member access of the same name, and a destructuring or
  // template subscript of something that is not the guest. Otherwise the rule is
  // "any destructuring anywhere reds" and every ordinary line of code trips it.
  // MUT: fail-closed-read-rule-overbroad
  const baseline = runGateAgainst(() => {});
  assert.equal(baseline.rc, 0, `the unmutated fixture must pass; got: ${baseline.out}`);
  assert.match(
    baseline.out,
    /0 guest reads in a shape the scan cannot judge \(destructuring \/ computed key \/ member cast\)/,
    `the detector must report its own count on every run; got: ${baseline.out}`,
  );
  const green = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    const needle = "const handler = guest?.onMutate;";
    writeFileSync(
      path,
      source.replace(
        needle,
        [
          needle,
          "      const rename = guest?.onMutate;",
          "      const { width, height } = globalThis.__elementHandles.get(observer) ?? {};",
          "      const label = `${width}x${height}`;",
          // The reviewer's counterexample to the FIRST version of the parameter
          // rule, which anchored on the whole world's declared names: `item` is a
          // `dom-token-list` op, so this unrelated line reded. The anchor is now
          // the interfaces THIS file binds.
          // MUT: arrow-param-anchored-on-whole-world
          "      new Map().forEach(({ item }) => item);",
          // A helper that hands back a WASI namespace matches the callback KEY
          // SHAPE (`<ns>/<iface>@<ver>`), so shape alone made it a guest source
          // and reported `const { status } = wasiStatus()` as a destructured guest
          // read (measured). Only an interface the composed WIT declares counts.
          // MUT: helper-source-shape-only
          "      function wasiStatus() {",
          '        return globalThis.__wasmExports?.["wasi:cli/exit@0.2.0"];',
          "      }",
          "      const { status } = wasiStatus();",
          "      void status;",
          // A cast that IS the value hides no read: the member is judged on its
          // own, nothing is read behind the cast, and the finding's own remedy
          // ("read the member the guest exports") is what this line already does.
          // Reporting it was a false positive introduced by the cast rule.
          // MUT: cast-of-a-value-reported
          "      const cb = (guest.onMutate as unknown as MutationCallback);",
          "      void cb;",
        ].join("\n"),
      ),
    );
  });
  assert.equal(
    green.rc,
    0,
    `member access, a non-guest destructuring and a non-guest template must stay green; got: ${green.out}`,
  );
});

test("check_wit_ops.py: an escaped `//` in a regex literal does NOT truncate the line", () => {
  // The docstring said "`//` inside a regular-expression literal" as if the two
  // spellings behaved alike. They do not, and the difference is worth a pin
  // because the safe one is what a URL-shaped pattern is written with: `/a\/\/b/`
  // contains no `//` at all once the escapes are read, so `strip_comments` copies
  // the line verbatim and the read after it IS seen.
  // MUT: strip-comments-eats-escaped-slash
  const needle = "const handler = guest?.onMutate;";
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(needle, `${needle}\n      const re = /a\\/\\/b/; guest?.onMutato;`),
    );
  });
  assert.equal(rc, 1, "an escaped `//` must not swallow the read that follows it");
  assert.match(out, /platformHelpers\.ts:\d+.*`guest\.onMutato`/, `got: ${out}`);
});

test("check_wit_ops.py: a `[//]` character class DOES truncate — pinned as the residual it is", () => {
  // The other half of the same conditional, measured: `strip_comments` reads the
  // `//` inside `[//]` as a comment and drops the rest of THAT line. A read
  // written after it is invisible, the binding survives (so the per-file pin is
  // silent) and the gate says OK — with the wrong callback name. This test pins
  // that behaviour on purpose: it is the recorded residual, not a repair, and it
  // is the reason the docstring says CONDITIONAL instead of "safe".
  // The control is the same line WITHOUT the character class: same read, rc 1.
  // If the truncation ever gets fixed, the first assertion flips and this test
  // becomes the "we closed it" pin — a failure here means the docstring moved.
  // MUT: regex-charclass-truncation-silent
  const needle = "const handler = guest?.onMutate;";
  const swallowed = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(needle, `${needle}\n      const re = /[//]/; guest?.onMutato;`),
    );
  });
  assert.equal(
    swallowed.rc,
    0,
    `the pin records today's behaviour: a [//] line swallows the read that follows it; got: ${swallowed.out}`,
  );
  // The base count is NOT asserted here: the swallowed text contains no
  // `__wasmExports` lookup, so no mutation of this rule can move it — the count
  // has its own pin above, and an assertion that cannot fail is a claim, not a
  // test. What this test proves is the rc, plus the control below.
  const control = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace(needle, `${needle}\n      guest?.onMutato;`));
  });
  assert.equal(control.rc, 1, `the SAME read without the regex must red; got: ${control.out}`);
  assert.match(control.out, /platformHelpers\.ts:\d+.*`guest\.onMutato`/, `got: ${control.out}`);
});

test("check_wit_ops.py: the `[//]` truncation is line-bounded", () => {
  // Third measurement of the same conditional: the comment rule stops at the
  // newline, so a read on the NEXT line is still seen. Without this, the pinned
  // silence above could equally be a scanner that stopped reading the file, and
  // the pin would be recording the wrong cause.
  // MUT: comment-rule-crosses-newline
  const needle = "const handler = guest?.onMutate;";
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(needle, `${needle}\n      const re = /[//]/;\n      guest?.onMutato;`),
    );
  });
  assert.equal(rc, 1, "the truncation must stop at the newline, so the next line is judged");
  assert.match(out, /platformHelpers\.ts:\d+.*`guest\.onMutato`/, `got: ${out}`);
});

test("check_wit_ops.py: the three cast shapes around the local are pinned end to end", () => {
  // The docstring lists them as shapes the cast reducer handles: `((guest as
  // any))` (double parens, so the inner group is the cast), `(guest as
  // Record<string, any>)` (a comma inside the type — the reason a type-syntax
  // white-list loses) and `(guest as Fn<(a: T) => U>)` (a function type, which
  // nests two parenthesis groups inside the cast). Claiming it is not enough:
  // each shape gets a wrong read that must red and the right read that must stay
  // green, so a reducer that stops matching one of them is a failing test rather
  // than a docstring that drifted.
  // MUT: cast-shape-not-reduced
  const needle = "const handler = guest?.onMutate;";
  for (const [label, cast] of [
    ["double parens", "((guest as any))"],
    ["generic with a comma", "(guest as Record<string, any>)"],
    ["function type", "(guest as Fn<(a: T) => U>)"],
    ["satisfies", "(guest satisfies any)"],
  ]) {
    const wrong = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      writeFileSync(path, source.replace(needle, `const handler = ${cast}?.onMutato;`));
    });
    assert.equal(wrong.rc, 1, `a wrong read behind a ${label} cast must fail the gate`);
    assert.match(
      wrong.out,
      /platformHelpers\.ts:\d+.*`guest\.onMutato`/,
      `the ${label} cast must be reduced to the local so the read is judged; got: ${wrong.out}`,
    );
    const right = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      writeFileSync(path, source.replace(needle, `const handler = ${cast}?.onMutate;`));
    });
    assert.equal(right.rc, 0, `the same ${label} cast with the right name must stay green; got: ${right.out}`);
  }
});

test("check_wit_ops.py: a member-core cast is judged through the member, not skipped", () => {
  // `(guest.sub as any)?.onX` is NOT reducible — there is no single local to put
  // back — so the cast stays and the member read inside it is judged like any
  // other read: `guest.sub` is not a declared export, so the gate reds. MEASURED
  // before this pin: rc 0. The member branch of `_ACCESS_RE` required `?.`, so
  // both `(guest.sub as any)?.onX` AND the plain-dot spelling `guest.onX` were
  // invisible, and the docstring's implicit claim that cast+read is always
  // resolved was wrong in the one direction that matters (a wrong name reading
  // green).
  // MUT: member-cast-silent-before
  const needle = "const handler = guest?.onMutate;";
  const memberCast = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace(needle, "const handler = (guest.sub as any)?.onMutato;"));
  });
  assert.equal(memberCast.rc, 1, "a cast whose core is a member must still red");
  assert.match(
    memberCast.out,
    /platformHelpers\.ts:\d+.*`guest\.sub`/,
    `the finding names the member the scan can see, not the property behind the cast; got: ${memberCast.out}`,
  );
  // The case the member read does NOT catch, and the reason the shape itself is
  // reported: when the member IS a declared export (`guest.onMutate`), the member
  // read is ACCEPTED and the property behind the cast (`?.onMutato`) is invisible
  // — measured rc 0 before this rule, with every count intact.
  // MUT: member-cast-declared-member-silent
  const declaredMember = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace(needle, "const handler = (guest.onMutate as any)?.onMutato;"));
  });
  assert.equal(
    declaredMember.rc,
    1,
    "a cast over a DECLARED member is the worst case: the member read passes and the read behind it is invisible",
  );
  assert.match(
    declaredMember.out,
    /platformHelpers\.ts:\d+ casts a member of `guest`/,
    `the gate must report the cast shape; got: ${declaredMember.out}`,
  );
  // Two spellings that defeated the first version of the rule: one redundant pair
  // of parentheses around the member (the rule's `[^()]` could not cross the `)`),
  // and a non-null assertion before `as` (the reducer rejected the core).
  // MUT: member-cast-parens-or-bang-missed
  for (const [label, read, pattern] of [
    [
      "redundant parens",
      "const handler = ((guest.onMutate) as any)?.onMutato;",
      /platformHelpers\.ts:\d+ casts a member of `guest`/,
    ],
    [
      "non-null assertion",
      "const handler = (guest! as any)?.onMutato;",
      /platformHelpers\.ts:\d+.*`guest\.onMutato`/,
    ],
  ]) {
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
      const source = readFileSync(path, "utf8");
      writeFileSync(path, source.replace(needle, read));
    });
    assert.equal(rc, 1, `a member cast with ${label} must fail the gate`);
    assert.match(out, pattern, `got: ${out}`);
  }
  // The plain-dot spelling: same read, no cast, no `?`. It used to read green.
  // MUT: plain-dot-read-invisible
  const plainWrong = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace(needle, "const handler = guest.onMutato;"));
  });
  assert.equal(plainWrong.rc, 1, "a plain-dot read must be judged like an optional-chained one");
  assert.match(plainWrong.out, /platformHelpers\.ts:\d+.*`guest\.onMutato`/, `got: ${plainWrong.out}`);
  // …and the reported line must be the READ's line, not the binding's. The scan
  // finds the read line with its own `\s*\?\.` probe; leaving that copy of the
  // pattern un-widened made every plain-dot finding quote the `const guest = …`
  // line instead (measured: 171 for a read on 180), which sends the reader to the
  // wrong code — the same defect the bound-lookup test above pins.
  // MUT: plain-dot-finding-points-at-binding-line
  const liveSource = readFileSync(join(PKG, "src", "runtime", "platformHelpers.ts"), "utf8");
  const trueLine = liveSource.slice(0, liveSource.indexOf(needle)).split("\n").length;
  const reported = /platformHelpers\.ts:(\d+).*`guest\.onMutato`/.exec(plainWrong.out);
  assert.ok(reported, `no line was reported; got: ${plainWrong.out}`);
  assert.equal(
    Number(reported[1]),
    trueLine,
    `the finding must name the read's line (${trueLine}), not the binding's`,
  );
  // …and both plain-dot and cast-wrapped CORRECT reads stay green: the rule is
  // about judging reads, not about banning the spellings.
  const plainRight = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "src", "runtime", "platformHelpers.ts");
    const source = readFileSync(path, "utf8");
    writeFileSync(
      path,
      source.replace(
        needle,
        "const handler = guest.onMutate;\n      const again = ((guest as any))?.onMutate;",
      ),
    );
  });
  assert.equal(plainRight.rc, 0, `a correct plain-dot read must stay green; got: ${plainRight.out}`);
});

test("check_wit_ops.py: the lexical layer carries its own selftest, and it is green", () => {
  // `just wit-ops-check` runs `--check` alone, and the rules that REPORT a shape
  // (instead of judging a name) can only ever print 0 on a clean tree — a
  // detector that stopped matching looks exactly like a tree that has no such
  // shape. The captured shapes and their verdicts live in the gate
  // (`--selftest`), the same way `check_wasip2_closure.py` pins its parser; this
  // runs them in the suite lane so the pin is executed, not shipped.
  // MUT: lexical-selftest-not-run
  const out = execFileSync("python3", [GATE, "--selftest"], { cwd: REPO, encoding: "utf8" });
  assert.match(out, /selftest ok: the lexical layer still produces every captured verdict/);
});

test("check_wit_ops.py: a wrong name on a FAR read names the read's line, not the binding's", () => {
  // `eventTarget.ts` binds `callbacks` on line 26 and reads it at 68, 86, 102, 107
  // and 112 — the five event bridges, which use the PLAIN dot the read scan only
  // started judging in this PR. Two things therefore have to hold, and both are
  // asserted here because nobody would notice either one failing:
  //   1. each of those five reads is judged at all (rc 1 when a name is wrong);
  //   2. the finding quotes THAT line. A fixed 12-line probe from the binding
  //      found nothing at 40-86 lines and reported line 26 for every one of them
  //      (measured), and keying on the local alone reported 68 for a defect on 86.
  // The binding pin (`EXPECTED_BINDINGS_BY_FILE["eventTarget.ts"] = 1`) cannot
  // see either: a renamed property leaves the binding count untouched.
  // MUT: far-read-line-attribution
  const liveSource = readFileSync(join(PKG, "src", "runtime", "eventTarget.ts"), "utf8");
  for (const [name, wrong] of [
    ["onMouseEvent", "onMouseEvento"],
    ["onKeyboardEvent", "onKeyboardEvento"],
    ["onFocusEvent", "onFocusEvento"],
    ["onInputEvent", "onInputEvento"],
    ["onGenericEvent", "onGenericEvento"],
  ]) {
    const trueLine = liveSource.slice(0, liveSource.indexOf(`callbacks.${name}(`)).split("\n").length;
    const { rc, out } = runGateAgainst((root) => {
      const path = join(root, "packages", "browser-glue", "src", "runtime", "eventTarget.ts");
      const source = readFileSync(path, "utf8");
      assert.ok(source.includes(`callbacks.${name}(`), `the fixture must find ${name}`);
      writeFileSync(path, source.replace(`callbacks.${name}(`, `callbacks.${wrong}(`));
    });
    assert.equal(rc, 1, `${name} renamed must fail the gate`);
    assert.match(out, new RegExp(`eventTarget\\.ts:\\d+ .*callbacks\\.${wrong}`), `got: ${out}`);
    const reported = new RegExp(`eventTarget\\.ts:(\\d+) .*callbacks\\.${wrong}`).exec(out);
    assert.ok(reported, `no line was reported for ${name}; got: ${out}`);
    assert.equal(
      Number(reported[1]),
      trueLine,
      `the finding for ${name} must name its own line (${trueLine}), not the binding's`,
    );
  }
});
