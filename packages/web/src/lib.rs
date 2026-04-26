#[cfg(feature = "wit-bindings")]
pub mod batch_ops;
#[cfg(feature = "wit-bindings")]
pub mod runtime_integration;
#[cfg(feature = "browser")]
pub mod browser;
#[cfg(feature = "wit-bindings")]
pub mod handle_cache;
#[cfg(feature = "i18n")]
pub mod i18n;
#[cfg(feature = "wit-bindings")]
pub mod navigation;
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
pub use i18n::{
    provide_i18n, set_locale, translate, translate_or_key, use_locale, I18nProvider, I18nState,
    Language, TextDirection,
};
#[cfg(feature = "wit-bindings")]
pub use navigation::{current_path, navigate, replace};
pub use prelude::*;
#[cfg(feature = "router")]
pub use router::*;
#[cfg(feature = "ssr")]
pub use ssr::SsrPlatform;
#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub use wit_platform::prevent_event_default;
#[cfg(feature = "wit-bindings")]
pub use wit_platform::{
    get_pathname, push_state, replace_state, WitElement, WitEvent, WitPlatform,
};
#[cfg(feature = "wit-bindings")]
pub use runtime_integration::init_runtime;

#[cfg(all(feature = "router", feature = "wit-bindings", target_family = "wasm"))]
pub mod client_router;
