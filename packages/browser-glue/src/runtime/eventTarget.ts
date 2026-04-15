// @ts-nocheck
import { lookupElement } from "./helpers";
import { wasmExports } from "./wasmExports";

// Initialize global handle tables for event listeners
globalThis.__listenerHandles = globalThis.__listenerHandles || new Map();
globalThis.__nextListenerHandle = globalThis.__nextListenerHandle || 1n;
// Event handles
globalThis.__eventHandles = globalThis.__eventHandles || new Map();
globalThis.__nextEventHandle = globalThis.__nextEventHandle || 1n;

export const eventTarget_exports = {
  /**
   * Add an event listener to an element.
   * @param target Handle to the element
   * @param eventType Event type (e.g., "click", "input")
   * @param useCapture Whether to use capture phase
   * @returns Handle to the listener or error message
   */
  addEventListener(target: bigint, eventType: string, useCapture: boolean): bigint | string {
    try {
      const element = lookupElement(target);

      const listener = function (event: Event) {
        if (wasmExports) {
          const callbacks = wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"];
          if (callbacks) {
            const eventHandle = globalThis.__nextEventHandle++;
            globalThis.__eventHandles.set(eventHandle, event);

            let listenerId = 0n;
            for (const [id, info] of globalThis.__listenerHandles) {
              if (info.element === element && info.type === eventType) {
                listenerId = id;
                break;
              }
            }

            if (listenerId !== 0n) {
              const evtType = event.type;
              try {
              if (evtType === "mouseenter" || evtType === "mouseleave" || evtType === "mousemove" ||
                  evtType === "mousedown" || evtType === "mouseup" || evtType === "click" ||
                  evtType === "dblclick" || evtType === "mouseover" || evtType === "mouseout" ||
                  evtType === "contextmenu" || evtType === "wheel") {
                callbacks.onMouseEvent(listenerId, eventHandle, {
                  clientX: event.clientX,
                  clientY: event.clientY,
                  offsetX: event.offsetX,
                  offsetY: event.offsetY,
                  button: event.button || 0,
                  buttons: event.buttons || 0,
                  ctrlKey: event.ctrlKey || false,
                  shiftKey: event.shiftKey || false,
                  altKey: event.altKey || false,
                  metaKey: event.metaKey || false,
                });
              } else if (evtType === "keydown" || evtType === "keyup" || evtType === "keypress") {
                callbacks.onKeyboardEvent(listenerId, eventHandle, {
                  key: event.key || "",
                  code: event.code || "",
                  keyCode: event.keyCode || 0,
                  ctrlKey: event.ctrlKey || false,
                  shiftKey: event.shiftKey || false,
                  altKey: event.altKey || false,
                  metaKey: event.metaKey || false,
                  repeat: event.repeat || false,
                });
              } else if (evtType === "focus" || evtType === "blur" || evtType === "focusin" || evtType === "focusout") {
                callbacks.onFocusEvent(listenerId, eventHandle, {
                  relatedTarget: undefined,
                });
              } else if (evtType === "input" || evtType === "change") {
                callbacks.onInputEvent(listenerId, eventHandle, {
                  data: event.data,
                  inputType: event.inputType || "",
                });
              } else {
                callbacks.onGenericEvent(listenerId, eventHandle, evtType);
              }
              } catch(e) { console.error("[tairitsu-glue] event dispatch error:", e); }
            }
          }
        }
      };

      element.addEventListener(eventType, listener, useCapture);

      // Store the listener for later reference
      const handle = globalThis.__nextListenerHandle++;
      globalThis.__listenerHandles.set(handle, { element, type: eventType, listener });

      return handle;
    } catch (error) {
      return `Error adding event listener: ${error}`;
    }
  },

  /**
   * Remove an event listener from an element.
   * @param target Handle to the element
   * @param eventType Event type
   * @param listenerHandle Handle to the listener
   */
  removeEventListener(target: bigint, eventType: string, listenerHandle: bigint): void {
    try {
      const element = lookupElement(target);
      const listenerInfo = globalThis.__listenerHandles.get(listenerHandle);

      if (listenerInfo && listenerInfo.element === element && listenerInfo.type === eventType) {
        element.removeEventListener(eventType, listenerInfo.listener);
        globalThis.__listenerHandles.delete(listenerHandle);
      }
    } catch (error) {
      console.error(`Error removing event listener: ${error}`);
    }
  },
  preventDefault(event: bigint): void {
    const ev = globalThis.__eventHandles.get(event);
    if (ev) ev.preventDefault();
  },
  stopPropagation(event: bigint): void {
    const ev = globalThis.__eventHandles.get(event);
    if (ev) ev.stopPropagation();
  },
};
