// node-list glue — implements the `tairitsu-browser:full/node-list` WIT
// import interface (0.5.26 world; consumed by diff/apply get_child_at for
// UpdateChild/InsertChild index resolution).
//
// Handles share the globalThis.__nodeListHandles table that dom glue's
// getChildNodes stores into — do NOT use a module-local table here.

/** Type definition for u64 */
export type u64 = bigint;

export const nodeList_exports = {
  getLength(self) {
    const list = globalThis.__nodeListHandles?.get(self);
    return list ? list.length : 0;
  },
  item(self, index) {
    const list = globalThis.__nodeListHandles?.get(self);
    if (!list || index >= list.length) return undefined;
    const node = list[index];
    if (!node) return undefined;
    return globalThis.__storeNode(node);
  },
} as {
  getLength(self: u64): number;
  item(self: u64, index: number): u64 | undefined;
};

export const INTERFACES = {
  "@tairitsu-glue/node-list": nodeList_exports,
};
