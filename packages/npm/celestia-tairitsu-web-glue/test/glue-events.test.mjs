// Behaviour tests for the `event-target` glue: which `event-callbacks` export a DOM
// event is delivered through.
//
// The module under test is the source file the glue package ships, imported directly
// (Node strips the TypeScript). Every host facility it reaches for is faked below, so
// the test drives the real dispatch chain with no browser and no WASM component.
//
// Run with: pnpm --filter @celestia-island/tairitsu-web-glue test
//      (or) node --test packages/npm/celestia-tairitsu-web-glue/test

import assert from "node:assert/strict";
import test from "node:test";

// ------
// Host surfaces the glue expects from the rest of the bundle
// ------

globalThis.__elementHandles = new Map();
globalThis.__lookupElement = (handle) => globalThis.__elementHandles.get(handle);

const { eventTarget_exports } = await import("../src/glue-events.ts");

const CALLBACKS_KEY = "tairitsu-browser:full/event-callbacks@0.2.0";

let nextElementHandle = 1n;

/** Register a fake element and record the listener the glue attaches to it. */
function makeElement() {
  const handle = nextElementHandle++;
  const listeners = new Map();
  const element = {
    listeners,
    addEventListener(type, listener) {
      listeners.set(type, listener);
    },
    removeEventListener(type) {
      listeners.delete(type);
    },
  };
  globalThis.__elementHandles.set(handle, element);
  return { handle, element };
}

/** Install a component export table and return the calls it received. */
function installComponent(overrides = {}) {
  const calls = [];
  const record = (name) => (...args) => calls.push([name, ...args]);
  globalThis.__wasmExports = {
    [CALLBACKS_KEY]: {
      onMouseEvent: record("onMouseEvent"),
      onKeyboardEvent: record("onKeyboardEvent"),
      onFocusEvent: record("onFocusEvent"),
      onInputEvent: record("onInputEvent"),
      onPointerEvent: record("onPointerEvent"),
      onGenericEvent: record("onGenericEvent"),
      ...overrides,
    },
  };
  return calls;
}

/** A `PointerEvent`-shaped object with every field the glue reads a distinct value. */
function pointerEventPayload(element, type) {
  return {
    type,
    target: element,
    currentTarget: element,
    pointerId: 7,
    pointerType: "pen",
    isPrimary: true,
    clientX: 11,
    clientY: 12,
    screenX: 13,
    screenY: 14,
    offsetX: 15,
    offsetY: 16,
    pageX: 17,
    pageY: 18,
    movementX: 19,
    movementY: 20,
    width: 21,
    height: 22,
    pressure: 0.5,
    tangentialPressure: 0.25,
    tiltX: 30,
    tiltY: -15,
    twist: 45,
    button: -1,
    buttons: 24,
    ctrlKey: false,
    shiftKey: true,
    altKey: false,
    metaKey: false,
    // Event-interface members the glue touches through the handle table.
    stopPropagation() {},
    stopImmediatePropagation() {},
    preventDefault() {},
    composedPath: () => [element],
    bubbles: true,
    cancelable: true,
    defaultPrevented: false,
    eventPhase: 2,
    timeStamp: 0,
    isTrusted: true,
  };
}

/** Attach a listener through the glue and return the DOM listener it registered. */
function listen(handle, element, type) {
  const listenerHandle = eventTarget_exports.addEventListener(handle, type, false);
  assert.equal(typeof listenerHandle, "bigint", "addEventListener should return a handle");
  const listener = element.listeners.get(type);
  assert.equal(typeof listener, "function", `no listener registered for "${type}"`);
  return listener;
}

// ------
// Tests
// ------

test("pointerdown is delivered as a typed pointer event", () => {
  const calls = installComponent();
  const { handle, element } = makeElement();
  const listener = listen(handle, element, "pointerdown");

  listener(pointerEventPayload(element, "pointerdown"));

  const generic = calls.filter(([name]) => name === "onGenericEvent");
  assert.deepEqual(generic, [], "pointerdown must not fall through to onGenericEvent");

  const pointer = calls.filter(([name]) => name === "onPointerEvent");
  assert.equal(pointer.length, 1, "pointerdown should reach onPointerEvent exactly once");

  const [, listenerId, , data] = pointer[0];
  assert.equal(typeof listenerId, "bigint");
  assert.deepEqual(data, {
    pointerId: 7,
    pointerType: "pen",
    isPrimary: true,
    clientX: 11,
    clientY: 12,
    screenX: 13,
    screenY: 14,
    offsetX: 15,
    offsetY: 16,
    pageX: 17,
    pageY: 18,
    movementX: 19,
    movementY: 20,
    width: 21,
    height: 22,
    pressure: 0.5,
    tangentialPressure: 0.25,
    tiltX: 30,
    tiltY: -15,
    twist: 45,
    button: -1,
    buttons: 24,
    ctrlKey: false,
    shiftKey: true,
    altKey: false,
    metaKey: false,
  });
});

test("every pointer event type is routed to onPointerEvent", () => {
  const types = [
    "pointerdown",
    "pointermove",
    "pointerup",
    "pointercancel",
    "pointerenter",
    "pointerleave",
    "pointerover",
    "pointerout",
    "gotpointercapture",
    "lostpointercapture",
  ];

  for (const type of types) {
    const calls = installComponent();
    const { handle, element } = makeElement();
    listen(handle, element, type)(pointerEventPayload(element, type));

    assert.deepEqual(
      calls.map(([name]) => name),
      ["onPointerEvent"],
      `"${type}" should reach only onPointerEvent`,
    );
  }
});

test("a component without onPointerEvent still delivers the event", () => {
  const calls = installComponent({ onPointerEvent: undefined });
  const { handle, element } = makeElement();
  listen(handle, element, "pointerdown")(pointerEventPayload(element, "pointerdown"));

  const names = calls.map(([name]) => name);
  assert.deepEqual(names, ["onGenericEvent"], "the fallback must be a single generic event");
  assert.equal(calls[0][3], "pointerdown", "the generic event should keep the event type");
});

test("non-pointer events keep their existing branches", () => {
  const cases = [
    ["click", "onMouseEvent"],
    ["mousemove", "onMouseEvent"],
    // wheel stays on the mouse branch: it has no pointer identity to carry.
    ["wheel", "onMouseEvent"],
    ["keydown", "onKeyboardEvent"],
    ["focus", "onFocusEvent"],
    ["input", "onInputEvent"],
    ["submit", "onGenericEvent"],
  ];

  for (const [type, expected] of cases) {
    const calls = installComponent();
    const { handle, element } = makeElement();
    listen(handle, element, type)({
      ...pointerEventPayload(element, type),
      key: "a",
      code: "KeyA",
      keyCode: 65,
      repeat: false,
      data: "x",
      inputType: "insertText",
    });

    assert.deepEqual(
      calls.map(([name]) => name),
      [expected],
      `"${type}" should reach only ${expected}`,
    );
  }
});
