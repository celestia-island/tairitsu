pub mod platform;
pub mod dom;
pub mod events;

pub use platform::WebPlatform;

#[cfg(feature = "web")]
pub use events::WebEvent;
