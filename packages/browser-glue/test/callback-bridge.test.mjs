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
// the notification is silently dropped. Sixteen wrong property names across 42
// dispatch sites were that way before this change:
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
// inferred from the WIT spelling: thirteen of the sixteen are not the
// `snake→camel` of the func they were meant to call (only `on_resize`,
// `on_scroll` and the four `on_web_socket_*` map onto a declared name, and those
// are the ones that read the wrong NAME for a real func).
//
// Run with: npm run test

import assert from "node:assert/strict";
import { register } from "node:module";
import { dirname, join } from "node:path";
import test from "node:test";
import { readFileSync } from "node:fs";
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
    const previous = { nodeType: 1, childNodes: [] };
    const next = { nodeType: 3, childNodes: [] };
    observer.callback(
      [
        fakeMutationRecord({
          type: "attributes",
          attributeName: "data-state",
          oldValue: "open",
          addedNodes: [],
          removedNodes: [{ nodeType: 1, childNodes: [] }],
          previousSibling: previous,
          nextSibling: next,
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
    // Sibling handles live in the NODE domain — `mutation-record`'s
    // `get-previous-sibling` / `get-next-sibling` are the readers, and they go
    // through `__storeNode`. An element-domain sibling is a handle the guest
    // can hold and never resolve (nothing else notices: both tables answer
    // `bigint`). MUT: mutation-bridge-sibling-handle-in-element-table
    assert.equal(typeof entry.previousSibling, "bigint");
    assert.equal(typeof entry.nextSibling, "bigint");
    assert.equal(
      globalThis.__nodeHandles.get(entry.previousSibling),
      previous,
      "the previous-sibling handle must resolve in the NODE table",
    );
    assert.equal(
      globalThis.__nodeHandles.get(entry.nextSibling),
      next,
      "a text node is still a node handle (nodeHandles or textHandles)",
    );
    assert.equal(
      currentInstance().tables.elementHandles.has(entry.previousSibling),
      false,
      "a sibling must NOT be an element handle — that is the wrong domain",
    );
    assert.ok("attributeNamespace" in entry && entry.attributeNamespace === undefined);
    // Handle DOMAINS, not just types: a sibling must resolve through the NODE
    // table (that is what `mutation-record`'s `get-previous-sibling` reads),
    // while `target` resolves through the ELEMENT table (what `get-target`
    // reads). Minting a sibling into the wrong table yields a bigint the guest
    // can hold and never resolve — and both tables answer `bigint`, so nothing
    // else in the suite notices.
    // MUT: mutation-bridge-sibling-handle-in-element-table
    assert.ok(
      currentInstance().tables.elementHandles.has(entry.target),
      "target must be an ELEMENT handle (mutation-record.get-target reads that table)",
    );
    assert.equal(
      currentInstance().tables.nodeHandles.has(entry.target),
      false,
      "target must not be a node handle — the two tables are different domains",
    );
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
    // …and the WIT getters that expose the FULL lists (the records the callback
    // carries truncate `added-nodes` to one handle, by declaration). These were
    // exported only by the generator's unwired `glue/dom.ts`, so the "ask the
    // record" recovery path the bridge relies on did not exist.
    // MUT: mutation-record-added-nodes-unwired
    const addedList = mutationRecord_exports.getAddedNodes(handle);
    assert.equal(typeof addedList, "bigint", "a node-list handle");
    assert.notEqual(addedList, 0n, "the record has an addedNodes list to hand back");
    // The fake record carries plain arrays (see `fakeMutationRecord`); a real
    // NodeList would have `.item`, and the handle is what the guest resolves.
    const added = globalThis.__nodeListHandles.get(addedList);
    assert.equal(added.length, rec.addedNodes.length);
    assert.equal(added[0], rec.addedNodes[0]);
    assert.notEqual(mutationRecord_exports.getRemovedNodes(handle), 0n);
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
    assert.equal(entries.length, 1);
    // DOMAIN, not (only) shape: `resize-observer-entry`'s `get-target` resolves
    // through `__storeElement`, so `target` must land in the ELEMENT table. An
    // earlier version of this assertion compared `target` with itself
    // (`target: entries[0].target`), which stays green when the handle is
    // minted into the node table instead — the same domain defect the mutation
    // entry had. MUT: resize-bridge-target-in-node-table
    assert.ok(
      currentInstance().tables.elementHandles.has(entries[0].target),
      "`target` must be an ELEMENT handle (resize-observer-entry.get-target reads that table)",
    );
    assert.equal(
      currentInstance().tables.nodeHandles.has(entries[0].target),
      false,
      "`target` must not be a node handle — the two tables are different domains",
    );
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
    assert.equal(calls[3][2][1], 1000, "the close code comes from the real event");
    assert.equal(calls[3][2][2], "bye");
    // Every event reports the callback id the bridge minted (the guest routes on
    // it), and the four ids differ — one shared id would route every event to
    // the same handler. MUT: ws-callback-ids-collapsed
    const ids = calls.map(([, , args]) => args[0]);
    assert.equal(
      new Set(ids).size,
      4,
      `open/message/close/error must carry distinct callback ids: ${ids}`,
    );
    for (const id of ids) assert.equal(typeof id, "number", "the ids the bridge mints are numbers");
    // …and `open` / `error` carry nothing else, so a reverted arity is visible.
    assert.equal(calls[0][2].length, 1, "on-web-socket-open takes only the callback id");
    assert.equal(calls[4][2].length, 1, "on-web-socket-error takes only the callback id");
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
    // `on-promise-resolved(promise-id, value: string)` — a required string, not
    // a `Response` object (`jco` cannot lower one into a `string`).
    // MUT: promise-bridge-passes-response-object
    assert.equal(
      typeof calls[0][2][1],
      "string",
      "the resolved value must be a string, or the call cannot be lowered",
    );

    // The rejection arm declares `error: string` just as firmly, and the bridge
    // used to call `handler(id)` there — an arity the wrapper cannot satisfy.
    // MUT: promise-bridge-rejects-without-error
    const rejected = withInstance(() => platformHelpers.fetchPromise("http://127.0.0.1:1/", null));
    await new Promise((resolve) => setTimeout(resolve, 400));
    const rejectCall = calls.find(([, name]) => name === "onPromiseRejected");
    assert.ok(rejectCall, "a failed fetch must report through onPromiseRejected");
    assert.equal(rejectCall[2][0], rejected);
    assert.equal(typeof rejectCall[2][1], "string", "`error` is a required string");
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
    // `on-position-success(callback-id, position: geo-position)` — the whole
    // record, not two loose numbers (`jco types`: `position: GeoPosition`).
    // MUT: geolocation-bridge-sends-loose-numbers
    const [callbackId, position] = calls[0][2];
    assert.equal(callbackId, 33n);
    assert.equal(typeof position, "object", "the position is a record, not a number");
    assert.equal(position.latitude, 12.5);
    assert.equal(position.longitude, -7.25);
    assert.equal(position.accuracy, 0, "required field is always present");
    assert.ok("altitude" in position && position.altitude === undefined);
  } finally {
    if (originalNavigator) Object.defineProperty(globalThis, "navigator", originalNavigator);
    else delete globalThis.navigator;
    delete globalThis.__wasmExports;
  }
});

test("every promise-callback dispatch passes the arity the WIT declares", () => {
  // `on-promise-resolved(promise-id, value: string)` and
  // `on-promise-rejected(promise-id, error: string)` both take a second,
  // REQUIRED string. Six arms dispatch into them (clipboard write, clipboard
  // read, `fetchPromise` resolve, and the fetch rejection arms); a per-arm test
  // would leave five of them pinned only by inspection, so this one walks the
  // source and asserts the CALL SHAPE at every site.
  // MUT: promise-arm-rejects-without-error
  const source = readFileSync(join(RUNTIME, "platformHelpers.ts"), "utf8");
  // `[^)]*` across newlines: the IndexedDB arms write `handler(\n callbackId,\n)`,
  // and a single-line slice silently skipped them (the "seven arms" count was
  // right by luck). MUT: arity-pin-single-line-slice
  const calls = [...source.matchAll(/if \(handler\) handler\(([^)]*)\)/gs)].map((m) =>
    m[1].replace(/\s+/g, " ").trim(),
  );
  assert.ok(calls.length >= 30, `expected every dispatch arm, saw ${calls.length}`);
  const promiseArms = calls.filter((args) => args === "id" || args.startsWith("id,"));
  // Seven: clipboard write + clipboard read + `fetchPromise` = 3 resolve arms,
  // and 4 rejection arms (write, read, JSON-parse, fetch).
  assert.equal(
    promiseArms.length,
    7,
    `the seven promise arms (3 resolve + 4 rejection) were expected, saw ${promiseArms.length}: ${promiseArms.join(" | ")}`,
  );
  for (const args of promiseArms) {
    assert.notEqual(
      args,
      "id",
      "a promise callback called with only the id cannot be lowered (the WIT requires a value/error string)",
    );
  }
});

