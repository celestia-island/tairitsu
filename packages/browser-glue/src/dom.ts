/**
 * DOM glue — implements the `tairitsu-browser:dom` WIT import interfaces.
 *
 * Each function here maps 1:1 to a WIT function in `wit/dom.wit`.
 * Node handles are underlying browser `Element` / `Text` / `Node`
 * objects stored in a shared handle table.
 */

import { getElement, getNode, registerNode } from "./handles";

// ---------------------------------------------------------------------------
// Diagnostic support
// ---------------------------------------------------------------------------

interface DiagnosticCallbacks {
  onError?: (error: DomDiagnosticError) => void;
  onWarning?: (warning: string) => void;
}

export interface DomDiagnosticError {
  kind: "invalid-handle" | "operation-failed" | "environment-error";
  operation: string;
  message: string;
  context?: Record<string, unknown>;
}

let _diagnosticCallbacks: DiagnosticCallbacks = {};

/**
 * Register diagnostic callbacks for observing DOM glue internals.
 */
export function registerDomDiagnosticCallbacks(callbacks: DiagnosticCallbacks): void {
  _diagnosticCallbacks = { ..._diagnosticCallbacks, ...callbacks };
}

function reportError(error: DomDiagnosticError): void {
  if (_diagnosticCallbacks.onError) {
    _diagnosticCallbacks.onError(error);
  }
  console.error(`[browser-glue DOM] ${error.operation} failed: ${error.message}`, error.context ?? "");
}

function reportWarning(message: string): void {
  if (_diagnosticCallbacks.onWarning) {
    _diagnosticCallbacks.onWarning(message);
  }
  console.warn(`[browser-glue DOM] ${message}`);
}

// ---------------------------------------------------------------------------
// Environment validation
// ---------------------------------------------------------------------------

/**
 * Check if the browser environment is properly initialized.
 * Returns true if all required DOM APIs are available.
 */
export function checkEnvironment(): { ok: boolean; issues: string[] } {
  const issues: string[] = [];

  if (typeof document === "undefined") {
    issues.push("document object is not available");
  }
  if (typeof window === "undefined") {
    issues.push("window object is not available");
  }
  if (typeof HTMLElement === "undefined") {
    issues.push("HTMLElement constructor is not available");
  }

  if (document) {
    if (!document.createElement) {
      issues.push("document.createElement is not available");
    }
    if (!document.body && document.readyState === "complete") {
      issues.push("document.body is not available (DOM fully loaded)");
    }
  }

  return { ok: issues.length === 0, issues };
}

// ---------------------------------------------------------------------------
// WIT interface: node
// ---------------------------------------------------------------------------

export function appendChild(parent: bigint, child: bigint): void {
  try {
    getNode(parent).appendChild(getNode(child));
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "appendChild",
      message: e instanceof Error ? e.message : String(e),
      context: { parent, child },
    });
    throw e;
  }
}

export function removeChild(parent: bigint, child: bigint): void {
  try {
    getNode(parent).removeChild(getNode(child));
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "removeChild",
      message: e instanceof Error ? e.message : String(e),
      context: { parent, child },
    });
    throw e;
  }
}

export function setAttribute(
  node: bigint,
  name: string,
  value: string,
): void {
  try {
    getElement(node).setAttribute(name, value);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "setAttribute",
      message: e instanceof Error ? e.message : String(e),
      context: { node, name, value },
    });
    throw e;
  }
}

export function getAttribute(
  node: bigint,
  name: string,
): string | undefined {
  try {
    return getElement(node).getAttribute(name) ?? undefined;
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "getAttribute",
      message: e instanceof Error ? e.message : String(e),
      context: { node, name },
    });
    return undefined;
  }
}

export function removeAttribute(node: bigint, name: string): void {
  try {
    getElement(node).removeAttribute(name);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "removeAttribute",
      message: e instanceof Error ? e.message : String(e),
      context: { node, name },
    });
    throw e;
  }
}

export function setTextContent(node: bigint, text: string): void {
  try {
    getNode(node).textContent = text;
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "setTextContent",
      message: e instanceof Error ? e.message : String(e),
      context: { node, textLength: text.length },
    });
    throw e;
  }
}

export function getTextContent(node: bigint): string | undefined {
  try {
    return getNode(node).textContent ?? undefined;
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "getTextContent",
      message: e instanceof Error ? e.message : String(e),
      context: { node },
    });
    return undefined;
  }
}

// ---------------------------------------------------------------------------
// WIT interface: document
// ---------------------------------------------------------------------------

export function createElement(tagName: string): bigint {
  try {
    const el = document.createElement(tagName);
    return registerNode(el);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "createElement",
      message: e instanceof Error ? e.message : String(e),
      context: { tagName },
    });
    throw e;
  }
}

export function createTextNode(data: string): bigint {
  try {
    const txt = document.createTextNode(data);
    return registerNode(txt);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "createTextNode",
      message: e instanceof Error ? e.message : String(e),
      context: { dataLength: data.length },
    });
    throw e;
  }
}

export function querySelector(selector: string): bigint | undefined {
  try {
    const el = document.querySelector(selector);
    if (!el) return undefined;
    return registerNode(el);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "querySelector",
      message: e instanceof Error ? e.message : String(e),
      context: { selector },
    });
    return undefined;
  }
}

export function querySelectorAll(selector: string): bigint[] {
  try {
    const nodes = document.querySelectorAll(selector);
    return Array.from(nodes).map(registerNode);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "querySelectorAll",
      message: e instanceof Error ? e.message : String(e),
      context: { selector },
    });
    return [];
  }
}

export function getBody(): bigint | undefined {
  const b = document.body;
  if (!b) {
    reportWarning("document.body is not available");
    return undefined;
  }
  return registerNode(b);
}

export function getHead(): bigint | undefined {
  const h = document.head;
  if (!h) {
    reportWarning("document.head is not available");
    return undefined;
  }
  return registerNode(h);
}

// Compatibility aliases expected by some jco-transpiled wrappers.
export const body = getBody;
export const head = getHead;

export function getElementById(id: string): bigint | undefined {
  try {
    const el = document.getElementById(id);
    if (!el) return undefined;
    return registerNode(el);
  } catch (e) {
    reportError({
      kind: "operation-failed",
      operation: "getElementById",
      message: e instanceof Error ? e.message : String(e),
      context: { id },
    });
    return undefined;
  }
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
