// @ts-nocheck

export const window_exports = {
  getInnerWidth() {
    return window.innerWidth;
  },
  getInnerHeight() {
    return window.innerHeight;
  },
  getComputedStyle(elt: number, pseudoElt?: string): number {
    const el = globalThis.__lookupElement(elt);
    const result = window.getComputedStyle(el, pseudoElt);
    const handle = globalThis.__nextCssStyleDeclarationHandle++;
    globalThis.__cssStyleDeclarationHandles.set(handle, result);
    return handle;
  },
};
