/**
 * Events glue — implements the `tairitsu-browser:events` WIT import interfaces.
 *
 * Manages event listener registration and dispatches serialised event data
 * back to WASM guest callbacks.
 */

import { getEventTarget, registerNode } from "./handle-table.js";

// ---------------------------------------------------------------------------
// Types matching WIT records
// ---------------------------------------------------------------------------

export interface MouseEventData {
  clientX: number;
  clientY: number;
  offsetX: number;
  offsetY: number;
  button: number;
  buttons: number;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}

export interface KeyboardEventData {
  key: string;
  code: string;
  keyCode: number;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  repeat: boolean;
}

export interface FocusEventData {
  relatedTarget: bigint | undefined;
}

export interface InputEventData {
  data: string | undefined;
  inputType: string;
}

// ---------------------------------------------------------------------------
// Listener registry
// ---------------------------------------------------------------------------

let _nextListenerId = 1n;

interface ListenerEntry {
  target: EventTarget;
  eventType: string;
  handler: EventListener;
}

const _listeners = new Map<bigint, ListenerEntry>();

// ---------------------------------------------------------------------------
// Event handle table for preventDefault/stopPropagation support
// ---------------------------------------------------------------------------

let _nextEventHandle = 1n;
const _activeEvents = new Map<bigint, Event>();

/**
 * Register an Event object for WASM-side manipulation (preventDefault/stopPropagation).
 * Called during event dispatch to provide a handle to the active event.
 */
function registerEvent(event: Event): bigint {
  const handle = _nextEventHandle++;
  _activeEvents.set(handle, event);
  return handle;
}

/**
 * Clean up an event handle after dispatch completes.
 * Called automatically after the WASM callback completes.
 */
function cleanupEventHandle(handle: bigint): void {
  _activeEvents.delete(handle);
}

// ---------------------------------------------------------------------------
// WASM callback hooks
// (To be set by the WASM host after instantiation)
// ---------------------------------------------------------------------------

type MouseCallback = (listenerId: bigint, eventHandle: bigint, data: MouseEventData) => void;
type KeyboardCallback = (listenerId: bigint, eventHandle: bigint, data: KeyboardEventData) => void;
type FocusCallback = (listenerId: bigint, eventHandle: bigint, data: FocusEventData) => void;
type InputCallback = (listenerId: bigint, eventHandle: bigint, data: InputEventData) => void;
type GenericCallback = (listenerId: bigint, eventHandle: bigint, eventType: string) => void;

let _onMouseEvent: MouseCallback | null = null;
let _onKeyboardEvent: KeyboardCallback | null = null;
let _onFocusEvent: FocusCallback | null = null;
let _onInputEvent: InputCallback | null = null;
let _onGenericEvent: GenericCallback | null = null;

/**
 * Register WASM-exported event callbacks. Call this after WASM instantiation
 * with the functions exported by the `event-callbacks` WIT interface.
 */
export function registerEventCallbacks(callbacks: {
  onMouseEvent?: MouseCallback;
  onKeyboardEvent?: KeyboardCallback;
  onFocusEvent?: FocusCallback;
  onInputEvent?: InputCallback;
  onGenericEvent?: GenericCallback;
}): void {
  _onMouseEvent = callbacks.onMouseEvent ?? null;
  _onKeyboardEvent = callbacks.onKeyboardEvent ?? null;
  _onFocusEvent = callbacks.onFocusEvent ?? null;
  _onInputEvent = callbacks.onInputEvent ?? null;
  _onGenericEvent = callbacks.onGenericEvent ?? null;
}

// ---------------------------------------------------------------------------
// Diagnostic / observability support
// ---------------------------------------------------------------------------

interface DiagnosticCallbacks {
  onError?: (error: DiagnosticError) => void;
  onWarning?: (warning: string) => void;
  onEventDispatch?: (info: EventDispatchInfo) => void;
}

export interface DiagnosticError {
  kind: "missing-callback" | "invalid-handle" | "dispatch-error" | "environment-error";
  message: string;
  context?: Record<string, unknown>;
}

export interface EventDispatchInfo {
  eventType: string;
  listenerId: bigint;
  success: boolean;
}

let _diagnosticCallbacks: DiagnosticCallbacks = {};

/**
 * Register diagnostic callbacks for observing browser-glue internals.
 * Useful for debugging and error reporting in development.
 */
export function registerDiagnosticCallbacks(callbacks: DiagnosticCallbacks): void {
  _diagnosticCallbacks = { ..._diagnosticCallbacks, ...callbacks };
}

function reportError(error: DiagnosticError): void {
  if (_diagnosticCallbacks.onError) {
    _diagnosticCallbacks.onError(error);
  }
  // Always log critical errors to console
  console.error(`[browser-glue] ${error.kind}: ${error.message}`, error.context ?? "");
}

function reportWarning(message: string): void {
  if (_diagnosticCallbacks.onWarning) {
    _diagnosticCallbacks.onWarning(message);
  }
  console.warn(`[browser-glue] ${message}`);
}

function reportEventDispatch(info: EventDispatchInfo): void {
  if (_diagnosticCallbacks.onEventDispatch) {
    _diagnosticCallbacks.onEventDispatch(info);
  }
}

// ---------------------------------------------------------------------------
// WIT interface: event-target
// ---------------------------------------------------------------------------

