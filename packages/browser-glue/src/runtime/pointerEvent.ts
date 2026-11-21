/**
 * `tairitsu-browser:full/pointer-event` glue.
 *
 * Same story as `mouse-event.ts`: the interface is part of the import set of any
 * component built against `tairitsu-web`, and the runtime registry wired none of
 * it. Handles come from the shared `__eventHandles` table (`eventTarget.ts`).
 *
 * List-valued getters (`coalesced-events`, `predicted-events`) register each
 * sub-event in the same table and return their handles, so the guest can read
 * them with the ordinary pointer-event getters.
 */

/** The host-side DOM event behind a `pointer-event-handle`. */
function eventOf(self: bigint): Partial<PointerEvent> {
  return globalThis.__eventHandles?.get(self) ?? {};
}

/**
 * Handles minted for coalesced / predicted sub-events, keyed by the sub-event
 * object itself.
 *
 * Without this, every `getCoalescedEvents()` call re-registered the same
 * sub-events, so a pointer-drawing app polling them added `len(coalesced)`
 * entries to `__eventHandles` per call, forever — multiplying the pre-existing
 * (never-deleted) per-dispatch growth. One handle per sub-event object instead.
 */
const subEventHandles = new WeakMap<Event, bigint>();

/** Register a sub-event once and return its stable handle. */
function storeEvent(event: Event): bigint {
  const cached = subEventHandles.get(event);
  if (cached !== undefined) return cached;
  const handle = globalThis.__nextEventHandle++;
  globalThis.__eventHandles.set(handle, event);
  subEventHandles.set(event, handle);
  return handle;
}

export const pointerEvent_exports = {
  getPointerId(self: bigint): number {
    return Number(eventOf(self).pointerId ?? 0);
  },
  getWidth(self: bigint): number {
    return Number(eventOf(self).width ?? 0);
  },
  getHeight(self: bigint): number {
    return Number(eventOf(self).height ?? 0);
  },
  getPressure(self: bigint): number {
    return Number(eventOf(self).pressure ?? 0);
  },
  getTangentialPressure(self: bigint): number {
    return Number(eventOf(self).tangentialPressure ?? 0);
  },
  getTiltX(self: bigint): number {
    return Number(eventOf(self).tiltX ?? 0);
  },
  getTiltY(self: bigint): number {
    return Number(eventOf(self).tiltY ?? 0);
  },
  getTwist(self: bigint): number {
    return Number(eventOf(self).twist ?? 0);
  },
  getAltitudeAngle(self: bigint): number {
    return Number(eventOf(self).altitudeAngle ?? 0);
  },
  getAzimuthAngle(self: bigint): number {
    return Number(eventOf(self).azimuthAngle ?? 0);
  },
  getPointerType(self: bigint): string {
    return String(eventOf(self).pointerType ?? "");
  },
  getIsPrimary(self: bigint): boolean {
    return Boolean(eventOf(self).isPrimary);
  },
  getPersistentDeviceId(self: bigint): number {
    return Number(eventOf(self).persistentDeviceId ?? 0);
  },
  getCoalescedEvents(self: bigint): bigint[] {
    const ev = eventOf(self);
    if (typeof ev.getCoalescedEvents !== "function") return [];
    return ev.getCoalescedEvents().map(storeEvent);
  },
  getPredictedEvents(self: bigint): bigint[] {
    const ev = eventOf(self);
    if (typeof ev.getPredictedEvents !== "function") return [];
    return ev.getPredictedEvents().map(storeEvent);
  },
};
