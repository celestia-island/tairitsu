# @celestia-island/tairitsu-transport

One request-transport contract for every tairitsu host. The same shape —
HTTP requests, WebSocket channels, SSE streams and JSON-RPC over WS —
runs on the browser (window APIs, optionally service-worker mediated) and
on desktop (a host-invoked local client). Hosts differ in implementation,
never in what callers see.

This package exists because of a real outage: a `typeof fetch` contract
leaked the host's receiver semantics, so `fetchFn(...)` threw
`Failed to execute 'fetch' on 'Window': Illegal invocation` on every
browser build while desktop (which injected its own function) worked.
The contract now has a receiver-safe way to obtain the host's fetch
(`bindHostFetch`) and every implementation is built on an injected host
object.

## Install

```bash
npm install @celestia-island/tairitsu-transport
```

## Usage

Browser:

```ts
import { createBrowserTransport, createJsonRpcChannel } from "@celestia-island/tairitsu-transport";

const transport = createBrowserTransport(); // window.fetch/WebSocket/EventSource
const res = await transport.request({ url: "/api/health" });
const ws = transport.openWs("wss://example.test/api/rpc");
const rpc = createJsonRpcChannel(ws);
const user = await rpc.request("auth.getMe");
```

Desktop (host bridge, e.g. Tauri `invoke`):

```ts
import { createDesktopTransport } from "@celestia-island/tairitsu-transport";

const transport = createDesktopTransport({
  invoke: (command, args) => invoke(command, args),
});
const res = await transport.request({ url: "/api/health" });
```

The contract is mirrored in `wit/transport.wit` (hand-written, independent
of the W3C-WebIDL-generated `browser-full` world).

## Fidelity notes

- **Errors**: both hosts map native failures onto `TransportError` codes
  (`network` / `timeout` / `aborted` / `unsupported-body` /
  `host-request-failed` / `bad-json` / `channel-closed`), so callers do not
  branch on which host they run on.
- **Desktop bodies** are text-only (`string` / `URLSearchParams`); binary or
  `FormData` bodies throw `unsupported-body` rather than being silently
  re-encoded. Serialize structured bodies with `JSON.stringify` at the call
  site.
- **Desktop `json()`** on an empty body throws `SyntaxError`, matching
  `Response.json()`.
- **`signal` / `timeoutMs`** are honoured on both hosts.
- **JSON-RPC** validates `jsonrpc: "2.0"`, routes method-bearing frames as
  notifications (never as responses), queues requests until the socket
  opens, and supports per-call `timeoutMs` / `signal`.

## Exports

| Entry | Contents |
| --- | --- |
| `tairitsu-transport` | everything below |
| `tairitsu-transport/contract` | types, `bindHostFetch`, `TransportError`, `CONTRACT_VERSION` |
| `tairitsu-transport/browser` | `createBrowserTransport` |
| `tairitsu-transport/desktop` | `createDesktopTransport` |
| `tairitsu-transport/rpc` | `createJsonRpcChannel`, `JsonRpcErrorException` |

## Tests

```bash
npm run build && npm test
```

Tests inject fake hosts (fetch/WebSocket/EventSource/host bridge), so the
receiver-safety contract is asserted without a real browser.
