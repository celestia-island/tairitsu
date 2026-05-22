// @ts-nocheck
import { document_exports } from "./document";
import { element_exports } from "./element";
import { node_exports } from "./node";
import { nonElementParentNode_exports } from "./nonElementParentNode";
import { parentNode_exports } from "./parentNode";
import { window_exports } from "./window";
import { platformHelpers_exports } from "./platformHelpers";
import { mutationRecord_exports } from "./mutationRecord";
import { resizeObserverEntry_exports } from "./resizeObserverEntry";
import { resizeObserverSize_exports } from "./resizeObserverSize";
import { resizeObserver_exports } from "./resizeObserver";
import { cssStyleDeclaration_exports } from "./cssStyleDeclaration";
import { elementCssInlineStyle_exports } from "./elementCssInlineStyle";
import { eventTarget_exports } from "./eventTarget";
import { domTokenList_exports } from "./domTokenList";
import { nodeList_exports } from "./nodeList";
import { location_exports } from "./location";
import { history_exports } from "./history";
import { observers_exports } from "./observers";
import { mutationObserver_exports } from "./mutationObserver";
import { event_exports } from "./event";

export const INTERFACES = {
  "@tairitsu-glue/document": document_exports,
  "@tairitsu-glue/element": element_exports,
  "@tairitsu-glue/node": node_exports,
  "@tairitsu-glue/non-element-parent-node": nonElementParentNode_exports,
  "@tairitsu-glue/parent-node": parentNode_exports,
  "@tairitsu-glue/window": window_exports,
  "@tairitsu-glue/platform-helpers": platformHelpers_exports,
  "@tairitsu-glue/mutation-record": mutationRecord_exports,
  "@tairitsu-glue/resize-observer-entry": resizeObserverEntry_exports,
  "@tairitsu-glue/resize-observer-size": resizeObserverSize_exports,
  "@tairitsu-glue/resize-observer": resizeObserver_exports,
  "@tairitsu-glue/css-style-declaration": cssStyleDeclaration_exports,
  "@tairitsu-glue/element-css-inline-style": elementCssInlineStyle_exports,
  "@tairitsu-glue/event-target": eventTarget_exports,
  "@tairitsu-glue/dom-token-list": domTokenList_exports,
  "@tairitsu-glue/node-list": nodeList_exports,
  "@tairitsu-glue/location": location_exports,
  "@tairitsu-glue/history": history_exports,
  "@tairitsu-glue/observers": observers_exports,
  "@tairitsu-glue/mutation-observer": mutationObserver_exports,
  "@tairitsu-glue/event": event_exports,
};

export function generateModuleCode(exports) {
  const lines = [];

  // Include helper functions needed by exports.
  // IMPORTANT: Always access globalThis directly, never cache in local variables.
  const helpers = `// Helper functions - always use globalThis directly
function storeElement(el) {
    if (!el) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__elementHandles.set(handle, el);
    return handle;
}

function storeNode(node) {
    if (!node) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__nodeHandles.set(handle, node);
    return handle;
}

function storeText(text) {
    if (!text) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__textHandles.set(handle, text);
    return handle;
}

function lookupElement(handle) {
    const el = globalThis.__elementHandles.get(handle);
    if (!el) throw new Error("Element handle " + handle + " not found");
    return el;
}

function lookupNode(handle) {
    const node = globalThis.__nodeHandles.get(handle) || globalThis.__elementHandles.get(handle) || globalThis.__textHandles.get(handle);
    if (!node) throw new Error("Node handle " + handle + " not found");
    return node;
}`;

  // State initializers for modules that keep mutable state on globalThis.
  // When generateModuleCode stringifies export functions into a blob module,
  // the blob has its own scope — any closure variables from the original file
  // are invisible.  The platform-helpers module stores timer/animation state
  // on globalThis so it survives the transition, but the blob must ensure
  // the globalThis properties exist before any export function runs.
  const stateInit = `// Ensure globalThis state objects exist (needed when this blob runs
// inside an import-map module that has no access to the IIFE closure).
if (!globalThis.__tairitsuTimerState) {
  globalThis.__tairitsuTimerState = { timeoutCallbacks: new Map(), nextTimeoutId: 1 };
}
if (!globalThis.__tairitsuAnimState) {
  globalThis.__tairitsuAnimState = { animationCallbacks: new Map(), nextAnimationId: 1 };
}`;

  lines.push(stateInit.trim());
  lines.push(helpers.trim());

  for (const [name, fn] of Object.entries(exports)) {
    let fnStr = fn.toString();
    // Ensure function syntax is complete (shorthand methods don't have 'function' keyword)
    if (!fnStr.startsWith('function')) {
      fnStr = 'function ' + fnStr;
    }
    lines.push('export const ' + name + ' = ' + fnStr + ';');
  }

  return lines.join("\n");
}

export function registerImportMap() {
  const imports: Record<string, string> = {};

  for (const [ifacePath, exports] of Object.entries(INTERFACES)) {
    const code = generateModuleCode(exports);
    const blob = new Blob([code], { type: "application/javascript" });
    const blobUrl = URL.createObjectURL(blob);
    imports[ifacePath] = blobUrl;
    const ifaceName = ifacePath.replace("@tairitsu-glue/", "");
    imports[`tairitsu-browser:full/${ifaceName}@0.2.0`] = blobUrl;
  }

  const shimModules = ["cli", "filesystem", "io", "random", "clocks", "sockets"];
  for (const mod of shimModules) {
    const spec = `@bytecodealliance/preview2-shim/${mod}`;
    imports[spec] = `/wasi-shim/${mod}.js`;
  }

  const existingMap = document.querySelector('script[type="importmap"]');
  if (existingMap) {
    existingMap.remove();
  }

  const script = document.createElement("script");
  script.type = "importmap";
  script.textContent = JSON.stringify({ imports });
  document.head.prepend(script);
}
