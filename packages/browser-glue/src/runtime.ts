/**
 * Tairitsu Browser Glue - Single Self-Contained Bundle
 * 
 * This bundle contains all browser glue implementations and automatically
 * registers them as importable modules via blob URLs and dynamic import maps.
 * 
 * Usage: <script type="module" src="/__tairitsu_glue__.js"></script>
 * 
 * After loading, the bundle will:
 * 1. Generate blob URLs for each interface
 * 2. Update the import map to map tairitsu-browser:full/* to these URLs
 */

// ============================================================================
// Handle Tables (shared across all glue functions)
// ============================================================================

const _elementHandles = new Map<bigint, Element>();
const _documentHandles = new Map<bigint, Document>();
const _nodeHandles = new Map<bigint, Node>();
const _textHandles = new Map<bigint, Text>();
let _nextHandle = 1n;

(globalThis as any).__elementHandles = _elementHandles;
(globalThis as any).__documentHandles = _documentHandles;
(globalThis as any).__nodeHandles = _nodeHandles;
(globalThis as any).__textHandles = _textHandles;

function storeElement(el: Element | null | undefined): bigint | undefined {
    if (!el) return undefined;
    const handle = _nextHandle++;
    _elementHandles.set(handle, el);
    return handle;
}

function storeNode(node: Node | null | undefined): bigint | undefined {
    if (!node) return undefined;
    const handle = _nextHandle++;
    _nodeHandles.set(handle, node);
    return handle;
}

function storeText(text: Text | null | undefined): bigint | undefined {
    if (!text) return undefined;
    const handle = _nextHandle++;
    _textHandles.set(handle, text);
    return handle;
}

function lookupElement(handle: bigint): Element {
    const el = _elementHandles.get(handle);
    if (!el) throw new Error(`Element handle ${handle} not found`);
    return el;
}

function lookupNode(handle: bigint): Node {
    const node = _nodeHandles.get(handle) || _elementHandles.get(handle);
    if (!node) throw new Error(`Node handle ${handle} not found`);
    return node;
}

// ============================================================================
// Console Interface (tairitsu-browser:full/console)
// ============================================================================

const console_exports = {
    log(message: string): void {
        console.log(message);
    },
    warn(message: string): void {
        console.warn(message);
    },
    error(message: string): void {
        console.error(message);
    },
};

// ============================================================================
// Style Interface (tairitsu-browser:full/style)
// ============================================================================

const style_exports = {
    setStyleProperty(element: bigint, property: string, value: string): void | string {
        try {
            (lookupElement(element) as HTMLElement).style.setProperty(property, value);
        } catch (e) {
            return String(e);
        }
    },
    getStyleProperty(element: bigint, property: string): string | undefined {
        return (lookupElement(element) as HTMLElement).style.getPropertyValue(property) || undefined;
    },
    removeStyleProperty(element: bigint, property: string): void | string {
        try {
            (lookupElement(element) as HTMLElement).style.removeProperty(property);
        } catch (e) {
            return String(e);
        }
    },
};

// ============================================================================
// Event Target Interface (tairitsu-browser:full/event-target)
// ============================================================================

let _nextListenerId = 1n;
const _listeners = new Map<bigint, {
    target: EventTarget;
    type: string;
    listener: EventListener;
    useCapture: boolean;
}>();
const _eventHandles = new Map<bigint, Event>();
let _nextEventHandle = 1n;

const event_target_exports = {
    addEventListener(target: bigint, eventType: string, useCapture: boolean): bigint | string {
        try {
            const element = _elementHandles.get(target) as EventTarget | undefined;
            if (!element) {
                return `Target handle ${target} not found`;
            }

            const listener: EventListener = (event: Event) => {
                const eventHandle = _nextEventHandle++;
                _eventHandles.set(eventHandle, event);
            };

            element.addEventListener(eventType, listener, useCapture);

            const listenerId = _nextListenerId++;
            _listeners.set(listenerId, {
                target: element,
                type: eventType,
                listener,
                useCapture,
            });

            return listenerId;
        } catch (e) {
            return String(e);
        }
    },
    removeEventListener(_target: bigint, listenerId: bigint): void | string {
        try {
            const info = _listeners.get(listenerId);
            if (!info) {
                return `Listener ID ${listenerId} not found`;
            }
            info.target.removeEventListener(info.type, info.listener, info.useCapture);
            _listeners.delete(listenerId);
        } catch (e) {
            return String(e);
        }
    },
    preventDefault(event: bigint): void {
        _eventHandles.get(event)?.preventDefault();
    },
    stopPropagation(event: bigint): void {
        _eventHandles.get(event)?.stopPropagation();
    },
};

