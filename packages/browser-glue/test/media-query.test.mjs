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
  // MUT: bridge-name-only-snake (reading only `on_change` reds this test; the
  // snake_case arm below still passes, which is why both arms exist).
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

  // (b) the snake_case spelling keeps working as a fallback (the sibling
  //     bridge in platformHelpers.ts accepts `onTimeout || on_timeout`), so a
  //     wrapper built with the other convention is not a silent no-op either.
  const snake = [];
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      on_change: (id, m) => snake.push([id, m]),
    },
  };
  mediaQueryList_exports.addListener(handle, 2n);
  media.reduce = false;
  list.flip();
  assert.deepEqual(snake, [[2n, false]], "the snake_case fallback must also deliver");

  // (c) neither name present: no throw, no delivery — the bridge is inert
  //     rather than crashing the DOM event loop.
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {},
  };
  mediaQueryList_exports.addListener(handle, 3n);
  media.reduce = true;
  assert.doesNotThrow(() => list.flip());
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
    assert.doesNotThrow(() => list.flip());
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
    const needle = "guest?.onMutate ?? guest?.on_mutation";
    assert.ok(source.includes(needle), "the fixture must find the dispatch it is about to break");
    writeFileSync(path, source.replace(needle, "guest?.on_mutation"));
  });
  assert.equal(rc, 1, "a callback property the guest does not export must fail the gate");
  assert.match(
    out,
    /`mutation-observer-callbacks@0\.2\.0` and reads `guest\.on_mutation`/,
    `the gate must name the interface and the property; got: ${out}`,
  );
});

test("check_wit_ops.py: an unexplained dead-interface bridge turns the gate RED", () => {
  // `resize-callbacks` is recorded in wit-callback-allow.tsv. Remove the line and
  // the same site must be reported: an allowance is the ONLY way to say "known",
  // so the exception is visible in review instead of implicit.
  // MUT: callback-allow-file-dropped
  const { rc, out } = runGateAgainst((root) => {
    const path = join(root, "packages", "browser-glue", "wit-callback-allow.tsv");
    writeFileSync(path, "# empty on purpose\n");
  });
  assert.equal(rc, 1, "an undeclared interface with no allowance must fail the gate");
  assert.match(out, /dispatches into `resize-callbacks@0\.2\.0`/, `got: ${out}`);
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
