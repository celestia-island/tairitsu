globalThis.__wasmExports = null;

globalThis.__setWasmExports = function (exports: Record<string, any>) { // ts-escape-ok: retires with L40
  globalThis.__wasmExports = exports;
};
