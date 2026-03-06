/**
 * Events glue — implements the `tairitsu-browser:events` WIT import interfaces.
 *
 * Manages event listener registration and dispatches serialised event data
 * back to WASM guest callbacks.
 *
 * Status: Phase 0 stubs — listener registration wired to real DOM.
 */

import { registerExternalNode } from "./dom-glue.js";

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
// WASM callback hooks
// (To be set by the WASM host after instantiation)
// ---------------------------------------------------------------------------

type MouseCallback = (listenerId: bigint, data: MouseEventData) => void;
type KeyboardCallback = (listenerId: bigint, data: KeyboardEventData) => void;
type FocusCallback = (listenerId: bigint, data: FocusEventData) => void;
type InputCallback = (listenerId: bigint, data: InputEventData) => void;
type GenericCallback = (listenerId: bigint, eventType: string) => void;

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
// WIT interface: event-target
// ---------------------------------------------------------------------------

function getNode(handle: bigint): EventTarget {
  // Re-use the handle table from dom-glue by querying the DOM directly.
  // NOTE: The `__tairitsu_node_*` pattern is also used in canvas-glue.ts.
  // In Phase 3 both modules will share a single exported handle table to
  // eliminate this coupling.
  const el = (globalThis as Record<string, unknown>)[`__tairitsu_node_${handle}`] as EventTarget | undefined;
  if (el) return el;
  throw new Error(`Event target handle ${handle} not found`);
}

export function addEventListener(
  target: bigint,
  eventType: string,
  _useCapture: boolean,
): bigint {
  const listenerId = _nextListenerId++;

  let domTarget: EventTarget;
  try {
    domTarget = getNode(target);
  } catch {
    // Fallback: use document as target for now (Phase 3 will fix this).
    domTarget = document;
  }

  const handler = (ev: Event) => {
    if (ev instanceof MouseEvent && _onMouseEvent) {
      _onMouseEvent(listenerId, {
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
    } else if (ev instanceof KeyboardEvent && _onKeyboardEvent) {
      _onKeyboardEvent(listenerId, {
        key: ev.key,
        code: ev.code,
        keyCode: ev.keyCode,
        ctrlKey: ev.ctrlKey,
        shiftKey: ev.shiftKey,
        altKey: ev.altKey,
        metaKey: ev.metaKey,
        repeat: ev.repeat,
      });
    } else if (ev instanceof FocusEvent && _onFocusEvent) {
      const rel = ev.relatedTarget as Node | null;
      _onFocusEvent(listenerId, {
        relatedTarget: rel ? registerExternalNode(rel) : undefined,
      });
    } else if (ev instanceof InputEvent && _onInputEvent) {
      _onInputEvent(listenerId, {
        data: ev.data ?? undefined,
        inputType: ev.inputType,
      });
    } else if (_onGenericEvent) {
      _onGenericEvent(listenerId, ev.type);
    }
  };

  domTarget.addEventListener(eventType, handler, _useCapture);
  _listeners.set(listenerId, { target: domTarget, eventType, handler });

  return listenerId;
}

export function removeEventListener(
  _target: bigint,
  listenerId: bigint,
): void {
  const entry = _listeners.get(listenerId);
  if (!entry) return;
  entry.target.removeEventListener(entry.eventType, entry.handler);
  _listeners.delete(listenerId);
}

export function preventDefault(_event: bigint): void {
  // preventDefault is called from WASM during an event handler; the actual
  // Event object reference is managed by the host dispatch loop.
  // Phase 3 will pass the Event reference through a separate handle table.
}

export function stopPropagation(_event: bigint): void {
  // Same as above — managed by host dispatch loop in Phase 3.
}
