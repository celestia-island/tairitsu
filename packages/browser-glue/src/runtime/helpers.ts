// Initialize global handle tables
globalThis.__elementHandles = globalThis.__elementHandles || new Map();
globalThis.__documentHandles = globalThis.__documentHandles || new Map();
globalThis.__nodeHandles = globalThis.__nodeHandles || new Map();
globalThis.__textHandles = globalThis.__textHandles || new Map();
globalThis.__nextHandle = globalThis.__nextHandle || 1n;

// Set globalThis helper functions (for use in generated blob URL modules)
// IMPORTANT: Always access globalThis directly, never cache in local variables
globalThis.__storeElement = function (el: any) {
  if (!el) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__elementHandles.set(handle, el);
  return handle;
};

globalThis.__storeNode = function (node: Node | null) {
  if (!node) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__nodeHandles.set(handle, node);
  return handle;
};

globalThis.__storeText = function (text: Text | null) {
  if (!text) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__textHandles.set(handle, text);
  return handle;
};

globalThis.__lookupElement = function (handle: bigint) {
  const el = globalThis.__elementHandles.get(handle);
  if (!el) throw new Error("Element handle " + handle + " not found");
  return el;
};

globalThis.__lookupNode = function (handle: bigint) {
  const node = globalThis.__nodeHandles.get(handle) || globalThis.__textHandles.get(handle);
  if (!node) {
    const el = globalThis.__elementHandles.get(handle);
    if (el) return el;
    throw new Error("Node handle " + handle + " not found");
  }
  return node;
};

// Local function exports for use within runtime modules
export function storeElement(el: any) {
  if (!el) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__elementHandles.set(handle, el);
  return handle;
}

export function storeNode(node: Node | null) {
  if (!node) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__nodeHandles.set(handle, node);
  return handle;
}

export function storeText(text: Text | null) {
  if (!text) return undefined;
  const handle = globalThis.__nextHandle++;
  globalThis.__textHandles.set(handle, text);
  return handle;
}

export function lookupElement(handle: bigint) {
  const el = globalThis.__elementHandles.get(handle);
  if (!el) throw new Error("Element handle " + handle + " not found");
  return el;
}

export function lookupNode(handle: bigint) {
  const node = globalThis.__nodeHandles.get(handle) || globalThis.__textHandles.get(handle);
  if (!node) {
    const el = globalThis.__elementHandles.get(handle);
    if (el) return el;
    throw new Error("Node handle " + handle + " not found");
  }
  return node;
}
