// @ts-nocheck
import { lookupElement } from "./helpers";

export const element_exports = {
  setAttribute(self, qualifiedName, value) {
    lookupElement(self).setAttribute(qualifiedName, value);
  },
  removeAttribute(self, qualifiedName) {
    lookupElement(self).removeAttribute(qualifiedName);
  },
};
