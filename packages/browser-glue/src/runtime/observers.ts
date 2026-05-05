// @ts-nocheck

export const observers_exports = {
  getBoundingClientRect(element) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  observe(self, target, options) {
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.observe(el, options || { childList: true, subtree: true });
    }
  },
};
