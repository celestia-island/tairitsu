// @ts-nocheck
import { lookupNode, storeNode } from "./helpers";

export const node_exports = {
  appendChild(self, child) {
    const parent = lookupNode(self);
    const childNode = lookupNode(child);
    const result = parent.appendChild(childNode);
    return storeNode(result);
  },
  removeChild(self, child) {
    const parent = lookupNode(self);
    const childNode = lookupNode(child);
    const result = parent.removeChild(childNode);
    return storeNode(result);
  },
  setTextContent(self, text) {
    lookupNode(self).textContent = text;
  },
  getTextContent(self) {
    return lookupNode(self).textContent || "";
  },
  getParentElement(self) {
    const el = lookupNode(self).parentElement;
    if (!el) return undefined;
    return globalThis.__elementHandles.get(el);
  },
};
