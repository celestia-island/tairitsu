/**
 * tairitsu-transport — the one request-transport contract.
 *
 * Same shape on every host: browser (window.fetch / WebSocket /
 * EventSource, optionally mediated by a service worker) and desktop
 * (host-invoked local client). Hosts differ in *implementation*, not in
 * what callers see — this is the lesson from the 2026-10-17
 * "Illegal invocation" outage: a bare `typeof fetch` contract leaked the
 * host's receiver semantics, so every default call threw in the browser
 * while desktop (which injected its own function) worked.
 *
 * The contract deliberately carries no receiver assumptions:
 * `bindHostFetch()` is the only sanctioned way to obtain the browser's
 * fetch, and every host implementation is built on an injected host
 * object rather than on bare global captures.
 *
 * WIT mirror: `wit/transport.wit` (hand-written, independent of the
 * W3C-WebIDL-generated `browser-full` world).
 */

/** Request options — the shared subset of `fetch` init. */
export interface TransportRequest {
  url: string;
  method?: string;
  headers?: Record<string, string>;
  body?: BodyInit | null;
  /** Browser only; desktop hosts map it to their cookie jar. */
  credentials?: "omit" | "same-origin" | "include";
  /** Browser only; desktop hosts ignore it. */
  mode?: "cors" | "no-cors" | "same-origin";
  /** Browser only; desktop hosts ignore it. */
  redirect?: "follow" | "error" | "manual";
  signal?: AbortSignal;
  /** Abort the request after this many milliseconds; 0 = disabled
   *  (both hosts, same as the JSON-RPC channel). */
  timeoutMs?: number;
}

/** Unified response — `Response` in the browser, a host-built twin on
 *  the desktop. Both expose the same four members. */
export interface TransportResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Headers;
  text(): Promise<string>;
  json<T = unknown>(): Promise<T>;
}

export type WsFrame = string | ArrayBuffer | ArrayBufferView | Blob;

export interface WsCloseInfo {
  code: number;
  reason: string;
  wasClean: boolean;
}

export interface WsOptions {
  protocols?: string[];
  /** Browser WebSocket cannot send custom headers; desktop may. */
  headers?: Record<string, string>;
}

export type WsReadyState = "connecting" | "open" | "closing" | "closed";

export interface WsChannel {
  readonly readyState: WsReadyState;
  send(frame: WsFrame): void;
  close(code?: number, reason?: string): void;
  /** Optional: browsers and most host bridges can report the open edge;
   *  JSON-RPC queues requests until it fires when present. */
  onOpen?(cb: () => void): () => void;
  onMessage(cb: (data: string | ArrayBuffer) => void): () => void;
  onClose(cb: (info: WsCloseInfo) => void): () => void;
  onError(cb: (err: unknown) => void): () => void;
}

export interface SseEvent {
  /** `event:` field; absent means the default "message" event. */
  event?: string;
  data: string;
  id?: string;
}

export interface SseOptions {
  /** Browser EventSource cannot send custom headers; desktop may. */
  headers?: Record<string, string>;
  withCredentials?: boolean;
  /** Additional named events to subscribe to (`event:` field values). */
  events?: string[];
}

export type SseReadyState = "connecting" | "open" | "closed";

export interface SseChannel {
  readonly readyState: SseReadyState;
  /** The stream's open edge (browsers fire it once the connection is
   *  established; progressive-connect callers gate on it). */
  onOpen?(cb: () => void): () => void;
  onEvent(cb: (ev: SseEvent) => void): () => void;
  onError(cb: (err: unknown) => void): () => void;
  close(): void;
}

/**
 * The transport capability. One interface, one implementation per host.
 */
export interface Transport {
  request(req: TransportRequest): Promise<TransportResponse>;
  openWs(url: string, opts?: WsOptions): WsChannel;
  openSse(url: string, opts?: SseOptions): SseChannel;
  /** Release every channel this transport opened (optional). */
  close?(): void;
}

/**
 * The desktop host's command bridge (e.g. Tauri `invoke`).
 * The transport never assumes a specific command protocol; the host
 * adapter names its own commands.
 */
export interface HostInvoke {
  invoke<T = unknown>(command: string, args: unknown): Promise<T>;
}

/**
 * Transport-level failure (network, host, protocol).
 *
 * Codes: `no-fetch` | `no-websocket` | `no-eventsource` | `network` |
 * `timeout` | `aborted` | `unsupported-body` | `host-request-failed` |
 * `bad-json` | `ws-unavailable` | `sse-unavailable` | `send-failed` |
 * `channel-closed`. Browser and desktop hosts map their native failures
 * onto these codes so callers see one error shape.
 */
export class TransportError extends Error {
  readonly code: string;

  constructor(code: string, message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "TransportError";
    this.code = code;
  }
}

/**
 * The only sanctioned way to take the browser's fetch: bind it to the
 * global so a later call has a receiver. A bare `fetch` capture is what
 * produced "Failed to execute 'fetch' on 'Window': Illegal invocation".
 */
export function bindHostFetch(host: typeof globalThis = globalThis): typeof fetch {
  const fn = host.fetch;
  if (typeof fn !== "function") {
    throw new TransportError(
      "no-fetch",
      "host has no fetch implementation to bind",
    );
  }
  return fn.bind(host);
}

/** Contract version — bump when the shape changes incompatibly. */
export const CONTRACT_VERSION = "0.1.0";
