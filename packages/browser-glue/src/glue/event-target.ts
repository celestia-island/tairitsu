/**
 * Event target glue — implements the `tairitsu-browser:full/event-target` WIT import interface.
 *
 * This interface is manually defined in browser-full.wit, providing simplified event handling.
 * DO NOT EDIT MANUALLY - this file provides event handling support for WASM components.
 */

// Listener ID counter
let _nextListenerId = 1n;

// Map of listener ID to { target, type, listener, useCapture }
const _listeners = new Map<bigint, {
  target: EventTarget;
  type: string;
  listener: EventListener;
  useCapture: boolean;
}>();

// Map of event handle to Event object
const _eventHandles = new Map<bigint, Event>();
let _nextEventHandle = 1n;

/**
 * Add an event listener to a target node.
 * Returns a listener-id that can be used to remove the listener.
 */
export function addEventListener(target: bigint, eventType: string, useCapture: boolean): bigint | string {
  try {
    // Get the target from element handles (assuming it's an Element)
    const element = (globalThis as any).__elementHandles?.get(target);
    if (!element) {
      return `Target handle ${target} not found`;
    }

    const listener: EventListener = (event: Event) => {
      // Store the event for later access
      const eventHandle = _nextEventHandle++;
      _eventHandles.set(eventHandle, event);

      // Call the component's event callback (this would be implemented by the component)
      // For now, we just store the event
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
}

/**
 * Remove a previously registered event listener.
 */
export function removeEventListener(target: bigint, listenerId: bigint): void | string {
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
}

/**
 * Prevent the default action for this event.
 */
export function preventDefault(event: bigint): void {
  const eventObj = _eventHandles.get(event);
  if (eventObj) {
    eventObj.preventDefault();
  }
}

/**
 * Stop the event from propagating further.
 */
export function stopPropagation(event: bigint): void {
  const eventObj = _eventHandles.get(event);
  if (eventObj) {
    eventObj.stopPropagation();
  }
}
