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
  getFirstChild(self) {
    const node = globalThis.__lookupNode(self);
    const first = node.firstChild;
    if (!first) return undefined;
    return globalThis.__storeNode(first);
  },
  getChildNodes(self) {
    const node = globalThis.__lookupNode(self);
    if (!node) return undefined;
    const children = node.childNodes;
    if (!globalThis.__nodeListHandles) {
      globalThis.__nodeListHandles = new Map();
      globalThis.__nextNodeList = 1n;
    }
    const handle = globalThis.__nextNodeList++;
    globalThis.__nodeListHandles.set(handle, children);
    return handle;
  },
  insertBefore(self, newChild, referenceChild) {
    const parent = globalThis.__lookupNode(self);
    const child = globalThis.__lookupNode(newChild);
    const refChild = referenceChild !== undefined ? globalThis.__lookupNode(referenceChild) : null;
    const result = parent.insertBefore(child, refChild);
    return globalThis.__storeNode(result);
  },
};
