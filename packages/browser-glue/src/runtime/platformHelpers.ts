// @ts-nocheck

// State stored on globalThis so that when generateModuleCode() stringifies
// these functions into a blob module, the state is still accessible.
// (Blob modules have their own scope — closure variables from this file are
// invisible inside the blob.)
if (!globalThis.__tairitsuTimerState) {
  globalThis.__tairitsuTimerState = {
    timeoutCallbacks: new Map(),
    nextTimeoutId: 1,
  };
}
if (!globalThis.__tairitsuAnimState) {
  globalThis.__tairitsuAnimState = {
    animationCallbacks: new Map(),
    nextAnimationId: 1,
  };
}

export const platformHelpers_exports = {
  innerWidth() {
    return window.innerWidth;
  },
  innerHeight() {
    return window.innerHeight;
  },
  setTimeout(callbackId, ms) {
    const s = globalThis.__tairitsuTimerState;
    const id = s.nextTimeoutId++;
    const timeoutId = window.setTimeout(() => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"]) {
        const exp = globalThis.__wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"];
        (exp.onTimeout || exp.on_timeout)?.(callbackId);
      }
    }, ms);
    s.timeoutCallbacks.set(id, timeoutId);
    return id;
  },
  clearTimeout(id) {
    const s = globalThis.__tairitsuTimerState;
    if (s.timeoutCallbacks.has(id)) {
      window.clearTimeout(s.timeoutCallbacks.get(id));
      s.timeoutCallbacks.delete(id);
    }
  },
  requestAnimationFrame(callbackId) {
    const s = globalThis.__tairitsuAnimState;
    const id = s.nextAnimationId++;
    const animationId = window.requestAnimationFrame((timestamp) => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"]) {
        const exp = globalThis.__wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"];
        (exp.onFrame || exp.on_animation_frame)?.(callbackId, timestamp);
      }
    });
    s.animationCallbacks.set(id, animationId);
    return id;
  },
  cancelAnimationFrame(id) {
    const s = globalThis.__tairitsuAnimState;
    if (s.animationCallbacks.has(id)) {
      window.cancelAnimationFrame(s.animationCallbacks.get(id));
      s.animationCallbacks.delete(id);
    }
  },
  getBoundingClientRect(element) {
    const el = globalThis.__elementHandles.get(element);
    if (!el) {
      return { x: 0, y: 0, width: 0, height: 0 };
    }
    const rect = el.getBoundingClientRect();
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
  },
  createResizeObserver(callbackId) {
    const observer = new ResizeObserver((entries) => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"]) {
        const entryHandles = entries.map(entry => {
          if (!globalThis.__resizeObserverEntryHandles) {
            globalThis.__resizeObserverEntryHandles = new Map();
            globalThis.__nextResizeObserverEntry = 1n;
          }
          const handle = globalThis.__nextResizeObserverEntry++;
          globalThis.__resizeObserverEntryHandles.set(handle, entry);
          return handle;
        });
        globalThis.__wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"]?.on_resize?.(callbackId, entryHandles);
      }
    });
    return globalThis.__storeElement(observer);
  },
  observeResize(observer, element) {
    const obs = globalThis.__elementHandles.get(observer);
    const el = globalThis.__elementHandles.get(element);
    if (obs && el) {
      obs.observe(el);
    }
  },
  unobserveResize(observer, element) {
    const obs = globalThis.__elementHandles.get(observer);
    const el = globalThis.__elementHandles.get(element);
    if (obs && el) {
      obs.unobserve(el);
    }
  },
  disconnectResize(observer) {
    const obs = globalThis.__elementHandles.get(observer);
    if (obs) {
      obs.disconnect();
    }
  },
  createMutationObserver(callbackId) {
    const observer = new MutationObserver((records) => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"]) {
        const recordHandles = records.map(record => {
          if (!globalThis.__mutationRecordHandles) {
            globalThis.__mutationRecordHandles = new Map();
            globalThis.__nextMutationRecord = 1n;
          }
          const handle = globalThis.__nextMutationRecord++;
          globalThis.__mutationRecordHandles.set(handle, record);
          return handle;
        });
        globalThis.__wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"]?.on_mutation?.(callbackId, recordHandles);
      }
    });
    return globalThis.__storeElement(observer);
  },
  observeMutations(observer, target, _options) {
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
  disconnectMutation(observer) {
    const obs = globalThis.__elementHandles.get(observer);
    if (obs) {
      obs.disconnect();
    }
  },

  createAudioContext() {
    try {
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      return globalThis.__storeElement(ctx);
    } catch {
      return 0n;
    }
  },

  scrollTo(top: number, behavior: string) {
    window.scrollTo({ top, behavior: behavior || "auto" });
  },

  clipboardWriteTextPromise(text: string) {
    if (!navigator.clipboard?.writeText) return 0n;
    const id = globalThis.__nextHandle ?? 1;
    if (globalThis.__nextHandle !== undefined) globalThis.__nextHandle++;
    navigator.clipboard.writeText(text).then(() => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_resolve?.(id);
      }
    }).catch(() => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_reject?.(id);
      }
    });
    return id;
  },

  clipboardReadTextPromise() {
    if (!navigator.clipboard?.readText) return 0n;
    const id = globalThis.__nextHandle ?? 1;
    if (globalThis.__nextHandle !== undefined) globalThis.__nextHandle++;
    navigator.clipboard.readText().then((text) => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_resolve?.(id, text);
      }
    }).catch(() => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_reject?.(id);
      }
    });
    return id;
  },

  fetchPromise(url: string, options: string | null) {
    const id = globalThis.__nextHandle ?? 1;
    if (globalThis.__nextHandle !== undefined) globalThis.__nextHandle++;
    const opts = options ? JSON.parse(options) : undefined;
    fetch(url, opts).then((resp) => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_resolve?.(id, resp);
      }
    }).catch(() => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/promise-callbacks@0.2.0"]?.on_promise_reject?.(id);
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]?.on_geolocation_success?.(
            successCallbackId,
            pos.coords.latitude,
            pos.coords.longitude,
          );
        }
      },
      (err) => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/geolocation-callbacks@0.2.0"]?.on_geolocation_error?.(
            errorCallbackId,
            err.code,
            err.message,
          );
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
    if (navigator.clipboard?.readText) {
      try {
        return (navigator.clipboard as any).readTextSync?.() ?? undefined;
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
    const handler = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/scroll-callbacks@0.2.0"]) {
        const scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
        const scrollHeight = document.documentElement.scrollHeight || document.body.scrollHeight;
        const clientHeight = document.documentElement.clientHeight || document.body.clientHeight;
        globalThis.__wasmExports["tairitsu-browser:full/scroll-callbacks@0.2.0"]?.on_scroll?.(
          callbackId,
          scrollTop,
          scrollHeight,
          clientHeight,
        );
      }
    };
    window.addEventListener("scroll", handler, { passive: true });
    return 0;
  },

  onResizeCallback(callbackId: bigint) {
    const handler = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/resize-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/resize-callbacks@0.2.0"]?.on_resize?.(callbackId);
      }
    };
    window.addEventListener("resize", handler);
    return 0;
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
    const canvas = document.getElementById(canvasId);
    if (!canvas || !(canvas as HTMLCanvasElement).getContext) return false;
    const ctx = (canvas as HTMLCanvasElement).getContext("2d");
    if (!ctx) return false;
    const cellSize = canvas.width / Number(modules);
    ctx.fillStyle = background || "#ffffff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = color || "#000000";
    for (let r = 0; r < matrix.length; r++) {
      for (let c = 0; c < matrix[r].length; c++) {
        if (matrix[r][c]) ctx.fillRect(c * cellSize, r * cellSize, cellSize, cellSize);
      }
    }
    return true;
  },

  fileReaderSyncReadAsText(blob: bigint, encoding: string | null) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return { tag: "err", val: "Invalid blob handle" };
    try {
      const reader = new FileReader();
      reader.readAsText(b, encoding || "utf-8");
      return { tag: "ok", val: reader.result || "" };
    } catch (e: any) {
      return { tag: "err", val: e.message || "Read failed" };
    }
  },

  fileReaderSyncReadAsArrayBuffer(blob: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return { tag: "err", val: "Invalid blob handle" };
    try {
      const reader = new FileReader();
      reader.readAsArrayBuffer(b);
      return reader.result
        ? { tag: "ok", val: Array.from(new Uint8Array(reader.result)) }
        : { tag: "err", val: "No result" };
    } catch (e: any) {
      return { tag: "err", val: e.message || "Read failed" };
    }
  },

  fileReaderReadAsText(blob: bigint, encoding: string | null, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]?.on_file_reader_result?.(callbackId, reader.result || "");
      }
    };
    reader.onerror = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]?.on_file_reader_error?.(
          callbackId,
          reader.error?.message || "Read failed",
        );
      }
    };
    reader.readAsText(b, encoding || "utf-8");
  },

  fileReaderReadAsArrayBuffer(blob: bigint, callbackId: bigint) {
    const b = globalThis.__elementHandles.get(blob);
    if (!b) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]?.on_file_reader_result?.(
          callbackId,
          reader.result ? Array.from(new Uint8Array(reader.result)) : [],
        );
      }
    };
    reader.onerror = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/file-reader-callbacks@0.2.0"]?.on_file_reader_error?.(
          callbackId,
          reader.error?.message || "Read failed",
        );
      }
    };
    reader.readAsArrayBuffer(b);
  },

  idbOpen(name: string, version: bigint | null, callbackId: bigint) {
    const req = indexedDB.open(name, version !== undefined && version !== null ? Number(version) : undefined);
    req.onsuccess = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_open?.(
          callbackId,
          globalThis.__storeElement?.(req.result) ?? 0n,
        );
      }
    };
    req.onerror = () => {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_success?.(callbackId);
        }
      };
      req.onerror = () => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, req.error?.message || "Put failed");
        }
      };
    } catch (e: any) {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, e.message);
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_success?.(callbackId, req.result);
        }
      };
      req.onerror = () => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, req.error?.message || "Get failed");
        }
      };
    } catch (e: any) {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, e.message);
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_success?.(callbackId);
        }
      };
      req.onerror = () => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, req.error?.message || "Delete failed");
        }
      };
    } catch (e: any) {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, e.message);
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_success?.(callbackId, req.result || []);
        }
      };
      req.onerror = () => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, req.error?.message || "GetAll failed");
        }
      };
    } catch (e: any) {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, e.message);
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
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_success?.(callbackId);
        }
      };
      req.onerror = () => {
        if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
          globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, req.error?.message || "Clear failed");
        }
      };
    } catch (e: any) {
      if (globalThis.__wasmExports && globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]) {
        globalThis.__wasmExports["tairitsu-browser:full/idb-callbacks@0.2.0"]?.on_idb_error?.(callbackId, e.message);
      }
    }
  },
};
