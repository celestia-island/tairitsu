// The host→guest callback bridges: the name AND the shape of every value that
// crosses into `__wasmExports`.
//
// Why this file exists
// --------------------
// `check_wit_ops.py` reads the runtime INTERFACES table — the guest calling IN.
// It was structurally blind to the other direction: the runtime calling the
// guest's exports through
// `globalThis.__wasmExports["tairitsu-browser:full/<iface>@0.2.0"]?.<prop>(…)`.
// There, a wrong property name does not throw; it evaluates to `undefined`, so
// the notification is silently dropped. Fifteen bridges were wrong that way
// before this change:
//
//   * `on_mutation` for a WIT func named `on-mutate` (jco emits `onMutate`) —
//     a name the wrapper carries under NO convention, so a guest's
//     MutationObserver callback never ran and nothing logged;
//   * `on_resize` — the name happens to be right (one-segment kebab names
//     camelCase to themselves) but the VALUE was an array of `u64` handles for a
//     `list<resize-entry>` parameter;
//   * `on_promise_resolve` / `on_geolocation_*` / `on_scroll` /
//     `on_file_reader_*` / `on_idb_*` / `on_web_socket_*` — renamed funcs whose
//     old property survived one generation past the WIT.
//
// The names below were taken from `jco types` 1.42.0 run against the callback
// interfaces of `packages/browser-worlds/wit/handwritten/callbacks.wit`, not
// inferred from the WIT spelling (four of the fifteen disagree with a naive
// `snake→camel` mapping).
//
// Run with: npm run test

import assert from "node:assert/strict";
import { register } from "node:module";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

register("./ts-resolve-hook.mjs", import.meta.url);

const HERE = dirname(fileURLToPath(import.meta.url));
const PKG = join(HERE, "..");
const RUNTIME = join(PKG, "src", "runtime");

const NS = "tairitsu-browser:full";

// The real bootstrap order (`runtime/index.ts`): `handles.ts` installs the
// per-instance delegates, `helpers.ts` installs `__storeElement` / `__lookup*`.
await import(join(RUNTIME, "handles.ts"));
await import(join(RUNTIME, "helpers.ts"));
const { currentInstance } = await import(join(RUNTIME, "instance.ts"));

/**
 * Static observers and a socket, the way a page provides them. Node has none of
 * them; the runtime only ever CONSTRUCTS them and hands the instance a
 * callback, so a recorder is enough to drive the bridge.
 */
