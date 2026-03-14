pub mod platform;
pub mod portal;

pub use platform::WebPlatform;
pub use portal::PortalRenderer;

#[cfg(feature = "web")]
pub use platform::{WebElement, WebEvent};

#[cfg(feature = "wit-bindings")]
pub mod wit_platform;
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{WitElement, WitEvent, WitPlatform};
