// @ts-nocheck
import { lookupElement } from "./helpers";

// Handle table for CSSStyleDeclaration instances
const _cssStyleDeclarationHandles = new Map<bigint, CSSStyleDeclaration>();
let _nextCssStyleDeclarationHandle = 1n;

/**
 * Store a CSSStyleDeclaration and return its handle.
 */
export function storeCssStyleDeclaration(obj: CSSStyleDeclaration | null): bigint | undefined {
  if (!obj) return undefined;
  const handle = _nextCssStyleDeclarationHandle++;
  _cssStyleDeclarationHandles.set(handle, obj);
  return handle;
}

/**
 * Lookup a CSSStyleDeclaration by handle, throwing if not found.
 */
export function lookupCssStyleDeclaration(handle: bigint): CSSStyleDeclaration {
  const obj = _cssStyleDeclarationHandles.get(handle);
  if (!obj) {
    throw new Error(`CSSStyleDeclaration handle ${handle} not found`);
  }
  return obj;
}

export const element_css_inline_style_exports = {
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
