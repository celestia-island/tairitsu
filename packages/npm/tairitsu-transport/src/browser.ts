/**
 * Browser transport — window.fetch / WebSocket / EventSource, obtained
 * through an injected host object (default: globalThis) so nothing is
 * ever held as a bare global capture.
 */

import {
  bindHostFetch,
  TransportError,
  type SseChannel,
  type SseEvent,
  type SseOptions,
  type Transport,
  type TransportRequest,
  type TransportResponse,
  type WsChannel,
  type WsCloseInfo,
  type WsFrame,
  type WsOptions,
  type WsReadyState,
} from "./contract.js";

/** The browser APIs this transport needs. Inject a fake in tests. */
export interface BrowserHost {
  fetch: typeof fetch;
  WebSocket: typeof WebSocket;
  EventSource: typeof EventSource;
}

const WS_STATE: Record<number, WsReadyState> = {
  0: "connecting",
  1: "open",
  2: "closing",
  3: "closed",
};

export interface BrowserTransportOptions {
  /** Defaults to a receiver-safe bind of the host's fetch. */
  fetch?: typeof fetch;
  /** Defaults to host.WebSocket (through the host object, not bare). */
  WebSocket?: typeof WebSocket;
  /** Defaults to host.EventSource. */
  EventSource?: typeof EventSource;
}

/**
 * Create the browser transport. All API access goes through the host
 * object's methods, so even an injected stub keeps its receiver
 * semantics (and a test stub can assert on them).
 */
export function createBrowserTransport(
  options: BrowserTransportOptions = {},
  host: typeof globalThis = globalThis,
): Transport {
  const fetchImpl = options.fetch ?? bindHostFetch(host);
  const WebSocketImpl = options.WebSocket ?? host.WebSocket;
  const EventSourceImpl = options.EventSource ?? host.EventSource;

  return {
    async request(req: TransportRequest): Promise<TransportResponse> {
      const init: RequestInit = {
        method: req.method ?? "GET",
        headers: req.headers,
        body: req.body as BodyInit | null | undefined,
        credentials: req.credentials,
        mode: req.mode,
        redirect: req.redirect,
        signal: mergeSignals(req.signal, req.timeoutMs),
      };
      let resp: Response;
      try {
        resp = await fetchImpl(req.url, init);
      } catch (err) {
        throw mapFetchError(err);
      }
      return {
        ok: resp.ok,
        status: resp.status,
        statusText: resp.statusText,
        headers: resp.headers,
        text: () => resp.text(),
        json: <T = unknown>() => resp.json() as Promise<T>,
      };
    },

    openWs(url: string, opts: WsOptions = {}): WsChannel {
      if (typeof WebSocketImpl !== "function") {
        throw new TransportError("no-websocket", "host has no WebSocket");
      }
      const ws = new WebSocketImpl(url, opts.protocols);
      // Predictable frames: binary arrives as ArrayBuffer, not Blob.
      try {
        ws.binaryType = "arraybuffer";
      } catch {
        /* read-only in some host shims */
      }
      return makeWsChannel(ws);
    },

    openSse(url: string, opts: SseOptions = {}): SseChannel {
      if (typeof EventSourceImpl !== "function") {
        throw new TransportError("no-eventsource", "host has no EventSource");
      }
      const es = new EventSourceImpl(url, {
        withCredentials: opts.withCredentials ?? false,
      });
      return makeSseChannel(es, opts.events ?? []);
    },
  };
}

function mergeSignals(
  signal: AbortSignal | undefined,
  timeoutMs: number | undefined,
): AbortSignal | undefined {
  // `0` means disabled on every host (matches rpc/desktop).
  if (timeoutMs === undefined || timeoutMs <= 0) return signal;
  const timeout = AbortSignal.timeout(timeoutMs);
  if (!signal) return timeout;
  return AbortSignal.any([signal, timeout]);
}

