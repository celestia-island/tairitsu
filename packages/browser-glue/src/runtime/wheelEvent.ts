/**
 * `tairitsu-browser:full/wheel-event` glue.
 *
 * The third of the event interfaces a `tairitsu-web` component imports
 * unconditionally; unwired in the runtime registry until now, so instantiation
 * of any browser component failed on the missing import object.
 *
 * Handles are shared `__eventHandles` entries (`eventTarget.ts`).
 */

/** The host-side DOM event behind a `wheel-event-handle`. */
function eventOf(self: bigint): Partial<WheelEvent> {
  return globalThis.__eventHandles?.get(self) ?? {};
}

export const wheelEvent_exports = {
  getDeltaX(self: bigint): number {
    return Number(eventOf(self).deltaX ?? 0);
  },
  getDeltaY(self: bigint): number {
    return Number(eventOf(self).deltaY ?? 0);
  },
  getDeltaZ(self: bigint): number {
    return Number(eventOf(self).deltaZ ?? 0);
  },
  getDeltaMode(self: bigint): number {
    return Number(eventOf(self).deltaMode ?? 0);
  },
};
