pub mod prelude;

pub use prelude::*;

#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "ssr")]
pub mod ssr;

#[cfg(feature = "wit-bindings")]
pub mod wit_platform;

#[cfg(feature = "browser")]
pub use browser::BrowserPlatform;

#[cfg(feature = "ssr")]
pub use ssr::SsrPlatform;

#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};