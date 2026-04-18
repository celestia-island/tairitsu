// @ts-nocheck

export const nonElementParentNode_exports = {
  getElementById(self, elementId) {
    const doc = globalThis.__documentHandles.get(self) || document;
    const el = doc.getElementById(elementId);
    return globalThis.__storeElement(el);
  },
};
