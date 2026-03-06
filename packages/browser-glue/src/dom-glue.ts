/**
 * DOM glue — implements the `tairitsu-browser:dom` WIT import interfaces.
 *
 * Each function here maps 1:1 to a WIT function in `wit/dom.wit`.
 * Node handles are the underlying browser `Element` / `Text` / `Node`
 * objects stored in a handle table.
 *
 * Status: Phase 0 stubs — core operations implemented.
 */

// ---------------------------------------------------------------------------
// Handle table
// ---------------------------------------------------------------------------

let _nextHandle = 1n;
const _nodes = new Map<bigint, Node>();

function registerNode(node: Node): bigint {
  const handle = _nextHandle++;
  _nodes.set(handle, node);
  return handle;
}

function getNode(handle: bigint): Node {
  const node = _nodes.get(handle);
  if (!node) throw new Error(`DOM node handle ${handle} not found`);
  return node;
}

function getElement(handle: bigint): Element {
  const node = getNode(handle);
  if (!(node instanceof Element)) {
    throw new Error(`DOM handle ${handle} is not an Element`);
  }
  return node;
}

// ---------------------------------------------------------------------------
// WIT interface: node
// ---------------------------------------------------------------------------

export function appendChild(parent: bigint, child: bigint): void {
  getNode(parent).appendChild(getNode(child));
}

export function removeChild(parent: bigint, child: bigint): void {
  getNode(parent).removeChild(getNode(child));
}

export function setAttribute(
  node: bigint,
  name: string,
  value: string,
): void {
  getElement(node).setAttribute(name, value);
}

export function getAttribute(
  node: bigint,
  name: string,
): string | undefined {
  return getElement(node).getAttribute(name) ?? undefined;
}

export function removeAttribute(node: bigint, name: string): void {
  getElement(node).removeAttribute(name);
}

export function setTextContent(node: bigint, text: string): void {
  getNode(node).textContent = text;
}

export function getTextContent(node: bigint): string | undefined {
  return getNode(node).textContent ?? undefined;
}

// ---------------------------------------------------------------------------
// WIT interface: document
// ---------------------------------------------------------------------------

export function createElement(tagName: string): bigint {
  const el = document.createElement(tagName);
  return registerNode(el);
}

export function createTextNode(data: string): bigint {
  const txt = document.createTextNode(data);
  return registerNode(txt);
}

export function querySelector(selector: string): bigint | undefined {
  const el = document.querySelector(selector);
  if (!el) return undefined;
  return registerNode(el);
}

export function querySelectorAll(selector: string): bigint[] {
  const nodes = document.querySelectorAll(selector);
  return Array.from(nodes).map(registerNode);
}

export function getBody(): bigint | undefined {
  const b = document.body;
  if (!b) return undefined;
  return registerNode(b);
}

export function getHead(): bigint | undefined {
  const h = document.head;
  if (!h) return undefined;
  return registerNode(h);
}

export function getElementById(id: string): bigint | undefined {
  const el = document.getElementById(id);
  if (!el) return undefined;
  return registerNode(el);
}

// ---------------------------------------------------------------------------
// WIT interface: window
// ---------------------------------------------------------------------------

export function innerWidth(): number {
  return window.innerWidth;
}

export function innerHeight(): number {
  return window.innerHeight;
}

export function consoleLog(message: string): void {
  console.log(message);
}

export function consoleWarn(message: string): void {
  console.warn(message);
}

export function consoleError(message: string): void {
  console.error(message);
}

// ---------------------------------------------------------------------------
// WIT interface: style
// ---------------------------------------------------------------------------

export function setStyleProperty(
  node: bigint,
  property: string,
  value: string,
): void {
  const el = getElement(node) as HTMLElement;
  el.style.setProperty(property, value);
}

export function getStyleProperty(
  node: bigint,
  property: string,
): string | undefined {
  const el = getElement(node) as HTMLElement;
  const v = el.style.getPropertyValue(property);
  return v !== "" ? v : undefined;
}

export function removeStyleProperty(node: bigint, property: string): void {
  const el = getElement(node) as HTMLElement;
  el.style.removeProperty(property);
}

// ---------------------------------------------------------------------------
// Utility: expose handle table for host integration
// ---------------------------------------------------------------------------

/**
 * Register an externally-created Node (e.g. the document root) so it can be
 * referenced by WIT handle. Returns the assigned handle.
 */
export function registerExternalNode(node: Node): bigint {
  return registerNode(node);
}
