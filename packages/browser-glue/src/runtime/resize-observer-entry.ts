// @ts-nocheck
import { storeElement } from "./helpers";

export const resize_observer_entry_exports = {
  getTarget(self) {
    if (!globalThis.__resizeObserverEntryHandles) return 0n;
    const entry = globalThis.__resizeObserverEntryHandles.get(self);
    if (!entry) return 0n;
    return storeElement(entry.target);
  },
  getContentRect(self) {
    if (!globalThis.__resizeObserverEntryHandles) return 0n;
    const entry = globalThis.__resizeObserverEntryHandles.get(self);
    if (!entry) return 0n;
    if (!globalThis.__domRectHandles) { globalThis.__domRectHandles = new Map(); globalThis.__nextDomRectHandle = 1n; }
    const handle = globalThis.__nextDomRectHandle++;
    globalThis.__domRectHandles.set(handle, entry.contentRect);
    return handle;
  },
  getBorderBoxSize(self) {
    if (!globalThis.__resizeObserverEntryHandles) return [];
    const entry = globalThis.__resizeObserverEntryHandles.get(self);
    if (!entry) return [];
    if (!globalThis.__resizeObserverSizeHandles) { globalThis.__resizeObserverSizeHandles = new Map(); globalThis.__nextResizeObserverSizeHandle = 1n; }
    return [...entry.borderBoxSize].map(function (size) {
      const handle = globalThis.__nextResizeObserverSizeHandle++;
      globalThis.__resizeObserverSizeHandles.set(handle, size);
      return handle;
    });
  },
  getContentBoxSize(self) {
    if (!globalThis.__resizeObserverEntryHandles) return [];
    const entry = globalThis.__resizeObserverEntryHandles.get(self);
    if (!entry) return [];
    if (!globalThis.__resizeObserverSizeHandles) { globalThis.__resizeObserverSizeHandles = new Map(); globalThis.__nextResizeObserverSizeHandle = 1n; }
    return [...entry.contentBoxSize].map(function (size) {
      const handle = globalThis.__nextResizeObserverSizeHandle++;
      globalThis.__resizeObserverSizeHandles.set(handle, size);
      return handle;
    });
  },
};
