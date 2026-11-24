export const mutationRecord_exports = {
  getType(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return "";
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? rec.type : "";
  },
  getTarget(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return 0n;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.target) return 0n;
    return globalThis.__storeElement(rec.target);
  },
  getPreviousSibling(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.previousSibling) return undefined;
    return globalThis.__storeNode(rec.previousSibling);
  },
  getNextSibling(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    if (!rec || !rec.nextSibling) return undefined;
    return globalThis.__storeNode(rec.nextSibling);
  },
  getAttributeName(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.attributeName ?? undefined) : undefined;
  },
  getAttributeNamespace(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.attributeNamespace ?? undefined) : undefined;
  },
  // WIT declares `get-added-nodes` / `get-removed-nodes` returning `u64`: a
  // NODE LIST handle, not a node handle (the same `u64`-as-handle shape the
  // generated glue stores in `nodeList_exports`). They were exported only by
  // the generator's `glue/dom.ts` — a table the registry does not wire — so a
  // guest calling them trapped, and `callbackValues.mutationEntryValue`'s
  // "ask the record for the rest" note pointed at nothing.
  getAddedNodes(self: bigint) {
    const rec = globalThis.__mutationRecordHandles?.get(self);
    if (!rec || !globalThis.__nodeListHandles) return 0n;
    const handle = globalThis.__nextNodeList++;
    globalThis.__nodeListHandles.set(handle, rec.addedNodes);
    return handle;
  },

  getRemovedNodes(self: bigint) {
    const rec = globalThis.__mutationRecordHandles?.get(self);
    if (!rec || !globalThis.__nodeListHandles) return 0n;
    const handle = globalThis.__nextNodeList++;
    globalThis.__nodeListHandles.set(handle, rec.removedNodes);
    return handle;
  },

  getOldValue(self: bigint) {
    if (!globalThis.__mutationRecordHandles) return undefined;
    const rec = globalThis.__mutationRecordHandles.get(self);
    return rec ? (rec.oldValue ?? undefined) : undefined;
  },
};
