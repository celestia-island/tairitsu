/**
 * `EventTarget` → element-handle reverse index.
 *
 * Two glue call sites need the same answer — "given a DOM node, which handle does
 * the guest already have for it?" — and both are hot paths:
 *
 *  - `event.ts` (`currentTarget`, `target`, `srcElement`, `composedPath`), on
 *    every dispatched event;
 *  - `mouseEvent.ts` (`relatedTarget`), on every `mouseover`/`mouseout`/
 *    `mouseenter`/`mouseleave` — i.e. on every element-boundary crossing.
 *
 * A linear walk of `__elementHandles` per call is O(elements-ever-stored) per
 * event, so the index below is O(1) after a one-time build. It is built from the
 * *live* table (not just future stores), so handles minted before this module was
 * imported are indexed too, and `__storeElement` is wrapped so later handles keep
 * it current.
 */

/** Narrowing helper — a `Node` is the only thing with an element handle. */
function isNode(value: EventTarget | null): value is Node {
  return value !== null && "nodeType" in value;
}

/** Build (once) and return the reverse index. */
function ensureIndex(): Map<EventTarget, bigint> {
  const existing = globalThis.__elementReverseMap;
  if (existing) return existing;

  // The table is normally created by `./handles` + `./helpers`, which the runtime
  // entry imports for their side effects. A consumer that imports this module (or
  // `registry.ts`) directly — tests, a hand-assembled page — has no such entry, and
  // a missing table must read as "no handles yet", not throw mid-event.
  const handles = (globalThis.__elementHandles ??= new Map());
  const index = new Map<EventTarget, bigint>();
  for (const [handle, element] of handles) {
    index.set(element, handle);
  }
  globalThis.__elementReverseMap = index;

  // Same reasoning for the store hook: without `./helpers` there is none yet, and
  // wrapping a missing function would turn every later store into a TypeError.
  const store = globalThis.__storeElement;
  if (typeof store !== "function") return index;
  globalThis.__storeElement = (obj) => {
    const handle = store(obj);
    if (handle !== undefined && obj) index.set(obj, handle);
    return handle;
  };
  return index;
}

/**
 * The handle a target already has, or `undefined` — never mints one. This is what
 * the event glue has always returned for a target outside the handle tables.
 */
export function existingElementHandle(
  target: EventTarget | null,
): bigint | undefined {
  if (!target) return undefined;
  return ensureIndex().get(target);
}

/**
 * Like {@link existingElementHandle}, but registers a `Node` that has no handle
 * yet instead of dropping it, so a guest reading `relatedTarget` always gets a
 * usable handle. Non-`Node` event targets (`window`) have no element handle in
 * this runtime and still resolve to `undefined`.
 */
export function elementHandleFor(
  target: EventTarget | null,
): bigint | undefined {
  const known = existingElementHandle(target);
  if (known !== undefined) return known;
  if (!isNode(target)) return undefined;
  // No `./helpers` in this realm => nothing can mint a handle; resolve to none
  // rather than throwing inside an event handler.
  if (typeof globalThis.__storeElement !== "function") return undefined;
  return globalThis.__storeElement(target);
}
