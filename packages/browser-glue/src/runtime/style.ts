// @ts-nocheck
import { lookupElement } from "./helpers";

export const style_exports = {
  setStyleProperty(element, property, value) {
    try {
      lookupElement(element).style.setProperty(property, value);
    } catch (e) {
      return String(e);
    }
  },
  getStyleProperty(element, property) {
    return lookupElement(element).style.getPropertyValue(property) || undefined;
  },
  removeStyleProperty(element, property) {
    try {
      lookupElement(element).style.removeProperty(property);
    } catch (e) {
      return String(e);
    }
  },
};
