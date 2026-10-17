/**
 * Desktop transport — routes every channel through a host-invoked local
 * client (e.g. the Tauri gateway). The host owns base URLs, cookies,
 * proxies and TLS; the contract only names the request shape.
 *
 * Fidelity rules (they exist because callers must not have to care which
 * host they run on):
 * - `signal` and `timeoutMs` are honoured here too (the contract declares
 *   them without a host qualifier).
 * - Bodies are text-only: a `Blob`/binary/`FormData` body is rejected with
 *   `unsupported-body` instead of being silently re-encoded.
 * - `json()` on an empty body throws like `Response.json()` does.
 * - Native host failures are wrapped in `TransportError`.
 */

import {
  TransportError,
  type HostInvoke,
  type SseChannel,
  type SseOptions,
  type Transport,
  type TransportRequest,
  type TransportResponse,
  type WsChannel,
  type WsOptions,
} from "./contract.js";

/** Host-side response envelope returned by the `http-proxy` command. */
export interface HostHttpResponse {
  status: number;
  statusText?: string;
  headers?: Record<string, string>;
  /** Either a string body or an already-parsed JSON value. */
  body?: unknown;
}

export interface DesktopHost extends HostInvoke {
  /** Optional: hosts that can bridge a bidirectional channel. */
  openWs?(url: string, opts?: WsOptions): WsChannel;
  /** Optional: hosts that can bridge an event stream. */
  openSse?(url: string, opts?: SseOptions): SseChannel;
}

export interface DesktopTransportOptions {
  /** Host command carrying an HTTP request; defaults to "http-proxy". */
  httpCommand?: string;
}

export function createDesktopTransport(
  host: DesktopHost,
  options: DesktopTransportOptions = {},
): Transport {
  const httpCommand = options.httpCommand ?? "http-proxy";

  return {
    async request(req: TransportRequest): Promise<TransportResponse> {
      if (req.signal?.aborted) {
        throw new TransportError("aborted", "request was aborted");
      }
      const body = serializeBody(req.body);

      let wire: HostHttpResponse;
      try {
        wire = await runWithGuards(
          () =>
            host.invoke<HostHttpResponse>(httpCommand, {
              method: req.method ?? "GET",
              path: req.url,
              headers: req.headers ?? {},
              body,
            }),
          { signal: req.signal, timeoutMs: req.timeoutMs },
        );
      } catch (err) {
        if (err instanceof TransportError) throw err;
        throw new TransportError("host-request-failed", "host request failed", {
          cause: err,
        });
      }

      if (!wire || typeof wire.status !== "number") {
        throw new TransportError(
          "host-request-failed",
          "host response carried no numeric status",
        );
      }

      const status = wire.status;
      const raw = wire.body;
      const text =
        typeof raw === "string"
          ? raw
          : raw === undefined || raw === null
            ? ""
            : JSON.stringify(raw);

      return {
        ok: status >= 200 && status < 300,
        status,
        statusText: wire.statusText ?? "",
        headers: new Headers(wire.headers ?? {}),
        text: async () => text,
        json: async <T = unknown>() => {
          if (text === "") {
            // Same failure a browser's Response.json() produces.
            throw new SyntaxError("Unexpected end of JSON input");
          }
          if (typeof raw === "string") {
            try {
              return JSON.parse(raw) as T;
            } catch (err) {
              throw new TransportError("bad-json", "host body is not JSON", {
                cause: err,
              });
            }
          }
          return raw as T;
        },
      };
    },

    openWs(url: string, opts?: WsOptions): WsChannel {
      if (!host.openWs) {
        throw new TransportError(
          "ws-unavailable",
          "this desktop host cannot bridge WebSocket channels",
        );
      }
      return host.openWs(url, opts);
    },

    openSse(url: string, opts?: SseOptions): SseChannel {
      if (!host.openSse) {
        throw new TransportError(
          "sse-unavailable",
          "this desktop host cannot bridge SSE channels",
        );
      }
      return host.openSse(url, opts);
    },
  };
}

/**
 * Text-only body contract. Structured bodies are the caller's job to
 * serialize (JSON.stringify); binary/FormData is outside the host bridge's
 * wire format today and must fail loudly.
 */
function serializeBody(body: TransportRequest["body"]): string | undefined {
  if (body === undefined || body === null) return undefined;
  if (typeof body === "string") return body;
  if (typeof URLSearchParams !== "undefined" && body instanceof URLSearchParams) {
    return body.toString();
  }
  throw new TransportError(
    "unsupported-body",
    "desktop transport carries text bodies only (string / URLSearchParams); serialize structured bodies at the call site",
  );
}

/** Race the host call against abort/timeout, always cleaning up guards. */
async function runWithGuards<T>(
  fn: () => Promise<T>,
  opts: { signal?: AbortSignal; timeoutMs?: number },
): Promise<T> {
  const { signal, timeoutMs } = opts;
  if (signal?.aborted) {
    throw new TransportError("aborted", "request was aborted");
  }

  const guards: Array<Promise<never>> = [];
  const cleanups: Array<() => void> = [];

  if (timeoutMs !== undefined && timeoutMs > 0) {
    guards.push(
      new Promise<never>((_, reject) => {
        const timer = setTimeout(() => {
          reject(
            new TransportError("timeout", `request timed out after ${timeoutMs}ms`),
          );
        }, timeoutMs);
        cleanups.push(() => clearTimeout(timer));
      }),
    );
  }

  if (signal) {
    guards.push(
      new Promise<never>((_, reject) => {
        const onAbort = () =>
          reject(new TransportError("aborted", "request was aborted"));
        signal.addEventListener("abort", onAbort, { once: true });
        cleanups.push(() => signal.removeEventListener("abort", onAbort));
      }),
    );
  }

  try {
    return guards.length === 0
      ? await fn()
      : await Promise.race([fn(), ...guards]);
  } finally {
    for (const cleanup of cleanups) cleanup();
  }
}
