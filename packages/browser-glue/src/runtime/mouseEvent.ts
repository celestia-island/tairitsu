/**
 * `tairitsu-browser:full/mouse-event` glue.
 *
 * Components built against `tairitsu-web` import this interface (the vdom event
 * bridge hands the guest an event handle for mouse events), but the runtime
 * registry never wired it — the jco wrapper therefore destructured `undefined`
 * and instantiation died before any rendering happened.
 *
 * Handles are entries of the shared `__eventHandles` table, i.e. the same handles
 * `event-target.addEventListener` hands to `on-mouse-event` callbacks
 * (`eventTarget.ts`), so pointer/wheel/keydown handles are accepted here as well:
 * `PointerEvent` and `WheelEvent` are both `MouseEvent` subclasses at runtime.
 *
 * `relatedTarget` resolves through the shared O(1) reverse index
 * (`elementReverse.ts`) rather than a per-event scan of `__elementHandles`:
 * mouseover/mouseout/mouseenter/mouseleave fire on every element-boundary
 * crossing, so this is a hot path.
 */

import { elementHandleFor } from "./elementReverse";

/** The host-side DOM event behind a `mouse-event-handle`. */
function eventOf(self: bigint): Partial<MouseEvent> {
  return globalThis.__eventHandles?.get(self) ?? {};
}

export const mouseEvent_exports = {
  getScreenX(self: bigint): number {
    return Number(eventOf(self).screenX ?? 0);
  },
  getScreenY(self: bigint): number {
    return Number(eventOf(self).screenY ?? 0);
  },
  getPageX(self: bigint): number {
    return Number(eventOf(self).pageX ?? 0);
  },
  getPageY(self: bigint): number {
    return Number(eventOf(self).pageY ?? 0);
  },
  getClientX(self: bigint): number {
    return Number(eventOf(self).clientX ?? 0);
  },
  getClientY(self: bigint): number {
    return Number(eventOf(self).clientY ?? 0);
  },
  getX(self: bigint): number {
    return Number(eventOf(self).x ?? 0);
  },
  getY(self: bigint): number {
    return Number(eventOf(self).y ?? 0);
  },
  getOffsetX(self: bigint): number {
    return Number(eventOf(self).offsetX ?? 0);
  },
  getOffsetY(self: bigint): number {
    return Number(eventOf(self).offsetY ?? 0);
  },
  getLayerX(self: bigint): number {
    return Number(eventOf(self).layerX ?? 0);
  },
  getLayerY(self: bigint): number {
    return Number(eventOf(self).layerY ?? 0);
  },
  getCtrlKey(self: bigint): boolean {
    return Boolean(eventOf(self).ctrlKey);
  },
  getShiftKey(self: bigint): boolean {
    return Boolean(eventOf(self).shiftKey);
  },
  getAltKey(self: bigint): boolean {
    return Boolean(eventOf(self).altKey);
  },
  getMetaKey(self: bigint): boolean {
    return Boolean(eventOf(self).metaKey);
  },
  getButton(self: bigint): number {
    return Number(eventOf(self).button ?? 0);
  },
  getButtons(self: bigint): number {
    return Number(eventOf(self).buttons ?? 0);
  },
  getRelatedTarget(self: bigint): bigint | undefined {
    return elementHandleFor(eventOf(self).relatedTarget ?? null);
  },
  getModifierState(self: bigint, keyArg: string): boolean {
    const ev = eventOf(self);
    if (typeof ev.getModifierState !== "function") return false;
    return Boolean(ev.getModifierState(keyArg));
  },
  // NOTE: `init-mouse-event()` is intentionally not wired — see the module
  // header and `test/registry-completeness.test.ts` (it needs a Window handle
  // for its `view` argument and is deprecated in the DOM).
};