// ============================================================================
// Document Interface (tairitsu-browser:full/document)
// ============================================================================

const document_exports = {
    createElement(localName: string): bigint | undefined {
        const el = document.createElement(localName);
        return storeElement(el);
    },
    createTextNode(data: string): bigint | undefined {
        const text = document.createTextNode(data);
        return storeText(text);
    },
    getBody(): bigint | undefined {
        return storeElement(document.body);
    },
};

// ============================================================================
// Element Interface (tairitsu-browser:full/element)
// ============================================================================

const element_exports = {
    setAttribute(self: bigint, qualifiedName: string, value: string): void {
        lookupElement(self).setAttribute(qualifiedName, value);
    },
    removeAttribute(self: bigint, qualifiedName: string): void {
        lookupElement(self).removeAttribute(qualifiedName);
    },
};

// ============================================================================
// Node Interface (tairitsu-browser:full/node)
// ============================================================================

const node_exports = {
    appendChild(self: bigint, child: bigint): bigint | undefined {
        const parent = lookupNode(self);
        const childNode = lookupNode(child);
        const result = parent.appendChild(childNode);
        return storeNode(result);
    },
    removeChild(self: bigint, child: bigint): bigint | undefined {
        const parent = lookupNode(self);
        const childNode = lookupNode(child);
        const result = parent.removeChild(childNode);
        return storeNode(result);
    },
    setTextContent(self: bigint, text: string): void {
        lookupNode(self).textContent = text;
    },
    getTextContent(self: bigint): string {
        return lookupNode(self).textContent || "";
    },
};

// ============================================================================
// Non-Element-Parent-Node Interface (tairitsu-browser:full/non-element-parent-node)
// ============================================================================

const non_element_parent_node_exports = {
    getElementById(self: bigint, elementId: string): bigint | undefined {
        const doc = _documentHandles.get(self) || document;
        const el = (doc as NonElementParentNode).getElementById(elementId);
        return storeElement(el);
    },
};

// ============================================================================
// Window Interface (tairitsu-browser:full/window)
// ============================================================================

const window_exports = {
    getInnerWidth(): number {
        return window.innerWidth;
    },
    getInnerHeight(): number {
        return window.innerHeight;
    },
};

// ============================================================================
// Interface Registry
// ============================================================================

const INTERFACES: Record<string, Record<string, Function>> = {
    "tairitsu-browser:full/console": console_exports,
    "tairitsu-browser:full/style": style_exports,
    "tairitsu-browser:full/event-target": event_target_exports,
    "tairitsu-browser:full/document": document_exports,
    "tairitsu-browser:full/element": element_exports,
    "tairitsu-browser:full/node": node_exports,
    "tairitsu-browser:full/non-element-parent-node": non_element_parent_node_exports,
    "tairitsu-browser:full/window": window_exports,
};

// ============================================================================
// Module Generation & Import Map Registration
// ============================================================================

function generateModuleCode(exports: Record<string, Function>): string {
    const lines: string[] = [];
    
    for (const [name, fn] of Object.entries(exports)) {
        let fnStr = fn.toString();
        // Ensure function syntax is complete (shorthand methods don't have 'function' keyword)
        if (!fnStr.startsWith('function')) {
            fnStr = `function ${fnStr}`;
        }
        lines.push(`export const ${name} = ${fnStr};`);
    }
    
    return lines.join("\n");
}

function registerImportMap(): void {
    const imports: Record<string, string> = {};
    
    for (const [ifacePath, exports] of Object.entries(INTERFACES)) {
        const code = generateModuleCode(exports);
        const blob = new Blob([code], { type: "application/javascript" });
        const blobUrl = URL.createObjectURL(blob);
        imports[ifacePath] = blobUrl;
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
(globalThis as any).__TAIRITSU_GLUE__ = {
    INTERFACES,
    handles: { _elementHandles, _nodeHandles, _documentHandles, _textHandles },
};
