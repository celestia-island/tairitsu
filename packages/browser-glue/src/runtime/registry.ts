// @ts-nocheck
import { document_exports } from "./document";
import { element_exports } from "./element";
import { node_exports } from "./node";
import { nonElementParentNode_exports } from "./nonElementParentNode";
import { window_exports } from "./window";
import { platformHelpers_exports } from "./platformHelpers";
import { mutationRecord_exports } from "./mutationRecord";
import { resizeObserverEntry_exports } from "./resizeObserverEntry";
import { resizeObserverSize_exports } from "./resizeObserverSize";
import { cssStyleDeclaration_exports } from "./cssStyleDeclaration";
import { elementCssInlineStyle_exports } from "./elementCssInlineStyle";
import { eventTarget_exports } from "./eventTarget";

export const INTERFACES = {
  "@tairitsu-glue/document": document_exports,
  "@tairitsu-glue/element": element_exports,
  "@tairitsu-glue/node": node_exports,
  "@tairitsu-glue/non-element-parent-node": nonElementParentNode_exports,
  "@tairitsu-glue/window": window_exports,
  "@tairitsu-glue/platform-helpers": platformHelpers_exports,
  "@tairitsu-glue/mutation-record": mutationRecord_exports,
  "@tairitsu-glue/resize-observer-entry": resizeObserverEntry_exports,
  "@tairitsu-glue/resize-observer-size": resizeObserverSize_exports,
  "@tairitsu-glue/css-style-declaration": cssStyleDeclaration_exports,
  "@tairitsu-glue/element-css-inline-style": elementCssInlineStyle_exports,
  "@tairitsu-glue/event-target": eventTarget_exports,
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
  // Start with static external URLs for WASI preview2-shim
  const imports = {
    // WASI preview2-shim (static external CDN URLs required by jco-transpiled wrappers)
    "@bytecodealliance/preview2-shim/cli": "https://esm.sh/@bytecodealliance/preview2-shim/cli",
    "@bytecodealliance/preview2-shim/filesystem": "https://esm.sh/@bytecodealliance/preview2-shim/filesystem",
    "@bytecodealliance/preview2-shim/io": "https://esm.sh/@bytecodealliance/preview2-shim/io",
    "@bytecodealliance/preview2-shim/random": "https://esm.sh/@bytecodealliance/preview2-shim/random",
  };

  for (const [ifacePath, exports] of Object.entries(INTERFACES)) {
    const code = generateModuleCode(exports);
    const blob = new Blob([code], { type: "application/javascript" });
    const blobUrl = URL.createObjectURL(blob);
    // Add bare module specifier (e.g., "@tairitsu-glue/console")
    imports[ifacePath] = blobUrl;
    // Also add the full WIT package name (e.g., "tairitsu-browser:full/console@0.2.0")
    const ifaceName = ifacePath.replace("@tairitsu-glue/", "");
    imports[`tairitsu-browser:full/${ifaceName}@0.2.0`] = blobUrl;
  }

  // Remove any pre-existing import map to avoid the browser's one-import-map
  // limitation in Chrome < 127, where a second importmap is silently ignored.
  const existingMap = document.querySelector('script[type="importmap"]');
  if (existingMap) {
    existingMap.remove();
  }

  // Create and prepend the single, complete import map (WASI + tairitsu-glue).
  // This script runs synchronously before any <script type="module"> is loaded,
  // so the import map is guaranteed to be registered first.
  const script = document.createElement("script");
  script.type = "importmap";
  script.textContent = JSON.stringify({ imports });
  document.head.prepend(script);
}
