/**
 * Contract tests — browser/desktop/JSON-RPC all through injected hosts.
 *
 * Node has no window/WebSocket/EventSource, which is exactly the point:
 * the transport takes its host as an argument, so a fake host can assert
 * receiver semantics the way a browser would.
 */

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  bindHostFetch,
  createBrowserTransport,
  createDesktopTransport,
  createJsonRpcChannel,
  JsonRpcErrorException,
  TransportError,
} from "../dist/index.js";

function messageEvent(type, data, lastEventId = "") {
  if (typeof MessageEvent !== "undefined") {
    return new MessageEvent(type, { data, lastEventId });
  }
  const ev = new Event(type);
  Object.defineProperty(ev, "data", { value: data });
  Object.defineProperty(ev, "lastEventId", { value: lastEventId });
  return ev;
}

// ── fake browser host ────────────────────────────────────────────────────

class FakeWebSocket extends EventTarget {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;
  static instances = [];

  readyState = 0;
  sent = [];
  url;

  constructor(url, _protocols) {
    super();
    this.url = url;
    FakeWebSocket.instances.push(this);
  }

  send(frame) {
    this.sent.push(frame);
  }

  close(code = 1000, reason = "") {
    this.readyState = FakeWebSocket.CLOSED;
    const ev = new Event("close");
    Object.defineProperty(ev, "code", { value: code });
    Object.defineProperty(ev, "reason", { value: reason });
    Object.defineProperty(ev, "wasClean", { value: true });
    this.dispatchEvent(ev);
  }

  open() {
    this.readyState = FakeWebSocket.OPEN;
    this.dispatchEvent(new Event("open"));
  }

  deliver(data) {
    this.dispatchEvent(messageEvent("message", data));
  }
}

class FakeEventSource extends EventTarget {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSED = 2;
  static instances = [];

  readyState = FakeEventSource.OPEN;
  url;
  closed = false;

  constructor(url, _init) {
    super();
    this.url = url;
    FakeEventSource.instances.push(this);
  }

  close() {
    this.closed = true;
    this.readyState = FakeEventSource.CLOSED;
  }

  deliver(data, type = "message", lastEventId = "") {
    this.dispatchEvent(messageEvent(type, data, lastEventId));
  }
}

function fakeBrowserHost() {
  const fetchCalls = [];
  const fetchImpl = function (url, init) {
    fetchCalls.push({ thisArg: this, url: String(url), init });
    return Promise.resolve(
      new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: { "content-type": "application/json" },
      }),
    );
  };
  FakeWebSocket.instances.length = 0;
  FakeEventSource.instances.length = 0;
  return {
    fetchCalls,
    host: {
      fetch: fetchImpl,
      WebSocket: FakeWebSocket,
      EventSource: FakeEventSource,
    },
  };
}

// ── receiver safety (the 2026-10-17 regression) ─────────────────────────

test("bindHostFetch keeps the receiver that a bare capture loses", async () => {
  const { host, fetchCalls } = fakeBrowserHost();

  const bound = bindHostFetch(host);
  await bound("https://example.test/a");
  assert.equal(fetchCalls.at(-1).thisArg, host);

  const bare = host.fetch;
  await bare("https://example.test/b");
  assert.equal(fetchCalls.at(-1).thisArg, undefined);
});

test("browser transport calls fetch with the injected host as receiver", async () => {
  const { host, fetchCalls } = fakeBrowserHost();
  const transport = createBrowserTransport({}, host);
  await transport.request({ url: "https://example.test/a", method: "POST" });
  assert.equal(fetchCalls.length, 1);
  assert.equal(fetchCalls[0].thisArg, host);
  assert.equal(fetchCalls[0].init.method, "POST");
});

test("browser transport carries WS frames and close info through the channel", () => {
  const { host } = fakeBrowserHost();
  const transport = createBrowserTransport({}, host);
  const ws = transport.openWs("wss://example.test/ws");
  const fake = FakeWebSocket.instances.at(-1);
  assert.ok(fake);

  const seen = [];
  let closeInfo = null;
  ws.onMessage((d) => seen.push(String(d)));
  ws.onClose((info) => (closeInfo = info));

  fake.open();
  fake.deliver("hello");
  ws.send("out");
  ws.close(1001, "bye");

  assert.deepEqual(seen, ["hello"]);
  assert.deepEqual(fake.sent, ["out"]);
  assert.deepEqual(closeInfo, { code: 1001, reason: "bye", wasClean: true });
  assert.equal(ws.readyState, "closed");
});

