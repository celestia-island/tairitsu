/**
 * JSON-RPC 2.0 channel over a transport WebSocket.
 *
 * "All WS JSON-RPC goes through transport management" (user direction,
 * 2026-10-17): callers never see a raw WebSocket — they get request/
 * notification/response semantics with pending-call cleanup on close.
 * Progressive-connect/reconnect policy stays in plana-rpc-client; this
 * layer owns the frame protocol only.
 */

import {
  TransportError,
  type WsChannel,
  type WsCloseInfo,
} from "./contract.js";

export type JsonRpcId = number | string;

export interface JsonRpcErrorShape {
  code: number;
  message: string;
  data?: unknown;
}

export class JsonRpcErrorException extends Error {
  readonly code: number;
  readonly data?: unknown;

  constructor(error: JsonRpcErrorShape) {
    super(error.message);
    this.name = "JsonRpcError";
    this.code = error.code;
    this.data = error.data;
  }
}

export type JsonRpcChannelState = "connecting" | "open" | "closed";

export interface JsonRpcRequestOptions {
  /** Per-call timeout; defaults to the channel-level timeout, 0 = none. */
  timeoutMs?: number;
  signal?: AbortSignal;
}

type Pending = {
  resolve: (value: unknown) => void;
  reject: (err: unknown) => void;
  timer?: ReturnType<typeof setTimeout>;
  cleanup?: () => void;
};

export interface JsonRpcChannel {
  readonly state: JsonRpcChannelState;
  request<T = unknown>(
    method: string,
    params?: unknown,
    options?: JsonRpcRequestOptions,
  ): Promise<T>;
  notify(method: string, params?: unknown): void;
  onNotification(cb: (method: string, params: unknown) => void): () => void;
  onState(cb: (state: JsonRpcChannelState) => void): () => void;
  close(): void;
}

const DEFAULT_TIMEOUT_MS = 30_000;

