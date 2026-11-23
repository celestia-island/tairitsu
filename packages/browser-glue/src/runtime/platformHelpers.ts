

import {
  geoPositionErrorValue,
  geoPositionValue,
  mutationEntryValue,
  resizeEntryValue,
} from "./callbackValues";

/** A rejection value as the required `string` the promise callbacks declare. */
function rejectionText(error: unknown): string {
  if (error instanceof Error) return error.name + ": " + error.message;
  return String(error ?? "rejected");
}

/** An IndexedDB request result as the `option<string>` the WIT declares. */
function idbResultText(result: unknown): string | undefined {
  if (typeof result === "string") return result;
  if (result === null || result === undefined) return undefined;
  return JSON.stringify(result);
}

export const platformHelpers_exports = {
  innerWidth() {
    return window.innerWidth;
  },
  innerHeight() {
    return window.innerHeight;
  },
  setTimeout(callbackId: bigint, ms: number) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.timerState;
    const id = s.nextTimeoutId++;
    const timeoutId = window.setTimeout(() => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"]
      ) {
        const exp = globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"];
        const onTimeout = exp?.onTimeout ?? exp?.on_timeout;
        if (onTimeout) onTimeout(callbackId);
      }
    }, ms);
    // W4: registered at creation (not when it fires), so a PENDING timer is
    // reclaimed by dispose and is never attributed to the wrong instance.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.timers.add(timeoutId);
    s.timeoutCallbacks.set(id, timeoutId);
    return id;
  },
  clearTimeout(id: number) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.timerState;
    if (s.timeoutCallbacks.has(id)) {
      window.clearTimeout(s.timeoutCallbacks.get(id));
      s.timeoutCallbacks.delete(id);
    }
  },
  setInterval(callbackId: bigint, ms: number) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.timerState;
    const id = s.nextTimeoutId++;
    const intervalId = window.setInterval(() => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"]
      ) {
        const exp = globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"];
        const onInterval = exp?.onInterval ?? exp?.on_interval;
        if (onInterval) onInterval(callbackId);
      }
    }, ms);
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.intervals.add(intervalId);
    s.intervalCallbacks.set(id, intervalId);
    return id;
  },
  clearInterval(id: number) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.timerState;
    if (s.intervalCallbacks.has(id)) {
      window.clearInterval(s.intervalCallbacks.get(id));
      s.intervalCallbacks.delete(id);
    }
  },
  requestAnimationFrame(callbackId: bigint) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.animState;
    const id = s.nextAnimationId++;
    const animationId = window.requestAnimationFrame((timestamp) => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"]
      ) {
        const exp = globalThis.__wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"];
        const onFrame = exp?.onFrame ?? exp?.on_frame;
        if (onFrame) onFrame(callbackId, timestamp);
      }
    });
    // W4: owned by the instance, cancelled on dispose.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.frames.add(animationId);
    s.animationCallbacks.set(id, animationId);
    return id;
  },
  cancelAnimationFrame(id: number) {
    const s = globalThis.__TAIRITSU_CURRENT__().tables.animState;
    if (s.animationCallbacks.has(id)) {
      window.cancelAnimationFrame(s.animationCallbacks.get(id) as number);
      s.animationCallbacks.delete(id);
    }
  },
  getBoundingClientRect(element: bigint) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  createResizeObserver(callbackId: bigint) {
    const observer = new ResizeObserver((entries) => {
      const guest =
        globalThis.__wasmExports?.["tairitsu-browser:full/resize-observer-callbacks@0.2.0"];
      // WIT declares `on-resize(callback-id, entries: list<resize-entry>)`; jco
      // emits `onResize(callbackId: bigint, entries: Array<ResizeEntry>)` — the
      // same identifier this read already used (one-segment kebab names
      // camelCase to themselves), so the NAME was never the bug here. The VALUE
      // was: the bridge handed over `u64` table handles for a
      // `list<resize-entry>`, which the wrapper cannot convert. Both halves are
      // fixed; the snake fallback stays for a wrapper built with that
      // convention.
      const handler = guest?.onResize ?? guest?.on_resize;
      if (handler) {
        // A throwing guest must not escape into the DOM observer loop — same
        // shape as `mediaQueryList.ts`'s `dispatchChange` (log, do not propagate):
        // one component bug would otherwise become an unhandled page error on
        // every resize of every element.
        try {
          handler(
            callbackId,
            entries.map((entry) =>
              resizeEntryValue(entry, {
                store: (handle, value) =>
                  globalThis.__resizeObserverEntryHandles.set(handle, value as never),
              }),
            ),
          );
        } catch (error) {
          console.error("[tairitsu-glue] resize-observer dispatch error:", error);
        }
      }
    });
    // W4: owned by the instance, disconnected on dispose.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.observers.add(observer);
    return globalThis.__storeElement(observer);
  },
  observeResize(observer: bigint, element: bigint) {
    const obs = globalThis.__elementHandles.get(observer);
    const el = globalThis.__elementHandles.get(element);
    if (obs && el) {
      obs.observe(el);
    }
  },
  unobserveResize(observer: bigint, element: bigint) {
    const obs = globalThis.__elementHandles.get(observer);
    const el = globalThis.__elementHandles.get(element);
    if (obs && el) {
      obs.unobserve(el);
    }
  },
  disconnectResize(observer: bigint) {
    const obs = globalThis.__elementHandles.get(observer);
    if (obs) {
      obs.disconnect();
    }
  },
  createMutationObserver(callbackId: bigint) {
    const observer = new MutationObserver((records) => {
      const guest =
        globalThis.__wasmExports?.["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"];
      // WIT declares `on-mutate` → jco emits `onMutate`. The bridge used to read
      // the WIT spelling with an underscore (`on` + `mutation`), which is not a
      // name the wrapper carries under ANY convention, so every mutation
      // notification was dropped with no error at all — a guest observer
      // callback that never ran and never logged. The argument had the same
      // problem as the resize bridge: `u64` handles for a
      // `list<mutation-entry>`.
      const handler = guest?.onMutate ?? guest?.on_mutation;
      if (handler) {
        // Same guard as the resize bridge and `mediaQueryList.ts`: a guest that
        // throws must not escape into the MutationObserver callback.
        try {
          handler(
            callbackId,
            records.map((record) =>
              mutationEntryValue(record, {
                store: (handle, value) =>
                  globalThis.__mutationRecordHandles.set(handle, value as never),
              }),
            ),
          );
        } catch (error) {
          console.error("[tairitsu-glue] mutation-observer dispatch error:", error);
        }
      }
    });
    // W4: owned by the instance, disconnected on dispose.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.observers.add(observer);
    return globalThis.__storeElement(observer);
  },
  observeMutations(observer: bigint, target: bigint, _options: any) { // ts-escape-ok: retires with L40
    const obs = globalThis.__elementHandles.get(observer);
    const el = globalThis.__elementHandles.get(target);
    if (obs && el) {
      obs.observe(el, {
        childList: true,
        attributes: true,
        characterData: true,
        subtree: true,
      });
    }
  },
  querySelector(selector: string) {
    const result = document.querySelector(selector);
    if (!result) return undefined;
    return globalThis.__storeElement(result);
  },
  querySelectorAll(selector: string) {
    const result = document.querySelectorAll(selector);
    return Array.from(result).map((el) => globalThis.__storeElement(el));
  },
  disconnectMutation(observer: bigint) {
    const obs = globalThis.__elementHandles.get(observer);
    if (obs) {
      obs.disconnect();
    }
  },

  createAudioContext() {
    try {
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)(); // ts-escape-ok: retires with L40
      return globalThis.__storeElement(ctx);
    } catch {
      return 0n;
    }
  },

  scrollTo(top: number, behavior: string) {
    window.scrollTo({ top, behavior: (behavior || "auto") as ScrollBehavior });
  },

  clipboardWriteTextPromise(text: string) {
    if (!navigator.clipboard?.writeText) return 0n;
    const id = globalThis.__nextHandle++;
    navigator.clipboard
      .writeText(text)
      .then(() => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseResolved ?? guest?.on_promise_resolved;
          // `on-promise-resolved(promise-id, value: string)` — the value is
          // REQUIRED, and `navigator.clipboard.writeText` resolves with nothing,
          // so the honest value is the empty string (not `undefined`, which is
          // not representable for a required `string`).
          if (handler) handler(id, "");
        }
      })
      .catch((error) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseRejected ?? guest?.on_promise_rejected;
          // `on-promise-rejected(promise-id, error: string)` — `error` is
          // required, so it is reported, not omitted.
          if (handler) handler(id, rejectionText(error));
        }
      });
    return id;
  },

  clipboardReadTextPromise() {
    if (!navigator.clipboard?.readText) return 0n;
    const id = globalThis.__nextHandle++;
    navigator.clipboard
      .readText()
      .then((text) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseResolved ?? guest?.on_promise_resolved;
          if (handler) handler(id, text);
        }
      })
      .catch((error) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseRejected ?? guest?.on_promise_rejected;
          if (handler) handler(id, rejectionText(error));
        }
      });
    return id;
  },

  fetchPromise(url: string, options: string | null) {
    const id = globalThis.__nextHandle++;
    let opts: RequestInit | undefined;
    if (options) {
      try {
        opts = JSON.parse(options);
      } catch (error) {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseRejected ?? guest?.on_promise_rejected;
          if (handler) handler(id, rejectionText(error));
        }
        return id;
      }
    }
    fetch(url, opts)
      .then((resp) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseResolved ?? guest?.on_promise_resolved;
          // `value` is a required `string`; a `Response` is not one. The bridge
          // reports what it can say synchronously — the status line — and the
          // guest reads the body through `fetch` itself, which is where the
          // content belongs (`_handwritten.wit` declares no response record).
          if (handler) handler(id, resp.status + " " + resp.statusText);
        }
      })
      .catch((error) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/promise-callbacks@0.2.0"];
          const handler = guest?.onPromiseRejected ?? guest?.on_promise_rejected;
          if (handler) handler(id, rejectionText(error));
        }
      });
    return id;
  },

  getGeolocationHandle() {
    return 0n;
  },

  getCurrentPosition(
    _geoHandle: bigint,
    successCallbackId: bigint,
    errorCallbackId: bigint,
    enableHighAccuracy: boolean,
    timeout: number,
    maximumAge: number,
  ) {
    if (!navigator.geolocation) return;
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/geolocation-callbacks@0.2.0"];
          const handler = guest?.onPositionSuccess ?? guest?.on_position_success;
          // `on-position-success(callback-id, position: geo-position)` — the
          // WHOLE record, not two loose numbers. Built inside the guard so the
          // record is only materialised when a guest is there to take it.
          if (handler) handler(successCallbackId, geoPositionValue(pos));
        }
      },
      (err) => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/geolocation-callbacks@0.2.0"];
          const handler = guest?.onPositionError ?? guest?.on_position_error;
          if (handler) handler(errorCallbackId, geoPositionErrorValue(err));
        }
      },
      { enableHighAccuracy, timeout, maximumAge },
    );
  },

  copyToClipboard(text: string) {
    if (navigator.clipboard?.writeText) {
      navigator.clipboard.writeText(text).catch(() => {});
      return true;
    }
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    try {
      document.execCommand("copy");
      return true;
    } catch {
      return false;
    } finally {
      document.body.removeChild(ta);
    }
  },

  readClipboard() {
    if (navigator.clipboard) {
      try {
        return (navigator.clipboard as any).readTextSync?.() ?? undefined; // ts-escape-ok: retires with L40
      } catch {}
    }
    return undefined;
  },

  setContentEditable(element: bigint, editable: boolean) {
    const el = globalThis.__elementHandles.get(element);
    if (el) el.contentEditable = editable ? "true" : "false";
  },

  getElementById(id: string) {
    const el = document.getElementById(id);
    if (!el) return undefined;
    return globalThis.__storeElement(el);
  },

  onScroll(callbackId: bigint) {
    const g = globalThis as any; // ts-escape-ok: retires with L40
    // W4: the scroll-listener table belongs to the current instance.
    const handler = () => {
      const guest = g.__wasmExports?.["tairitsu-browser:full/scroll-callbacks@0.2.0"];
      // WIT declares `on-scroll-event(callback-id, scroll-x, scroll-y)` → jco
      // emits `onScrollEvent` taking TWO coordinates. The old read passed
      // `on_scroll` (a name the wrapper never has) with three derived numbers
      // (top / height / client height) — neither the name nor the arity matched.
      const onScrollEvent = guest?.onScrollEvent ?? guest?.on_scroll;
      if (onScrollEvent) {
        const scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
        const scrollLeft = document.documentElement.scrollLeft || document.body.scrollLeft;
        onScrollEvent(callbackId, scrollLeft, scrollTop);
      }
    };
    window.addEventListener("scroll", handler, { passive: true });
    // W4: owned by the instance, removed on dispose.
    if (g.__TAIRITSU_INSTANCES__) {
      g.__TAIRITSU_INSTANCES__.current().reclaimables.listeners.add({
        target: window,
        type: "scroll",
        handler,
        options: { passive: true },
      });
    }
    const handle = g.__nextHandle++;
    g.__tairitsuScrollListeners.set(handle, handler);
    return handle;
  },

  onResizeCallback(callbackId: bigint) {
    const g = globalThis as any; // ts-escape-ok: retires with L40
    if (!g.__tairitsuResizeListeners) {
      g.__tairitsuResizeListeners = new Map();
    }
    // `on-resize-callback` is DECLARED and IMPLEMENTED (`onResizeCallback`), so a
    // guest can call it and expect its window-resize notification. The bridge
    // used to read `…/resize-callbacks@0.2.0"]?.on_resize` — an interface the
    // world does not have and a property the wrapper does not carry, so the call
    // went nowhere. The real interface is `window-resize-callbacks` and jco
    // emits `onWindowResize(callbackId, width, height)`.
    const handler = () => {
      const guest = g.__wasmExports?.["tairitsu-browser:full/window-resize-callbacks@0.2.0"];
      const onWindowResize = guest?.onWindowResize ?? guest?.on_window_resize;
      if (onWindowResize) onWindowResize(callbackId, window.innerWidth, window.innerHeight);
    };
    window.addEventListener("resize", handler);
    // W4: owned by the instance, removed on dispose.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.listeners.add({
      target: window,
      type: "resize",
      handler,
      options: undefined,
    });
    const handle = g.__nextHandle++;
    g.__tairitsuResizeListeners.set(handle, handler);
    return handle;
  },

  getScrollTopFromPoint(x: number, y: number) {
    return document.elementFromPoint(x, y)?.scrollTop ?? 0;
  },

  getScrollTopBySelector(selector: string) {
    const el = document.querySelector(selector);
    return el?.scrollTop ?? 0;
  },

  getElementRectById(id: string) {
    const el = document.getElementById(id);
    if (!el) return undefined;
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },

  getBoundingRectByClass(className: string, element: bigint) {
    const parent = globalThis.__elementHandles?.get(element) || document;
    const el = parent.getElementsByClassName(className)[0];
    if (!el) return undefined;
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },

  prefersDarkMode() {
    return window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
  },

  getContenteditableState(element: bigint) {
    const el = globalThis.__elementHandles.get(element);
    if (!el || !el.isContentEditable) return undefined;
    return {
      html: el.innerHTML,
      text: el.innerText,
      isEditable: true,
      selectionStart: el.selectionStart,
      selectionEnd: el.selectionEnd,
    };
  },

  getSelectionStart(element: bigint) {
    const el = globalThis.__elementHandles.get(element);
    if (!el || typeof el.selectionStart !== "number") return undefined;
    return el.selectionStart;
  },

  getSelectionEnd(element: bigint) {
    const el = globalThis.__elementHandles.get(element);
    if (!el || typeof el.selectionEnd !== "number") return undefined;
    return el.selectionEnd;
  },

  analyserGetFrequencyData(analyser: bigint) {
    const node = globalThis.__elementHandles.get(analyser);
    if (!node || !node.frequencyBinCount) return [];
    const data = new Uint8Array(node.frequencyBinCount);
    node.getByteFrequencyData(data);
    return Array.from(data);
  },

  analyserGetTimeDomainData(analyser: bigint) {
    const node = globalThis.__elementHandles.get(analyser);
    if (!node || !node.frequencyBinCount) return [];
    const data = new Uint8Array(node.frequencyBinCount);
    node.getByteTimeDomainData(data);
    return Array.from(data);
  },

  drawQrcodeOnCanvasById(
    canvasId: string,
    matrix: boolean[][],
    modules: bigint,
    color: string,
    background: string,
  ) {
    const canvasEl =
      globalThis.__elementHandles.get(BigInt(canvasId)) || document.getElementById(canvasId);
    if (!canvasEl || !(canvasEl as HTMLCanvasElement).getContext) return false;
    const ctx = (canvasEl as HTMLCanvasElement).getContext("2d");
    if (!ctx) return false;
    const cellSize = canvasEl.width / Number(modules);
    ctx.fillStyle = background || "#ffffff";
    ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);
    ctx.fillStyle = color || "#000000";
    for (let r = 0; r < matrix.length; r++) {
      for (let c = 0; c < matrix[r].length; c++) {
        if (matrix[r][c]) ctx.fillRect(c * cellSize, r * cellSize, cellSize, cellSize);
      }
    }
    return true;
  },

  fileReaderSyncReadAsText(blob: bigint, encoding: string | null, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderLoad ?? guest?.on_file_reader_load;
        if (handler) handler(callbackId, reader.result || "");
      }
    };
    reader.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderError ?? guest?.on_file_reader_error;
        if (handler) handler(callbackId, reader.error?.message || "Read failed");
      }
    };
    reader.readAsText(b, encoding || "utf-8");
  },

  fileReaderSyncReadAsArrayBuffer(blob: bigint, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderLoad ?? guest?.on_file_reader_load;
        if (handler) handler(
          callbackId,
          reader.result ? Array.from(new Uint8Array(reader.result as ArrayBuffer)) : [],
        );
      }
    };
    reader.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderError ?? guest?.on_file_reader_error;
        if (handler) handler(callbackId, reader.error?.message || "Read failed");
      }
    };
    reader.readAsArrayBuffer(b);
  },

  fileReaderReadAsText(blob: bigint, encoding: string | null, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderLoad ?? guest?.on_file_reader_load;
        if (handler) handler(callbackId, reader.result || "");
      }
    };
    reader.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderError ?? guest?.on_file_reader_error;
        if (handler) handler(callbackId, reader.error?.message || "Read failed");
      }
    };
    reader.readAsText(b, encoding || "utf-8");
  },

  fileReaderReadAsArrayBuffer(blob: bigint, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderLoad ?? guest?.on_file_reader_load;
        if (handler) handler(
          callbackId,
          reader.result ? Array.from(new Uint8Array(reader.result as ArrayBuffer)) : [],
        );
      }
    };
    reader.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/file-reader-callbacks@0.2.0"];
        const handler = guest?.onFileReaderError ?? guest?.on_file_reader_error;
        if (handler) handler(callbackId, reader.error?.message || "Read failed");
      }
    };
    reader.readAsArrayBuffer(b);
  },

  idbOpen(name: string, version: bigint | null, callbackId: bigint) {
    const req = indexedDB.open(
      name,
      version !== undefined && version !== null ? Number(version) : undefined,
    );
    req.onsuccess = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
        // `on-idb-request-success(callback-id, result: option<string>)` —
        // the second argument is a STRING or `undefined`, never a handle.
        if (handler) handler(callbackId, "open:" + String(req.result?.version ?? ""));
      }
    };
    req.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          req.error?.message || "Open failed",
        );
      }
    };
    return 0n;
  },

  idbPut(db: bigint, storeName: string, value: string, key: string | null, callbackId: bigint) {
    const database = globalThis.__elementHandles.get(db);
    if (!database) return;
    try {
      const tx = database.transaction(storeName, "readwrite");
      const store = tx.objectStore(storeName);
      const req = key !== undefined && key !== null ? store.put(value, key) : store.put(value);
      req.onsuccess = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
          // A `put` request carries no value: `undefined` is the honest result.
          if (handler) handler(callbackId, undefined);
        }
      };
      req.onerror = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
          if (handler) handler(
            callbackId,
            req.error?.message || "Put failed",
          );
        }
      };
    } catch (e: any) { // ts-escape-ok: retires with L40
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          e.message,
        );
      }
    }
  },

  idbGet(db: bigint, storeName: string, key: string, callbackId: bigint) {
    const database = globalThis.__elementHandles.get(db);
    if (!database) return;
    try {
      const tx = database.transaction(storeName, "readonly");
      const store = tx.objectStore(storeName);
      const req = store.get(key);
      req.onsuccess = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
          // `option<string>`: a missing key is `undefined`, not `""`.
          if (handler) handler(callbackId, idbResultText(req.result));
        }
      };
      req.onerror = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
          if (handler) handler(
            callbackId,
            req.error?.message || "Get failed",
          );
        }
      };
    } catch (e: any) { // ts-escape-ok: retires with L40
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          e.message,
        );
      }
    }
  },

  idbDelete(db: bigint, storeName: string, key: string, callbackId: bigint) {
    const database = globalThis.__elementHandles.get(db);
    if (!database) return;
    try {
      const tx = database.transaction(storeName, "readwrite");
      const store = tx.objectStore(storeName);
      const req = store.delete(key);
      req.onsuccess = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
          if (handler) handler(callbackId, undefined);
        }
      };
      req.onerror = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
          if (handler) handler(
            callbackId,
            req.error?.message || "Delete failed",
          );
        }
      };
    } catch (e: any) { // ts-escape-ok: retires with L40
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          e.message,
        );
      }
    }
  },

  idbGetAll(db: bigint, storeName: string, callbackId: bigint) {
    const database = globalThis.__elementHandles.get(db);
    if (!database) return;
    try {
      const tx = database.transaction(storeName, "readonly");
      const store = tx.objectStore(storeName);
      const req = store.getAll();
      req.onsuccess = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
          // The WIT value is ONE string; a list arrives JSON-encoded.
          if (handler) handler(callbackId, JSON.stringify(req.result ?? []));
        }
      };
      req.onerror = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
          if (handler) handler(
            callbackId,
            req.error?.message || "GetAll failed",
          );
        }
      };
    } catch (e: any) { // ts-escape-ok: retires with L40
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          e.message,
        );
      }
    }
  },

  idbClear(db: bigint, storeName: string, callbackId: bigint) {
    const database = globalThis.__elementHandles.get(db);
    if (!database) return;
    try {
      const tx = database.transaction(storeName, "readwrite");
      const store = tx.objectStore(storeName);
      const req = store.clear();
      req.onsuccess = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestSuccess ?? guest?.on_idb_request_success;
          if (handler) handler(callbackId, undefined);
        }
      };
      req.onerror = () => {
        if (
          globalThis.__wasmExports &&
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
        ) {
          const guest =
            globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
          const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
          if (handler) handler(
            callbackId,
            req.error?.message || "Clear failed",
          );
        }
      };
    } catch (e: any) { // ts-escape-ok: retires with L40
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/idb-callbacks@0.2.0"];
        const handler = guest?.onIdbRequestError ?? guest?.on_idb_request_error;
        if (handler) handler(
          callbackId,
          e.message,
        );
      }
    }
  },

  connectWebSocket(url: string) {
    const ws = new WebSocket(url);
    // W4: owned by the instance, closed on dispose.
    globalThis.__TAIRITSU_CURRENT__?.().reclaimables.sockets.add(ws);
    const handle = globalThis.__storeElement(ws);

    const s = globalThis.__TAIRITSU_CURRENT__().tables.wsState;
    const openCbId = s.nextCallbackId++;
    const msgCbId = s.nextCallbackId++;
    const closeCbId = s.nextCallbackId++;
    const errCbId = s.nextCallbackId++;

    const g = globalThis as any; // ts-escape-ok: retires with L40
    if (!g.__tairitsuWsCallbacks) {
      g.__tairitsuWsCallbacks = new Map();
    }
    g.__tairitsuWsCallbacks.set(handle, { openCbId, msgCbId, closeCbId, errCbId });

    ws.onopen = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/web-socket-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/web-socket-callbacks@0.2.0"];
        const handler = guest?.onWebSocketOpen ?? guest?.on_web_socket_open;
        if (handler) handler(openCbId);
      }
    };
    ws.onmessage = (ev: MessageEvent) => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/web-socket-callbacks@0.2.0"]
      ) {
        const data = typeof ev.data === "string" ? ev.data : JSON.stringify(ev.data);
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/web-socket-callbacks@0.2.0"];
        const handler = guest?.onWebSocketMessage ?? guest?.on_web_socket_message;
        if (handler) handler(msgCbId, data);
      }
    };
    ws.onclose = (ev: CloseEvent) => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/web-socket-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/web-socket-callbacks@0.2.0"];
        const handler = guest?.onWebSocketClose ?? guest?.on_web_socket_close;
        if (handler) handler(closeCbId, ev.code, ev.reason || "");
      }
      if (g.__tairitsuWsCallbacks) {
        g.__tairitsuWsCallbacks.delete(handle);
      }
    };
    ws.onerror = () => {
      if (
        globalThis.__wasmExports &&
        globalThis.__wasmExports["tairitsu-browser:full/web-socket-callbacks@0.2.0"]
      ) {
        const guest =
          globalThis.__wasmExports?.["tairitsu-browser:full/web-socket-callbacks@0.2.0"];
        const handler = guest?.onWebSocketError ?? guest?.on_web_socket_error;
        if (handler) handler(errCbId);
      }
    };

    return handle;
  },
};
