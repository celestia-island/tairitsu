// @ts-nocheck

export const resizeObserver_exports = {
  observe(self, target, options) {
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.observe(el, options);
    }
  },
  unobserve(self, target) {
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.unobserve(el);
    }
  },
  disconnect(self) {
    const observer = globalThis.__lookupElement(self);
    if (observer) {
      observer.disconnect();
    }
  },
};
