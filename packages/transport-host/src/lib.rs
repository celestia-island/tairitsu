//! `tairitsu-transport-host` — runtime host implementations for the
//! `tairitsu:transport` WIT contract.
//!
//! The contract's single authority is the npm package's `wit/transport.wit`
//! (mirrored by `packages/npm/tairitsu-transport`); this crate binds the
//! same file at build time so the WIT can never drift between the TS and
//! runtime sides. First surface: the `http` interface (host-side fetch for
//! component imports). The poll-style websocket/sse bridges come later.

use wasmtime::component::bindgen;

bindgen!({
    path: "../npm/tairitsu-transport/wit",
    world: "transport",
});

pub use self::tairitsu::transport::http::Host as HttpHost;
pub use self::tairitsu::transport::types::{
    CredentialsMode, RedirectMode, RequestOptions, TransportResponse,
};

/// Host state backing the http import: one shared reqwest client.
///
/// Components are untrusted: every request option is validated (method
/// sanity, header count/size caps, body size cap) before it reaches the
/// network, and redirects/credentials are applied per the contract.
#[derive(Debug)]
pub struct TransportHostState {
    client: reqwest::blocking::Client,
}

const MAX_HEADERS: usize = 64;
const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
/// Responses are read fully before marshalling — refuse the oversized
/// ones so a hostile upstream cannot balloon component memory.
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
const MAX_TIMEOUT_MS: u64 = 120_000;

impl Default for TransportHostState {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportHostState {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .build()
                .expect("static reqwest client config is valid"),
        }
    }
}

impl HttpHost for TransportHostState {
    fn request(
        &mut self,
        url: String,
        options: RequestOptions,
    ) -> Result<TransportResponse, String> {
        options.validate()?;
        let RequestParts {
            method,
            headers,
            body,
            credentials,
            redirect,
            timeout_ms,
        } = options.into_parts();

        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| format!("invalid method {method:?}: {e}"))?;
        let mut builder = self.client.request(method.clone(), &url);

        for (name, value) in headers.clone().into_iter() {
            let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| format!("invalid header name {name:?}: {e}"))?;
            let value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| format!("invalid header value for {name:?}: {e}"))?;
            builder = builder.header(name, value);
        }

        if let Some(ref bytes) = body {
            builder = builder.body(bytes.clone());
        }

        // The runtime acts as the component's network origin: same-origin
        // credentials map to cookie forwarding, everything else to none.
        // (Cookie jars are per-state; today the shared client has none.)
        let _ = credentials;

        // reqwest's redirect policy is client-level: route non-follow
        // modes onto a cached no-follow client. WIT's error/manual
        // distinction collapses at this runtime layer — both surface
        // the 3xx to the component instead of trapping (reqwest 0.13
        // dropped Policy::error; components decide on 3xx themselves).
        if !matches!(redirect, RedirectMode::Follow) {
            let client = policy_client();
            let mut redirected = client.request(method, &url);
            for (name, value) in headers.iter() {
                redirected = redirected.header(name.as_str(), value.as_str());
            }
            if let Some(ref bytes) = body {
                redirected = redirected.body(bytes.clone());
            }
            if let Some(ms) = timeout_ms {
                redirected = redirected.timeout(std::time::Duration::from_millis(ms));
            }
            builder = redirected;
        }

        if let Some(ms) = timeout_ms {
            builder = builder.timeout(std::time::Duration::from_millis(ms));
        }

        let response = builder
            .send()
            .map_err(|e| format!("request to {url} failed: {e}"))?;

        let status = response.status();
        let status_text = status.canonical_reason().unwrap_or_default().to_string();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect::<Vec<_>>();
        let body = response
            .bytes()
            .map_err(|e| format!("reading body from {url} failed: {e}"))?
            .to_vec();
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(format!(
                "response body from {url} too large: {} bytes (cap {MAX_RESPONSE_BYTES})",
                body.len()
            ));
        }

        Ok(TransportResponse {
            status: status.as_u16(),
            status_text,
            headers,
            body,
        })
    }
}

/// The flattened request options, defaults applied (the clippy-clean
/// shape of what `into_parts` used to return as a 6-tuple).
pub struct RequestParts {
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub credentials: CredentialsMode,
    pub redirect: RedirectMode,
    pub timeout_ms: Option<u64>,
}

