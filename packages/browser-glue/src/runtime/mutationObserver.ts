export const mutationObserver_exports = {
  observe(self: bigint, target: bigint, options: any) { // ts-escape-ok: retires with L40
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.observe(el, options || { childList: true, subtree: true });
    }
  },
  disconnect(self: bigint) {
    const observer = globalThis.__lookupElement(self);
    if (observer) {
      observer.disconnect();
    }
  },
};
