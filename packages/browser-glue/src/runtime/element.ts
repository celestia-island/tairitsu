// @ts-nocheck
import { lookupElement } from "./helpers";

export const element_exports = {
  setAttribute(self, qualifiedName, value) {
    lookupElement(self).setAttribute(qualifiedName, value);
  },
  removeAttribute(self, qualifiedName) {
    lookupElement(self).removeAttribute(qualifiedName);
  },
  getBoundingClientRect(element) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
};
