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
  /**
   * `momentum` attribute — WebKit-only. Safari synthesises a follow-up wheel
   * event for inertial scrolling and flags it here; every other engine leaves
   * it undefined, which reads as "not momentum" (the honest default: a wheel
   * event that cannot be identified is treated as a real user gesture).
   */
  getMomentum(self: bigint): boolean {
    const momentum = (eventOf(self) as { momentum?: boolean }).momentum;
    return momentum === true;
  },
};
