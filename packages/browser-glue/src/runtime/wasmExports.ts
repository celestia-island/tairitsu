// @ts-nocheck
// Shared WASM exports reference, used by platform-helpers.
// ES module named exports are live bindings, so importers always read the current value.

export let wasmExports = null;

globalThis.__setWasmExports = function (exports) {
  wasmExports = exports;
};
