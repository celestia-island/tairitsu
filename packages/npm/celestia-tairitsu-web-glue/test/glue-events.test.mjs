// Behaviour tests for the `event-target`, `mouse-event`, `pointer-event` and
// `wheel-event` glue.
//
// Two things are asserted here:
//   1. the event getters read the real property off the host event object the glue
//      stored for that handle (this is the whole of the pointer payload the
//      component can see — the WIT callback record carries none of it), and
//   2. the dispatch whitelist still sends each DOM event type down its old branch,
//      with wheel no longer arriving as a mouse event.
//
// The module under test is the shipped source file, imported directly (Node strips
// the TypeScript); the host facilities it reaches for are faked below, so this runs
// with no browser and no WASM component.
//
// Run with: npm run test   (or: node --test "test/**/*.test.mjs")

import assert from "node:assert/strict";
import test from "node:test";

// ------
// Host surfaces the glue expects from the rest of the bundle
// ------

globalThis.__elementHandles = new Map();
globalThis.__lookupElement = (handle) => globalThis.__elementHandles.get(handle);

const glue = await import("../src/glue-events.ts");
const {
  INTERFACES,
  eventTarget_exports,
  mouseEvent_exports,
  pointerEvent_exports,
  wheelEvent_exports,
} = glue;

const CALLBACKS_KEY = "tairitsu-browser:full/event-callbacks@0.2.0";

let nextElementHandle = 1n;
let nextEventHandle = 1000n;

/** Put a fake host event in the handle table and return its handle. */
function hostEvent(fields) {
  const handle = nextEventHandle++;
  globalThis.__eventHandles.set(handle, fields);
  return handle;
}

/** A `PointerEvent`-shaped object with every getter-visible field distinct. */
function pointerEventFields(overrides = {}) {
  return {
    type: "pointerdown",
    pointerId: 7,
    pointerType: "pen",
    isPrimary: true,
    width: 21,
    height: 22,
    pressure: 0.5,
    tangentialPressure: 0.25,
    tiltX: 30,
    tiltY: -15,
    twist: 45,
    altitudeAngle: 1.25,
    azimuthAngle: 0.5,
    persistentDeviceId: 3,
    clientX: 11,
    clientY: 12,
    screenX: 13,
    screenY: 14,
    offsetX: 15,
    offsetY: 16,
    pageX: 17,
    pageY: 18,
    button: -1,
    buttons: 24,
    ctrlKey: false,
    shiftKey: true,
    altKey: false,
    metaKey: false,
    ...overrides,
  };
}

// ------
// Getters
// ------

test("mouse-event getters read the host event", () => {
  const handle = hostEvent({
    clientX: 11,
    clientY: 12,
    screenX: 13,
    screenY: 14,
    offsetX: 15,
    offsetY: 16,
    pageX: 17,
    pageY: 18,
    x: 19,
    y: 20,
    layerX: 21,
    layerY: 22,
    button: -1,
    buttons: 24,
    ctrlKey: false,
    shiftKey: true,
    altKey: false,
    metaKey: false,
    getModifierState: (key) => key === "Alt",
  });

  assert.equal(mouseEvent_exports.getClientX(handle), 11);
  assert.equal(mouseEvent_exports.getClientY(handle), 12);
  assert.equal(mouseEvent_exports.getScreenX(handle), 13);
  assert.equal(mouseEvent_exports.getScreenY(handle), 14);
  assert.equal(mouseEvent_exports.getOffsetX(handle), 15);
  assert.equal(mouseEvent_exports.getOffsetY(handle), 16);
  assert.equal(mouseEvent_exports.getPageX(handle), 17);
  assert.equal(mouseEvent_exports.getPageY(handle), 18);
  assert.equal(mouseEvent_exports.getX(handle), 19);
  assert.equal(mouseEvent_exports.getY(handle), 20);
  assert.equal(mouseEvent_exports.getLayerX(handle), 21);
  assert.equal(mouseEvent_exports.getLayerY(handle), 22);
  // Signed and wide enough that "-1" and the back/forward bits survive.
  assert.equal(mouseEvent_exports.getButton(handle), -1);
  assert.equal(mouseEvent_exports.getButtons(handle), 24);
  assert.equal(mouseEvent_exports.getCtrlKey(handle), false);
  assert.equal(mouseEvent_exports.getShiftKey(handle), true);
  assert.equal(mouseEvent_exports.getAltKey(handle), false);
  assert.equal(mouseEvent_exports.getMetaKey(handle), false);
  assert.equal(mouseEvent_exports.getModifierState(handle, "Alt"), true);
  assert.equal(mouseEvent_exports.getModifierState(handle, "Control"), false);
});

