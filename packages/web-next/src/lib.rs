//! Tairitsu Web Next - Unified Web Platform
//!
//! This package consolidates all Tairitsu web-related packages into a single,
//! feature-gated module. Use this for simplified dependency management and
//! better user experience.

#![cfg_attr(not(feature = "std"), no_std)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/skiptests/tairitsu/main/docs/logo.svg")]

pub mod prelude {
    //! Public exports for common functionality
    pub use tairitsu_runtime::*;

    #[cfg(feature = "vdom")]
    pub use tairitsu_vdom::*;

    #[cfg(feature = "hooks")]
    pub use tairitsu_hooks::*;

    #[cfg(feature = "macros")]
    pub use tairitsu_macros::*;

    #[cfg(feature = "style")]
    pub use tairitsu_style::*;

    #[cfg(feature = "router")]
    pub use tairitsu_router::*;

    #[cfg(feature = "ssr")]
    pub use tairitsu_ssr::*;

    #[cfg(feature = "data-fetcher")]
    pub use tairitsu_data_fetcher::*;

    #[cfg(feature = "hmr")]
    pub use tairitsu_hmr::*;

    #[cfg(feature = "fast-refresh")]
    pub use tairitsu_fast_refresh::*;

    #[cfg(feature = "error-overlay")]
    pub use tairitsu_error_overlay::*;

    #[cfg(feature = "css-values")]
    pub use tairitsu_css_values::*;

    #[cfg(feature = "i18n")]
    pub use tairitsu_i18n::*;

    #[cfg(feature = "packager")]
    pub use tairitsu_packager::*;
}

/// Browser-specific functionality
#[cfg(feature = "browser")]
pub mod browser;

/// WIT bindings
pub mod wit_bindings;

/// Re-export common types and functions
pub use prelude::*;

/// Platform-specific initialization
///
/// Call this function to initialize the Tairitsu runtime for the current platform.
/// This sets up the global state and necessary configuration.
pub fn init() {
    #[cfg(feature = "ssr")]
    {
        // Initialize SSR context if needed
        tairitsu_ssr::init_ssr();
    }

    #[cfg(feature = "browser")]
    {
        // Initialize browser context if needed
        #[cfg(feature = "error-overlay")]
        tairitsu_error_overlay::init_error_overlay();

        #[cfg(feature = "hmr")]
        tairitsu_hmr::init_hmr();
    }
}

/// Check if Tairitsu is running in SSR mode
#[cfg(feature = "ssr")]
pub fn is_ssr() -> bool {
    tairitsu_ssr::is_ssr_mode()
}

/// Check if Tairitsu is running in browser mode
#[cfg(feature = "browser")]
pub fn is_browser() -> bool {
    !is_ssr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
        // Should not panic
    }

    #[test]
    #[cfg(feature = "vdom")]
    fn test_vdom_features() {
        use tairitsu_vdom::VNode;
        let vnode = VNode::Text("test".into());
        assert_eq!(vnode.text().unwrap(), "test");
    }

    #[test]
    #[cfg(feature = "hooks")]
    fn test_hooks_features() {
        use tairitsu_hooks::use_signal;
        let signal = use_signal(|| 42);
        assert_eq!(*signal.get(), 42);
    }
}