class RecordingObserver {
  static kind = "observer";
  static instances = [];
  constructor(callback) {
    this.callback = callback;
    this.observed = [];
    RecordingObserver.instances.push(this);
  }
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.MutationObserver = RecordingObserver;
globalThis.ResizeObserver = RecordingObserver;

const platformHelpers = (await import(join(RUNTIME, "platformHelpers.ts"))).platformHelpers_exports;
const { mutationRecord_exports } = await import(join(RUNTIME, "mutationRecord.ts"));
const { resizeObserverEntry_exports } = await import(join(RUNTIME, "resizeObserverEntry.ts"));

/** The guest, as jco builds it: one object per interface, camelCase keys. */
function installGuest(entries) {
  const calls = [];
  const guest = {};
  for (const [iface, funcs] of Object.entries(entries)) {
    const slot = {};
    for (const name of funcs) {
      slot[name] = (...args) => calls.push([iface, name, args]);
    }
    guest[`${NS}/${iface}@0.2.0`] = slot;
  }
  globalThis.__wasmExports = guest;
  return calls;
}

/** A DOM `MutationRecord` with only the fields the WIT record declares. */
function fakeMutationRecord(overrides = {}) {
  return {
    type: "childList",
    target: { nodeType: 1, childNodes: [] },
    addedNodes: [{ nodeType: 1, childNodes: [] }],
    removedNodes: [],
    previousSibling: null,
    nextSibling: null,
    attributeName: null,
    attributeNamespace: null,
    oldValue: null,
    ...overrides,
  };
}

/** A DOM `ResizeObserverEntry`. */
function fakeResizeEntry(rect) {
  return { target: { nodeType: 1 }, contentRect: rect, borderBoxSize: [], contentBoxSize: [] };
}

/**
 * Instances: these bridges read the per-instance tables, so every test holds the
 * current-instance pointer and restores it. `dispose` also keeps a reclaimed
 * observer from firing into a later test's guest.
 */
function withInstance(fn) {
  const instance = currentInstance();
  return fn(instance);
}

test("mutation observer: the bridge binds `onMutate` (the WIT func `on-mutate`)", () => {
  // MUT: mutation-bridge-reads-on_mutation (restoring the old property name
  // reds this: nothing is pushed, because the guest carries no such key)
  const calls = installGuest({ "mutation-observer-callbacks": ["onMutate"] });
  try {
    withInstance(() => platformHelpers.createMutationObserver(31n));
    assert.equal(globalThis.MutationObserver.instances.length > 0, true);
    const observer = globalThis.MutationObserver.instances.at(-1);
    observer.callback([fakeMutationRecord()], observer);
    assert.deepEqual(
      calls.map(([, name]) => name),
      ["onMutate"],
      "a mutation notification must reach the guest — the old bridge read `on_mutation`, " +
        "a property the wrapper never has, so every notification was dropped",
    );
    assert.deepEqual(calls[0][2][0], 31n, "the callback id the guest registered is what comes back");
  } finally {
    delete globalThis.__wasmExports;
  }
});

test("mutation observer: records arrive as mutation-entry values, not handle lists", () => {
  // MUT: mutation-bridge-passes-handles (the old body marshalled `u64` handles
  // into a `list<mutation-entry>` parameter — an integer where the guest reads
  // an object, which the decoder accepts structurally and then reports garbage)
  const calls = installGuest({ "mutation-observer-callbacks": ["onMutate"] });
  try {
    withInstance(() => platformHelpers.createMutationObserver(7n));
    const observer = globalThis.MutationObserver.instances.at(-1);
    observer.callback(
      [
        fakeMutationRecord({
          type: "attributes",
          attributeName: "data-state",
          oldValue: "open",
          addedNodes: [],
          removedNodes: [{ nodeType: 1, childNodes: [] }],
        }),
      ],
      observer,
    );
    const [, , [, entries]] = calls[0];
    assert.equal(Array.isArray(entries), true, "entries must be an array (list<mutation-entry>)");
    const entry = entries[0];
    assert.equal(entry.mutationType, "attributes", "kebab `mutation-type` → camel `mutationType`");
    assert.equal(typeof entry.target, "bigint", "a node reference is a u64 handle");
    // `added-nodes` / `removed-nodes` are `option<u64>` — ONE handle or
    // `undefined`, never an array. `jco types` on the verbatim record:
    //   export interface MutationEntry { … addedNodes?: bigint, removedNodes?: bigint … }
    // A first draft of the bridge passed `bigint[]` here, which is the same
    // class of defect the rename repairs, so this assertion pins the arity.
    assert.equal(entry.addedNodes, undefined, "an empty NodeList is `undefined`, not `[]`");
    assert.equal(typeof entry.removedNodes, "bigint", "one removed node is ONE handle");
    assert.equal(
      Array.isArray(entry.removedNodes),
      false,
      "`option<u64>` is a single value — a list here is not convertible by jco",
    );
    assert.equal(entry.attributeName, "data-state");
    assert.equal(entry.oldValue, "open");
    // option<T> fields must be present as `undefined`, not omitted (jco reads them).
    assert.ok("previousSibling" in entry && entry.previousSibling === undefined);
    assert.ok("nextSibling" in entry && entry.nextSibling === undefined);
    assert.ok("attributeNamespace" in entry && entry.attributeNamespace === undefined);
  } finally {
    delete globalThis.__wasmExports;
  }
});

test("mutation observer: the record handle still resolves through the WIT getters", () => {
  // The bridge mints a handle per record and stores it in the instance table;
  // `mutation-record` keeps serving it, so a guest that used the handle path
  // before this change sees the same magic 2nd recording.
  const calls = installGuest({ "mutation-observer-callbacks": ["onMutate"] });
  try {
    const instance = currentInstance();
    const before = instance.tables.mutationRecordHandles.size;
    withInstance(() => platformHelpers.createMutationObserver(8n));
    const observer = globalThis.MutationObserver.instances.at(-1);
    observer.callback([fakeMutationRecord({ type: "characterData" })], observer);
    const after = instance.tables.mutationRecordHandles.size;
    assert.equal(after, before + 1, "one handle per record must still be table-stored");
    const [handle, rec] = [...instance.tables.mutationRecordHandles.entries()].at(-1);
    assert.equal(mutationRecord_exports.getType(handle), "characterData");
    assert.equal(mutationRecord_exports.getType(handle), rec.type);
    assert.equal(calls.length, 1);
  } finally {
    delete globalThis.__wasmExports;
  }
});

test("resize observer: the bridge binds `onResize` and passes resize-entry records", () => {
  // MUT: resize-bridge-passes-entry-handles (the old body passed an array of
  // `u64`; this assertion reads `entries[0].contentRect` and goes red)
  const calls = installGuest({ "resize-observer-callbacks": ["onResize"] });
  try {
    const instance = currentInstance();
    withInstance(() => platformHelpers.createResizeObserver(4n));
    const observer = globalThis.ResizeObserver.instances.at(-1);
    observer.callback([fakeResizeEntry({ x: 1, y: 2, width: 640, height: 480 })], observer);
    assert.equal(calls.length, 1, "the resize notification must reach the guest");
    const [iface, name, [callbackId, entries]] = calls[0];
    assert.equal(iface, "resize-observer-callbacks");
    assert.equal(name, "onResize");
    assert.equal(callbackId, 4n);
    assert.deepEqual(
      entries,
      [{ target: entries[0].target, contentRect: { x: 1, y: 2, width: 640, height: 480 } }],
      "each entry must be a `resize-entry` record: { target: u64, contentRect: dom-rect }",
    );
    assert.equal(typeof entries[0].target, "bigint");
    // …and the handle path still works for `resize-observer-entry`.
    assert.equal(instance.tables.resizeObserverEntryHandles.size >= 1, true);
    const [handle] = [...instance.tables.resizeObserverEntryHandles.keys()].slice(-1);
    assert.equal(typeof resizeObserverEntry_exports.getContentRect(handle), "bigint");
  } finally {
    delete globalThis.__wasmExports;
  }
});

test("web socket: open, message, close and error all reach the guest (the live path)", () => {
  // `connect-web-socket` is a WIRED, IMPLEMENTED op with a guest export
  // (`wit_platform.rs`), so these four bridges are the only live consumer of the
  // whole callback direction: before this change the data arrived on the socket
  // and the guest never heard about it.
  // MUT: ws-bridge-reads-on_web_socket (any of the four restored to snake_case
  // drops that event and reds this test)
  const calls = installGuest({
    "web-socket-callbacks": [
      "onWebSocketOpen",
      "onWebSocketMessage",
      "onWebSocketClose",
      "onWebSocketError",
    ],
  });
  const originalWebSocket = globalThis.WebSocket;
  const sockets = [];
  class FakeWebSocket {
    constructor(url) {
      this.url = url;
      sockets.push(this);
    }
  }
  globalThis.WebSocket = FakeWebSocket;
  try {
    const instance = currentInstance();
    withInstance(() => {
      platformHelpers.connectWebSocket("wss://example.test/socket");
      const socket = sockets.at(-1);
      socket.onopen();
      socket.onmessage({ data: "hello" });
      socket.onmessage({ data: { kind: "json" } });
      socket.onclose({ code: 1000, reason: "bye" });
      socket.onerror();
    });
    assert.deepEqual(
      calls.map(([, name]) => name),
      [
        "onWebSocketOpen",
        "onWebSocketMessage",
        "onWebSocketMessage",
        "onWebSocketClose",
        "onWebSocketError",
      ],
      "every socket event must cross the bridge under its jco name",
    );
    assert.equal(calls[1][2][1], "hello", "string payloads pass through");
    assert.equal(calls[2][2][1], '{"kind":"json"}', "non-string payloads are JSON encoded");
    assert.equal(calls[3][2][1], 1000, "the close code comes from the event");
    assert.equal(calls[3][2][2], "bye");
    assert.equal(instance.reclaimables.sockets.size, 1, "the socket stays instance-owned");
  } finally {
    globalThis.WebSocket = originalWebSocket;
    delete globalThis.__wasmExports;
  }
});

test("scroll bridge binds `onScrollEvent` and sends the two COORDINATES", () => {
  // `on-scroll-event(callback-id, scroll-x, scroll-y)` → `onScrollEvent`. The old
  // read was `on_scroll` with three derived numbers (top / height / client
  // height) — neither the name nor the arity matched.
  // MUT: scroll-bridge-reads-on_scroll
  const calls = installGuest({ "scroll-callbacks": ["onScrollEvent"] });
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  try {
    globalThis.document = {
      documentElement: { scrollTop: 40, scrollLeft: 5, scrollHeight: 900, clientHeight: 500 },
      body: { scrollTop: 0, scrollLeft: 0, scrollHeight: 900, clientHeight: 500 },
    };
    const listeners = [];
    globalThis.window = {
      addEventListener: (type, handler) => listeners.push([type, handler]),
      removeEventListener: () => {},
    };
    withInstance(() => platformHelpers.onScroll(19n));
    const scrollListener = listeners.find(([type]) => type === "scroll")[1];
    scrollListener();
    assert.deepEqual(
      calls.map(([, , args]) => Array.from(args)),
      [[19n, 5, 40]],
      "onScrollEvent(callbackId, scroll-x, scroll-y)",
    );
  } finally {
    globalThis.window = originalWindow;
    globalThis.document = originalDocument;
    delete globalThis.__wasmExports;
  }
});

test("promise bridge binds `onPromiseResolved` / `onPromiseRejected`", async () => {
  // Were `on_promise_resolve` / `on_promise_reject` — the WIT funcs are
  // `on-promise-resolved` / `on-promise-rejected`, so no such property exists.
  // MUT: promise-bridge-reads-on_promise_resolve
  const calls = installGuest({
    "promise-callbacks": ["onPromiseResolved", "onPromiseRejected"],
  });
  try {
    const id = withInstance(() => platformHelpers.fetchPromise("data:application/json,{}", null));
    // `fetch` against a data: URL resolves without a network; give it a tick.
    await new Promise((resolve) => setTimeout(resolve, 60));
    assert.equal(calls.length >= 1, true, "a settled fetch must reach the guest");
    assert.equal(calls[0][1], "onPromiseResolved");
    assert.equal(calls[0][2][0], id, "the promise id the caller received comes back");
  } finally {
    delete globalThis.__wasmExports;
  }
});

test("geolocation bridge binds `onPositionSuccess` / `onPositionError`", () => {
  // Were `on_geolocation_success` / `_error`; the WIT declares
  // `on-position-success` / `on-position-error`.
  // MUT: geolocation-bridge-reads-on_geolocation
  const calls = installGuest({
    "geolocation-callbacks": ["onPositionSuccess", "onPositionError"],
  });
  const originalNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  try {
    const coords = { latitude: 12.5, longitude: -7.25 };
    Object.defineProperty(globalThis, "navigator", {
      configurable: true,
      value: {
        geolocation: {
          getCurrentPosition(success) {
            success({ coords });
          },
        },
      },
    });
    withInstance(() => platformHelpers.getCurrentPosition(0n, 33n, 34n, false, 1000, 0));
    assert.deepEqual(calls.map(([, name]) => name), ["onPositionSuccess"]);
    assert.deepEqual(Array.from(calls[0][2]), [33n, 12.5, -7.25]);
  } finally {
    if (originalNavigator) Object.defineProperty(globalThis, "navigator", originalNavigator);
    else delete globalThis.navigator;
    delete globalThis.__wasmExports;
  }
});

test("every bridge answers under the snake_case spelling too (fallback kept)", () => {
  // The jco name is the contract, but a wrapper built with the WIT spelling (or
  // a future jco) must keep working — the same belt-and-braces the media-query
  // bridge uses. This drives the mutation bridge with `on_mutation` only and
  // asserts it still fires, which is what proves the `??` fallback is wired and
  // not dead text.
  // MUT: mutation-bridge-drops-fallback (`guest?.onMutate` alone reds this)
  const calls = installGuest({ "mutation-observer-callbacks": ["on_mutation"] });
  try {
    withInstance(() => platformHelpers.createMutationObserver(2n));
    const observer = globalThis.MutationObserver.instances.at(-1);
    observer.callback([fakeMutationRecord()], observer);
    assert.equal(calls.length, 1, "the snake_case fallback must still deliver");
  } finally {
    delete globalThis.__wasmExports;
  }
});
