// @ts-nocheck
import { storeElement } from "./helpers";

export const non_element_parent_node_exports = {
  getElementById(self, elementId) {
    const doc = globalThis.__documentHandles.get(self) || document;
    const el = doc.getElementById(elementId);
    return storeElement(el);
  },
};