export function addEventListener(
  target: bigint,
  eventType: string,
  useCapture: boolean,
): bigint {
  const listenerId = _nextListenerId++;

  let domTarget: EventTarget;
  try {
    domTarget = getEventTarget(target);
  } catch (e) {
    reportError({
      kind: "invalid-handle",
      message: `Invalid node handle ${target} when adding event listener`,
      context: { eventType, listenerId, error: e instanceof Error ? e.message : String(e) },
    });
    throw e;
  }

  const handler = (ev: Event) => {
    const eventHandle = registerEvent(ev);
    let dispatchSuccess = false;

    try {
      if (ev instanceof MouseEvent) {
        if (_onMouseEvent) {
          _onMouseEvent(listenerId, eventHandle, {
            clientX: ev.clientX,
            clientY: ev.clientY,
            offsetX: ev instanceof MouseEvent ? (ev as MouseEvent).offsetX : 0,
            offsetY: ev instanceof MouseEvent ? (ev as MouseEvent).offsetY : 0,
            button: ev.button,
            buttons: ev.buttons,
            ctrlKey: ev.ctrlKey,
            shiftKey: ev.shiftKey,
            altKey: ev.altKey,
            metaKey: ev.metaKey,
          });
          dispatchSuccess = true;
        } else {
          reportWarning(`MouseEvent dispatched but no callback registered (listener: ${listenerId})`);
        }
      } else if (ev instanceof KeyboardEvent) {
        if (_onKeyboardEvent) {
          _onKeyboardEvent(listenerId, eventHandle, {
            key: ev.key,
            code: ev.code,
            keyCode: ev.keyCode,
            ctrlKey: ev.ctrlKey,
            shiftKey: ev.shiftKey,
            altKey: ev.altKey,
            metaKey: ev.metaKey,
            repeat: ev.repeat,
          });
          dispatchSuccess = true;
        } else {
          reportWarning(`KeyboardEvent dispatched but no callback registered (listener: ${listenerId})`);
        }
      } else if (ev instanceof FocusEvent) {
        if (_onFocusEvent) {
          const rel = ev.relatedTarget as Node | null;
          _onFocusEvent(listenerId, eventHandle, {
            relatedTarget: rel ? registerNode(rel) : undefined,
          });
          dispatchSuccess = true;
        } else {
          reportWarning(`FocusEvent dispatched but no callback registered (listener: ${listenerId})`);
        }
      } else if (ev instanceof InputEvent) {
        if (_onInputEvent) {
          _onInputEvent(listenerId, eventHandle, {
            data: ev.data ?? undefined,
            inputType: ev.inputType,
          });
          dispatchSuccess = true;
        } else {
          reportWarning(`InputEvent dispatched but no callback registered (listener: ${listenerId})`);
        }
      } else if (_onGenericEvent) {
        _onGenericEvent(listenerId, eventHandle, ev.type);
        dispatchSuccess = true;
      } else {
        reportWarning(`Generic event '${ev.type}' dispatched but no callback registered (listener: ${listenerId})`);
      }
    } catch (e) {
      reportError({
        kind: "dispatch-error",
        message: `Error during event dispatch for '${eventType}'`,
        context: {
          listenerId,
          eventType: ev.type,
          error: e instanceof Error ? e.message : String(e),
        },
      });
    } finally {
      // Clean up event handle after dispatch
      cleanupEventHandle(eventHandle);
      reportEventDispatch({ eventType, listenerId, success: dispatchSuccess });
    }
  };

  domTarget.addEventListener(eventType, handler, useCapture);
  _listeners.set(listenerId, { target: domTarget, eventType, handler });

  return listenerId;
}

export function removeEventListener(
  _target: bigint,
  listenerId: bigint,
): void {
  const entry = _listeners.get(listenerId);
  if (!entry) {
    reportWarning(`Attempted to remove non-existent listener ${listenerId}`);
    return;
  }
  entry.target.removeEventListener(entry.eventType, entry.handler);
  _listeners.delete(listenerId);
}

export function preventDefault(eventHandle: bigint): void {
  const event = _activeEvents.get(eventHandle);
  if (!event) {
    reportError({
      kind: "invalid-handle",
      message: `Invalid event handle ${eventHandle} in preventDefault`,
      context: { eventHandle },
    });
    return;
  }
  event.preventDefault();
}

export function stopPropagation(eventHandle: bigint): void {
  const event = _activeEvents.get(eventHandle);
  if (!event) {
    reportError({
      kind: "invalid-handle",
      message: `Invalid event handle ${eventHandle} in stopPropagation`,
      context: { eventHandle },
    });
    return;
  }
  event.stopPropagation();
}

/**
 * Check if a callback has been registered. Useful for validation.
 */
export function hasCallback(type: "mouse" | "keyboard" | "focus" | "input" | "generic"): boolean {
  switch (type) {
    case "mouse":
      return _onMouseEvent !== null;
    case "keyboard":
      return _onKeyboardEvent !== null;
    case "focus":
      return _onFocusEvent !== null;
    case "input":
      return _onInputEvent !== null;
    case "generic":
      return _onGenericEvent !== null;
  }
}

/**
 * Get current listener count for diagnostics.
 */
export function getListenerCount(): number {
  return _listeners.size;
}

/**
 * Get active event count for diagnostics.
 */
export function getActiveEventCount(): number {
  return _activeEvents.size;
}
