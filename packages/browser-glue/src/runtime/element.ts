// @ts-nocheck
import { lookupElement } from "./helpers";

export const element_exports = {
  setAttribute(self, qualifiedName, value) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    lookupElement(self).setAttribute(name, value);
  },
  removeAttribute(self, qualifiedName) {
    const name = typeof qualifiedName === 'string' && qualifiedName.startsWith('r#')
      ? qualifiedName.slice(2)
      : qualifiedName;
    lookupElement(self).removeAttribute(name);
  },
  getBoundingClientRect(element) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  setInnerHtml(self, html) {
    lookupElement(self).innerHTML = html;
  },
};
