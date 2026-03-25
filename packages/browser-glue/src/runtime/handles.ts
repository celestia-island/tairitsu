// @ts-nocheck
// Initialize global handle tables if not already set
globalThis.__elementHandles = globalThis.__elementHandles || new Map();
globalThis.__documentHandles = globalThis.__documentHandles || new Map();
globalThis.__nodeHandles = globalThis.__nodeHandles || new Map();
globalThis.__textHandles = globalThis.__textHandles || new Map();
globalThis.__nextHandle = globalThis.__nextHandle || 1n;
globalThis.__mutationRecordHandles = globalThis.__mutationRecordHandles || new Map();
globalThis.__nextMutationRecord = globalThis.__nextMutationRecord || 1n;
globalThis.__resizeObserverEntryHandles = globalThis.__resizeObserverEntryHandles || new Map();
globalThis.__nextResizeObserverEntry = globalThis.__nextResizeObserverEntry || 1n;
globalThis.__resizeObserverSizeHandles = globalThis.__resizeObserverSizeHandles || new Map();
globalThis.__nextResizeObserverSizeHandle = globalThis.__nextResizeObserverSizeHandle || 1n;
globalThis.__domRectHandles = globalThis.__domRectHandles || new Map();
globalThis.__nextDomRectHandle = globalThis.__nextDomRectHandle || 1n;
