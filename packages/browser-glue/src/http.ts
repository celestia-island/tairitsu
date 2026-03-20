/**
 * Fetch glue — implements the `tairitsu-browser:fetch` WIT import interfaces.
 *
 * Browser runtimes cannot execute network requests synchronously, so
 * `fetchSync` returns an explicit error and async polling is the primary path.
 */

// ---------------------------------------------------------------------------
// Types matching WIT records
// ---------------------------------------------------------------------------

export type HttpMethod =
  | "get"
  | "post"
  | "put"
  | "patch"
  | "delete"
  | "head"
  | "options";

export interface HeaderEntry {
  name: string;
  value: string;
}

export type RequestBody =
  | { tag: "none" }
  | { tag: "text"; value: string }
  | { tag: "bytes"; value: Uint8Array };

export interface RequestDescriptor {
  url: string;
  method: HttpMethod;
  headers: HeaderEntry[];
  body: RequestBody;
}

export interface ResponseData {
  status: number;
  statusText: string;
  headers: HeaderEntry[];
  body: Uint8Array;
  ok: boolean;
}

// ---------------------------------------------------------------------------
// Async handle table
// ---------------------------------------------------------------------------

let _nextFetchHandle = 1n;

interface FetchHandle {
  promise: Promise<ResponseData>;
  result: { ok: true; value: ResponseData } | { ok: false; error: string } | null;
}

const _fetchHandles = new Map<bigint, FetchHandle>();

// ---------------------------------------------------------------------------
// WIT interface: fetch-api
// ---------------------------------------------------------------------------

/**
 * fetchSync is not supported in browsers.
 * This function always throws — use `fetchAsync` / `pollFetch` instead.
 */
export function fetchSync(_request: RequestDescriptor): never {
  throw new Error(
    "fetchSync is not supported in the browser environment. " +
    "Use fetchAsync + pollFetch instead.",
  );
}

// ---------------------------------------------------------------------------
// WIT interface: async-fetch
// ---------------------------------------------------------------------------

function buildInit(request: RequestDescriptor): RequestInit {
  const method = request.method.toUpperCase();
  const headers: Record<string, string> = {};
  for (const { name, value } of request.headers) {
    headers[name] = value;
  }
  let body: BodyInit | undefined;
  if (request.body.tag === "text") {
    body = request.body.value;
  } else if (request.body.tag === "bytes") {
    body = request.body.value as BodyInit;
  }
  return { method, headers, body };
}

export function fetchAsync(request: RequestDescriptor): bigint {
  const handle = _nextFetchHandle++;

  const promise = fetch(request.url, buildInit(request)).then(
    async (resp) => {
      const respHeaders: HeaderEntry[] = [];
      resp.headers.forEach((value, name) => {
        respHeaders.push({ name, value });
      });
      const buf = await resp.arrayBuffer();
      const data: ResponseData = {
        status: resp.status,
        statusText: resp.statusText,
        headers: respHeaders,
        body: new Uint8Array(buf),
        ok: resp.ok,
      };
      const entry = _fetchHandles.get(handle);
      if (entry) entry.result = { ok: true, value: data };
      return data;
    },
  ).catch((err: Error | DOMException | TypeError) => {
    const entry = _fetchHandles.get(handle);
    if (entry) {
      entry.result = {
        ok: false,
        error: err instanceof Error ? err.message : String(err),
      };
    }
    throw err;
  });

  _fetchHandles.set(handle, { promise, result: null });
  return handle;
}

export function pollFetch(
  handle: bigint,
): ({ ok: true; value: ResponseData } | { ok: false; error: string }) | undefined {
  const entry = _fetchHandles.get(handle);
  if (!entry) return { ok: false, error: `Unknown fetch handle ${handle}` };
  if (entry.result === null) return undefined; // still in flight
  return entry.result;
}

export function cancelFetch(handle: bigint): void {
  // This removes the handle entry; the browser Fetch API itself does not
  // currently expose an AbortController from this path.
  _fetchHandles.delete(handle);
}

// ---------------------------------------------------------------------------
// Request/Response interceptors
// ---------------------------------------------------------------------------

/**
 * Interface for request/response interceptor functions.
 */
export interface RequestInterceptor {
  /**
   * Called before the request is sent.
   * Can modify the request descriptor or throw to abort the request.
   *
   * @param request - Original request descriptor
   * @returns Modified request descriptor or the same one
   */
  onRequest?: (request: RequestDescriptor) => RequestDescriptor | Promise<RequestDescriptor>;

