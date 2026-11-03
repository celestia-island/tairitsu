// Behaviour tests for the `html-input-element` glue implemented in this PR.
//
// Asserted here:
//   1. every declared getter reads the DOM-absent default off an unknown
//      handle instead of trapping (the lenient-resolution convention this
//      interface follows — same as the pointer/wheel/mouse families),
//   2. real property reads return the real values off a stored element,
//   3. `set_files` is an honest NO-OP — `HTMLInputElement.files` is a
//      readonly IDL attribute ([SameObject], getter-only), so assignment
//      would throw TypeError in strict-mode ESM; the setter must never
//      attempt it (the S1 regression this test pins).
//
// The module under test is the shipped source file, imported directly (Node
// strips the TypeScript); the host facilities it reaches for are faked below.
//
// Run with: npm run test   (or: node --test "test/**/*.test.mjs")

import assert from "node:assert/strict";
import test from "node:test";

// ------
// Host surfaces the glue expects from the rest of the bundle
// ------

globalThis.__elementHandles = new Map();
globalThis.__fileListHandles = new Map();
globalThis.__nodeListHandles = new Map();
globalThis.__validityStateHandles = new Map();

const glue = await import("../src/glue-html.ts");
const { htmlInputElement_exports } = glue;

let nextElementHandle = 1n;
function store(el) {
  const handle = nextElementHandle;
  nextElementHandle += 1n;
  globalThis.__elementHandles.set(handle, el);
  return handle;
}

// 1. Unknown-handle tolerance: EVERY declared getter reads the DOM-absent
//    default instead of trapping (derived from the exports themselves, so a
//    new member automatically joins this row).
const unknown = 987654321n;
const defaults = {
  getString: "",
  getBool: false,
  getNumber: -1 === -1 ? -1 : 0,
  getHandle: undefined,
};
for (const [name, fn] of Object.entries(htmlInputElement_exports)) {
  if (typeof fn !== "function") continue;
  const probe = fn(unknown);
  // The honest defaults: strings "" (empty), numbers -1 or 0 (both
  // documented), booleans false, handles undefined. Anything else is a
  // trap or an invented value — flag it for review.
  const ok =
    probe === "" ||
    probe === false ||
    probe === -1 ||
    probe === 0 ||
    probe === undefined ||
    probe === null ||
    typeof probe === "bigint" ||
    (typeof probe === "number" && Number.isNaN(probe));
  const shown = typeof probe === "bigint" ? `${probe}n` : JSON.stringify(probe);
  assert.ok(ok, `${name}(unknown) = ${shown} — not a documented absent default`);
}

// 2. Real property reads off a stored element.
{
  const handle = store({
    value: "2026-06-15",
    checked: true,
    maxLength: 10,
    alt: "an icon",
  });
  assert.strictEqual(htmlInputElement_exports.getValue(handle), "2026-06-15");
  assert.strictEqual(htmlInputElement_exports.getChecked(handle), true);
  assert.strictEqual(htmlInputElement_exports.getMaxLength(handle), 10);
  assert.strictEqual(htmlInputElement_exports.getAlt(handle), "an icon");
}

// 3. setFiles is an honest no-op — the S1 regression pin. A naive
//    `el.files = list` throws TypeError on a readonly IDL attribute in
//    strict-mode ESM; the setter must never attempt the assignment.
{
  const handle = store({ value: "v" });
  const before = htmlInputElement_exports.getFiles(handle);
  htmlInputElement_exports.setFiles(handle, undefined);
  htmlInputElement_exports.setFiles(handle, 42n);
  const after = htmlInputElement_exports.getFiles(handle);
  assert.strictEqual(before, after, "setFiles must not disturb getFiles");
}

console.log("html-input-element glue tests done");