export function createJsonRpcChannel(
  ws: WsChannel,
  options: { timeoutMs?: number } = {},
): JsonRpcChannel {
  const defaultTimeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const pending = new Map<JsonRpcId, Pending>();
  const notificationHandlers = new Set<(m: string, p: unknown) => void>();
  const stateHandlers = new Set<(s: JsonRpcChannelState) => void>();
  const outbox: string[] = [];
  let nextId = 1;
  let state: JsonRpcChannelState =
    ws.readyState === "open"
      ? "open"
      : ws.readyState === "connecting"
        ? "connecting"
        : "closed";

  const setState = (next: JsonRpcChannelState) => {
    if (state === next) return;
    state = next;
    for (const cb of stateHandlers) cb(next);
  };

  const flush = () => {
    while (outbox.length > 0) {
      const frame = outbox.shift();
      if (frame !== undefined) ws.send(frame);
    }
  };

  const failAll = (err: unknown) => {
    for (const [, p] of pending) {
      if (p.timer) clearTimeout(p.timer);
      p.cleanup?.();
      p.reject(err);
    }
    pending.clear();
  };

  const offOpen = ws.onOpen?.(() => {
    setState("open");
    flush();
  });

  const offMessage = ws.onMessage((data) => {
    if (typeof data !== "string") return; // binary frames are not JSON-RPC
    let frame: unknown;
    try {
      frame = JSON.parse(data);
    } catch {
      return; // malformed frame: ignore rather than kill the channel
    }
    if (frame === null || typeof frame !== "object") return;
    const msg = frame as Record<string, unknown>;
    if (msg.jsonrpc !== "2.0") return; // reject other protocol versions

    // A method-bearing frame is a notification (or a server->client
    // request, which this client does not implement). Route it before any
    // id matching so an id collision can never resolve a pending call.
    if (typeof msg.method === "string") {
      for (const cb of notificationHandlers) cb(msg.method, msg.params);
      return;
    }

    if (msg.id === undefined || msg.id === null) return;
    const key = msg.id as JsonRpcId;
    const p = pending.get(key);
    if (!p) return;
    if (!("result" in msg) && !("error" in msg)) return; // not a response
    pending.delete(key);
    if (p.timer) clearTimeout(p.timer);
    p.cleanup?.();
    if ("error" in msg) {
      p.reject(normalizeRpcError(msg.error));
    } else {
      p.resolve(msg.result);
    }
  });

  const offClose = ws.onClose((info: WsCloseInfo) => {
    setState("closed");
    outbox.length = 0;
    failAll(
      new TransportError(
        "channel-closed",
        `JSON-RPC channel closed (code ${info.code}${info.reason ? `: ${info.reason}` : ""})`,
      ),
    );
  });

  // Browser WebSockets fire `error` before `close`; the close handler is
  // what settles pending calls, so error is only observed (and unsubscribed
  // on close) rather than treated as terminal.
  const offError = ws.onError(() => {});

  if (state === "closed") setState("closed");

  return {
    get state() {
      return state;
    },

    request<T = unknown>(
      method: string,
      params?: unknown,
      requestOptions: JsonRpcRequestOptions = {},
    ): Promise<T> {
      if (state === "closed") {
        return Promise.reject(
          new TransportError("channel-closed", "JSON-RPC channel is closed"),
        );
      }
      const id = nextId++;
      const timeoutMs = requestOptions.timeoutMs ?? defaultTimeoutMs;
      const frame = JSON.stringify({
        jsonrpc: "2.0",
        id,
        method,
        ...(params === undefined ? {} : { params }),
      });

      return new Promise<T>((resolve, reject) => {
        const timer =
          timeoutMs > 0
            ? setTimeout(() => {
                if (pending.delete(id)) {
                  p.cleanup?.();
                  reject(
                    new TransportError(
                      "timeout",
                      `JSON-RPC request '${method}' timed out after ${timeoutMs}ms`,
                    ),
                  );
                }
              }, timeoutMs)
            : undefined;

        let cleanup: (() => void) | undefined;
        if (requestOptions.signal) {
          if (requestOptions.signal.aborted) {
            if (timer) clearTimeout(timer);
            reject(new DOMException("the request was aborted", "AbortError"));
            return;
          }
          const onAbort = () => {
            if (pending.delete(id)) {
              if (timer) clearTimeout(timer);
              reject(new DOMException("the request was aborted", "AbortError"));
            }
          };
          requestOptions.signal.addEventListener("abort", onAbort, {
            once: true,
          });
          cleanup = () =>
            requestOptions.signal?.removeEventListener("abort", onAbort);
        }

        const p: Pending = {
          resolve: (v) => resolve(v as T),
          reject,
          timer,
          cleanup,
        };
        pending.set(id, p);

        try {
          // Queue until the socket opens (browser throws if you send while
          // CONNECTING); channels without an open edge send immediately.
          if (state === "connecting" && ws.onOpen) {
            outbox.push(frame);
          } else {
            ws.send(frame);
          }
        } catch (err) {
          pending.delete(id);
          if (timer) clearTimeout(timer);
          cleanup?.();
          reject(
            new TransportError("send-failed", "failed to send JSON-RPC frame", {
              cause: err,
            }),
          );
        }
      });
    },

    notify(method: string, params?: unknown): void {
      if (state === "closed") return;
      const frame = JSON.stringify({
        jsonrpc: "2.0",
        method,
        ...(params === undefined ? {} : { params }),
      });
      if (state === "connecting" && ws.onOpen) {
        outbox.push(frame);
      } else {
        ws.send(frame);
      }
    },

    onNotification(cb) {
      notificationHandlers.add(cb);
      return () => notificationHandlers.delete(cb);
    },

    onState(cb) {
      stateHandlers.add(cb);
      // Deliver the current state immediately so callers never race a
      // transition that already happened.
      cb(state);
      return () => stateHandlers.delete(cb);
    },

    close(): void {
      offOpen?.();
      offMessage();
      offClose();
      offError();
      outbox.length = 0;
      failAll(
        new TransportError("channel-closed", "JSON-RPC channel closed by caller"),
      );
      ws.close();
      setState("closed");
    },
  };
}

function normalizeRpcError(error: unknown): JsonRpcErrorException {
  if (error !== null && typeof error === "object" && "message" in error) {
    const shape = error as { code?: unknown; message?: unknown; data?: unknown };
    return new JsonRpcErrorException({
      code: typeof shape.code === "number" ? shape.code : -32000,
      message: typeof shape.message === "string" ? shape.message : "JSON-RPC error",
      data: shape.data,
    });
  }
  return new JsonRpcErrorException({
    code: -32000,
    message: typeof error === "string" ? error : "JSON-RPC error",
  });
}
