// @ts-nocheck

export const resizeObserverSize_exports = {
  getInlineSize(self) {
    if (!globalThis.__resizeObserverSizeHandles) return 0;
    const size = globalThis.__resizeObserverSizeHandles.get(self);
    return size ? size.inlineSize : 0;
  },
  getBlockSize(self) {
    if (!globalThis.__resizeObserverSizeHandles) return 0;
    const size = globalThis.__resizeObserverSizeHandles.get(self);
    return size ? size.blockSize : 0;
  },
};