test("browser transport surfaces named SSE events", () => {
  const { host } = fakeBrowserHost();
  const transport = createBrowserTransport({}, host);
  const sse = transport.openSse("https://example.test/sse", { events: ["update"] });
  const fake = FakeEventSource.instances.at(-1);
  assert.ok(fake);

  const events = [];
  sse.onEvent((ev) => events.push(ev));
  fake.deliver("one", "message", "id-1");
  fake.deliver("two", "update", "id-2");
  sse.close();

  assert.deepEqual(events, [
    { data: "one", id: "id-1" },
    { event: "update", data: "two", id: "id-2" },
  ]);
  assert.equal(fake.closed, true);
});

// ── desktop transport ───────────────────────────────────────────────────

test("desktop transport forwards http requests through the host bridge", async () => {
  const calls = [];
  const transport = createDesktopTransport({
    invoke: async (command, args) => {
      calls.push({ command, args });
      return { status: 200, body: { hello: "world" } };
    },
  });

  const resp = await transport.request({
    url: "https://api.example.test/v1/things",
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ a: 1 }),
  });

  assert.equal(calls.length, 1);
  assert.equal(calls[0].command, "http-proxy");
  assert.deepEqual(calls[0].args, {
    method: "POST",
    path: "https://api.example.test/v1/things",
    headers: { "content-type": "application/json" },
    body: '{"a":1}',
  });
  assert.equal(resp.ok, true);
  assert.deepEqual(await resp.json(), { hello: "world" });
});

test("desktop transport reports missing WS/SSE bridges explicitly", () => {
  const transport = createDesktopTransport({ invoke: async () => ({ status: 200 }) });
  assert.throws(
    () => transport.openWs("wss://example.test/ws"),
    (err) => err instanceof TransportError && err.code === "ws-unavailable",
  );
  assert.throws(
    () => transport.openSse("https://example.test/sse"),
    (err) => err instanceof TransportError && err.code === "sse-unavailable",
  );
});

// ── JSON-RPC channel ────────────────────────────────────────────────────

function fakeWsChannel() {
  const messageHandlers = new Set();
  const closeHandlers = new Set();
  const errorHandlers = new Set();
  const ch = {
    sent: [],
    closed: false,
    get readyState() {
      return "open";
    },
    send(frame) {
      ch.sent.push(frame);
    },
    close() {
      ch.closed = true;
      for (const cb of closeHandlers) cb({ code: 1000, reason: "", wasClean: true });
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
    deliver(frame) {
      const text = typeof frame === "string" ? frame : JSON.stringify(frame);
      for (const cb of messageHandlers) cb(text);
    },
  };
  return ch;
}

test("json-rpc channel matches responses, errors and notifications", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const notes = [];
  rpc.onNotification((m, p) => notes.push([m, p]));

  const p1 = rpc.request("thing.list", { page: 1 });
  const p2 = rpc.request("thing.fail");
  const sent = ws.sent.map((s) => JSON.parse(s));
  assert.equal(sent[0].method, "thing.list");
  assert.equal(sent[1].method, "thing.fail");

  ws.deliver({ jsonrpc: "2.0", id: sent[0].id, result: { x: 1 } });
  ws.deliver({
    jsonrpc: "2.0",
    id: sent[1].id,
    error: { code: -32000, message: "boom" },
  });
  ws.deliver({ jsonrpc: "2.0", method: "thing.changed", params: { id: 7 } });

  assert.deepEqual(await p1, { x: 1 });
  await assert.rejects(
    p2,
    (err) => err instanceof JsonRpcErrorException && err.code === -32000,
  );
  assert.deepEqual(notes, [["thing.changed", { id: 7 }]]);
});

test("json-rpc channel fails pending calls on close and refuses new ones", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const pending = rpc.request("thing.slow");
  rpc.close();
  await assert.rejects(
    pending,
    (err) => err instanceof TransportError && err.code === "channel-closed",
  );
  await assert.rejects(
    rpc.request("thing.after"),
    (err) => err instanceof TransportError && err.code === "channel-closed",
  );
});

// ── R2 hardening: error shapes, desktop guards, RPC protocol ─────────────

test("browser transport wraps fetch failures as TransportError(network)", async () => {
  const host = {
    ...fakeBrowserHost().host,
    fetch: () => Promise.reject(new TypeError("boom")),
  };
  const transport = createBrowserTransport({}, host);
  await assert.rejects(
    transport.request({ url: "https://example.test/x" }),
    (err) => err instanceof TransportError && err.code === "network",
  );
});

test("browser transport maps fetch timeout to TransportError(timeout)", async () => {
  const host = {
    ...fakeBrowserHost().host,
    fetch: () => Promise.reject(new DOMException("t", "TimeoutError")),
  };
  const transport = createBrowserTransport({}, host);
  await assert.rejects(
    transport.request({ url: "https://example.test/x" }),
    (err) => err instanceof TransportError && err.code === "timeout",
  );
});

