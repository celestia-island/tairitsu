/**
 * Fetch glue — implements the `tairitsu-browser:fetch` WIT import interfaces.
 *
 * Status: Phase 0 — synchronous fetch is not possible in the browser;
 * `fetchSync` always rejects. Async fetch is stubbed via the handle table.
 * Full async implementation is planned for Phase 2.
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
  ).catch((err: unknown) => {
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
  // AbortController support planned for Phase 2.
  _fetchHandles.delete(handle);
}