/** Commas at bracket depth 0 outside string literals. */
function topLevelCommas(text) {
  let depth = 0;
  let inString = null;
  let commas = 0;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (inString) {
      if (char === "\\") index += 1;
      else if (char === inString) inString = null;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") inString = char;
    else if (char === "(" || char === "[" || char === "{") depth += 1;
    else if (char === ")" || char === "]" || char === "}") depth -= 1;
    else if (char === "," && depth === 0) commas += 1;
  }
  return commas;
}

test("every idb success arm passes the `result` the WIT declares", () => {
  // `on-idb-request-success(callback-id, result: option<string>)` — `jco types`
  // emits `onIdbRequestSuccess(callbackId: bigint, result: string | undefined)`.
  // Renaming the bridge from `on_idb_success` to `onIdbRequestSuccess` made
  // these arms FIRE; two of them passed only `callbackId`, one passed a handle
  // and one an array. A latent shape bug became a live one, so it is pinned.
  // MUT: idb-arms-missing-result
  const source = readFileSync(join(RUNTIME, "platformHelpers.ts"), "utf8");
  // Only the IDB arms: the file-reader arms also start with `callbackId` and do
  // carry their result, so they are not what this pins.
  // Parse the ARGUMENT LIST, not a `[^)]*` slice: `String(req.result?.version ??
  // "")` contains a `)`, so a slice ends early and one argument looks like two.
  // Parenthesis depth does the counting.
  // `[\s\S]*?` rather than `[^]*?` — the latter is a needless super-linear
  // backtracker that could exhaust a timeout and read as a pass.
  const arms = [...source.matchAll(/guest\?\.onIdbRequest(?:Success|Error)[\s\S]*?if \(handler\) handler\(/g)].map((m) => {
    let depth = 1;
    let index = m.index + m[0].length;
    const start = index;
    while (index < source.length && depth > 0) {
      if (source[index] === "(") depth += 1;
      else if (source[index] === ")") depth -= 1;
      if (depth === 0) break;
      index += 1;
    }
    return source.slice(start, index).replace(/\s+/g, " ").trim();
  });
  assert.ok(arms.length >= 5, `expected the idb arms, saw ${arms.length}`);
  for (const args of arms) {
    // A trailing comma is part of the slice, so "how many ARGUMENTS" means "how
    // many COMMAS" — `handler(\n callbackId,\n)` is one argument, not two.
    // Count TOP-LEVEL COMMAS with a depth counter. A hand-rolled loop here
    // silently under-counted (one arm with a nested `String(...)` swallowed the
    // comma that separated the arguments), which made the pin read green on a
    // real revert — so the counting is a small explicit helper whose behaviour
    // is asserted by this very loop.
    const normalised = args.replace(/[,\s]+$/, "");
    assert.notEqual(
      normalised,
      "callbackId",
      `an idb arm called with only the callback id leaves \`result\` undefined by accident, not by declaration: ${args}`,
    );
    assert.ok(
      topLevelCommas(normalised) >= 1,
      `an idb arm must pass a second argument (result): ${args}`,
    );
  }
});

test("window resize: onResizeCallback answers through `on-window-resize`", () => {
  // `on-resize-callback` is DECLARED and IMPLEMENTED, so the guest can call it —
  // and it used to dispatch into `…/resize-callbacks@0.2.0"]?.on_resize`, an
  // interface the world does not have, while an allow-list line called it
  // "never dispatched". The real contract is
  // `on-window-resize(callback-id, width: s32, height: s32)`.
  // MUT: resize-bridge-reads-resize-callbacks
  const calls = installGuest({ "window-resize-callbacks": ["onWindowResize"] });
  const originalWindow = globalThis.window;
  const listeners = [];
  try {
    globalThis.window = {
      addEventListener: (type, handler) => listeners.push([type, handler]),
      removeEventListener: () => {},
      innerWidth: 1024,
      innerHeight: 768,
    };
    withInstance(() => platformHelpers.onResizeCallback(21n));
    const resize = listeners.find(([type]) => type === "resize")[1];
    resize();
    assert.deepEqual(
      calls.map(([, name, args]) => [name, Array.from(args)]),
      [["onWindowResize", [21n, 1024, 768]]],
      "the bridge must call on-window-resize with the callback id and the two dimensions",
    );
  } finally {
    globalThis.window = originalWindow;
    delete globalThis.__wasmExports;
  }
});

test("a throwing guest callback does not escape into the DOM observer loop", () => {
  // The media-query bridge has wrapped its dispatch since #156; the resize and
  // mutation bridges were rewritten here without one, so a component bug became
  // an unhandled page error on every resize / mutation. Both are guarded now,
  // and the guard is what this pins — the sibling paths (`eventTarget.ts`,
  // `mediaQueryList.ts`) keep the same shape: log, do not propagate.
  // MUT: observer-bridge-unwrapped
  const originalError = console.error;
  const logged = [];
  console.error = (...args) => logged.push(args);
  try {
    const throwing = installGuest({
      "resize-observer-callbacks": ["onResize"],
      "mutation-observer-callbacks": ["onMutate"],
    });
    globalThis.__wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"].onResize = () => {
      throw new Error("guest exploded");
    };
    globalThis.__wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"].onMutate = () => {
      throw new Error("guest exploded");
    };
    withInstance(() => platformHelpers.createResizeObserver(1n));
    withInstance(() => platformHelpers.createMutationObserver(2n));
    const resizeObserver = globalThis.ResizeObserver.instances.at(-2);
    const mutationObserver = globalThis.ResizeObserver.instances.at(-1);
    assert.doesNotThrow(() =>
      resizeObserver.callback([fakeResizeEntry({ x: 0, y: 0, width: 1, height: 1 })], resizeObserver),
    );
    assert.doesNotThrow(() => mutationObserver.callback([fakeMutationRecord()], mutationObserver));
    assert.equal(logged.length, 2, `both throws must be logged, saw ${logged.length}`);
    assert.match(String(logged[0][0]), /resize-observer dispatch error/);
    assert.match(String(logged[1][0]), /mutation-observer dispatch error/);
    void throwing;
  } finally {
    console.error = originalError;
    delete globalThis.__wasmExports;
  }
});

test("every bridge reads the jco name alone (no fallback that nobody emits)", () => {
  // The fallback rule was tightened until it matched reality: no declared
  // callback name is single-segment, so no WIT-spelling alias exists, so a
  // bridge carrying `?? on_timeout`-style text would be advertising a
  // compatibility path that can never run. This asserts the tree is clean of
  // them — the naming gate refuses such a pair anyway, and this is the
  // behavioural side of the same statement.
  // MUT: bridge-carries-dead-fallback
  const source = readFileSync(join(RUNTIME, "platformHelpers.ts"), "utf8");
  const pairs = [
    ...source.matchAll(/(\w+)\?\.(on[A-Z][A-Za-z]*) \?\? \1\?\.(on_[a-z_]+)/g),
  ].map((m) => `${m.group(2)} ?? ${m.group(3)}`);
  assert.deepEqual(pairs, [], `dead fallbacks in the tree: ${pairs.join(", ")}`);
});
