// W4: per-instance handle tables, dispose-time reclaim, and instance isolation.
//
// The three regressions these pin, in the order the task names them:
//   1. two instances on one page must not see each other's handles;
//   2. a disposed instance's handles must stop resolving (the table is
//      cleared), and its listeners/observers/timers must be reclaimed;
//   3. `globalThis` must carry no mutable handle state — only delegating
//      accessors to the current instance.

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const INSTANCE_MODULE = resolve(here, "../src/runtime/instance.ts");
const {
  TairitsuInstance,
  createInstance,
  currentInstance,
  disposeInstance,
  installGlobalDelegates,
  installInstanceApi,
  listInstances,
  setCurrentInstance,
  withInstance,
} = await import(INSTANCE_MODULE);

installGlobalDelegates();
installInstanceApi();

/** Store a value the way the generated glue does, on a chosen instance. */
function store(instance, value) {
  return withInstance(instance, () => {
    const handle = globalThis.__nextHandle++;
    globalThis.__elementHandles.set(handle, value);
    return handle;
  });
}

test("handles are stored per instance, not per page", () => {
  const a = createInstance();
  const b = createInstance();

  const fromA = store(a, { owner: "a" });
  const fromB = store(b, { owner: "b" });

  // Handle numbers are instance-scoped (`nextHandle` is seeded from the
  // instance id), so a handle minted by B never resolves in A — it fails as
  // "not found" instead of silently naming A's object with the same number.
  assert.notEqual(fromA, fromB, "two instances minted the same handle number");
  assert.equal(a.tables.elementHandles.get(fromA).owner, "a");
  assert.equal(b.tables.elementHandles.get(fromB).owner, "b");
  assert.equal(a.tables.elementHandles.get(fromB), undefined, "B's handle resolved in A");
  assert.equal(b.tables.elementHandles.get(fromA), undefined, "A's handle resolved in B");

  // … and the delegating accessor follows the current instance.
  assert.equal(withInstance(a, () => globalThis.__elementHandles.get(fromA).owner), "a");
  assert.equal(withInstance(b, () => globalThis.__elementHandles.get(fromB).owner), "b");

  disposeInstance(a);
  disposeInstance(b);
});

test("EVERY handle counter is per instance and never collides", () => {
  const a = createInstance();
  const b = createInstance();
  const names = [
    "nextHandle",
    "nextMutationRecord",
    "nextResizeObserverEntry",
    "nextResizeObserverSizeHandle",
    "nextDomRectHandle",
    "nextNodeList",
    "nextCssStyleDeclarationHandle",
    "nextListenerHandle",
    "nextEventHandle",
  ];
  for (const name of names) {
    assert.notEqual(a.counters[name], b.counters[name], `${name} is shared across instances`);
    assert.equal(typeof a.counters[name], "bigint", `${name} is not a bigint counter`);
  }
  disposeInstance(a);
  disposeInstance(b);
});

test("the page-global indexes are delegated too", () => {
  for (const name of [
    "__elementReverseMap",
    "__listenerByElement",
    "__tairitsuResizeListeners",
    "__tairitsuScrollListeners",
    "__tairitsuWsCallbacks",
    "__subEventHandles",
  ]) {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
    assert.ok(descriptor, `${name} is not defined`);
    assert.equal(typeof descriptor.get, "function", `${name} is not a delegating accessor`);
  }
});

test("disposing an instance clears its handles (old handles stop resolving)", () => {
  const instance = createInstance();
  const handle = store(instance, { owner: "gone" });
  assert.equal(instance.tables.elementHandles.has(handle), true);

  assert.equal(disposeInstance(instance), true);
  assert.equal(instance.disposed, true);
  assert.equal(instance.tables.elementHandles.has(handle), false, "handle survived dispose");
  // The contract W4 pins: the existing lookups throw for an unknown handle.
  withInstance(instance, () => {
    assert.throws(
      () => {
        const el = globalThis.__elementHandles.get(handle);
        if (!el) throw new Error(`Element handle ${handle} not found`);
      },
      /not found/,
    );
  });
});

