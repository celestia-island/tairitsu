// @ts-nocheck
import { storeElement } from "./helpers";
import { wasmExports } from "./wasmExports";

let _timeoutCallbacks = new Map();
let _nextTimeoutId = 1;
let _animationCallbacks = new Map();
let _nextAnimationId = 1;

export const platformHelpers_exports = {
  innerWidth() {
    return window.innerWidth;
  },
  innerHeight() {
    return window.innerHeight;
  },
  setTimeout(callbackId, ms) {
    const id = _nextTimeoutId++;
    const timeoutId = setTimeout(() => {
      if (wasmExports && wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"]) {
        wasmExports["tairitsu-browser:full/timer-callbacks@0.2.0"].on_timeout(callbackId);
      }
    }, ms);
    _timeoutCallbacks.set(id, timeoutId);
    return id;
  },
  clearTimeout(id) {
    if (_timeoutCallbacks.has(id)) {
      clearTimeout(_timeoutCallbacks.get(id));
      _timeoutCallbacks.delete(id);
    }
  },
  requestAnimationFrame(callbackId) {
    const id = _nextAnimationId++;
    const animationId = requestAnimationFrame((timestamp) => {
      if (wasmExports && wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"]) {
        wasmExports["tairitsu-browser:full/animation-callbacks@0.2.0"].on_animation_frame(callbackId, timestamp);
      }
    });
    _animationCallbacks.set(id, animationId);
    return id;
  },
  cancelAnimationFrame(id) {
    if (_animationCallbacks.has(id)) {
      cancelAnimationFrame(_animationCallbacks.get(id));
      _animationCallbacks.delete(id);
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
      if (wasmExports && wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"]) {
        const entryHandles = entries.map(entry => {
          if (!globalThis.__resizeObserverEntryHandles) {
            globalThis.__resizeObserverEntryHandles = new Map();
            globalThis.__nextResizeObserverEntry = 1n;
          }
          const handle = globalThis.__nextResizeObserverEntry++;
          globalThis.__resizeObserverEntryHandles.set(handle, entry);
          return handle;
        });
        wasmExports["tairitsu-browser:full/resize-observer-callbacks@0.2.0"].on_resize(callbackId, entryHandles);
      }
    });
    return storeElement(observer);
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
      if (wasmExports && wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"]) {
        const recordHandles = records.map(record => {
          if (!globalThis.__mutationRecordHandles) {
            globalThis.__mutationRecordHandles = new Map();
            globalThis.__nextMutationRecord = 1n;
          }
          const handle = globalThis.__nextMutationRecord++;
          globalThis.__mutationRecordHandles.set(handle, record);
          return handle;
        });
        wasmExports["tairitsu-browser:full/mutation-observer-callbacks@0.2.0"].on_mutation(callbackId, recordHandles);
      }
    });
    return storeElement(observer);
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
  disconnectMutation(observer) {
    const obs = globalThis.__elementHandles.get(observer);
    if (obs) {
      obs.disconnect();
    }
  },
};
