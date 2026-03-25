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

// Store reference to WASM exports for event callbacks
let wasmExports = null;
globalThis.__setWasmExports = function(exports) {
    wasmExports = exports;
};

const event_target_exports = {
    addEventListener(target, eventType, useCapture) {
        try {
            const element = globalThis.__elementHandles.get(target);
            if (!element) {
                return "Target handle " + target + " not found";
            }

            const listener = function (event) {
                // Store the event for later access
                const eventHandle = globalThis.__nextEventHandle++;
                globalThis.__eventHandles.set(eventHandle, event);

                // Dispatch to WASM component if exports are available
                if (wasmExports && wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"]) {
                    const callbacks = wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"];
                    const listeners = globalThis.__listeners;

                    // Find the listener ID for this element and event type
                    let listenerId = 0n;
                    for (const [id, info] of listeners) {
                        if (info.target === element && info.type === eventType) {
                            listenerId = id;
                            break;
                        }
                    }

                    if (listenerId !== 0n) {
                        // Determine event type and call appropriate callback
                        if (eventType === "click" || eventType === "mousedown" || eventType === "mouseup" ||
                            eventType === "mousemove" || eventType === "mouseenter" || eventType === "mouseleave") {
                            if (callbacks.on_mouse_event) {
                                const mouseEvent = event;
                                callbacks.on_mouse_event(
                                    listenerId,
                                    eventHandle,
                                    {
                                        target: target,
                                        client_x: mouseEvent.clientX,
                                        client_y: mouseEvent.clientY,
                                        screen_x: mouseEvent.screenX,
                                        screen_y: mouseEvent.screenY,
                                        offset_x: mouseEvent.offsetX,
                                        offset_y: mouseEvent.offsetY,
                                        page_x: mouseEvent.pageX,
                                        page_y: mouseEvent.pageY,
                                        movement_x: mouseEvent.movementX,
                                        movement_y: mouseEvent.movementY,
                                        button: mouseEvent.button || 0,
                                        buttons: mouseEvent.buttons || 0,
                                        ctrl_key: mouseEvent.ctrlKey || false,
                                        shift_key: mouseEvent.shiftKey || false,
                                        alt_key: mouseEvent.altKey || false,
                                        meta_key: mouseEvent.metaKey || false,
                                    }
                                );
                            }
                        } else if (eventType === "keydown" || eventType === "keyup" || eventType === "keypress") {
                            if (callbacks.on_keyboard_event) {
                                const keyboardEvent = event;
                                callbacks.on_keyboard_event(
                                    listenerId,
                                    eventHandle,
                                    {
                                        target: target,
                                        key: keyboardEvent.key || "",
                                        code: keyboardEvent.code || "",
                                        key_code: keyboardEvent.keyCode || 0,
                                        ctrl_key: keyboardEvent.ctrlKey || false,
                                        shift_key: keyboardEvent.shiftKey || false,
                                        alt_key: keyboardEvent.altKey || false,
                                        meta_key: keyboardEvent.metaKey || false,
                                        repeat: keyboardEvent.repeat || false,
                                    }
                                );
                            }
                        } else if (eventType === "focus" || eventType === "blur") {
                            if (callbacks.on_focus_event) {
                                callbacks.on_focus_event(
                                    listenerId,
                                    eventHandle,
                                    {
                                        target: target,
                                        related_target: event.relatedElement ?
                                            (globalThis.__elementHandles.get(event.relatedElement) || 0n) : undefined,
                                    }
                                );
                            }
                        } else if (eventType === "input") {
                            if (callbacks.on_input_event) {
                                const inputEvent = event;
                                callbacks.on_input_event(
                                    listenerId,
                                    eventHandle,
                                    {
                                        target: target,
                                        data: inputEvent.data,
                                        input_type: inputEvent.inputType || "",
                                    }
                                );
                            }
                        } else {
                            if (callbacks.on_generic_event) {
                                callbacks.on_generic_event(listenerId, eventHandle, eventType);
                            }
                        }
                    }
                }
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
// Platform Helpers Interface (tairitsu-browser:full/platform-helpers)
// ============================================================================

let _timeoutCallbacks = new Map();
let _nextTimeoutId = 1;
let _animationCallbacks = new Map();
let _nextAnimationId = 1;

const platform_helpers_exports = {
    innerWidth() {
        return window.innerWidth;
    },
    innerHeight() {
        return window.innerHeight;
    },
    setTimeout(callbackId, ms) {
        const id = _nextTimeoutId++;
        const timeoutId = setTimeout(() => {
            if (wasmExports && wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"]) {
                wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"].on_timeout(callbackId);
            }
        }, ms);
        _timeoutCallbacks.set(id, timeoutId);
        return id;
    },
    clearTimeout(id) {
        if (_timeoutCallbacks.has(id)) {
            clearTimeout(_timeoutCallbacks.get(id));
            _timeoutCallbacks.delete(id);
        }
    },
    requestAnimationFrame(callbackId) {
        const id = _nextAnimationId++;
        const animationId = requestAnimationFrame((timestamp) => {
            if (wasmExports && wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"]) {
                wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"].on_animation_frame(callbackId, timestamp);
            }
        });
        _animationCallbacks.set(id, animationId);
        return id;
    },
    cancelAnimationFrame(id) {
        if (_animationCallbacks.has(id)) {
            cancelAnimationFrame(_animationCallbacks.get(id));
            _animationCallbacks.delete(id);
        }
    },
    getBoundingClientRect(element) {
        const el = globalThis.__elementHandles.get(element);
        if (!el) {
            return { x: 0, y: 0, width: 0, height: 0 };
        }
        const rect = el.getBoundingClientRect();
        return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
    },
    createResizeObserver(callbackId) {
        const observer = new ResizeObserver((entries) => {
            if (wasmExports && wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"]) {
                const entryHandles = entries.map(entry => {
                    if (!globalThis.__resizeObserverEntryHandles) {
                        globalThis.__resizeObserverEntryHandles = new Map();
                        globalThis.__nextResizeObserverEntry = 1n;
                    }
                    const handle = globalThis.__nextResizeObserverEntry++;
                    globalThis.__resizeObserverEntryHandles.set(handle, entry);
                    return handle;
                });
                wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"].on_resize(callbackId, entryHandles);
            }
        });
        return storeElement(observer);
    },
    observeResize(observer, element) {
        const obs = globalThis.__elementHandles.get(observer);
        const el = globalThis.__elementHandles.get(element);
        if (obs && el) {
            obs.observe(el);
        }
    },
    unobserveResize(observer, element) {
        const obs = globalThis.__elementHandles.get(observer);
        const el = globalThis.__elementHandles.get(element);
        if (obs && el) {
            obs.unobserve(el);
        }
    },
    disconnectResize(observer) {
        const obs = globalThis.__elementHandles.get(observer);
        if (obs) {
            obs.disconnect();
        }
    },
    createMutationObserver(callbackId) {
        const observer = new MutationObserver((records) => {
            if (wasmExports && wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"]) {
                const recordHandles = records.map(record => {
                    if (!globalThis.__mutationRecordHandles) {
                        globalThis.__mutationRecordHandles = new Map();
                        globalThis.__nextMutationRecord = 1n;
                    }
                    const handle = globalThis.__nextMutationRecord++;
                    globalThis.__mutationRecordHandles.set(handle, record);
                    return handle;
                });
                wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"].on_mutation(callbackId, recordHandles);
            }
        });
        return storeElement(observer);
    },
    observeMutations(observer, target, _options) {
        const obs = globalThis.__elementHandles.get(observer);
        const el = globalThis.__elementHandles.get(target);
        if (obs && el) {
            obs.observe(el, {
                childList: true,
                attributes: true,
                characterData: true,
                subtree: true,
            });
        }
    },
    disconnectMutation(observer) {
        const obs = globalThis.__elementHandles.get(observer);
        if (obs) {
            obs.disconnect();
        }
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
    "@tairitsu-glue/platform-helpers": platform_helpers_exports,
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
