/**
 * Panel transport helpers — the shared webui request/vocabulary layer.
 *
 * Migrated from shittim-chest's `api/client.ts` (2026-10-22 user ruling:
 * the family gets one request base instead of per-panel hand-rolled
 * fetch/.upload/error-policy code). App-specific policy — token injection,
 * 401 → auth-lost routing, the actual transport binding (Tauri vs browser) —
 * stays in each panel behind the {@link PanelTransport} port; this module
 * owns only the vocabulary that every panel shares.
 *
 * Scope:
 *   - {@link ApiError} — one error class for non-2xx after retry.
 *   - {@link configureErrorHandler} / the silent-on-error contract — the
 *     central toast hook, so stores stop toasting in every catch.
 *   - {@link requestJson} / {@link upload} / {@link download} — the three
 *     call shapes every panel uses, over an injected transport.
 *   - Accept-Language resolution (panel supplies its storage key).
 */

/** One non-2xx API failure (after any transport-level retry). */
export class ApiError extends Error {
  constructor(
    public status: number,
    public body: string,
  ) {
    super(`API Error ${status}: ${body}`);
  }
}

/** The transport port a panel injects. Chest's Tauri/web pair and any
 * future desktop bridge implement these three methods. */
export interface PanelTransport {
  /** JSON-envelope request (method, path, optional JSON body, headers). */
  httpProxy(
    method: string,
    path: string,
    body: unknown,
    headers: Record<string, string>,
  ): Promise<{ status?: number; body?: unknown }>;
  /** Multipart upload (always PUT — the only verb panels use). */
  upload(
    path: string,
    body: FormData,
    headers: Record<string, string>,
  ): Promise<{ status?: number; body?: unknown }>;
  /** Download / raw passthrough (token carried for same-origin paths). */
  download(
    path: string,
    options: { headers?: Record<string, string>; body?: BodyInit },
  ): Promise<Response>;
}

/** Hook fired whenever an API call fails (non-2xx after retry). Registered
 * during app bootstrap to push a toast — stores no longer toast per catch. */
let onError: ((_err: ApiError, _path: string, _method: string) => void) | null =
  null;

/** Register the global error handler. The 401/auth-lost flow stays in each
 * panel's auth module; this hook is only the toast surface. */
export function configureErrorHandler(
  handler: (err: ApiError, path: string, method: string) => void,
): void {
  onError = handler;
}

/** Notify the global error handler (if registered and not silenced). */
function reportError(
  err: ApiError,
  path: string,
  method: string,
  silent?: boolean,
): void {
  if (silent || !onError) return;
  try {
    onError(err, path, method);
  } catch {
    // The error handler itself must never throw
  }
}

/** Panel-specific Accept-Language resolution (storage key differs per app). */
export type ResolveAcceptLanguage = () => string;

/** Default resolver: navigator language, no panel storage. */
export const navigatorLanguage: ResolveAcceptLanguage = () =>
  typeof navigator === "undefined" ? "en" : (navigator.language || "en");

export interface RequestOptions {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
  /** Suppress the auto-toast on failure — use when the caller handles the
   * error itself (e.g. inline form feedback). */
  silent?: boolean;
}

function toApiError(result: { status?: number; body?: unknown }): ApiError {
  const bodyText =
    typeof result.body === "string" && result.body.length > 0
      ? result.body
      : result.body != null
        ? JSON.stringify(result.body)
        : `HTTP ${result.status || "error"}`;
  return new ApiError(result.status ?? 0, bodyText);
}

/** JSON-envelope request over the injected transport. */
export async function requestJson<T>(
  transport: PanelTransport,
  lang: ResolveAcceptLanguage,
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const { method = "GET", body, headers, silent } = options;
  const result = await transport.httpProxy(method, path, body, {
    "Accept-Language": lang(),
    ...headers,
  });
  if (!result.status || result.status >= 400) {
    const err = toApiError(result);
    reportError(err, path, method, silent);
    throw err;
  }
  return (result.body ?? undefined) as T;
}

export interface UploadOptions {
  body?: FormData;
  headers?: Record<string, string>;
  silent?: boolean;
}

/** Multipart upload over the injected transport (always PUT). */
export async function upload<T>(
  transport: PanelTransport,
  lang: ResolveAcceptLanguage,
  path: string,
  options: UploadOptions = {},
): Promise<T> {
  const { body, headers = {}, silent } = options;
  if (!(body instanceof FormData)) {
    throw new Error("upload requires a FormData body");
  }
  const result = await transport.upload(path, body, {
    "Accept-Language": lang(),
    ...headers,
  });
  if (!result.status || result.status >= 400) {
    const err = toApiError(result);
    reportError(err, path, "PUT", silent);
    throw err;
  }
  return (result.body ?? undefined) as T;
}

export interface RawFetchOptions {
  method?: string;
  headers?: Record<string, string>;
  body?: BodyInit;
}

/** Download / raw passthrough (token carried for same-origin paths). */
export async function download(
  transport: PanelTransport,
  path: string,
  options: RawFetchOptions = {},
): Promise<Response> {
  const { headers: extraHeaders, body } = options;
  return transport.download(path, { headers: extraHeaders, body });
}