test("mouse-event getRelatedTarget maps the DOM node back to its handle", () => {
  const nodeHandle = nextElementHandle++;
  const node = {};
  globalThis.__elementHandles.set(nodeHandle, node);

  const withTarget = hostEvent({ relatedTarget: node });
  assert.equal(mouseEvent_exports.getRelatedTarget(withTarget), nodeHandle);

  const withoutTarget = hostEvent({ relatedTarget: null });
  assert.equal(mouseEvent_exports.getRelatedTarget(withoutTarget), undefined);
});

test("pointer-event getters read the host event", () => {
  const handle = hostEvent(pointerEventFields());

  assert.equal(pointerEvent_exports.getPointerId(handle), 7);
  assert.equal(pointerEvent_exports.getPointerType(handle), "pen");
  assert.equal(pointerEvent_exports.getIsPrimary(handle), true);
  assert.equal(pointerEvent_exports.getWidth(handle), 21);
  assert.equal(pointerEvent_exports.getHeight(handle), 22);
  assert.equal(pointerEvent_exports.getPressure(handle), 0.5);
  assert.equal(pointerEvent_exports.getTangentialPressure(handle), 0.25);
  assert.equal(pointerEvent_exports.getTiltX(handle), 30);
  assert.equal(pointerEvent_exports.getTiltY(handle), -15);
  assert.equal(pointerEvent_exports.getTwist(handle), 45);
  assert.equal(pointerEvent_exports.getAltitudeAngle(handle), 1.25);
  assert.equal(pointerEvent_exports.getAzimuthAngle(handle), 0.5);
  assert.equal(pointerEvent_exports.getPersistentDeviceId(handle), 3);
});

test("pointer-event getters survive a pointercancel with no coordinates", () => {
  // `pointercancel` carries no position and no contact geometry; every getter has to
  // come back with a usable default instead of throwing.
  const handle = hostEvent({ type: "pointercancel" });

  assert.equal(pointerEvent_exports.getPointerId(handle), 0);
  assert.equal(pointerEvent_exports.getPointerType(handle), "");
  assert.equal(pointerEvent_exports.getIsPrimary(handle), false);
  assert.equal(pointerEvent_exports.getWidth(handle), 0);
  assert.equal(pointerEvent_exports.getHeight(handle), 0);
  assert.equal(pointerEvent_exports.getPressure(handle), 0);
  assert.equal(pointerEvent_exports.getTiltX(handle), 0);
  assert.equal(pointerEvent_exports.getTwist(handle), 0);
  assert.equal(mouseEvent_exports.getClientX(handle), 0);
  assert.equal(mouseEvent_exports.getShiftKey(handle), false);
  assert.deepEqual(pointerEvent_exports.getCoalescedEvents(handle), []);
});

test("pointer-event coalesced events get their own handles", () => {
  const first = { type: "pointermove", clientX: 1 };
  const second = { type: "pointermove", clientX: 2 };
  const handle = hostEvent({
    type: "pointermove",
    getCoalescedEvents: () => [first, second],
    getPredictedEvents: () => [first],
  });

  const coalesced = pointerEvent_exports.getCoalescedEvents(handle);
  assert.equal(coalesced.length, 2);
  assert.equal(globalThis.__eventHandles.get(coalesced[0]), first);
  assert.equal(globalThis.__eventHandles.get(coalesced[1]), second);

  const predicted = pointerEvent_exports.getPredictedEvents(handle);
  assert.equal(predicted.length, 1);
  assert.equal(globalThis.__eventHandles.get(predicted[0]), first);
});

