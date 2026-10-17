/**
 * tairitsu-transport — public entry.
 */

export {
  bindHostFetch,
  CONTRACT_VERSION,
  TransportError,
  type HostInvoke,
  type SseChannel,
  type SseEvent,
  type SseOptions,
  type SseReadyState,
  type Transport,
  type TransportRequest,
  type TransportResponse,
  type WsChannel,
  type WsCloseInfo,
  type WsFrame,
  type WsOptions,
  type WsReadyState,
} from "./contract.js";

export {
  createBrowserTransport,
  type BrowserHost,
  type BrowserTransportOptions,
} from "./browser.js";

export {
  createDesktopTransport,
  type DesktopHost,
  type DesktopTransportOptions,
  type HostHttpResponse,
} from "./desktop.js";

export {
  createJsonRpcChannel,
  JsonRpcErrorException,
  type JsonRpcChannel,
  type JsonRpcChannelState,
  type JsonRpcErrorShape,
  type JsonRpcId,
  type JsonRpcRequestOptions,
} from "./rpc.js";
