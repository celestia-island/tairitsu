/**
 * Shared browser object handle tables used across glue modules.
 *
 * Provides both:
 * - Legacy DOM-specific handle table (nodes, elements)
 * - Generic typed handle tables for auto-generated glue code
 */

// ---------------------------------------------------------------------------
// Diagnostic Support
// ---------------------------------------------------------------------------

interface HandleDiagnosticCallbacks {
  onHandleError?: (error: HandleDiagnosticError) => void;
}

export interface HandleDiagnosticError {
  kind: "handle-not-found" | "type-mismatch";
  handle: bigint;
  expectedType: string;
  actualType?: string;
}

let _diagnosticCallbacks: HandleDiagnosticCallbacks = {};

/**
 * Register diagnostic callbacks for handle table operations.
 */
export function registerHandleDiagnosticCallbacks(callbacks: HandleDiagnosticCallbacks): void {
  _diagnosticCallbacks = { ..._diagnosticCallbacks, ...callbacks };
}

function reportHandleError(error: HandleDiagnosticError): void {
  if (_diagnosticCallbacks.onHandleError) {
    _diagnosticCallbacks.onHandleError(error);
  }
  console.error(`[browser-glue handle-table] ${error.kind}: handle ${error.handle} - ${error.expectedType}`);
}

// ---------------------------------------------------------------------------
// Legacy DOM Handle Table
// ---------------------------------------------------------------------------

let nextNodeHandle = 1n;
const nodes = new Map<bigint, Node>();

export function registerNode(node: Node): bigint {
  const handle = nextNodeHandle++;
  nodes.set(handle, node);
  return handle;
}

export function getNode(handle: bigint): Node {
  const node = nodes.get(handle);
  if (!node) {
    reportHandleError({
      kind: "handle-not-found",
      handle,
      expectedType: "Node",
    });
    throw new Error(`DOM node handle ${handle} not found. This usually indicates:
1. The handle was already freed/destroyed
2. The handle is from a different component instance
3. A use-after-free bug in the component code

Current handle table contains ${nodes.size} handles, next handle is ${nextNodeHandle}`);
  }
  return node;
}

export function getElement(handle: bigint): Element {
  const node = getNode(handle);
  if (!(node instanceof Element)) {
    reportHandleError({
      kind: "type-mismatch",
      handle,
      expectedType: "Element",
      actualType: node.constructor.name,
    });
    throw new Error(`DOM handle ${handle} is not an Element (got ${node.constructor.name})`);
  }
  return node;
}

export function getEventTarget(handle: bigint): EventTarget {
  const node = getNode(handle);
  if (!(node instanceof EventTarget)) {
    const actualType = (node as object).constructor?.name ?? "unknown";
    reportHandleError({
      kind: "type-mismatch",
      handle,
      expectedType: "EventTarget",
      actualType,
    });
    throw new Error(`DOM handle ${handle} is not an EventTarget (got ${actualType})`);
  }
  return node;
}

export function getCanvasElement(handle: bigint): HTMLCanvasElement {
  const node = getNode(handle);
  if (!(node instanceof HTMLCanvasElement)) {
    reportHandleError({
      kind: "type-mismatch",
      handle,
      expectedType: "HTMLCanvasElement",
      actualType: node.constructor.name,
    });
    throw new Error(`Node handle ${handle} is not an HTMLCanvasElement (got ${node.constructor.name})`);
  }
  return node;
}

// ---------------------------------------------------------------------------
// Generic Typed Handle Registry
// ---------------------------------------------------------------------------

/**
 * Generic handle registry for type-specific handle tables.
 * Used by auto-generated glue code for browser APIs.
 */
interface TypeRegistry<T = unknown> {
  nextHandle: bigint;
  handles: Map<bigint, T>;
}

const _typeRegistries = new Map<string, TypeRegistry<unknown>>();

function getOrCreateRegistry<T>(typeName: string): TypeRegistry<T> {
  let registry = _typeRegistries.get(typeName) as TypeRegistry<T> | undefined;
  if (!registry) {
    registry = {
      nextHandle: 1n,
      handles: new Map(),
    };
    _typeRegistries.set(typeName, registry as TypeRegistry<unknown>);
  }
  return registry;
}

/**
 * Register an object in a type-specific handle table.
 * Returns the assigned handle.
 *
 * @param typeName - Unique type identifier (e.g., "Headers", "StorageManager")
 * @param obj - The object to register
 * @returns The assigned bigint handle
 */
export function registerTypedHandle<T>(typeName: string, obj: T): bigint {
  const registry = getOrCreateRegistry<T>(typeName);
  const handle = registry.nextHandle++;
  registry.handles.set(handle, obj);
  return handle;
}

/**
 * Get an object from a type-specific handle table.
 * Throws if the handle is not found.
 *
 * @param typeName - Unique type identifier
 * @param handle - The bigint handle
 * @returns The registered object
 */
export function getTypedHandle<T>(typeName: string, handle: bigint): T {
  const registry = _typeRegistries.get(typeName);
  if (!registry) {
    reportHandleError({
      kind: "handle-not-found",
      handle,
      expectedType: typeName,
    });
    throw new Error(`No handle registry found for type "${typeName}"`);
  }
  const obj = registry.handles.get(handle);
  if (obj === undefined) {
    reportHandleError({
      kind: "handle-not-found",
      handle,
      expectedType: typeName,
    });
    throw new Error(`${typeName} handle ${handle} not found`);
  }
  return obj as T;
}

/**
 * Remove an object from a type-specific handle table.
 * Returns true if the handle was found and removed.
 */
export function unregisterTypedHandle(typeName: string, handle: bigint): boolean {
  const registry = _typeRegistries.get(typeName);
  if (!registry) return false;
  return registry.handles.delete(handle);
}

/**
 * Get statistics for all type-specific handle tables.
 */
export function getGenericHandleStats(): Record<string, { count: number; nextHandle: bigint }> {
  const stats: Record<string, { count: number; nextHandle: bigint }> = {};
  for (const [typeName, registry] of _typeRegistries) {
    stats[typeName] = {
      count: registry.handles.size,
      nextHandle: registry.nextHandle,
    };
  }
  return stats;
}

/**
 * Clear all type-specific handle tables (for testing/reset).
 */
export function clearAllTypedHandles(): void {
  _typeRegistries.clear();
}

// ---------------------------------------------------------------------------
// Combined Statistics
// ---------------------------------------------------------------------------

/**
 * Get statistics about all handle tables for diagnostics.
 */
export function getHandleStats(): {
  totalHandles: number;
  nextHandle: bigint;
  typedHandles: Record<string, { count: number; nextHandle: bigint }>;
} {
  return {
    totalHandles: nodes.size,
    nextHandle: nextNodeHandle,
    typedHandles: getGenericHandleStats(),
  };
}