test("dispose reclaims listeners, observers, timers and intervals", () => {
  const instance = createInstance();

  const listeners = [];
  const fakeTarget = {
    removeEventListener(type, handler) {
      listeners.push([type, handler]);
    },
  };
  const handler = () => {};
  instance.reclaimables.listeners.add({ target: fakeTarget, handler, type: "click" });

  let disconnected = 0;
  instance.reclaimables.observers.add({ disconnect: () => disconnected++ });

  const cleared = [];
  const originalClearTimeout = globalThis.clearTimeout;
  const originalClearInterval = globalThis.clearInterval;
  globalThis.clearTimeout = (id) => cleared.push(["timeout", id]);
  globalThis.clearInterval = (id) => cleared.push(["interval", id]);
  try {
    instance.reclaimables.timers.add(11);
    instance.reclaimables.intervals.add(22);
    disposeInstance(instance);
  } finally {
    globalThis.clearTimeout = originalClearTimeout;
    globalThis.clearInterval = originalClearInterval;
  }

  assert.deepEqual(listeners, [["click", handler]], "the listener was not removed");
  assert.equal(disconnected, 1, "the observer was not disconnected");
  assert.deepEqual(cleared, [["timeout", 11], ["interval", 22]]);
  assert.equal(instance.reclaimables.listeners.size, 0);
  assert.equal(instance.reclaimables.timers.size, 0);
});

test("listeners and observers created through the glue are owned by the instance", async () => {
  // The reclaimables table is only useful if the registration sites actually
  // feed it. This drives the real `addEventListener` export against a fake
  // element and checks the instance owns the listener (and dispose removes it).
  // Import the helper module first: it installs `__lookupElement`, which the
  // the `__lookupElement` helper the exports rely on (the shipped bundle does
  // the same through `runtime/index.ts`).
  await import("../src/runtime/helpers.ts");
  const { eventTarget_exports } = await import("../src/runtime/eventTarget.ts");
  const instance = createInstance();
  const previous = setCurrentInstance(instance);
  const removed = [];
  const fakeElement = {
    addEventListener: () => {},
    removeEventListener: (type, handler) => removed.push([type, handler]),
  };
  try {
    instance.tables.elementHandles.set(7n, fakeElement);
    const listenerHandle = eventTarget_exports.addEventListener(7n, "click", false);
    assert.equal(typeof listenerHandle, "bigint");
    assert.equal(instance.reclaimables.listeners.size, 1, "the listener was not registered");
    const [entry] = [...instance.reclaimables.listeners];
    assert.equal(entry.target, fakeElement);
    assert.equal(entry.type, "click");
    disposeInstance(instance);
    assert.equal(removed.length, 1, "dispose did not remove the listener");
    assert.equal(removed[0][0], "click");
  } finally {
    setCurrentInstance(previous);
  }
});

test("disposing twice is reported, not silently repeated", () => {
  const instance = createInstance();
  assert.equal(disposeInstance(instance), true);
  assert.equal(disposeInstance(instance), false);
});

test("disposing one instance never touches another", () => {
  const a = createInstance();
  const b = createInstance();
  const keepB = store(b, { owner: "survivor" });

  disposeInstance(a);
  assert.equal(b.disposed, false);
  assert.equal(b.tables.elementHandles.get(keepB).owner, "survivor");
  assert.equal(withInstance(b, () => globalThis.__elementHandles.get(keepB).owner), "survivor");
  disposeInstance(b);
});

test("currentInstance() creates the page default on first use", () => {
  setCurrentInstance(null);
  const implicit = currentInstance();
  assert.ok(implicit instanceof TairitsuInstance);
  assert.equal(implicit.disposed, false);
  assert.ok(listInstances().includes(implicit));
});

test("every delegate name maps to a real table (the scroll regression)", async () => {
  // The first version shipped `__tairitsuScrollListeners` pointing at a table
  // key that did not exist (the field was `scrollListeners`), so the getter was
  // undefined and `onScroll` threw on every guest call. Every delegate must
  // resolve to a Map/Set/WeakMap, never undefined.
  await import("../src/runtime/helpers.ts");
  const instance = createInstance();
  const previous = setCurrentInstance(instance);
  const originalWindow = globalThis.window;
  try {
    const names = [
      "__elementHandles",
      "__documentHandles",
      "__nodeHandles",
      "__textHandles",
      "__mutationRecordHandles",
      "__resizeObserverEntryHandles",
      "__resizeObserverSizeHandles",
      "__domRectHandles",
      "__listenerHandles",
      "__eventHandles",
      "__dispatchingEvents",
      "__tairitsuScrollListeners",
      "__listenerByElement",
      "__tairitsuResizeListeners",
      "__tairitsuWsCallbacks",
      "__subEventHandles",
    ];
    // `__elementReverseMap` is deliberately absent until the reverse index is
    // built (an eager empty Map would make its builder early-return), so it is
    // checked below rather than here.
    for (const name of names) {
      const value = globalThis[name];
      assert.ok(
        value instanceof Map || value instanceof Set || value instanceof WeakMap,
        `${name} resolves to ${typeof value} instead of a table`,
      );
    }
    // …and the guest-facing port that used it still works (node has no
    // `window`, so give it the two methods the port calls).
    const windowCalls = [];
    globalThis.window = {
      addEventListener: (type) => windowCalls.push(["add", type]),
      removeEventListener: (type) => windowCalls.push(["remove", type]),
    };
    const { platformHelpers_exports } = await import("../src/runtime/platformHelpers.ts");
    const handle = platformHelpers_exports.onScroll(1n);
    assert.equal(typeof handle, "bigint", "onScroll no longer returns a handle");
    assert.equal(instance.tables.tairitsuScrollListeners.size, 1, "the scroll handler was not stored");
    assert.deepEqual(windowCalls[0], ["add", "scroll"]);
    disposeInstance(instance);
    assert.deepEqual(windowCalls.at(-1), ["remove", "scroll"], "dispose did not remove the listener");
  } finally {
    globalThis.window = originalWindow;
    setCurrentInstance(previous);
  }
});

