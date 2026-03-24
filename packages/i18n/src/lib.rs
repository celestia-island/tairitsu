//! # Tairitsu i18n System
//!
//! Internationalization (i18n) system for Tairitsu UI applications.
//!
//! ## Architecture
//!
//! - **[`language`]** - Language definitions and utilities
//! - **[`keys`]** - Language key structures
//! - **[`loader`]** - TOML loader
//! - **[`context`]** - I18n context using Tairitsu hooks
//! - **[`macros`]** - Convenience macros for i18n access
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tairitsu_i18n::{Language, loader::load_toml, provide_i18n, use_i18n};
//!
//! // Load language keys from TOML
//! let toml_content = r#"
//!     [common.button]
//!     submit = "Submit"
//!     cancel = "Cancel"
//!     confirm = "Confirm"
//!     delete = "Delete"
//!     edit = "Edit"
//!     save = "Save"
//!
//!     [common.navigation]
//!     home = "Home"
//!     about = "About"
//!     documentation = "Documentation"
//!     components = "Components"
//!     theme = "Theme"
//!
//!     [common.status]
//!     loading = "Loading..."
//!     success = "Success"
//!     error = "Error"
//!     warning = "Warning"
//!
//!     [theme]
//!     light = "Light"
//!     dark = "Dark"
//!     auto = "Auto"
//!
//!     [page.home.hero]
//!     title = "Welcome"
//!     subtitle = "Subtitle"
//!     description = "Description"
//!     tagline = "Tagline"
//!     explore = "Explore"
//!
//!     [page.home.features]
//!     title = "Features"
//!     description = "Features description"
//!
//!     [page.components]
//!     title = "Components"
//!     description = "Components description"
//!
//!     [page.documentation]
//!     title = "Documentation"
//!     description = "Documentation description"
//!     getting_started = "Getting Started"
//!     quick_start = "Quick Start"
//!
//!     [language]
//!     english = "English"
//!     chinese_simplified = "简体中文"
//!     chinese_traditional = "繁體中文"
//!     french = "Français"
//!     russian = "Русский"
//!     spanish = "Español"
//!     arabic = "العربية"
//!     japanese = "日本語"
//!     korean = "한국어"
//!
//!     [sidebar.overview]
//!     title = "Overview"
//!
//!     [sidebar.components]
//!     title = "Components"
//!
//!     [sidebar.system]
//!     title = "System"
//!
//!     [sidebar.demos]
//!     title = "Demos"
//!
//!     [sidebar.items]
//!     button = "Button"
//!     form = "Form"
//!     number_input = "Number Input"
//!     search = "Search"
//!     switch = "Switch"
//!     feedback = "Feedback"
//!     display = "Display"
//!     avatar = "Avatar"
//!     image = "Image"
//!     tag = "Tag"
//!     empty = "Empty"
//!     comment = "Comment"
//!     description_list = "Description List"
//!     navigation = "Navigation"
//!     collapsible = "Collapsible"
//!     data = "Data"
//!     table = "Table"
//!     tree = "Tree"
//!     pagination = "Pagination"
//!     qrcode = "QR Code"
//!     timeline = "Timeline"
//!     cascader = "Cascader"
//!     transfer = "Transfer"
//!     media = "Media"
//!     editor = "Editor"
//!     visualization = "Visualization"
//!     user_guide = "User Guide"
//!     zoom_controls = "Zoom Controls"
//!     form_demo = "Form Demo"
//!     dashboard_demo = "Dashboard Demo"
//!     video_demo = "Video Demo"
//! "#;
//! let keys = load_toml(toml_content).unwrap();
//!
//! // Provide i18n context
//! provide_i18n(Language::English, keys);
//!
//! // In your component
//! fn component() {
//!     let i18n = use_i18n();
//!     let submit_text = &i18n.keys.common.button.submit;
//! }
//! ```

pub mod context;
pub mod keys;
pub mod language;
pub mod loader;
pub mod macros;

pub use context::*;
pub use keys::*;
pub use language::*;
