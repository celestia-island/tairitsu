/**
 * Shared browser object handle tables used across glue modules.
 */

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
    throw new Error(`DOM node handle ${handle} not found`);
  }
  return node;
}

export function getElement(handle: bigint): Element {
  const node = getNode(handle);
  if (!(node instanceof Element)) {
    throw new Error(`DOM handle ${handle} is not an Element`);
  }
  return node;
}

export function getEventTarget(handle: bigint): EventTarget {
  const node = getNode(handle);
  if (!(node instanceof EventTarget)) {
    throw new Error(`DOM handle ${handle} is not an EventTarget`);
  }
  return node;
}

export function getCanvasElement(handle: bigint): HTMLCanvasElement {
  const node = getNode(handle);
  if (!(node instanceof HTMLCanvasElement)) {
    throw new Error(`Node handle ${handle} is not an HTMLCanvasElement`);
  }
  return node;
}
