// @ts-nocheck
import { lookupElement } from "./helpers";

// Initialize global handle tables for CSSStyleDeclaration
globalThis.__cssStyleDeclarationHandles = globalThis.__cssStyleDeclarationHandles || new Map();
globalThis.__nextCssStyleDeclarationHandle = globalThis.__nextCssStyleDeclarationHandle || 1n;

/**
 * Store a CSSStyleDeclaration and return its handle.
 */
export function storeCssStyleDeclaration(obj: CSSStyleDeclaration | null): bigint | undefined {
  if (!obj) return undefined;
  const handle = globalThis.__nextCssStyleDeclarationHandle++;
  globalThis.__cssStyleDeclarationHandles.set(handle, obj);
  return handle;
}

/**
 * Lookup a CSSStyleDeclaration by handle, throwing if not found.
 */
export function lookupCssStyleDeclaration(handle: bigint): CSSStyleDeclaration {
  const obj = globalThis.__cssStyleDeclarationHandles.get(handle);
  if (!obj) {
    throw new Error(`CSSStyleDeclaration handle ${handle} not found`);
  }
  return obj;
}

export const elementCssInlineStyle_exports = {
  /**
   * `style` attribute — getter.
   * Returns a handle to the CSSStyleDeclaration for this element.
   */
  getStyle(self: bigint): bigint {
    const el = lookupElement(self);
    const style = el.style;
    return storeCssStyleDeclaration(style);
  },
};
