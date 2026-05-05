// @ts-nocheck

export const mutationRecord_exports = {
  getType(self) {
    if (!globalThis.__mutationRecordHandles) return '';
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? rec.type : '';
  },
  getTarget(self) {
    if (!globalThis.__mutationRecordHandles) return 0n;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.target) return 0n;
    return globalThis.__storeElement(rec.target);
  },
  getPreviousSibling(self) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.previousSibling) return undefined;
    return globalThis.__storeNode(rec.previousSibling);
  },
  getNextSibling(self) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.nextSibling) return undefined;
    return globalThis.__storeNode(rec.nextSibling);
  },
  getAttributeName(self) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.attributeName ?? undefined) : undefined;
  },
  getAttributeNamespace(self) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.attributeNamespace ?? undefined) : undefined;
  },
  getOldValue(self) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.oldValue ?? undefined) : undefined;
  },
};
