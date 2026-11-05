export const window_exports = {
  getInnerWidth() {
    return window.innerWidth;
  },
  getInnerHeight() {
    return window.innerHeight;
  },
  getComputedStyle(elt: bigint, pseudoElt?: string): bigint {
    const el = (globalThis as any).__lookupElement(elt);
    const result = window.getComputedStyle(el, pseudoElt);
    const handle = (globalThis as any).__nextCssStyleDeclarationHandle++ as unknown as bigint;
    (globalThis as any).__cssStyleDeclarationHandles.set(handle, result);
    return handle;
  },
};
