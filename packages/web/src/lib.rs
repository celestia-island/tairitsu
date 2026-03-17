pub mod platform;
pub mod portal;
#[cfg(feature = "wit-bindings")]
pub mod wit_platform;

pub use platform::WebPlatform;
pub use portal::PortalRenderer;
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};
