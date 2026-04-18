// @ts-nocheck
import { storeCssStyleDeclaration, lookupCssStyleDeclaration } from "./elementCssInlineStyle";

export const cssStyleDeclaration_exports = {
  /**
   * `cssText` attribute — getter.
   */
  getCssText(self: bigint): string {
    const obj = lookupCssStyleDeclaration(self);
    return obj.cssText;
  },

  /**
   * `cssText` attribute — setter.
   */
  setCssText(self: bigint, value: string): void {
    const obj = lookupCssStyleDeclaration(self);
    obj.cssText = value;
  },

  /**
   * `length` attribute — getter.
   */
  getLength(self: bigint): number {
    const obj = lookupCssStyleDeclaration(self);
    return obj.length;
  },

  /**
   * `item()` operation.
   */
  item(self: bigint, index: number): bigint {
    const obj = lookupCssStyleDeclaration(self);
    const result = obj.item(index);
    return globalThis.__storeText(result);
  },

  /**
   * `getPropertyValue()` operation.
   */
  getPropertyValue(self: bigint, property: string): string {
    const obj = lookupCssStyleDeclaration(self);
    return obj.getPropertyValue(property);
  },

  /**
   * `getPropertyPriority()` operation.
   */
  getPropertyPriority(self: bigint, property: string): string {
    const obj = lookupCssStyleDeclaration(self);
    return obj.getPropertyPriority(property);
  },

  /**
   * `setProperty()` operation.
   */
  setProperty(self: bigint, property: string, value: string, priority: string | undefined): void {
    const obj = lookupCssStyleDeclaration(self);
    obj.setProperty(property, value, priority || "");
  },

  /**
   * `removeProperty()` operation.
   */
  removeProperty(self: bigint, property: string): bigint {
    const obj = lookupCssStyleDeclaration(self);
    const result = obj.removeProperty(property);
    return globalThis.__storeText(result);
  },

  /**
   * `parentRule` attribute — getter.
   */
  getParentRule(self: bigint): bigint | undefined {
    const obj = lookupCssStyleDeclaration(self);
    const parentRule = obj.parentRule;
    if (!parentRule) return undefined;
    // Store the CSSRule and return its handle
    // Note: We'd need to import storeCssRule from a css-rule module
    // For now, return undefined as this is a simplified implementation
    return undefined;
  },
};
