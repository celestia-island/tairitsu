// @ts-nocheck

export const parentNode_exports = {
  querySelector(self, selectors) {
    const el = globalThis.__lookupElement(self);
    const result = el.querySelector(selectors);
    if (!result) return undefined;
    return globalThis.__storeElement(result);
  },
  querySelectorAll(self, selectors) {
    const el = globalThis.__lookupElement(self);
    const result = el.querySelectorAll(selectors);
    if (!globalThis.__nodeListHandles) {
      globalThis.__nodeListHandles = new Map();
      globalThis.__nextNodeList = 1n;
    }
    const handle = globalThis.__nextNodeList++;
    globalThis.__nodeListHandles.set(handle, result);
    return handle;
  },
};
