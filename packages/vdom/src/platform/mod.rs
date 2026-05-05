pub mod element;
pub mod event;
pub mod r#trait;
pub use element::ElementHandle;
pub use event::EventHandle;
pub use r#trait::{
    CanvasContext, ContentEditableState, DomRect, GeoPosition, GeoPositionError,
    MutationObserverInit, MutationRecord, Platform, ResizeObserverEntry, ResizeObserverSize,
};
