function reverseLookupElement(target: EventTarget | null): bigint | undefined {
  if (!target) return undefined;
  const g = globalThis as any;
  if (!g.__elementReverseMap) {
    g.__elementReverseMap = new Map();
    const orig = g.__storeElement;
    if (orig) {
      g.__storeElement = function (el: any) {
        const handle = orig(el);
        if (handle !== undefined) {
          g.__elementReverseMap.set(el, handle);
        }
        return handle;
      };
    }
  }
  return g.__elementReverseMap.get(target);
}

export const event_exports = {
  getCurrentTarget(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !ev.currentTarget) return undefined;
    return reverseLookupElement(ev.currentTarget);
  },

  getTarget(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !ev.target) return undefined;
    return reverseLookupElement(ev.target);
  },

  getEventType(eventHandle: bigint): string {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.type : "";
  },

  getSrcElement(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !(ev as any).srcElement) return undefined;
    return reverseLookupElement((ev as any).srcElement);
  },

  getEventPhase(eventHandle: bigint): number {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.eventPhase : 0;
  },

  eventStopPropagation(eventHandle: bigint): void {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (ev) ev.stopPropagation();
  },

  stopImmediatePropagation(eventHandle: bigint): void {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (ev) ev.stopImmediatePropagation();
  },

  getBubbles(eventHandle: bigint): boolean {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.bubbles : false;
  },

  getCancelable(eventHandle: bigint): boolean {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.cancelable : false;
  },

  getDefaultPrevented(eventHandle: bigint): boolean {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.defaultPrevented : false;
  },

  getTimeStamp(eventHandle: bigint): number {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.timeStamp : 0;
  },

  getIsTrusted(eventHandle: bigint): boolean {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.isTrusted : false;
  },

  getCancelBubble(eventHandle: bigint): boolean {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? !!(ev as any).cancelBubble : false;
  },

  setCancelBubble(eventHandle: bigint, value: boolean): void {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (ev) (ev as any).cancelBubble = value;
  },

  composedPath(eventHandle: bigint): bigint[] {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev) return [];
    return ev
      .composedPath()
      .map((t: EventTarget) => reverseLookupElement(t))
      .filter((h: bigint | undefined): h is bigint => h !== undefined);
  },
};
