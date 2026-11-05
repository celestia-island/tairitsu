globalThis.__cssStyleDeclarationHandles = globalThis.__cssStyleDeclarationHandles || new Map();
globalThis.__nextCssStyleDeclarationHandle = globalThis.__nextCssStyleDeclarationHandle || 1n;

globalThis.__storeCssStyleDeclaration = function (obj: CSSStyleDeclaration | null) {
  if (!obj) return undefined;
  const handle = globalThis.__nextCssStyleDeclarationHandle++;
  globalThis.__cssStyleDeclarationHandles.set(handle, obj);
  return handle;
};

globalThis.__lookupCssStyleDeclaration = function (handle: bigint) {
  const obj = globalThis.__cssStyleDeclarationHandles.get(handle);
  if (!obj) {
    throw new Error("CSSStyleDeclaration handle " + handle + " not found");
  }
  return obj;
};

export const elementCssInlineStyle_exports = {
  getStyle(self: bigint) {
    const el = globalThis.__lookupElement(self);
    const style = el.style;
    return globalThis.__storeCssStyleDeclaration(style);
  },
};
