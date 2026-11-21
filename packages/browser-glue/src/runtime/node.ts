export const node_exports = {
  appendChild(self: bigint, child: bigint) {
    const parent = globalThis.__lookupNode(self);
    const childNode = globalThis.__lookupNode(child);
    const result = parent.appendChild(childNode);
    return globalThis.__storeNode(result);
  },
  removeChild(self: bigint, child: bigint) {
    const parent = globalThis.__lookupNode(self);
    const childNode = globalThis.__lookupNode(child);
    const result = parent.removeChild(childNode);
    return globalThis.__storeNode(result);
  },
  setTextContent(self: bigint, text: string) {
    globalThis.__lookupNode(self).textContent = text;
  },
  getTextContent(self: bigint) {
    return globalThis.__lookupNode(self).textContent || "";
  },
  getParentElement(self: bigint) {
    const el = globalThis.__lookupNode(self).parentElement;
    if (!el) return undefined;
    for (const [handle, elem] of globalThis.__elementHandles.entries()) {
      if (elem === el) return handle;
    }
    return undefined;
  },
  getFirstChild(self: bigint) {
    const node = globalThis.__lookupNode(self);
    const first = node.firstChild;
    if (!first) return undefined;
    return globalThis.__storeNode(first);
  },
  getChildNodes(self: bigint) {
    const node = globalThis.__lookupNode(self);
    if (!node) return undefined;
    const children = node.childNodes;
    const handle = globalThis.__nextNodeList++;
    globalThis.__nodeListHandles.set(handle, children);
    return handle;
  },
  /** `nodeType` — `Node.nodeType` (1 = element, 3 = text, 9 = document, …). */
  getNodeType(self: bigint) {
    return Number(globalThis.__lookupNode(self).nodeType ?? 0);
  },
  /**
   * `replaceChild()` — the DOM returns the *removed* node, so the handle the
   * guest gets back is the detached child, matching the WIT contract
   * (`_domain-dom.wit`), not the inserted one.
   */
  replaceChild(self: bigint, node: bigint, child: bigint) {
    const parent = globalThis.__lookupNode(self);
    const replacement = globalThis.__lookupNode(node);
    const removed = globalThis.__lookupNode(child);
    const result = parent.replaceChild(replacement, removed);
    return globalThis.__storeNode(result);
  },
  insertBefore(self: bigint, newChild: bigint, referenceChild: bigint | undefined) {
    const parent = globalThis.__lookupNode(self);
    const child = globalThis.__lookupNode(newChild);
    const refChild = referenceChild !== undefined ? globalThis.__lookupNode(referenceChild) : null;
    const result = parent.insertBefore(child, refChild);
    return globalThis.__storeNode(result);
  },
};
