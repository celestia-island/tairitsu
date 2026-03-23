// @ts-nocheck
/**
 * Tairitsu Browser Glue - Single Self-Contained Bundle
 * 
 * This bundle contains all browser glue implementations and automatically
 * registers them as importable modules via blob URLs and dynamic import maps.
 * 
 * Usage: <script type="module" src="/__tairitsu_glue__.js"></script>
 * 
 * After loading, bundle will:
 * 1. Generate blob URLs for each interface
 * 2. Update the import map to map tairitsu-browser:full/* to these URLs
 */

// ============================================================================
// Handle Tables (shared across all glue functions via globalThis)
// ============================================================================

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

// Helper functions for blob URL modules
globalThis.__storeElement = function (el) {
    if (!el) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__elementHandles.set(handle, el);
    return handle;
};

globalThis.__storeNode = function (node) {
    if (!node) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__nodeHandles.set(handle, node);
    return handle;
};

globalThis.__storeText = function (text) {
    if (!text) return undefined;
    const handle = globalThis.__nextHandle++;
    globalThis.__textHandles.set(handle, text);
    return handle;
};

globalThis.__lookupElement = function (handle) {
    const el = globalThis.__elementHandles.get(handle);
    if (!el) throw new Error("Element handle " + handle + " not found");
    return el;
};

globalThis.__lookupNode = function (handle) {
    const node = globalThis.__nodeHandles.get(handle) || globalThis.__elementHandles.get(handle) || globalThis.__textHandles.get(handle);
    if (!node) throw new Error("Node handle " + handle + " not found");
    return node;
};

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
}

// ============================================================================
// Console Interface (tairitsu-browser:full/console)
// ============================================================================

const console_exports = {
    log(message) {
        console.log(message);
    },
    warn(message) {
        console.warn(message);
    },
    error(message) {
        console.error(message);
    },
};

// ============================================================================
// Style Interface (tairitsu-browser:full/style)
// ============================================================================

const style_exports = {
    setStyleProperty(element, property, value) {
        try {
            lookupElement(element).style.setProperty(property, value);
        } catch (e) {
            return String(e);
        }
    },
    getStyleProperty(element, property) {
        return lookupElement(element).style.getPropertyValue(property) || undefined;
    },
    removeStyleProperty(element, property) {
        try {
            lookupElement(element).style.removeProperty(property);
        } catch (e) {
            return String(e);
        }
    },
};

// ============================================================================
// Event Target Interface (tairitsu-browser:full/event-target)
// ============================================================================

globalThis.__nextListenerId = globalThis.__nextListenerId || 1n;
globalThis.__listeners = globalThis.__listeners || new Map();
globalThis.__eventHandles = globalThis.__eventHandles || new Map();
globalThis.__nextEventHandle = globalThis.__nextEventHandle || 1n;

const event_target_exports = {
    addEventListener(target, eventType, useCapture) {
        try {
            const element = globalThis.__elementHandles.get(target);
            if (!element) {
                return "Target handle " + target + " not found";
            }

            const listener = function (event) {
                const eventHandle = globalThis.__nextEventHandle++;
                globalThis.__eventHandles.set(eventHandle, event);
            };

            element.addEventListener(eventType, listener, useCapture);

            const listenerId = globalThis.__nextListenerId++;
            globalThis.__listeners.set(listenerId, {
                target: element,
                type: eventType,
                listener: listener,
                useCapture: useCapture,
            });

            return listenerId;
        } catch (e) {
            return String(e);
        }
    },
    removeEventListener(_target, listenerId) {
        try {
            const info = globalThis.__listeners.get(listenerId);
            if (!info) {
                return "Listener ID " + listenerId + " not found";
            }
            info.target.removeEventListener(info.type, info.listener, info.useCapture);
            globalThis.__listeners.delete(listenerId);
        } catch (e) {
            return String(e);
        }
    },
    preventDefault(event) {
        globalThis.__eventHandles.get(event)?.preventDefault();
    },
    stopPropagation(event) {
        globalThis.__eventHandles.get(event)?.stopPropagation();
    },
};

// ============================================================================
// Document Interface (tairitsu-browser:full/document)
// ============================================================================

const document_exports = {
    createElement(localName) {
        const el = document.createElement(localName);
        return storeElement(el);
    },
    createTextNode(data) {
        const text = document.createTextNode(data);
        return storeText(text);
    },
    getBody() {
        return storeElement(document.body);
    },
};

// ============================================================================
// Element Interface (tairitsu-browser:full/element)
// ============================================================================

const element_exports = {
    setAttribute(self, qualifiedName, value) {
        lookupElement(self).setAttribute(qualifiedName, value);
    },
    removeAttribute(self, qualifiedName) {
        lookupElement(self).removeAttribute(qualifiedName);
    },
};

// ============================================================================
// Node Interface (tairitsu-browser:full/node)
// ============================================================================

const node_exports = {
    appendChild(self, child) {
        const parent = lookupNode(self);
        const childNode = lookupNode(child);
        const result = parent.appendChild(childNode);
        return storeNode(result);
    },
    removeChild(self, child) {
        const parent = lookupNode(self);
        const childNode = lookupNode(child);
        const result = parent.removeChild(childNode);
        return storeNode(result);
    },
    setTextContent(self, text) {
        lookupNode(self).textContent = text;
    },
    getTextContent(self) {
        return lookupNode(self).textContent || "";
    },
};

// ============================================================================
// Non-Element-Parent-Node Interface (tairitsu-browser:full/non-element-parent-node)
// ============================================================================

