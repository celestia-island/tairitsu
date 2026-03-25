pub mod platform;
pub mod portal;
#[cfg(feature = "wit-bindings")]
pub mod wit_platform;
#[cfg(feature = "wit-bindings")]
pub mod runtime_integration;

pub use platform::WebPlatform;
pub use portal::PortalRenderer;
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};
#[cfg(feature = "wit-bindings")]
pub use runtime_integration::{init_runtime, ComponentRenderer};
