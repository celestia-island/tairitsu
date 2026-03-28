//! WIT bindings for browser worlds
//!
//! This module re-exports the WIT interface definitions that were previously
//! in the browser-worlds package.

pub mod browser_full {
    //! Browser full WIT interfaces
    //!
    //! Contains all browser-specific interfaces needed for Tairitsu.

    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    #[allow(non_upper_case_globals)]
    #[allow(clippy::all)]
    pub mod browser {
        include!(concat!(env!("OUT_DIR"), "/browser.rs"));
    }
}

pub mod browser_geometry {
    //! Browser geometry WIT interfaces
    //!
    //! Contains DOM geometry related interfaces.

    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    #[allow(non_upper_case_globals)]
    #[allow(clippy::all)]
    pub mod geometry {
        include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
    }
}

pub mod browser_animation {
    //! Browser animation WIT interfaces
    //!
    //! Contains animation related interfaces.

    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    #[allow(non_upper_case_globals)]
    #[allow(clippy::all)]
    pub mod animation {
        include!(concat!(env!("OUT_DIR"), "/animation.rs"));
    }
}

pub mod browser_media {
    //! Browser media query WIT interfaces
    //!
    //! Contains media query related interfaces.

    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    #[allow(non_upper_case_globals)]
    #[allow(clippy::all)]
    pub mod media {
        include!(concat!(env!("OUT_DIR"), "/media.rs"));
    }
}

/// Export all browser WIT interfaces
pub use browser_full::*;
pub use browser_geometry::*;
pub use browser_animation::*;
pub use browser_media::*;