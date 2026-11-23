export const window_exports = {
  getInnerWidth() {
    return window.innerWidth;
  },
  getInnerHeight() {
    return window.innerHeight;
  },
  getComputedStyle(elt: bigint, pseudoElt?: string): bigint {
    const el = (globalThis as any).__lookupElement(elt); // ts-escape-ok: retires with L40
    const result = window.getComputedStyle(el, pseudoElt);
    const handle = (globalThis as any).__nextCssStyleDeclarationHandle++ as unknown as bigint; // ts-escape-ok: retires with L40
    (globalThis as any).__cssStyleDeclarationHandles.set(handle, result); // ts-escape-ok: retires with L40
    return handle;
  },
  // `matchMedia()` — the WIT `window.match-media`. The returned handle names a
  // live `MediaQueryList` for the current instance; every op that consumes it
  // lives in `mediaQueryList.ts`.
  //
  // This is not a convenience wrapper: a guest that reads
  // `(prefers-reduced-motion: reduce)` through it decides whether to suppress
  // animation at all. Before this op existed the import had no host
  // implementation, so `hikari`'s AnimationProvider read `false` on every page
  // and honoured the user's reduced-motion setting only by accident.
  matchMedia(query: string): bigint {
    const list = window.matchMedia(query);
    const handle = globalThis.__nextMediaQueryListHandle++;
    globalThis.__mediaQueryListHandles.set(handle, list);
    return handle;
  },
};
