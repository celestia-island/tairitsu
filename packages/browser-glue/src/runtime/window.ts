export const window_exports = {
  getInnerWidth() {
    return window.innerWidth;
  },
  getInnerHeight() {
    return window.innerHeight;
  },
  getComputedStyle(elt: bigint, pseudoElt?: string): bigint {
    const el = (globalThis as any).__lookupElement(elt); // ts-escape-ok: retires with L40
    const result = window.getComputedStyle(el, pseudoElt);
    const handle = (globalThis as any).__nextCssStyleDeclarationHandle++ as unknown as bigint; // ts-escape-ok: retires with L40
    (globalThis as any).__cssStyleDeclarationHandles.set(handle, result); // ts-escape-ok: retires with L40
    return handle;
  },
};
