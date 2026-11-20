/**
 * Registry completeness + behaviour tests for the browser-glue runtime registry.
 *
 * Why this file exists: a transpiled tairitsu component imports every
 * `tairitsu-browser:full/*` interface its code path touches, and jco looks each
 * one up in the `imports` object the page builds from
 * `src/runtime/registry.ts`'s `INTERFACES`. A missing registration is not a
 * degraded feature — instantiation throws immediately:
 *
 *   TypeError: Cannot destructure property 'getValue' of
 *   'imports.tairitsu-browser:full/html-input-element' as it is undefined.
 *
 * That is exactly how every browser component was broken at HEAD: the world
 * declared hundreds of interfaces, the registry wired a hand-curated subset,
 * and four of the ones `tairitsu-web` itself imports were among the missing.
 * This suite pins the wiring against the WIT declarations, so the same gap
 * cannot come back silently (the interface-level counterpart lives in
 * `scripts/check_wit_surface.py`, which also carries the honest inventory of
 * interfaces that are still unwired):
 *
 *   1. every declared function of the four wired interfaces is either exported
 *      by the glue or listed in `KNOWN_UNWIRED` (adding one without wiring it,
 *      or removing a wired one, fails),
 *   2. the registry source registers each of them under the exact
 *      `@tairitsu-glue/<kebab-name>` key the page's `tairitsu-browser:full/<name>`
 *      mapping is derived from, and the imported symbol really exists,
 *   3. the functions behave like the DOM (fake event/element objects, no browser).
 *
 * Run with: npm run test
 *       or: node --experimental-strip-types --test test/<glob>.test.mjs
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
// Two composed copies exist and they must be byte-identical (single source =
// packages/browser-worlds/wit/composed; `just wit-sync` regenerates the copies,
// `just wit-sync-check` gates them):
//   - packages/web/wit/composed        — what in-tree builds resolve (build.rs
//                                        prefers this embedded copy);
//   - packages/browser-worlds/wit/composed — the source of truth.
const WIT_DIR = join(REPO, "packages", "web", "wit", "composed");
const WIT_SYNC_DIR = join(REPO, "packages", "browser-worlds", "wit", "composed");
const REGISTRY_SRC = readFileSync(join(PKG, "src", "runtime", "registry.ts"), "utf8");

// ---------------------------------------------------------------------------
// The four interfaces this module set wires, with the WIT file that declares
// them (the composed world a component is actually built against).
// ---------------------------------------------------------------------------
const WIRED = {
  "html-input-element": {
    wit: "_domain-html.wit",
    module: "htmlInputElement",
    // Declared-but-unwired functions, each with the reason. Kept explicit so a
    // newly declared function fails the suite until it is either wired or listed.
    unwired: [
      "get-files", // FileList handle — needs a FileList table
      "set-files",
      "get-form", // HTMLFormElement handle
      "get-labels", // NodeList handle
      "get-list", // HTMLDataListElement handle
      "get-validity", // ValidityState handle
    ],
  },
  "mouse-event": {
    wit: "_domain-css.wit",
    module: "mouseEvent",
    unwired: [
      "init-mouse-event", // deprecated DOM ctor API; needs a Window handle for `view`
    ],
  },
  "pointer-event": {
    wit: "_domain-events.wit",
    module: "pointerEvent",
    unwired: [],
  },
  "wheel-event": {
    wit: "_domain-events.wit",
    module: "wheelEvent",
    unwired: [],
  },
};

/** Function names declared by an interface in the composed WIT. */
function declaredFunctions(witFile, interfaceName, dir = WIT_DIR) {
  const text = readFileSync(join(dir, witFile), "utf8");
  const start = text.indexOf(`interface ${interfaceName} {`);
  assert.notEqual(start, -1, `${interfaceName} not found in ${witFile}`);
  const body = text.slice(start, text.indexOf("\n}", start));
  return [...body.matchAll(/^\s+([a-z][a-z0-9-]*): func\(/gm)].map((m) => m[1]);
}

// ---------------------------------------------------------------------------
// Host surfaces the glue expects (same contract the runtime bundle provides).
// ---------------------------------------------------------------------------
globalThis.__elementHandles = new Map();
globalThis.__nodeHandles = new Map();
globalThis.__textHandles = new Map();
globalThis.__eventHandles = new Map();
globalThis.__nextHandle = 1n;
globalThis.__nextEventHandle = 1n;
// Faithful to helpers.ts: __storeElement writes ONLY __elementHandles and
// __storeNode writes ONLY __nodeHandles — a more lenient fixture would mask
// handle-table routing bugs instead of catching them.
globalThis.__storeElement = (el) => {
  if (!el) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__elementHandles.set(handle, el);
  return handle;
};
globalThis.__storeNode = (node) => {
  if (!node) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__nodeHandles.set(handle, node);
  return handle;
};
globalThis.__lookupNode = (handle) => globalThis.__nodeHandles.get(handle);
globalThis.__lookupElement = (handle) => globalThis.__elementHandles.get(handle);

// Import the real registry: the assertions below then describe the object the
// page actually hands to jco, not a reconstruction of it.
const { INTERFACES } = await import("../src/runtime/registry.ts");
const node_exports = INTERFACES["@tairitsu-glue/node"];
assert.ok(node_exports, "the registry must wire @tairitsu-glue/node");

/** The registered glue object for one interface. */
function exportsOf(iface) {
  const exported = INTERFACES[`@tairitsu-glue/${iface}`];
  assert.ok(
    exported,
    `the registry must wire "@tairitsu-glue/${iface}" — a page cannot instantiate without it`,
  );
  return exported;
}

// ---------------------------------------------------------------------------
// 1. WIT coverage: declared === wired ∪ known-unwired, in both directions.
// ---------------------------------------------------------------------------
for (const [iface, spec] of Object.entries(WIRED)) {
  test(`${iface}: glue covers the whole WIT interface except the pinned gap`, () => {
    const declared = declaredFunctions(spec.wit, iface);
    assert.ok(declared.length > 0, `${iface} declares no functions?`);
    const wired = Object.keys(exportsOf(iface)).map((n) =>
      n.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`),
    );
    const covered = new Set([...wired, ...spec.unwired]);

    const missing = declared.filter((fn) => !covered.has(fn));
    assert.deepEqual(
      missing,
      [],
      `${iface}: declared in WIT but neither wired nor listed as known-unwired: ` +
        `${missing.join(", ")} — wire it in src/runtime/${spec.module}.ts or add it to ` +
        "KNOWN_UNWIRED in this test with the reason",
    );

    const extra = [...covered].filter((fn) => !declared.includes(fn));
    assert.deepEqual(
      extra,
      [],
      `${iface}: wired/listed but not declared by the WIT interface: ${extra.join(", ")}`,
    );
  });
}

test("wheel-event: momentum is read from the event, not hard-coded", async () => {
  // WebKit synthesises inertial wheel events and flags them with `momentum`;
  // other engines leave it undefined. The glue must forward the flag honestly —
  // a wrong default would make a host treat inertial scroll as a real gesture
  // (or the reverse). Signature coverage alone cannot catch this, so the
  // behaviour is pinned here.
  const { wheelEvent_exports } = await import("../src/runtime/wheelEvent.ts");
  const handle = 9001n;
  const previous = globalThis.__eventHandles;
  try {
    globalThis.__eventHandles = new Map([[handle, { momentum: true }]]);
    assert.equal(wheelEvent_exports.getMomentum(handle), true);
    globalThis.__eventHandles = new Map([[handle, { momentum: false }]]);
    assert.equal(wheelEvent_exports.getMomentum(handle), false);
    globalThis.__eventHandles = new Map([[handle, { deltaY: 4 }]]);
    assert.equal(wheelEvent_exports.getMomentum(handle), false, "absent momentum reads as a real gesture");
    globalThis.__eventHandles = new Map();
    assert.equal(wheelEvent_exports.getMomentum(handle), false, "unknown handle must not claim momentum");
  } finally {
    globalThis.__eventHandles = previous;
  }
});

test("wheel-event: the embedded and single-source copies agree", () => {
  // Until W2 these two copies desynced (the source declared `get-momentum`,
  // the embedded copy did not) and this test pinned that gap. The copies are
  // now regenerated from one source, `getMomentum` is wired in
  // `src/runtime/wheelEvent.ts`, and the pin becomes an equality assertion so
  // the same gap can never come back silently.
  const embedded = declaredFunctions("_domain-events.wit", "wheel-event");
  const synced = declaredFunctions("_domain-events.wit", "wheel-event", WIT_SYNC_DIR);
  assert.deepEqual(
    embedded,
    synced,
    "the embedded and single-source WIT disagree: run `just wit-sync`",
  );
  assert.ok(
    embedded.includes("get-momentum"),
    "the wheel-event interface lost get-momentum — regenerate the WIT instead of editing copies",
  );
});

// ---------------------------------------------------------------------------
// 2. Registry wiring: the page's import keys are derived from these entries.
// ---------------------------------------------------------------------------
test("every wired interface is registered under its bare WIT name", () => {
  for (const iface of Object.keys(WIRED)) {
    const exported = exportsOf(iface);
    assert.equal(typeof exported, "object");
    // The page derives the component-side specifier by stripping the
    // "@tairitsu-glue/" prefix (index.html.template), so the key has to be the
    // bare WIT kebab name: "@tairitsu-glue/mouse-event" -> "mouse-event".
    assert.ok(
      Object.keys(INTERFACES).includes(`@tairitsu-glue/${iface}`),
      `registry must key ${iface} as "@tairitsu-glue/${iface}"`,
    );
    assert.ok(
      declaredFunctions(WIRED[iface].wit, iface).length > 0,
      `${iface} must be a real WIT interface`,
    );
  }
});

test("registry.ts imports the glue modules rather than inlining them", () => {
  // Source-level companion to the runtime assertion above: a copy-pasted object
  // literal in registry.ts would stop tracking its module silently.
  for (const [iface, spec] of Object.entries(WIRED)) {
    assert.ok(
      REGISTRY_SRC.includes(`"@tairitsu-glue/${iface}"`),
      `registry.ts must register "@tairitsu-glue/${iface}"`,
    );
    const importRe = new RegExp(
      `import \\{ ${spec.module}_exports \\} from "\\./${spec.module}"`,
    );
    assert.match(
      REGISTRY_SRC,
      importRe,
      `registry.ts must import ${spec.module}_exports from ./${spec.module}`,
    );
  }
});

test("node glue gained the two functions the transpiled wrapper destructures", () => {
  for (const fn of ["getNodeType", "replaceChild"]) {
    assert.equal(
      typeof node_exports[fn],
      "function",
      `node_exports.${fn} is required: tairitsu-web's vdom calls WIT node.get-node-type / node.replace-child`,
    );
  }
  // Everything node exports must exist in the WIT interface (no invented names).
  const declared = new Set(declaredFunctions("_domain-dom.wit", "node"));
  for (const name of Object.keys(node_exports)) {
    const kebab = name.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
    assert.ok(declared.has(kebab), `node_exports.${name} is not declared by WIT node.${kebab}`);
  }
});

// ---------------------------------------------------------------------------
// 3. Behaviour: the functions must map WIT to DOM the way the DOM does.
// ---------------------------------------------------------------------------
test("html-input-element: value/checked/selection round-trip on the element", () => {
  const input = {
    nodeType: 1,
    value: "hello",
    defaultValue: "hello",
    checked: true,
    disabled: false,
    required: false,
    type: "text",
    minLength: 0,
    maxLength: 20,
    selectionStart: 1,
    selectionEnd: 4,
    selectionDirection: "forward",
    valueAsNumber: 42,
    valueAsDate: new Date(Date.UTC(2026, 0, 2)),
    willValidate: true,
    validationMessage: "",
    setSelectionRange(start, end, direction) {
      this.selectionStart = start;
      this.selectionEnd = end;
      this.selectionDirection = direction ?? this.selectionDirection;
    },
    checkValidity: () => true,
    reportValidity: () => false,
    setCustomValidity(msg) {
      this.validationMessage = msg;
    },
    select() {
      this.selected = true;
    },
  };
  const h = globalThis.__storeElement(input);

  assert.equal(exportsOf("html-input-element").getValue(h), "hello");
  assert.equal(exportsOf("html-input-element").getChecked(h), true);
  assert.equal(exportsOf("html-input-element").getMaxLength(h), 20);
  assert.equal(exportsOf("html-input-element").getValueAsNumber(h), 42);
  assert.equal(
    exportsOf("html-input-element").getValueAsDate(h),
    BigInt(input.valueAsDate.getTime()),
  );

  exportsOf("html-input-element").setValue(h, "world");
  assert.equal(input.value, "world");
  // The setter must go through the DOM property, not a shadow copy: this is the
  // path ChangeEvent.value takes after a user edit.
  assert.equal(exportsOf("html-input-element").getValue(h), "world");

  exportsOf("html-input-element").setValueAsNumber(h, 7);
  assert.equal(input.valueAsNumber, 7);

  assert.equal(exportsOf("html-input-element").getSelectionStart(h), 1);
  exportsOf("html-input-element").setSelectionRange(h, 2, 5, "backward");
  assert.deepEqual(
    [input.selectionStart, input.selectionEnd, input.selectionDirection],
    [2, 5, "backward"],
  );

  exportsOf("html-input-element").setCustomValidity(h, "nope");
  assert.equal(exportsOf("html-input-element").getValidationMessage(h), "nope");
  assert.equal(exportsOf("html-input-element").checkValidity(h), true);
  assert.equal(exportsOf("html-input-element").reportValidity(h), false);
});

test("html-input-element: pre-1970 dates resolve to none, not a wrapped u64", () => {
  // 1969-12-31 is a valid date-state value whose epoch is negative; WIT says u64.
  const birthday = {
    nodeType: 1,
    valueAsDate: new Date(Date.UTC(1969, 11, 31)),
    value: "1969-12-31",
  };
  const h = globalThis.__storeElement(birthday);
  assert.equal(
    exportsOf("html-input-element").getValueAsDate(h),
    undefined,
    "a negative epoch cannot cross the u64 boundary; it must be none, never 0 or a wrapped value",
  );
  // The lossless path for those dates stays available.
  assert.equal(exportsOf("html-input-element").getValue(h), "1969-12-31");

  // Exactly the epoch is representable and must NOT be confused with "none".
  const epoch = { nodeType: 1, valueAsDate: new Date(0) };
  assert.equal(exportsOf("html-input-element").getValueAsDate(globalThis.__storeElement(epoch)), 0n);
});

test("html-input-element: absent DOM values map to the WIT defaults, not NaN/throw", () => {
  const bare = { nodeType: 1 };
  const h = globalThis.__storeElement(bare);
  assert.equal(exportsOf("html-input-element").getValue(h), "");
  assert.equal(exportsOf("html-input-element").getChecked(h), false);
  assert.equal(exportsOf("html-input-element").getValueAsDate(h), undefined);
  assert.equal(exportsOf("html-input-element").getSelectionStart(h), undefined);
});

test("mouse-event: coordinates and modifiers come from the event handle", () => {
  const ev = {
    clientX: 10,
    clientY: 20,
    screenX: 1,
    screenY: 2,
    pageX: 3,
    pageY: 4,
    offsetX: 5,
    offsetY: 6,
    x: 10,
    y: 20,
    layerX: 7,
    layerY: 8,
    ctrlKey: false,
    shiftKey: true,
    altKey: false,
    metaKey: true,
    button: 2,
    buttons: 2,
    relatedTarget: null,
    getModifierState: (key) => key === "Shift",
  };
  const h = globalThis.__nextEventHandle++;
  globalThis.__eventHandles.set(h, ev);

  assert.equal(exportsOf("mouse-event").getClientX(h), 10);
  assert.equal(exportsOf("mouse-event").getOffsetY(h), 6);
  assert.equal(exportsOf("mouse-event").getLayerX(h), 7);
  assert.equal(exportsOf("mouse-event").getShiftKey(h), true);
  assert.equal(exportsOf("mouse-event").getMetaKey(h), true);
  assert.equal(exportsOf("mouse-event").getButton(h), 2);
  assert.equal(exportsOf("mouse-event").getRelatedTarget(h), undefined);
  assert.equal(exportsOf("mouse-event").getModifierState(h, "Shift"), true);
  assert.equal(exportsOf("mouse-event").getModifierState(h, "Alt"), false);
});

test("mouse-event: relatedTarget resolves to a usable element handle", () => {
  const target = { nodeType: 1, tagName: "BUTTON" };
  const ev = { relatedTarget: target };
  const h = globalThis.__nextEventHandle++;
  globalThis.__eventHandles.set(h, ev);

  const related = exportsOf("mouse-event").getRelatedTarget(h);
  assert.equal(typeof related, "bigint", "related target must come back as a handle");
  assert.equal(globalThis.__lookupElement(related), target);
});

test("pointer-event: pointerType is the DOM string and sub-events get handles", () => {
  const coalesced = [{ pointerId: 1 }, { pointerId: 2 }];
  const ev = {
    pointerId: 7,
    width: 2.5,
    height: 3.5,
    pressure: 0.5,
    tangentialPressure: 0.1,
    tiltX: 4,
    tiltY: -4,
    twist: 90,
    altitudeAngle: 1.2,
    azimuthAngle: 0.3,
    pointerType: "pen",
    isPrimary: true,
    persistentDeviceId: 9,
    getCoalescedEvents: () => coalesced,
    getPredictedEvents: () => [],
  };
  const h = globalThis.__nextEventHandle++;
  globalThis.__eventHandles.set(h, ev);

  const p = exportsOf("pointer-event");
  assert.equal(p.getPointerType(h), "pen");
  assert.equal(p.getPointerId(h), 7);
  assert.equal(p.getPressure(h), 0.5);
  assert.equal(p.getTiltY(h), -4);
  assert.equal(p.getIsPrimary(h), true);

  const handles = p.getCoalescedEvents(h);
  assert.equal(handles.length, 2);
  assert.equal(globalThis.__eventHandles.get(handles[1]), coalesced[1]);
  // Same sub-event object => same handle: polling must not grow the table.
  assert.deepEqual(p.getCoalescedEvents(h), handles);
  assert.deepEqual(p.getPredictedEvents(h), []);
});

test("wheel-event: deltas come from the event handle", () => {
  const h = globalThis.__nextEventHandle++;
  globalThis.__eventHandles.set(h, { deltaX: 1.5, deltaY: -120, deltaZ: 0, deltaMode: 1 });
  const w = exportsOf("wheel-event");
  assert.equal(w.getDeltaY(h), -120);
  assert.equal(w.getDeltaMode(h), 1);
});

test("elementReverse tolerates a realm without the runtime entry (no handles/helpers)", async () => {
  // The registry (and so this index) can be imported without `runtime/index.ts`,
  // which is what installs `./handles` + `./helpers`. That realm has no handle
  // table and no store hook: lookups must resolve to none instead of throwing
  // inside an event handler.
  const savedHandles = globalThis.__elementHandles;
  const savedStore = globalThis.__storeElement;
  const savedIndex = globalThis.__elementReverseMap;
  try {
    delete globalThis.__elementHandles;
    delete globalThis.__storeElement;
    delete globalThis.__elementReverseMap;
    const fresh = await import("../src/runtime/elementReverse.ts?no-runtime-entry");
    const node = { nodeType: 1 };
    assert.equal(fresh.existingElementHandle(node), undefined);
    assert.equal(fresh.elementHandleFor(node), undefined);
    assert.equal(fresh.elementHandleFor(null), undefined);
    assert.ok(
      globalThis.__elementHandles instanceof Map,
      "seeding an empty table keeps later stores cheap instead of re-deriving",
    );
  } finally {
    globalThis.__elementHandles = savedHandles;
    globalThis.__storeElement = savedStore;
    globalThis.__elementReverseMap = savedIndex;
  }
});

test("node: getNodeType and replaceChild follow the DOM contract", () => {
  const parent = {
    nodeType: 1,
    children: [],
    replaceChild(next, old) {
      const at = this.children.indexOf(old);
      if (at === -1) throw new Error("not a child");
      this.children[at] = next;
      next.parentElement = this;
      old.parentElement = null;
      return old;
    },
  };
  const oldChild = { nodeType: 3, parentElement: parent };
  const newChild = { nodeType: 1, parentElement: null };
  parent.children.push(oldChild);

  const parentHandle = globalThis.__storeNode(parent);
  const oldHandle = globalThis.__storeNode(oldChild);
  const newHandle = globalThis.__storeNode(newChild);

  assert.equal(node_exports.getNodeType(parentHandle), 1);
  assert.equal(node_exports.getNodeType(oldHandle), 3);

  const removedHandle = node_exports.replaceChild(parentHandle, newHandle, oldHandle);
  assert.equal(
    globalThis.__lookupNode(removedHandle),
    oldChild,
    "replaceChild must return the REMOVED node (what the WIT contract says)",
  );
  assert.equal(parent.children[0], newChild, "the replacement must be in place");
});
