pub mod platform;
pub mod portal;
#[cfg(feature = "wit-bindings")]
pub mod runtime_integration;
#[cfg(feature = "wit-bindings")]
pub mod wit_platform;

// Integration tests are available in non-wasm environments
#[cfg(test)]
#[cfg(target_arch = "x86_64")]
pub mod integration_tests;

pub use platform::WebPlatform;
pub use portal::PortalRenderer;
#[cfg(feature = "wit-bindings")]
pub use runtime_integration::{init_runtime, ComponentRenderer};
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};
