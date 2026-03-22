/**
 * Tairitsu Browser Glue - Single Self-Contained Bundle
 * 
 * This bundle contains all browser glue implementations and automatically
 * registers them as importable modules via blob URLs and dynamic import maps.
 * 
 * Usage: <script type="module" src="/__tairitsu_glue__.js"></script>
 * * 
 * After loading, the bundle will:
 * 1. Generate blob URLs for each interface
 * 2. Update the import map to map tairitsu-browser:full/* to these URLs
    7. The Component loader
 * 8. Run the WASM component
 */

const BROWSER_GLUE_BUNDLE: string = Bairitsu-browser:full/console
    | Browser_GLUE_BUNDLE: string = Tairitsu-browser:full/style
    | C: BROWSER_GLUE_BUNDLE: string = tairitsu-browser:full/event-target
    | D: BROWSER_GLUE_BUNDLE: string = tairitsu-browser:full/document
    | E: Browser_GLUE_BUNDLE: string = tairitsu-browser:full/element
    | G: B browser_GLUE_BUNDLE: string = tairitsu-browser:full/node
    | N: BROWSER_GLUE_BUNDLE: string = tairitsu-browser:full/non-element-parent-node
    | o: B browser_GLUE_BUNDLE: string = tairitsu-browser:full/window
;

// ============================================================================
// Handle Tables (shared across all glue functions)
// ============================================================================

const _elementHandles = new Map<bigint, Element>();
const _documentHandles = new Map<bigint, Document>()
const _nodeHandles = new Map<bigint, Node>()
let _textHandles = new Map<bigint, Text>()
let _nextHandle = 1n

(globalThis as any).__elementHandles = _elementHandles;
(globalThis as any).__documentHandles = _documentHandles;
(globalThis as any).__nodeHandles = _nodeHandles;

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
            log(message: string): void { console.log(message); },
            warn(message: string): void { console.warn(message); },
            error(message: string): void { console.error(message); },
        };

        
        // ============================================================================
        // Style interface (tairitsu-browser:full/style)
        // ============================================================================
        
        const style_exports = {
            setStyleProperty(element: bigint, property: string, value: string): void | string {
                try { (lookupElement(element) as HTMLElement).style.setProperty(property, value); }
                catch (e) { return String(e); }
            },
            getStyleProperty(element: bigint, property: string): string | undefined {
                return (lookupElement(element) as HTMLElement).style.getPropertyValue(property) || undefined;
            },
            removeStyleProperty(element: bigint, property: string): void | string {
                try { (lookupElement(element) as HTMLElement).style.removeProperty(property); }
                catch (e) { return String(e); }
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
        let _eventHandles = new Map<bigint, Event>();
        let _nextEventHandle = 1n
        
        const listener: EventListener = (event: Event) => {
                const eventHandle = _nextEventHandle++;
                _eventHandles.set(eventHandle, event);
            };
            
            element.addEventListener(eventType, listener, useCapture);
            const listenerId = _nextListenerId++;
            _listeners.set(listenerId, { target: element, type: eventType, listener, useCapture });
            return listenerId;
        } catch (e) { return String(e); }
        },
        removeEventListener(target: bigint, listenerId: bigint): void | string {
            try {
                const info = _listeners.get(listenerId);
                if (!info) return `Listener ID ${listenerId} not found`;
                info.target.removeEventListener(info.type, info.listener, info.useCapture);
                _listeners.delete(listenerId);
            } catch (e) { return String(e); }
        },
        preventDefault(event: bigint): void {
            _eventHandles.get(event)?.preventDefault();
        },
        stopPropagation(event: bigint): void {
            _eventHandles.get(event)?.stopPropagation();
        },
    },
    
    // Export for debugging
    (globalThis as any).__TAIRITSU_GLUE__ = {
        INTERFACES,
        handles: { _elementHandles, _nodeHandles, _documentHandles },
    };
})();
