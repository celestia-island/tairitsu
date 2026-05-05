// @ts-nocheck

export const mutationObserver_exports = {
  observe(self, target, options) {
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.observe(el, options || { childList: true, subtree: true });
    }
  },
  disconnect(self) {
    const observer = globalThis.__lookupElement(self);
    if (observer) {
      observer.disconnect();
    }
  },
};
