// @ts-nocheck

export const node_exports = {
  appendChild(self, child) {
    const parent = globalThis.__lookupNode(self);
    const childNode = globalThis.__lookupNode(child);
    const result = parent.appendChild(childNode);
    return globalThis.__storeNode(result);
  },
  removeChild(self, child) {
    const parent = globalThis.__lookupNode(self);
    const childNode = globalThis.__lookupNode(child);
    const result = parent.removeChild(childNode);
    return globalThis.__storeNode(result);
  },
  setTextContent(self, text) {
    globalThis.__lookupNode(self).textContent = text;
  },
  getTextContent(self) {
    return globalThis.__lookupNode(self).textContent || "";
  },
  getParentElement(self) {
    const el = globalThis.__lookupNode(self).parentElement;
    if (!el) return undefined;
    return globalThis.__elementHandles.get(el);
  },
};
