/**
 * Async scheduling and Promise bridging for WIT.
 *
 * Provides utilities to bridge JavaScript Promises and async operations
 * to the WIT asynchronous model, enabling cross-language async interop.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * Result of a Promise that has settled.
 */
export type PromiseResult<T> = { ok: true; value: T } | { ok: false; error: string };

/**
 * Result from an async iterator's next() method.
 */
export type IteratorResult<T> = { done: boolean; value: T | undefined };

/**
 * Async iterator interface.
 */
export interface AsyncIterator<T> {
  next(): Promise<IteratorResult<T>>;
}

/**
 * Internal state for a registered Promise.
 */
interface PromiseState<T> {
  promise: Promise<T>;
  settled: boolean;
  result?: PromiseResult<T>;
}

/**
 * Internal state for a registered async iterator.
 */
interface IteratorState<T> {
  iterator: AsyncIterator<T>;
  nextPromise: Promise<IteratorResult<T>> | null;
}

// ---------------------------------------------------------------------------
// Promise Registry
// ---------------------------------------------------------------------------

let nextRequestId = 1n;
const promises = new Map<bigint, PromiseState<unknown>>();

/**
 * Register a Promise, returning a request ID.
 * WIT can poll this ID to get the result.
 *
 * @param promise - The Promise to register
 * @returns The assigned request ID (bigint)
 */
export function registerPromise<T>(promise: Promise<T>): bigint {
  const requestId = nextRequestId++;

  const state: PromiseState<T> = {
    promise,
    settled: false,
  };

  promise
    .then((value) => {
      state.settled = true;
      state.result = { ok: true, value };
    })
    .catch((error) => {
      state.settled = true;
      const errorMessage =
        error instanceof Error
          ? error.message
          : typeof error === "string"
            ? error
            : String(error);
      state.result = { ok: false, error: errorMessage };
    });

  promises.set(requestId, state as PromiseState<unknown>);
  return requestId;
}

/**
 * Poll a Promise's result.
 *
 * @param requestId - The request ID from registerPromise
 * @returns undefined if still pending, or the result if settled
 * @throws Error if the request ID is not found
 */
export function pollPromise<T>(requestId: bigint): PromiseResult<T> | undefined {
  const state = promises.get(requestId);
  if (!state) {
    throw new Error(`Promise request ID ${requestId} not found`);
  }

  if (!state.settled) {
    return undefined;
  }

  return state.result as PromiseResult<T> | undefined;
}

// ---------------------------------------------------------------------------
// Timer Management
// ---------------------------------------------------------------------------

let nextTimerId = 1n;
const timers = new Map<bigint, ReturnType<typeof setTimeout> | ReturnType<typeof setInterval>>();

/**
 * Delay execution of a callback.
 *
 * @param callback - Function to execute after delay
 * @param delay - Delay in milliseconds
 * @returns The timer ID (bigint)
 */
export function setTimeoutAsync(callback: () => void, delay: number): bigint {
  const timerId = nextTimerId++;
  const timer = setTimeout(() => {
    try {
      callback();
    } catch (error) {
      console.error(`[async] setTimeout callback error:`, error);
    }
  }, delay);

  timers.set(timerId, timer);
  return timerId;
}

/**
 * Periodically execute a callback.
 *
 * @param callback - Function to execute at each interval
 * @param interval - Interval in milliseconds
 * @returns The timer ID (bigint)
 */
export function setIntervalAsync(callback: () => void, interval: number): bigint {
  const timerId = nextTimerId++;
  const timer = setInterval(() => {
    try {
      callback();
    } catch (error) {
      console.error(`[async] setInterval callback error:`, error);
    }
  }, interval);

  timers.set(timerId, timer);
  return timerId;
}

/**
 * Clear a timer created by setTimeoutAsync or setIntervalAsync.
 *
 * @param timerId - The timer ID to clear
 */
export function clearTimer(timerId: bigint): void {
  const timer = timers.get(timerId);
  if (!timer) {
    throw new Error(`Timer ID ${timerId} not found`);
  }

  if (typeof timer === "object" && timer !== null && "unref" in timer) {
    clearInterval(timer);
  } else {
    clearTimeout(timer);
  }

  timers.delete(timerId);
}

// ---------------------------------------------------------------------------
// Async Iterator Registry
// ---------------------------------------------------------------------------

let nextIteratorId = 1n;
const iterators = new Map<bigint, IteratorState<unknown>>();

/**
 * Register an async iterator, returning an iterator ID.
 *
 * @param iterator - The async iterator to register
 * @returns The assigned iterator ID (bigint)
 */
export function registerAsyncIterator<T>(iterator: AsyncIterator<T>): bigint {
  const iteratorId = nextIteratorId++;

  const state: IteratorState<T> = {
    iterator,
    nextPromise: null,
  };

  iterators.set(iteratorId, state as IteratorState<unknown>);
  return iteratorId;
}

/**
 * Poll for the next value from an async iterator.
 *
 * If the iterator is ready, returns the result immediately.
 * If the iterator is still fetching, returns undefined (caller should poll again).
 *
 * @param iteratorId - The iterator ID from registerAsyncIterator
 * @returns undefined if still fetching, or the IteratorResult if ready
 * @throws Error if the iterator ID is not found
 */
export function pollIterator<T>(iteratorId: bigint): IteratorResult<T> | undefined {
  const state = iterators.get(iteratorId);
  if (!state) {
    throw new Error(`Async iterator ID ${iteratorId} not found`);
  }

  if (!state.nextPromise) {
    state.nextPromise = state.iterator.next();
  }

  const result = handleIteratorPromise(state.nextPromise);
  if (result !== undefined) {
    state.nextPromise = null;
  }

  return result as IteratorResult<T> | undefined;
}

/**
 * Handle an iterator's next() promise.
 *
 * Returns undefined if the promise is still pending,
 * or the IteratorResult if it has settled.
 */
function handleIteratorPromise<T>(
  promise: Promise<IteratorResult<T>>
): IteratorResult<T> | undefined {
  let settled = false;
  let result: IteratorResult<T> | undefined;

  promise
    .then((value) => {
      settled = true;
      result = value;
    })
    .catch((error) => {
      settled = true;
      console.error(`[async] iterator.next() error:`, error);
      result = { done: true, value: undefined };
    });

  if (!settled) {
    return undefined;
  }

  return result;
}

// ---------------------------------------------------------------------------
// Statistics and Diagnostics
// ---------------------------------------------------------------------------

/**
 * Get statistics about async operations.
 */
export function getAsyncStats(): {
  pendingPromises: number;
  nextRequestId: bigint;
  activeTimers: number;
  nextTimerId: bigint;
  activeIterators: number;
  nextIteratorId: bigint;
} {
  let pendingPromises = 0;
  for (const state of promises.values()) {
    if (!state.settled) {
      pendingPromises++;
    }
  }

  return {
    pendingPromises,
    nextRequestId,
    activeTimers: timers.size,
    nextTimerId: nextTimerId,
    activeIterators: iterators.size,
    nextIteratorId,
  };
}

/**
 * Clear all async state (for testing/reset).
 */
export function clearAllAsyncState(): void {
  for (const timer of timers.values()) {
    if (typeof timer === "object" && timer !== null && "unref" in timer) {
      clearInterval(timer);
    } else {
      clearTimeout(timer);
    }
  }
  timers.clear();
  promises.clear();
  iterators.clear();
  nextRequestId = 1n;
  nextTimerId = 1n;
  nextIteratorId = 1n;
}