function mapFetchError(err: unknown): TransportError {
  if (err instanceof TransportError) return err;
  if (err instanceof DOMException) {
    if (err.name === "TimeoutError") {
      return new TransportError("timeout", "request timed out", { cause: err });
    }
    if (err.name === "AbortError") {
      return new TransportError("aborted", "request was aborted", { cause: err });
    }
  }
  return new TransportError("network", "network request failed", { cause: err });
}

function makeWsChannel(ws: WebSocket): {
  readyState: WsReadyState;
  send: (frame: WsFrame) => void;
  close: (code?: number, reason?: string) => void;
  onOpen: (cb: () => void) => () => void;
  onMessage: (cb: (data: string | ArrayBuffer) => void) => () => void;
  onClose: (cb: (info: WsCloseInfo) => void) => () => void;
  onError: (cb: (err: unknown) => void) => () => void;
} {
  const openHandlers = new Set<() => void>();
  const messageHandlers = new Set<(data: string | ArrayBuffer) => void>();
  const closeHandlers = new Set<(info: WsCloseInfo) => void>();
  const errorHandlers = new Set<(err: unknown) => void>();

  ws.addEventListener("open", () => {
    for (const cb of openHandlers) cb();
  });
  ws.addEventListener("message", (ev: MessageEvent) => {
    const data = ev.data as string | ArrayBuffer;
    for (const cb of messageHandlers) cb(data);
  });
  ws.addEventListener("close", (ev: CloseEvent) => {
    const info: WsCloseInfo = {
      code: ev.code,
      reason: ev.reason,
      wasClean: ev.wasClean,
    };
    for (const cb of closeHandlers) cb(info);
  });
  ws.addEventListener("error", (ev: Event) => {
    for (const cb of errorHandlers) cb(ev);
  });

  return {
    get readyState(): WsReadyState {
      return WS_STATE[ws.readyState] ?? "closed";
    },
    onOpen(cb: () => void) {
      openHandlers.add(cb);
      return () => openHandlers.delete(cb);
    },
    send(frame: WsFrame): void {
      // WsFrame is exactly the DOM's accepted union (BufferSource | Blob |
      // string); the old double cast was for a lib.dom generation whose
      // WebSocket.send overloads predated the unified signature.
      ws.send(frame);
    },
    close(code?: number, reason?: string): void {
      ws.close(code, reason);
    },
    onMessage(cb) {
      messageHandlers.add(cb);
      return () => messageHandlers.delete(cb);
    },
    onClose(cb) {
      closeHandlers.add(cb);
      return () => closeHandlers.delete(cb);
    },
    onError(cb) {
      errorHandlers.add(cb);
      return () => errorHandlers.delete(cb);
    },
  };
}

function makeSseChannel(es: EventSource, events: string[]): SseChannel {
  const openHandlers = new Set<() => void>();
  const eventHandlers = new Set<(ev: SseEvent) => void>();
  const errorHandlers = new Set<(err: unknown) => void>();

  es.addEventListener("open", () => {
    for (const cb of openHandlers) cb();
  });

  const dispatch = (ev: MessageEvent) => {
    const out: SseEvent = {
      data: typeof ev.data === "string" ? ev.data : String(ev.data),
    };
    if (ev.type !== "message") out.event = ev.type;
    if (typeof ev.lastEventId === "string" && ev.lastEventId !== "") {
      out.id = ev.lastEventId;
    }
    for (const cb of eventHandlers) cb(out);
  };
  es.addEventListener("message", dispatch);
  for (const name of events) es.addEventListener(name, dispatch);
  es.addEventListener("error", (ev: Event) => {
    for (const cb of errorHandlers) cb(ev);
  });

  return {
    get readyState(): "connecting" | "open" | "closed" {
      if (es.readyState === 0) return "connecting";
      if (es.readyState === 1) return "open";
      return "closed";
    },
    onOpen(cb) {
      openHandlers.add(cb);
      return () => openHandlers.delete(cb);
    },
    onEvent(cb) {
      eventHandlers.add(cb);
      return () => eventHandlers.delete(cb);
    },
    onError(cb) {
      errorHandlers.add(cb);
      return () => errorHandlers.delete(cb);
    },
    close() {
      es.close();
    },
  };
}
