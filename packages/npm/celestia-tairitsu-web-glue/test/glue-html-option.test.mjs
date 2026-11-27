// Behaviour tests for the `html-option-element` glue.
//
// Asserted here:
//   1. the glue exports a function for every member the WIT interface
//      declares, read off the WIT file — a member added there without an
//      export fails here instead of trapping in the host;
//   2. the registry binds that table under the interface's name, which is the
//      only route the host takes to it;
//   3. every declared getter reads the DOM-absent default off an unknown
//      handle instead of trapping, and no setter stores anything for one;
//   4. real property reads return the real values off a stored option;
//   5. `setSelected` assigns the live selectedness — what `getSelected`
//      reports — and leaves the `selected` attribute to `setDefaultSelected`.
//
// The module under test is the shipped source file, imported directly (Node
// strips the TypeScript); the host surfaces it reaches for are faked below.
// The element in cases 4 and 5 is a plain object, so these pin which property
// each member touches; what the DOM does with that property is the browser's
// to answer, and is not claimed here.
//
// Run with: npm run test   (or: node --test "test/**/*.test.mjs")

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

// ------
// Host surfaces the glue expects from the rest of the bundle
// ------

globalThis.__elementHandles = new Map();

const glue = await import("../src/glue-html.ts");
const { INTERFACES, htmlOptionElement_exports } = glue;

let nextElementHandle = 1n;
function store(el) {
  const handle = nextElementHandle;
  nextElementHandle += 1n;
  globalThis.__elementHandles.set(handle, el);
  return handle;
}

test("the glue exports every function of the WIT html-option-element interface", () => {
  const wit = readFileSync(
    new URL("../../../browser-worlds/wit/browser-full.wit", import.meta.url),
    "utf8",
  );
  const start = wit.indexOf("interface html-option-element {");
  assert.notEqual(start, -1, "the WIT file has no `interface html-option-element`");
  const block = wit.slice(start, wit.indexOf("\n}", start));
  // `get-default-selected: func(...)` → `getDefaultSelected`. The
  // `type html-option-element-handle = u64;` line carries no `: func`, so it
  // stays out.
  const names = Array.from(block.matchAll(/^\s+([a-z][a-z-]*):\s*func\b/gm), ([, kebab]) =>
    kebab.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()),
  );
  assert.equal(names.length, 14, `read ${names.length} functions off the WIT file`);
  for (const name of names) {
    assert.equal(
      typeof htmlOptionElement_exports[name],
      "function",
      `WIT html-option-element.${name} has no export in htmlOptionElement_exports`,
    );
  }
});

test("the registry binds the table under the interface's name", () => {
  assert.strictEqual(INTERFACES["@tairitsu-glue/html-option-element"], htmlOptionElement_exports);
});

test("an unknown handle reads as absent and takes no write", () => {
  const unknown = 987654321n;
  const expected = {
    getDisabled: false,
    getForm: undefined,
    getLabel: "",
    getDefaultSelected: false,
    getSelected: false,
    getValue: "",
    getText: "",
    getIndex: -1,
  };
  const setters = {
    setDisabled: true,
    setLabel: "label",
    setDefaultSelected: true,
    setSelected: true,
    setValue: "value",
    setText: "text",
  };
  const members = Object.keys(htmlOptionElement_exports).sort();
  assert.deepEqual(
    members,
    [...Object.keys(expected), ...Object.keys(setters)].sort(),
    "every export is covered by one of the two tables",
  );
  for (const [name, absent] of Object.entries(expected)) {
    assert.strictEqual(htmlOptionElement_exports[name](unknown), absent, name);
  }
  const stored = globalThis.__elementHandles.size;
  for (const [name, value] of Object.entries(setters)) {
    htmlOptionElement_exports[name](unknown, value);
  }
  assert.equal(globalThis.__elementHandles.size, stored, "a setter stored nothing");
  assert.equal(globalThis.__elementHandles.has(unknown), false, "an unknown handle stays unknown");
});

test("real property reads come off the stored option", () => {
  const formHandle = 500n;
  globalThis.__elementHandles.set(formHandle, { id: "the-form" });
  const handle = store({
    disabled: true,
    form: globalThis.__elementHandles.get(formHandle),
    label: "Blue",
    defaultSelected: true,
    selected: false,
    value: "blue",
    text: "  Blue  ",
    index: 2,
  });
  assert.strictEqual(htmlOptionElement_exports.getDisabled(handle), true);
  assert.strictEqual(htmlOptionElement_exports.getForm(handle), formHandle);
  assert.strictEqual(htmlOptionElement_exports.getLabel(handle), "Blue");
  assert.strictEqual(htmlOptionElement_exports.getDefaultSelected(handle), true);
  assert.strictEqual(htmlOptionElement_exports.getSelected(handle), false);
  assert.strictEqual(htmlOptionElement_exports.getValue(handle), "blue");
  assert.strictEqual(htmlOptionElement_exports.getText(handle), "  Blue  ");
  assert.strictEqual(htmlOptionElement_exports.getIndex(handle), 2);
});

test("setSelected writes the live selectedness and leaves the default alone", () => {
  const handle = store({ selected: false, defaultSelected: false });
  htmlOptionElement_exports.setSelected(handle, true);
  assert.strictEqual(htmlOptionElement_exports.getSelected(handle), true);
  assert.strictEqual(htmlOptionElement_exports.getDefaultSelected(handle), false);

  htmlOptionElement_exports.setDefaultSelected(handle, true);
  assert.strictEqual(htmlOptionElement_exports.getDefaultSelected(handle), true);

  // What a removed `selected` attribute comes to.
  htmlOptionElement_exports.setSelected(handle, false);
  assert.strictEqual(htmlOptionElement_exports.getSelected(handle), false);
  assert.strictEqual(
    htmlOptionElement_exports.getDefaultSelected(handle),
    true,
    "unselecting is not the default's to change",
  );
});