test("wheel-event getters read the host event", () => {
  const handle = hostEvent({ deltaX: -1.5, deltaY: 120, deltaZ: 0.5, deltaMode: 1 });

  assert.equal(wheelEvent_exports.getDeltaX(handle), -1.5);
  assert.equal(wheelEvent_exports.getDeltaY(handle), 120);
  assert.equal(wheelEvent_exports.getDeltaZ(handle), 0.5);
  assert.equal(wheelEvent_exports.getDeltaMode(handle), 1);
});

test("every getter tolerates an unknown handle", () => {
  const missing = 999_999n;

  for (const [iface, exports] of Object.entries({
    mouse: mouseEvent_exports,
    pointer: pointerEvent_exports,
    wheel: wheelEvent_exports,
  })) {
    for (const [name, fn] of Object.entries(exports)) {
      if (typeof fn !== "function") continue;
      if (name === "getCoalescedEvents" || name === "getPredictedEvents") continue;
      const arity = fn.length;
      const value = arity > 1 ? fn(missing, "Alt") : fn(missing);
      assert.ok(
        value === 0 || value === "" || value === false || value === undefined,
        `${iface}.${name} should return a default for an unknown handle, got ${String(value)}`,
      );
    }
  }
});

test("the three getter interfaces are registered for the component", () => {
  assert.equal(INTERFACES["@tairitsu-glue/mouse-event"], mouseEvent_exports);
  assert.equal(INTERFACES["@tairitsu-glue/pointer-event"], pointerEvent_exports);
  assert.equal(INTERFACES["@tairitsu-glue/wheel-event"], wheelEvent_exports);
});

// ------
// Dispatch routing
// ------

/** Register a fake element and return the handle the glue can attach listeners to. */
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
function installComponent() {
  const calls = [];
  const record = (name) => (...args) => calls.push([name, ...args]);
  globalThis.__wasmExports = {
    [CALLBACKS_KEY]: {
      onMouseEvent: record("onMouseEvent"),
      onKeyboardEvent: record("onKeyboardEvent"),
      onFocusEvent: record("onFocusEvent"),
      onInputEvent: record("onInputEvent"),
      onGenericEvent: record("onGenericEvent"),
    },
  };
  return calls;
}

/** Attach a listener through the glue and return the DOM listener it registered. */
function listen(handle, element, type) {
  const listenerHandle = eventTarget_exports.addEventListener(handle, type, false);
  assert.equal(typeof listenerHandle, "bigint", "addEventListener should return a handle");
  const listener = element.listeners.get(type);
  assert.equal(typeof listener, "function", `no listener registered for "${type}"`);
  return listener;
}

test("pointer and wheel reach the component through the generic callback", () => {
  // The callback record carries no pointer payload, so the component reads it back
  // through the getter interfaces; the glue must pass the event handle through
  // untouched for that to work.
  for (const type of ["pointerdown", "pointermove", "pointercancel", "wheel"]) {
    const calls = installComponent();
    const { handle, element } = makeElement();
    const payload = { ...pointerEventFields(), type, target: element, currentTarget: element };
    listen(handle, element, type)(payload);

    assert.deepEqual(
      calls.map(([name]) => name),
      ["onGenericEvent"],
      `"${type}" should reach the generic callback only`,
    );

    const [, listenerId, eventHandle, reportedType] = calls[0];
    assert.equal(typeof listenerId, "bigint");
    // The component reads the host event back through this handle.
    assert.equal(globalThis.__eventHandles.get(eventHandle), payload);
    assert.equal(reportedType, type);
  }
});

test("non-pointer events keep their existing branches", () => {
  const cases = [
    ["click", "onMouseEvent"],
    ["mousemove", "onMouseEvent"],
    ["contextmenu", "onMouseEvent"],
    ["keydown", "onKeyboardEvent"],
    ["focus", "onFocusEvent"],
    ["input", "onInputEvent"],
    ["submit", "onGenericEvent"],
  ];

  for (const [type, expected] of cases) {
    const calls = installComponent();
    const { handle, element } = makeElement();
    listen(handle, element, type)({
      ...pointerEventFields(),
      type,
      target: element,
      currentTarget: element,
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
