/**
 * Shared browser object handle tables used across glue modules.
 */

let nextNodeHandle = 1n;
const nodes = new Map<bigint, Node>();

// Diagnostic callback interface
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

/**
 * Get statistics about the handle table for diagnostics.
 */
export function getHandleStats(): { totalHandles: number; nextHandle: bigint } {
  return {
    totalHandles: nodes.size,
    nextHandle: nextNodeHandle,
  };
}

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
    reportHandleError({
      kind: "type-mismatch",
      handle,
      expectedType: "EventTarget",
      actualType: node.constructor.name,
    });
    throw new Error(`DOM handle ${handle} is not an EventTarget (got ${node.constructor.name})`);
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
