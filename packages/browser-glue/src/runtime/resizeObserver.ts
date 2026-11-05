export const resizeObserver_exports = {
  observe(self: bigint, target: bigint, options: any) { // ts-escape-ok: retires with L40
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.observe(el, options);
    }
  },
  unobserve(self: bigint, target: bigint) {
    const observer = globalThis.__lookupElement(self);
    const el = globalThis.__lookupElement(target);
    if (observer && el) {
      observer.unobserve(el);
    }
  },
  disconnect(self: bigint) {
    const observer = globalThis.__lookupElement(self);
    if (observer) {
      observer.disconnect();
    }
  },
};
