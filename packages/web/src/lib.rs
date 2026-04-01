#[cfg(feature = "wit-bindings")]
pub mod batch_ops;
#[cfg(feature = "browser")]
pub mod browser;
#[cfg(feature = "wit-bindings")]
pub mod handle_cache;
#[cfg(feature = "i18n")]
pub mod i18n;
pub mod prelude;
#[cfg(feature = "router")]
pub mod router;
#[cfg(feature = "ssr")]
pub mod ssr;
#[cfg(feature = "wit-bindings")]
pub mod wit_platform;

#[cfg(feature = "browser")]
pub use browser::BrowserPlatform;
#[cfg(feature = "i18n")]
pub use i18n::*;
pub use prelude::*;
#[cfg(feature = "router")]
pub use router::*;
#[cfg(feature = "ssr")]
pub use ssr::SsrPlatform;
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};
