pub mod dom;
pub mod platform;

#[cfg(feature = "web")]
pub mod events;

pub use platform::WebPlatform;

#[cfg(feature = "web")]
pub use events::WebEvent;
