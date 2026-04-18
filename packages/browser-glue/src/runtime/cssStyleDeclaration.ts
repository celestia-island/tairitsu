// @ts-nocheck

export const cssStyleDeclaration_exports = {
  getCssText(self) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.cssText;
  },
  setCssText(self, value) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    obj.cssText = value;
  },
  getLength(self) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.length;
  },
  item(self, index) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const result = obj.item(index);
    return globalThis.__storeText(result);
  },
  getPropertyValue(self, property) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.getPropertyValue(property);
  },
  getPropertyPriority(self, property) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    return obj.getPropertyPriority(property);
  },
  setProperty(self, property, value, priority) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    obj.setProperty(property, value, priority || "");
  },
  removeProperty(self, property) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const result = obj.removeProperty(property);
    return globalThis.__storeText(result);
  },
  getParentRule(self) {
    const obj = globalThis.__lookupCssStyleDeclaration(self);
    const parentRule = obj.parentRule;
    if (!parentRule) return undefined;
    return undefined;
  },
};
