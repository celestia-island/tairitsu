// @ts-nocheck

export const nodeList_exports = {
  getLength(self) {
    const list = globalThis.__nodeListHandles
      ? globalThis.__nodeListHandles.get(self)
      : null;
    if (!list) throw new Error("NodeList handle " + self + " not found");
    return list.length;
  },

  item(self, index) {
    const list = globalThis.__nodeListHandles
      ? globalThis.__nodeListHandles.get(self)
      : null;
    if (!list) throw new Error("NodeList handle " + self + " not found");
    const result = list.item(index);
    if (result === null) return undefined;
    return globalThis.__storeNode(result);
  },
};
