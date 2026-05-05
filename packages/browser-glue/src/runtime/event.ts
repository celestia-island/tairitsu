// @ts-nocheck

export const event_exports = {
  getCurrentTarget(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !ev.currentTarget) return undefined;
    for (const [h, el] of globalThis.__elementHandles) {
      if (el === ev.currentTarget) return h;
    }
    return undefined;
  },

  getTarget(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !ev.target) return undefined;
    for (const [h, el] of globalThis.__elementHandles) {
      if (el === ev.target) return h;
    }
    return undefined;
  },

  getEventType(eventHandle: bigint): string {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    return ev ? ev.type : "";
  },

  getSrcElement(eventHandle: bigint): bigint | undefined {
    const ev = globalThis.__eventHandles?.get(eventHandle);
    if (!ev || !(ev as any).srcElement) return undefined;
    for (const [h, el] of globalThis.__elementHandles) {
      if (el === (ev as any).srcElement) return h;
    }
    return undefined;
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
    return ev.composedPath()
      .map((t: EventTarget) => {
        for (const [h, el] of globalThis.__elementHandles) {
          if (el === t) return h;
        }
        return undefined;
      })
      .filter((h: bigint | undefined): h is bigint => h !== undefined);
  },
};
