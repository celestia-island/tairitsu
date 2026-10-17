//! Internal bridge types + conversions from the bindgen-generated WIT
//! shapes (the bridges speak these; the Host impl converts at the edge).

/// Mirrors the WIT `ws-frame` variant.
#[derive(Debug, Clone)]
pub(super) enum WsFrameInternal {
    Text(String),
    Binary(Vec<u8>),
}

/// Mirrors the WIT `ws-close-info` record.
#[derive(Debug, Clone)]
pub(super) struct WsCloseInfoInternal {
    pub code: u16,
    pub reason: String,
    pub clean: bool,
}

/// Mirrors the WIT `sse-event` record.
#[derive(Debug, Clone)]
pub(super) struct SseEventInternal {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
}
