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
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      on_change(callbackId, matches) {
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
      on_change(callbackId, matches) {
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

test("media-query-list: a throwing guest callback does not escape into the event loop", () => {
  installDom({ reduce: false });
  globalThis.__wasmExports = {
    "tairitsu-browser:full/media-query-list-callbacks@0.2.0": {
      on_change() {
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
    const witDir = join(root, "packages", "browser-worlds", "wit", "composed");
    mkdirSync(witDir, { recursive: true });
    for (const file of ["_domain-css.wit", "_handwritten.wit"]) {
      copy(join(WIT_DIR, file), join(witDir, file));
    }
    // Allowances: everything the real tree allows, so only the injected defect
    // is left for the gate to report.
    for (const file of ["wit-op-allow.tsv", "wit-interface-allow.tsv"]) {
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
  assert.match(out, /stale allowance: media-query-list\.get-matches/, `got: ${out}`);
});