const non_element_parent_node_exports = {
    getElementById(self, elementId) {
        const doc = globalThis.__documentHandles.get(self) || document;
        const el = doc.getElementById(elementId);
        return storeElement(el);
    },
};

// ============================================================================
// Window Interface (tairitsu-browser:full/window)
// ============================================================================

const window_exports = {
    getInnerWidth() {
        return window.innerWidth;
    },
    getInnerHeight() {
        return window.innerHeight;
    },
};

// ============================================================================
// Mutation-Record Interface (tairitsu-browser:full/mutation-record)
// ============================================================================

const mutation_record_exports = {
    getType(self) {
        if (!globalThis.__mutationRecordHandles) return '';
        const rec = globalThis.__mutationRecordHandles.get(self);
        return rec ? rec.type : '';
    },
    getTarget(self) {
        if (!globalThis.__mutationRecordHandles) return 0n;
        const rec = globalThis.__mutationRecordHandles.get(self);
        if (!rec || !rec.target) return 0n;
        return storeElement(rec.target);
    },
    getPreviousSibling(self) {
        if (!globalThis.__mutationRecordHandles) return undefined;
        const rec = globalThis.__mutationRecordHandles.get(self);
        if (!rec || !rec.previousSibling) return undefined;
        return storeNode(rec.previousSibling);
    },
    getNextSibling(self) {
        if (!globalThis.__mutationRecordHandles) return undefined;
        const rec = globalThis.__mutationRecordHandles.get(self);
        if (!rec || !rec.nextSibling) return undefined;
        return storeNode(rec.nextSibling);
    },
    getAttributeName(self) {
        if (!globalThis.__mutationRecordHandles) return undefined;
        const rec = globalThis.__mutationRecordHandles.get(self);
        return rec ? (rec.attributeName ?? undefined) : undefined;
    },
    getAttributeNamespace(self) {
        if (!globalThis.__mutationRecordHandles) return undefined;
        const rec = globalThis.__mutationRecordHandles.get(self);
        return rec ? (rec.attributeNamespace ?? undefined) : undefined;
    },
    getOldValue(self) {
        if (!globalThis.__mutationRecordHandles) return undefined;
        const rec = globalThis.__mutationRecordHandles.get(self);
        return rec ? (rec.oldValue ?? undefined) : undefined;
    },
};

// ============================================================================
// Resize-Observer-Entry Interface (tairitsu-browser:full/resize-observer-entry)
// ============================================================================

const resize_observer_entry_exports = {
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

// ============================================================================
// Resize-Observer-Size Interface (tairitsu-browser:full/resize-observer-size)
// ============================================================================

const resize_observer_size_exports = {
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

// ============================================================================
// Interface Registry
// ============================================================================

const INTERFACES = {
    "@tairitsu-glue/console": console_exports,
    "@tairitsu-glue/style": style_exports,
    "@tairitsu-glue/event-target": event_target_exports,
    "@tairitsu-glue/document": document_exports,
    "@tairitsu-glue/element": element_exports,
    "@tairitsu-glue/node": node_exports,
    "@tairitsu-glue/non-element-parent-node": non_element_parent_node_exports,
    "@tairitsu-glue/window": window_exports,
    "@tairitsu-glue/mutation-record": mutation_record_exports,
    "@tairitsu-glue/resize-observer-entry": resize_observer_entry_exports,
    "@tairitsu-glue/resize-observer-size": resize_observer_size_exports,
};

// ============================================================================
// Module Generation & Import Map Registration
// ============================================================================

function generateModuleCode(exports) {
    const lines = [];

    // Include helper functions needed by exports
    // IMPORTANT: Always access globalThis directly, never cache in local variables
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

function registerImportMap() {
    const imports = {};

    for (const [ifacePath, exports] of Object.entries(INTERFACES)) {
        const code = generateModuleCode(exports);
        const blob = new Blob([code], { type: "application/javascript" });
        const blobUrl = URL.createObjectURL(blob);
        // Add bare module specifier (e.g., "@tairitsu-glue/console")
        imports[ifacePath] = blobUrl;
        // Also add the full package name for WASM imports (e.g., "tairitsu-browser:full/console@0.2.0")
        // Extract the interface name from the path
        const ifaceName = ifacePath.replace("@tairitsu-glue/", "");
        imports[`tairitsu-browser:full/${ifaceName}@0.2.0`] = blobUrl;
    }

    // Create or update import map
    const importMap = { imports };

    // Check if there's already an import map
    const existingMap = document.querySelector('script[type="importmap"]');
    if (existingMap) {
        try {
            const existing = JSON.parse(existingMap.textContent || "{}");
            Object.assign(existing.imports || {}, importMap.imports);
            existingMap.textContent = JSON.stringify(existing);
        } catch {
            existingMap.textContent = JSON.stringify(importMap);
        }
    } else {
        const script = document.createElement("script");
        script.type = "importmap";
        script.textContent = JSON.stringify(importMap);
        document.head.appendChild(script);
    }
}

// Auto-register when loaded
registerImportMap();

// Export for debugging
globalThis.__TAIRITSU_GLUE__ = {
    INTERFACES,
    handles: {
        get elementHandles() { return globalThis.__elementHandles; },
        get nodeHandles() { return globalThis.__nodeHandles; },
        get documentHandles() { return globalThis.__documentHandles; },
        get textHandles() { return globalThis.__textHandles; },
        get nextHandle() { return globalThis.__nextHandle; },
    },
};
