export const cssStyleDeclaration_exports = {
  getCssText(self: bigint) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.cssText;
  },
  setCssText(self: bigint, value: string) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    obj.cssText = value;
  },
  getLength(self: bigint) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.length;
  },
  item(self: bigint, index: number) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const result = obj.item(index);
    return globalThis.__storeText(result as any);
  },
  getPropertyValue(self: bigint, property: string) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.getPropertyValue(property);
  },
  getPropertyPriority(self: bigint, property: string) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.getPropertyPriority(property);
  },
  setProperty(self: bigint, property: string, value: string, priority: string) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    obj.setProperty(property, value, priority || "");
  },
  removeProperty(self: bigint, property: string) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const result = obj.removeProperty(property);
    return globalThis.__storeText(result as any);
  },
  getParentRule(self: bigint) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const parentRule = obj.parentRule;
    if (!parentRule) return undefined;
    return undefined;
  },
};