test("desktop transport honours an already-aborted signal without invoking the host", async () => {
  let calls = 0;
  const transport = createDesktopTransport({
    invoke: async () => {
      calls += 1;
      return { status: 200 };
    },
  });
  const ac = new AbortController();
  ac.abort();
  await assert.rejects(
    transport.request({ url: "/x", signal: ac.signal }),
    (err) => err instanceof TransportError && err.code === "aborted",
  );
  assert.equal(calls, 0);
});

test("desktop transport times out slow host calls", async () => {
  const transport = createDesktopTransport({
    invoke: () => new Promise((resolve) => setTimeout(() => resolve({ status: 200 }), 200)),
  });
  await assert.rejects(
    transport.request({ url: "/x", timeoutMs: 20 }),
    (err) => err instanceof TransportError && err.code === "timeout",
  );
});

test("desktop transport rejects binary bodies explicitly", async () => {
  const transport = createDesktopTransport({ invoke: async () => ({ status: 200 }) });
  await assert.rejects(
    transport.request({ url: "/x", body: new Uint8Array([1, 2]) }),
    (err) => err instanceof TransportError && err.code === "unsupported-body",
  );
});

test("desktop transport json() on an empty body throws like Response.json()", async () => {
  const transport = createDesktopTransport({ invoke: async () => ({ status: 204 }) });
  const resp = await transport.request({ url: "/x" });
  await assert.rejects(resp.json(), (err) => err instanceof SyntaxError);
});

test("desktop transport wraps a synchronously-throwing host", async () => {
  const transport = createDesktopTransport({
    invoke: () => {
      throw new Error("sync");
    },
  });
  await assert.rejects(
    transport.request({ url: "/x" }),
    (err) => err instanceof TransportError && err.code === "host-request-failed",
  );
});

function fakeConnectingWs() {
  const messageHandlers = new Set();
  const closeHandlers = new Set();
  const openHandlers = new Set();
  const ch = {
    sent: [],
    state: "connecting",
    get readyState() {
      return ch.state;
    },
    send(frame) {
      ch.sent.push(frame);
    },
    close() {
      ch.state = "closed";
      for (const cb of closeHandlers) cb({ code: 1000, reason: "", wasClean: true });
    },
    onOpen(cb) {
      openHandlers.add(cb);
      return () => openHandlers.delete(cb);
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
      return () => {};
    },
    open() {
      ch.state = "open";
      for (const cb of openHandlers) cb();
    },
    deliver(frame) {
      const text = typeof frame === "string" ? frame : JSON.stringify(frame);
      for (const cb of messageHandlers) cb(text);
    },
  };
  return ch;
}

test("json-rpc queues requests until the socket opens and reports the open state", async () => {
  const ws = fakeConnectingWs();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const states = [];
  rpc.onState((s) => states.push(s));
  assert.equal(rpc.state, "connecting");

  const p = rpc.request("thing.list");
  assert.equal(ws.sent.length, 0, "request must be queued while connecting");
  ws.open();
  assert.equal(rpc.state, "open");
  const sent = JSON.parse(ws.sent[0]);
  ws.deliver({ jsonrpc: "2.0", id: sent.id, result: [1, 2] });
  assert.deepEqual(await p, [1, 2]);
  assert.ok(states.includes("open"));
});

test("json-rpc ignores frames with the wrong protocol version", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const p = rpc.request("thing.list");
  const sent = JSON.parse(ws.sent[0]);
  ws.deliver({ jsonrpc: "1.0", id: sent.id, result: "bogus" });
  assert.equal(rpc.state, "open");
  ws.deliver({ jsonrpc: "2.0", id: sent.id, result: "good" });
  assert.equal(await p, "good");
});

test("json-rpc routes method-bearing frames as notifications, not responses", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const notes = [];
  rpc.onNotification((m, p) => notes.push([m, p]));
  const p = rpc.request("thing.list");
  const sent = JSON.parse(ws.sent[0]);
  // Server->client request whose id collides with the pending call.
  ws.deliver({ jsonrpc: "2.0", id: sent.id, method: "server.hello", params: {} });
  assert.deepEqual(notes, [["server.hello", {}]]);
  ws.deliver({ jsonrpc: "2.0", id: sent.id, result: "real" });
  assert.equal(await p, "real");
});

test("json-rpc normalizes non-object errors", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  const p = rpc.request("thing.fail");
  const sent = JSON.parse(ws.sent[0]);
  ws.deliver({ jsonrpc: "2.0", id: sent.id, error: "boom" });
  await assert.rejects(
    p,
    (err) => err instanceof JsonRpcErrorException && err.code === -32000 && err.message === "boom",
  );
});

test("json-rpc honours per-call timeouts", async () => {
  const ws = fakeWsChannel();
  const rpc = createJsonRpcChannel(ws, { timeoutMs: 0 });
  await assert.rejects(
    rpc.request("thing.slow", undefined, { timeoutMs: 20 }),
    (err) => err instanceof TransportError && err.code === "timeout",
  );
});