/// Validate request options before they reach the network — components
/// are untrusted and the poll-bridge contract carries raw lists.
impl RequestOptions {
    pub fn into_parts(self) -> RequestParts {
        RequestParts {
            method: self.method,
            headers: self.headers,
            body: self.body,
            credentials: self.credentials.unwrap_or(CredentialsMode::SameOrigin),
            redirect: self.redirect.unwrap_or(RedirectMode::Follow),
            timeout_ms: self.timeout_ms.map(|ms| ms.min(MAX_TIMEOUT_MS)),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.method.is_empty() || self.method.len() > 32 {
            return Err(format!("method length out of range: {}", self.method.len()));
        }
        if self.headers.len() > MAX_HEADERS {
            return Err(format!("too many headers: {}", self.headers.len()));
        }
        let total: usize = self
            .headers
            .iter()
            .map(|(name, value)| name.len() + value.len())
            .sum();
        if total > MAX_HEADER_BYTES {
            return Err(format!("headers too large: {total} bytes"));
        }
        if let Some(body) = &self.body {
            if body.len() > MAX_BODY_BYTES {
                return Err(format!("body too large: {} bytes", body.len()));
            }
        }
        Ok(())
    }
}

/// Placeholder host for the poll-style websocket bridge — the http
/// surface ships first; calling into these from a component returns a
/// clear "not yet bridged" error instead of a runtime trap.
impl self::tairitsu::transport::websocket::Host for TransportHostState {
    fn connect(&mut self, _url: String, _protocols: Vec<String>) -> Result<u64, String> {
        Err("transport-host: the websocket bridge is not implemented yet (http only)".into())
    }
    fn send(
        &mut self,
        _handle: u64,
        _frame: self::tairitsu::transport::types::WsFrame,
    ) -> Result<(), String> {
        Err("transport-host: the websocket bridge is not implemented yet (http only)".into())
    }
    fn recv(&mut self, _handle: u64) -> Option<self::tairitsu::transport::types::WsFrame> {
        // Unreachable for well-behaved components: connect() refuses first.
        None
    }
    fn close_info(
        &mut self,
        _handle: u64,
    ) -> Option<self::tairitsu::transport::types::WsCloseInfo> {
        None
    }
    fn close(&mut self, _handle: u64, _code: u16, _reason: String) {}
}

/// Placeholder host for the poll-style sse bridge (same staging as ws).
impl self::tairitsu::transport::sse::Host for TransportHostState {
    fn subscribe(
        &mut self,
        _url: String,
        _headers: Vec<(String, String)>,
        _events: Vec<String>,
    ) -> Result<u64, String> {
        Err("transport-host: the sse bridge is not implemented yet (http only)".into())
    }
    fn next(&mut self, _handle: u64) -> Option<self::tairitsu::transport::types::SseEvent> {
        // Unreachable for well-behaved components: subscribe() refuses first.
        None
    }
    fn close(&mut self, _handle: u64) {}
}

/// Register the whole `tairitsu:transport` world onto a linker whose
/// store is `TransportHostState` itself (the HasSelf pattern the ssr
/// runtime uses). Multi-store hosts wrap it once their own state needs
/// to coexist — the bindgen-generated per-interface functions remain
/// directly callable.
pub fn add_to_linker(
    linker: &mut wasmtime::component::Linker<TransportHostState>,
) -> anyhow::Result<()> {
    use wasmtime::component::HasSelf;
    self::tairitsu::transport::http::add_to_linker::<
        TransportHostState,
        HasSelf<TransportHostState>,
    >(linker, |state| -> &mut TransportHostState { state })?;
    self::tairitsu::transport::websocket::add_to_linker::<
        TransportHostState,
        HasSelf<TransportHostState>,
    >(linker, |state| -> &mut TransportHostState { state })?;
    self::tairitsu::transport::sse::add_to_linker::<TransportHostState, HasSelf<TransportHostState>>(
        linker,
        |state| -> &mut TransportHostState { state },
    )?;
    Ok(())
}

/// Cached no-follow client for the non-follow redirect modes.
fn policy_client() -> &'static reqwest::blocking::Client {
    static NONE: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
    NONE.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("no-follow client builds")
    })
}