  /**
   * Called after the response is received.
   * Can modify the response data or throw to handle errors.
   *
   * @param response - Response data from the fetch
   * @returns Modified response data or the same one
   */
  onResponse?: (response: ResponseData) => ResponseData | Promise<ResponseData>;

  /**
   * Called when the request fails.
   * Can handle errors or throw to propagate them.
   *
   * @param error - Error that occurred during the fetch
   * @returns Transformed error or the same one
   */
  onError?: (error: string) => string | Promise<string>;
}

const _interceptors: RequestInterceptor[] = [];

/**
 * Add a request/response interceptor to the fetch pipeline.
 *
 * @param interceptor - Interceptor with optional onRequest, onResponse, and onError handlers
 * @returns Function to remove the interceptor
 */
export function addRequestInterceptor(interceptor: RequestInterceptor): () => void {
  _interceptors.push(interceptor);
  return () => {
    const index = _interceptors.indexOf(interceptor);
    if (index > -1) {
      _interceptors.splice(index, 1);
    }
  };
}

/**
 * Apply request interceptors to the request descriptor.
 */
async function applyRequestInterceptors(request: RequestDescriptor): Promise<RequestDescriptor> {
  let currentRequest = request;
  for (const interceptor of _interceptors) {
    if (interceptor.onRequest) {
      try {
        currentRequest = await interceptor.onRequest(currentRequest);
      } catch (e) {
        throw new Error(`Request interceptor failed: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
  }
  return currentRequest;
}

/**
 * Apply response interceptors to the response data.
 */
async function applyResponseInterceptors(response: ResponseData): Promise<ResponseData> {
  let currentResponse = response;
  for (const interceptor of _interceptors) {
    if (interceptor.onResponse) {
      try {
        currentResponse = await interceptor.onResponse(currentResponse);
      } catch (e) {
        throw new Error(`Response interceptor failed: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
  }
  return currentResponse;
}

/**
 * Apply error interceptors to the error message.
 */
async function applyErrorInterceptors(error: string): Promise<string> {
  let currentError = error;
  for (const interceptor of _interceptors) {
    if (interceptor.onError) {
      try {
        currentError = await interceptor.onError(currentError);
      } catch (e) {
        return `Error interceptor failed: ${e instanceof Error ? e.message : String(e)}`;
      }
    }
  }
  return currentError;
}

// Override fetchAsync to use interceptors
const originalFetchAsync = fetchAsync;

export function fetchAsyncWithInterceptors(request: RequestDescriptor): bigint {
  const handle = _nextFetchHandle++;

  const promise = (async () => {
    try {
      const interceptedRequest = await applyRequestInterceptors(request);
      const response = await fetch(interceptedRequest.url, buildInit(interceptedRequest));

      const respHeaders: HeaderEntry[] = [];
      response.headers.forEach((value, name) => {
        respHeaders.push({ name, value });
      });
      const buf = await response.arrayBuffer();

      let responseData: ResponseData = {
        status: response.status,
        statusText: response.statusText,
        headers: respHeaders,
        body: new Uint8Array(buf),
        ok: response.ok,
      };

      responseData = await applyResponseInterceptors(responseData);

      const entry = _fetchHandles.get(handle);
      if (entry) entry.result = { ok: true, value: responseData };
      return responseData;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      const interceptedError = await applyErrorInterceptors(errorMessage);
      const entry = _fetchHandles.get(handle);
      if (entry) {
        entry.result = {
          ok: false,
          error: interceptedError,
        };
      }
      throw new Error(interceptedError);
    }
  })();

  _fetchHandles.set(handle, { promise, result: null });
  return handle;
}

// ---------------------------------------------------------------------------
// Response caching
// ---------------------------------------------------------------------------

/**
 * Configuration for response caching.
 */
export interface ResponseCache {
  /**
   * Time to live for cached responses in milliseconds.
   */
  ttl: number;

  /**
   * Maximum number of entries to store in the cache.
   */
  maxSize: number;

  /**
   * Optional function to determine if a response should be cached.
   * Defaults to caching only successful responses (status 200-299).
   */
  shouldCache?: (response: ResponseData) => boolean;
}

/**
 * Cache entry with timestamp and data.
 */
interface CacheEntry {
  timestamp: number;
  data: ResponseData;
}

let _cacheEnabled = false;
let _cacheConfig: ResponseCache = { ttl: 60000, maxSize: 100 };
const _responseCache = new Map<string, CacheEntry>();

/**
 * Generate a cache key from a request descriptor.
 *
 * @param request - Request descriptor
 * @returns Cache key string
 */
export function getCacheKey(request: RequestDescriptor): string {
  const method = request.method.toUpperCase();
  const headersStr = request.headers
    .map(h => `${h.name.toLowerCase()}:${h.value}`)
    .sort()
    .join("|");

  let bodyStr = "";
  if (request.body.tag === "text") {
    bodyStr = request.body.value;
  } else if (request.body.tag === "bytes") {
    bodyStr = Array.from(request.body.value).join(",");
  }

  return `${method}:${request.url}:${headersStr}:${bodyStr}`;
}

/**
 * Enable response caching with the specified configuration.
 *
 * @param config - Cache configuration
 */
export function enableResponseCache(config: Partial<ResponseCache> = {}): void {
  _cacheEnabled = true;
  _cacheConfig = { ..._cacheConfig, ...config };
}

/**
 * Disable response caching and clear all cached entries.
 */
export function disableResponseCache(): void {
  _cacheEnabled = false;
  _responseCache.clear();
}

/**
 * Clear all cached entries regardless of cache enabled state.
 */
export function clearResponseCache(): void {
  _responseCache.clear();
}

/**
 * Get a cached response if available and not expired.
 *
 * @param request - Request descriptor
 * @returns Cached response data or undefined
 */
function getCachedResponse(request: RequestDescriptor): ResponseData | undefined {
  if (!_cacheEnabled) return undefined;

  const key = getCacheKey(request);
  const entry = _responseCache.get(key);

  if (!entry) return undefined;

  const now = Date.now();
  if (now - entry.timestamp > _cacheConfig.ttl) {
    _responseCache.delete(key);
    return undefined;
  }

  return entry.data;
}

/**
 * Store a response in the cache.
 *
 * @param request - Request descriptor
 * @param response - Response data to cache
 */
function setCachedResponse(request: RequestDescriptor, response: ResponseData): void {
  if (!_cacheEnabled) return;

  const shouldCache = _cacheConfig.shouldCache?.(response) ?? response.ok;
  if (!shouldCache) return;

  const key = getCacheKey(request);

  if (_responseCache.size >= _cacheConfig.maxSize && !_responseCache.has(key)) {
    const oldestKey = _responseCache.keys().next().value;
    if (oldestKey) {
      _responseCache.delete(oldestKey);
    }
  }

  _responseCache.set(key, {
    timestamp: Date.now(),
    data: response,
  });
}

// Override fetchAsyncWithInterceptors to use caching
const originalFetchAsyncWithInterceptors = fetchAsyncWithInterceptors;

export function fetchAsyncWithCache(request: RequestDescriptor): bigint {
  const handle = _nextFetchHandle++;

  const promise = (async () => {
    try {
      const cached = getCachedResponse(request);
      if (cached) {
        const entry = _fetchHandles.get(handle);
        if (entry) entry.result = { ok: true, value: cached };
        return cached;
      }

      const interceptedRequest = await applyRequestInterceptors(request);
      const response = await fetch(interceptedRequest.url, buildInit(interceptedRequest));

      const respHeaders: HeaderEntry[] = [];
      response.headers.forEach((value, name) => {
        respHeaders.push({ name, value });
      });
      const buf = await response.arrayBuffer();

      let responseData: ResponseData = {
        status: response.status,
        statusText: response.statusText,
        headers: respHeaders,
        body: new Uint8Array(buf),
        ok: response.ok,
      };

      responseData = await applyResponseInterceptors(responseData);
      setCachedResponse(request, responseData);

      const entry = _fetchHandles.get(handle);
      if (entry) entry.result = { ok: true, value: responseData };
      return responseData;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      const interceptedError = await applyErrorInterceptors(errorMessage);
      const entry = _fetchHandles.get(handle);
      if (entry) {
        entry.result = {
          ok: false,
          error: interceptedError,
        };
      }
      throw new Error(interceptedError);
    }
  })();

  _fetchHandles.set(handle, { promise, result: null });
  return handle;
}

// ---------------------------------------------------------------------------
// Retry logic with exponential backoff
// ---------------------------------------------------------------------------

/**
 * Configuration for fetch retry behavior.
 */
export interface RetryConfig {
  /**
   * Maximum number of retry attempts.
   */
  maxRetries: number;

  /**
   * Initial delay in milliseconds.
   */
  initialDelay: number;

  /**
   * Multiplier for exponential backoff.
   */
  backoffMultiplier: number;

  /**
   * Maximum delay between retries in milliseconds.
   */
  maxDelay: number;

  /**
   * Function to determine if a response should trigger a retry.
   * Defaults to retrying on network errors and 5xx status codes.
   */
  shouldRetry?: (error: string | null, status?: number) => boolean;

  /**
   * Optional function to be called before each retry attempt.
   */
  onRetry?: (attempt: number, error: string | null, status?: number) => void;
}

/**
 * Default retry configuration.
 */
const defaultRetryConfig: RetryConfig = {
  maxRetries: 3,
  initialDelay: 1000,
  backoffMultiplier: 2,
  maxDelay: 30000,
};

/**
 * Create a delay promise that resolves after the specified milliseconds.
 */
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Calculate the delay for a given retry attempt using exponential backoff.
 *
 * @param attempt - Retry attempt number (0-based)
 * @param config - Retry configuration
 * @returns Delay in milliseconds
 */
function calculateRetryDelay(attempt: number, config: RetryConfig): number {
  const delay = config.initialDelay * Math.pow(config.backoffMultiplier, attempt);
  return Math.min(delay, config.maxDelay);
}

/**
 * Determine if a fetch should be retried based on error or status code.
 *
 * @param error - Error message (null if no error)
 * @param status - HTTP status code (undefined if error occurred)
 * @returns True if the request should be retried
 */
function defaultShouldRetry(error: string | null, status?: number): boolean {
  if (error !== null) {
    return true;
  }
  if (status !== undefined && status >= 500 && status < 600) {
    return true;
  }
  if (status === 429) {
    return true;
  }
  return false;
}

/**
 * Fetch with retry logic and exponential backoff.
 *
 * @param request - Request descriptor
 * @param config - Retry configuration
 * @returns Fetch handle for polling
 */
export function fetchWithRetry(
  request: RequestDescriptor,
  config: Partial<RetryConfig> = {},
): bigint {
  const handle = _nextFetchHandle++;

  const retryConfig: RetryConfig = { ...defaultRetryConfig, ...config };
  const shouldRetry = retryConfig.shouldRetry ?? defaultShouldRetry;

  const promise = (async () => {
    let lastError: string | null = null;
    let lastStatus: number | undefined;

    for (let attempt = 0; attempt <= retryConfig.maxRetries; attempt++) {
      try {
        if (attempt > 0) {
          const retryDelay = calculateRetryDelay(attempt - 1, retryConfig);
          retryConfig.onRetry?.(attempt, lastError, lastStatus);
          await delay(retryDelay);
        }

        const interceptedRequest = await applyRequestInterceptors(request);
        const response = await fetch(interceptedRequest.url, buildInit(interceptedRequest));

        const respHeaders: HeaderEntry[] = [];
        response.headers.forEach((value, name) => {
          respHeaders.push({ name, value });
        });
        const buf = await response.arrayBuffer();

        let responseData: ResponseData = {
          status: response.status,
          statusText: response.statusText,
          headers: respHeaders,
          body: new Uint8Array(buf),
          ok: response.ok,
        };

        lastStatus = response.status;

        if (shouldRetry(null, response.status) && attempt < retryConfig.maxRetries) {
          continue;
        }

        responseData = await applyResponseInterceptors(responseData);
        setCachedResponse(request, responseData);

        const entry = _fetchHandles.get(handle);
        if (entry) entry.result = { ok: true, value: responseData };
        return responseData;
      } catch (err: unknown) {
        lastError = err instanceof Error ? err.message : String(err);

        if (shouldRetry(lastError) && attempt < retryConfig.maxRetries) {
          continue;
        }

        const interceptedError = await applyErrorInterceptors(lastError);
        const entry = _fetchHandles.get(handle);
        if (entry) {
          entry.result = {
            ok: false,
            error: interceptedError,
          };
        }
        throw new Error(interceptedError);
      }
    }

    const interceptedError = await applyErrorInterceptors(
      lastError ?? "Max retries exceeded without success",
    );
    const entry = _fetchHandles.get(handle);
    if (entry) {
      entry.result = {
        ok: false,
        error: interceptedError,
      };
    }
    throw new Error(interceptedError);
  })();

  _fetchHandles.set(handle, { promise, result: null });
  return handle;
}
