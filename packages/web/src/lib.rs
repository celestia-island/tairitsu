pub mod platform;

pub use platform::WebPlatform;

#[cfg(feature = "web")]
pub use platform::{WebElement, WebEvent};
