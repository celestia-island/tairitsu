// Behaviour tests for the `dom` node glue's getParentElement (this PR).
//
// Asserted here:
//   1. getParentElement resolves the parent via the node handle table —
//      the pre-fix code looked the *element object* up in the
//      handle→element map (`__elementHandles.get(el)`), which never hits,
//      so the parent chain broke on the first hop (event delegation's
//      closest semantics were dead). The fix follows the appendChild /
//      getFirstChild convention: route through `__storeNode`, which
//      stores-or-reuses the handle for a node object.
//   2. handle round-trip: asking for the same parent twice yields the SAME
//      handle (identity reuse via `__storeNode`), so attribute reads off
//      the returned handle hit the real parent.
//   3. root nodes (no parentElement) return undefined, not a trap.
//
// The host facilities the glue reaches for are faked below, mirroring the
// runtime contract: `__lookupNode(handle) → node`,
// `__storeNode(node) → handle` (identity-reusing),
// `__elementHandles: Map<handle, element>`.
//
// Run with: npm run test   (or: node --test "test/**/*.test.mjs")

import assert from "node:assert/strict";
import test from "node:test";

// ------
// Host surfaces the glue expects from the rest of the bundle
// ------

globalThis.__elementHandles = new Map();
globalThis.__nodeHandles = new Map();
globalThis.__nodeIdentity = new Map();

// The runtime contract the rest of glue-dom already relies on:
// store-or-reuse. Element and text nodes share the node table.
let nextHandle = 1n;
globalThis.__storeNode = function (node) {
  let handle = globalThis.__nodeIdentity.get(node);
  if (handle === undefined) {
    handle = nextHandle;
    nextHandle += 1n;
    globalThis.__nodeIdentity.set(node, handle);
    globalThis.__nodeHandles.set(handle, node);
    if (node && node.nodeType === 1) {
      globalThis.__elementHandles.set(handle, node);
    }
  }
  return handle;
};
globalThis.__lookupNode = function (handle) {
  return globalThis.__nodeHandles.get(handle);
};
globalThis.__storeElement = function (el) {
  return globalThis.__storeNode(el);
};
globalThis.__lookupElement = function (handle) {
  return globalThis.__elementHandles.get(handle);
};

const glue = await import("../src/glue-dom.ts");
const { node_exports } = glue;

// Minimal DOM stand-in: three-deep element chain + a root-less node.
function makeEl(name) {
  return {
    nodeType: 1,
    tagName: name,
    parentElement: null,
    childNodes: { length: 0, item: () => undefined, [0]: undefined },
    appendChild(child) {
      child.parentElement = this;
      return child;
    },
  };
}

test("getParentElement walks the chain instead of breaking on hop one", () => {
  const root = makeEl("ROOT");
  const mid = makeEl("MID");
  const leaf = makeEl("LEAF");
  root.appendChild(mid);
  mid.appendChild(leaf);

  const leafHandle = globalThis.__storeNode(leaf);
  const midHandle = globalThis.__storeNode(mid);

  const parentHandle = node_exports.getParentElement(leafHandle);
  assert.ok(parentHandle !== undefined, "parent handle must resolve");
  assert.equal(
    globalThis.__lookupNode(parentHandle),
    mid,
    "leaf's parent must be the mid node",
  );
  assert.equal(
    parentHandle,
    midHandle,
    "must reuse the existing handle (identity), not mint a fresh one",
  );

  const grandparentHandle = node_exports.getParentElement(parentHandle);
  assert.equal(globalThis.__lookupNode(grandparentHandle), root);
  const none = node_exports.getParentElement(grandparentHandle);
  assert.equal(none, undefined, "root has no parentElement → undefined");
});

test("getParentElement never consults the handle→element map with an element key", () => {
  // The pre-fix bug: __elementHandles.get(el). Poison the map so any
  // element-keyed lookup would return a WRONG handle instead of silently
  // undefined — if the fix regresses, this fails loudly.
  const root = makeEl("ROOT");
  const child = makeEl("CHILD");
  root.appendChild(child);
  const childHandle = globalThis.__storeNode(child);
  const rootHandle = globalThis.__storeNode(root);

  globalThis.__elementHandles.set(root, 424242n); // element object as key

  const parentHandle = node_exports.getParentElement(childHandle);
  assert.equal(parentHandle, rootHandle);
  assert.notEqual(parentHandle, 424242n);
});

test("appendChild/getFirstChild keep the same convention as the fix", () => {
  const parent = makeEl("P");
  const child = makeEl("C");
  const parentHandle = globalThis.__storeNode(parent);

  const returned = node_exports.appendChild(parentHandle, globalThis.__storeNode(child));
  assert.equal(globalThis.__lookupNode(returned), child, "append returns the child handle");
  assert.equal(child.parentElement, parent);
});
