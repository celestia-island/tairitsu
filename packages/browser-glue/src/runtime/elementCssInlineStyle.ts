import { installGlobalDelegates } from "./instance.ts";

installGlobalDelegates();

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
