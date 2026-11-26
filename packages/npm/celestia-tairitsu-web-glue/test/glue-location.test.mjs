// Behaviour tests for the `location` glue (this PR).
//
// Asserted here:
//   1. every getter reads its IDL attribute off `window.location`,
//   2. every setter writes through to `window.location` — the guest sets the
//      property and the platform navigates, exactly as a page would,
//   3. `assign` / `replace` / `reload` call the `Location` methods of the same
//      names (and hand `assign` / `replace` their URL untouched),
//   4. `getAncestorOrigins` hands back a plain array, and an empty one where
//      the platform has no such property,
//   5. the export object covers every function of the WIT `location`
//      interface, with the names read off the WIT file itself — the gap this
//      change closes (`setHref` missing, so a guest importing it failed at
//      module link time) had no guard.
//
// The module under test is the shipped source file, imported directly (Node
// strips the TypeScript). `window` does not exist in Node, so each test
// installs a recording stand-in and takes it back down again.
//
// Run with: npm run test   (or: node --test "test/**/*.test.mjs")

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

// Host surface the rest of the module expects at import time.
globalThis.__elementHandles = new Map();
globalThis.__fileListHandles = new Map();
globalThis.__nodeListHandles = new Map();
globalThis.__validityStateHandles = new Map();

const { location_exports } = await import("../src/glue-html.ts");

/** A recording stand-in for `window.location`: plain properties for the IDL
 * attributes and methods that push their argument, so a test can tell a
 * write-through from a swallowed call. */
function fakeLocation(overrides = {}) {
  return {
    href: "https://forum.example.test/admin?tab=1#top",
    origin: "https://forum.example.test",
    protocol: "https:",
    host: "forum.example.test:8443",
    hostname: "forum.example.test",
    port: "8443",
    pathname: "/admin",
    search: "?tab=1",
    hash: "#top",
    ancestorOrigins: ["https://parent.example.test"],
    calls: [],
    assign(url) {
      this.calls.push(["assign", url]);
    },
    replace(url) {
      this.calls.push(["replace", url]);
    },
    reload() {
      this.calls.push(["reload"]);
    },
    ...overrides,
  };
}

/** Install `location` as `window.location` for the duration of `run`. */
async function withLocation(location, run) {
  const previous = Object.getOwnPropertyDescriptor(globalThis, "window");
  Object.defineProperty(globalThis, "window", {
    value: { location },
    configurable: true,
    writable: true,
  });
  try {
    return await run(location);
  } finally {
    if (previous) {
      Object.defineProperty(globalThis, "window", previous);
    } else {
      delete globalThis.window;
    }
  }
}

test("location getters read the Location object", async () => {
  await withLocation(fakeLocation(), (loc) => {
    assert.equal(location_exports.getHref(), loc.href);
    assert.equal(location_exports.getOrigin(), loc.origin);
    assert.equal(location_exports.getProtocol(), "https:");
    assert.equal(location_exports.getHost(), "forum.example.test:8443");
    assert.equal(location_exports.getHostname(), "forum.example.test");
    assert.equal(location_exports.getPort(), "8443");
    assert.equal(location_exports.getPathname(), "/admin");
    assert.equal(location_exports.getSearch(), "?tab=1");
    assert.equal(location_exports.getHash(), "#top");
  });
});

test("location getters keep their empty-string fallbacks", async () => {
  // The two reads that pre-date this change fall back rather than handing
  // back an empty string; that contract is unchanged.
  await withLocation(fakeLocation({ pathname: "", hash: "" }), () => {
    assert.equal(location_exports.getPathname(), "/");
    assert.equal(location_exports.getHash(), "");
  });
});

test("location setters write through to the Location object", async () => {
  await withLocation(fakeLocation(), (loc) => {
    location_exports.setHref("https://forum.example.test/logout");
    location_exports.setProtocol("http:");
    location_exports.setHost("other.example.test");
    location_exports.setHostname("other.example.test");
    location_exports.setPort("8080");
    location_exports.setPathname("/plugin.php");
    location_exports.setSearch("?id=pokemon:game");
    location_exports.setHash("#nav");

    assert.equal(loc.href, "https://forum.example.test/logout");
    assert.equal(loc.protocol, "http:");
    assert.equal(loc.host, "other.example.test");
    assert.equal(loc.hostname, "other.example.test");
    assert.equal(loc.port, "8080");
    assert.equal(loc.pathname, "/plugin.php");
    assert.equal(loc.search, "?id=pokemon:game");
    assert.equal(loc.hash, "#nav");
    // Writes are writes: none of them detoured through a navigation method.
    assert.deepEqual(loc.calls, []);
  });
});

test("a setter surfaces the platform's own error", async () => {
  // A cross-origin or malformed target makes the browser throw a
  // SecurityError / TypeError from the setter. The glue must not swallow it:
  // the guest is the one that decided to navigate and the one to report it.
  const loc = fakeLocation();
  Object.defineProperty(loc, "href", {
    get() {
      return "https://forum.example.test/";
    },
    set() {
      throw new TypeError("bad href");
    },
  });
  await withLocation(loc, () => {
    assert.throws(
      () => location_exports.setHref("not a url"),
      /bad href/,
    );
  });
});

test("assign, replace and reload call the Location methods", async () => {
  await withLocation(fakeLocation(), (loc) => {
    location_exports.assign("https://forum.example.test/next");
    location_exports.replace("https://forum.example.test/other");
    location_exports.reload();
    assert.deepEqual(loc.calls, [
      ["assign", "https://forum.example.test/next"],
      ["replace", "https://forum.example.test/other"],
      ["reload"],
    ]);
  });
});

test("getAncestorOrigins hands back a plain array, empty where the platform has none", async () => {
  await withLocation(
    fakeLocation({ ancestorOrigins: { length: 2, 0: "https://a.example.test", 1: "https://b.example.test" } }),
    () => {
      // A DOMStringList is iterable in the browser; an array-like that is not
      // must still not be mistaken for the property being absent.
      assert.deepEqual(location_exports.getAncestorOrigins(), [
        "https://a.example.test",
        "https://b.example.test",
      ]);
    },
  );
  const bare = fakeLocation();
  delete bare.ancestorOrigins;
  await withLocation(bare, () => {
    assert.deepEqual(location_exports.getAncestorOrigins(), []);
  });
});

test("the glue exports every function of the WIT location interface", () => {
  const wit = readFileSync(
    new URL("../../../browser-worlds/wit/browser-full.wit", import.meta.url),
    "utf8",
  );
  const start = wit.indexOf("interface location {");
  assert.notEqual(start, -1, "the WIT file has no `interface location`");
  const block = wit.slice(start, wit.indexOf("\n}", start));
  // `get-href: func(...)` → `getHref`. The `type location-handle = u64;` line
  // carries no `: func`, so it stays out.
  const names = Array.from(block.matchAll(/^\s+([a-z][a-z-]*):\s*func\b/gm), ([, kebab]) =>
    kebab.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()),
  );
  assert.ok(names.length >= 20, `read ${names.length} location functions off the WIT file`);
  for (const name of names) {
    assert.equal(
      typeof location_exports[name],
      "function",
      `WIT location.${name} has no export in location_exports`,
    );
  }
});
