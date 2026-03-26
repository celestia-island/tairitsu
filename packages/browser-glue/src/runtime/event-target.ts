// @ts-nocheck
import { lookupElement } from "./helpers";

// Handle tables for event listeners
const _listenerHandles = new Map<bigint, { element: Element; type: string; listener: EventListener }>();
let _nextListenerHandle = 1n;

export const event_target_exports = {
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
        // The actual event handling will be done by the WASM component
        // through the event-callbacks export
        if (globalThis.__wasmExports) {
          const callbacks = globalThis.__wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"];
          if (callbacks && callbacks.onEvent) {
            // Store the event for later access
            const eventHandle = globalThis.__nextEventHandle++;
            globalThis.__eventHandles.set(eventHandle, event);

            // Find the listener ID
            let listenerId = 0n;
            for (const [id, info] of _listenerHandles) {
              if (info.element === element && info.type === eventType) {
                listenerId = id;
                break;
              }
            }

            if (listenerId !== 0n && callbacks.onEvent) {
              callbacks.onEvent(listenerId, eventHandle, target);
            }
          }
        }
      };

      element.addEventListener(eventType, listener, useCapture);

      // Store the listener for later reference
      const handle = _nextListenerHandle++;
      _listenerHandles.set(handle, { element, type: eventType, listener });

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
      const listenerInfo = _listenerHandles.get(listenerHandle);

      if (listenerInfo && listenerInfo.element === element && listenerInfo.type === eventType) {
        element.removeEventListener(eventType, listenerInfo.listener);
        _listenerHandles.delete(listenerHandle);
      }
    } catch (error) {
      console.error(`Error removing event listener: ${error}`);
    }
  },
};