test("withInstance restores the previous instance even when the call throws", () => {
  const a = createInstance();
  const b = createInstance();
  setCurrentInstance(a);
  assert.throws(() =>
    withInstance(b, () => {
      assert.equal(currentInstance(), b);
      throw new Error("boom");
    }),
  );
  assert.equal(currentInstance(), a, "the previous instance was not restored");
  setCurrentInstance(null);
  disposeInstance(a);
  disposeInstance(b);
});

test("disposing the current instance clears the pointer, and counters reset to the seed", () => {
  const instance = createInstance();
  setCurrentInstance(instance);
  const before = instance.counters.nextHandle;
  store(instance, "x");
  assert.ok(instance.counters.nextHandle > before);
  disposeInstance(instance);
  assert.equal(globalThis.__TAIRITSU_INSTANCES__.list().includes(instance), false);
  assert.equal(currentInstance() === instance, false, "the disposed instance stayed current");
  assert.equal(instance.counters.nextHandle, instance.seed, "the counter did not reset to the seed");
});

test("the boot template disposes its instance on a failed boot (source pin)", async () => {
  const page = await readFile(resolve(here, "../../packager/src/wasm/index.html.template"), "utf8");
  assert.match(page, /instanceApi\.dispose\(bootInstance\)/, "a failed boot no longer reclaims");
  assert.match(page, /instanceApi\.create\(\)/, "the boot no longer creates an instance");
});

test("NEGATIVE CONTROL — globalThis holds no mutable handle tables", async () => {
  // The tables must be accessors whose storage is the instance, not data
  // properties holding Maps. A regression to `globalThis.__elementHandles =
  // new Map()` (the pre-W4 shape) fails here.
  for (const name of [
    "__elementHandles",
    "__nodeHandles",
    "__eventHandles",
    "__listenerHandles",
    "__tairitsuScrollListeners",
    "__dispatchingEvents",
  ]) {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
    assert.ok(descriptor, `${name} is not defined`);
    assert.equal(typeof descriptor.get, "function", `${name} is not a delegating accessor`);
    assert.equal(typeof descriptor.set, "function", `${name} cannot be assigned`);
    assert.equal("value" in descriptor, false, `${name} is still a data property`);
  }
  // Writing through the compat name lands in the instance.
  const instance = createInstance();
  const previous = setCurrentInstance(instance);
  try {
    globalThis.__elementHandles = new Map([["k", "v"]]);
    assert.equal(instance.tables.elementHandles.get("k"), "v");
  } finally {
    setCurrentInstance(previous);
    disposeInstance(instance);
  }
});

test("the glue sources no longer initialise handle tables on globalThis", async () => {
  // Source-level half of the same pin: a fresh `globalThis.__xHandles =
  // new Map()` initialiser is exactly the pre-W4 shape.
  const sources = [
    "../src/runtime/handles.ts",
    "../src/runtime/eventTarget.ts",
    "../src/runtime/helpers.ts",
    "../src/runtime/elementCssInlineStyle.ts",
    "../src/runtime/platformHelpers.ts",
    "../src/runtime/parentNode.ts",
    "../src/runtime/pointerEvent.ts",
    "../src/runtime/node.ts",
    "../src/runtime/resizeObserverEntry.ts",
  ];
  for (const relative of sources) {
    const source = await readFile(resolve(here, relative), "utf8");
    // Both pre-W4 shapes: `globalThis.__x = new Map()` and the lazy
    // `globalThis.__x = globalThis.__x || new Map()`.
    const eager =
      source.match(
        /globalThis\.__\w*Handles\s*=\s*(?:globalThis\.__\w*\s*\|\||new Map\()/g,
      ) ?? [];
    assert.deepEqual(eager, [], `${relative} still creates a global handle table`);
  }
});