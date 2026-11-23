// `media-query-list` — host side of the WIT interface of the same name.
//
// The list objects are created by `window.match-media` (see `window.ts`) and
// live in the per-instance `__mediaQueryListHandles` table, exactly like every
// other handle in this runtime (W4: a destroyed instance's handles stop
// resolving).
//
// `add-listener` / `remove-listener` are the deprecated `MediaQueryListListener`
// shape WIT declares: `callback` is an opaque id the guest mints, and the host
// answers through the guest's `media-query-list-callbacks` world export
// (`on-change(callback-id, matches)`). That bridge is the same one the resize /
// mutation observers use in `platformHelpers.ts`, so a component that changed
// its media preference is told — it is not expected to poll `matches`.
//
// `onchange` (getter/setter, `u64`) is NOT event-handler machinery: WIT models
// both `add-listener` and `onchange` as callbacks, and the guest's own
// `wit_platform.rs` keeps them in the same callback registry. `set-onchange`
// with `0` therefore clears the slot; a non-zero id is registered as the list's
// single change callback.

/** Thrown when a handle does not name a live MediaQueryList of this instance. */
export function requireMediaQueryList(handle: bigint): MediaQueryList {
  const list = globalThis.__mediaQueryListHandles?.get(handle);
  if (!list) throw new Error("MediaQueryList handle " + handle + " not found");
  return list;
}

/** The guest's `media-query-list-callbacks` export, when the guest provides it. */
function callbacks() {
  return globalThis.__wasmExports?.["tairitsu-browser:full/media-query-list-callbacks@0.2.0"];
}

/**
 * Report a change to one guest callback id.
 *
 * A throwing guest callback must not escape into the DOM event loop: that would
 * turn one component bug into an unhandled page error on every media flip. It
 * is logged instead, matching the event bridge in `eventTarget.ts`.
 */
function dispatchChange(callbackId: bigint, matches: boolean): void {
  const guest = callbacks();
  if (!guest?.on_change) return;
  try {
    guest.on_change(callbackId, matches);
  } catch (error) {
    console.error("[tairitsu-glue] media-query-list change dispatch error:", error);
  }
}

export const mediaQueryList_exports = {
  getMedia(self: bigint): string {
    return requireMediaQueryList(self).media;
  },

  getMatches(self: bigint): boolean {
    return requireMediaQueryList(self).matches;
  },

  addListener(self: bigint, callback?: bigint): void {
    const list = requireMediaQueryList(self);
    if (callback === undefined) return;
    const callbackId = callback;
    const handler = (event: MediaQueryListEvent) => {
      // The event object is the truth for THIS transition; re-reading
      // `list.matches` here could already describe a later flip.
      dispatchChange(callbackId, event.matches);
    };
    list.addEventListener("change", handler);
    let listeners = globalThis.__mediaQueryListListeners?.get(self);
    if (!listeners) {
      listeners = new Map();
      globalThis.__mediaQueryListListeners?.set(self, listeners);
    }
    listeners.set(callbackId, handler);
    // W4: owned by the instance, removed on dispose (instance.ts reclaims
    // `listeners` by walking this table too — see `dispose`).
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.listeners.add({
      target: list,
      type: "change",
      handler: handler as unknown as EventListener,
      options: undefined,
    });
  },

  removeListener(self: bigint, callback?: bigint): void {
    const list = requireMediaQueryList(self);
    if (callback === undefined) return;
    const listeners = globalThis.__mediaQueryListListeners?.get(self);
    const handler = listeners?.get(callback);
    if (!handler) return;
    list.removeEventListener("change", handler);
    listeners?.delete(callback);
  },

  getOnchange(self: bigint): bigint {
    return globalThis.__mediaQueryListOnchange?.get(self) ?? 0n;
  },

  setOnchange(self: bigint, value: bigint): void {
    const list = requireMediaQueryList(self);
    const previous = globalThis.__mediaQueryListOnchange?.get(self);
    if (previous !== undefined && previous !== 0n) {
      mediaQueryList_exports.removeListener(self, previous);
    }
    if (value === 0n) {
      globalThis.__mediaQueryListOnchange?.delete(self);
      return;
    }
    mediaQueryList_exports.addListener(self, value);
    globalThis.__mediaQueryListOnchange?.set(self, value);
  },
};
