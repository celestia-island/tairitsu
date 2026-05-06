//! # Tairitsu i18n System
//!
//! Internationalization (i18n) system for Tairitsu UI applications.
//!
//! ## Architecture
//!
//! - **[`I18nProvider`]** — Reactive context-based locale provider
//! - **[`I18nState`]** — Core state: current locale + all translations
//! - **[`Language`]** — Type-safe locale representation (full ISO 639-1/639-3 coverage via `iso639_enum`)
//! - **[`t!`]** — Translate macro: `t!("common.button.submit")` → `String`
//! - **[`tr!`]** — Translate macro returning `Option<String>`
//! - **[`provide_i18n`]** — Shorthand to provide i18n context
//! - **[`use_locale`]** — Get current locale from context
//! - **[`set_locale`]** — Switch locale at runtime
//! - **[`load_toml_flat`]** — Load TOML into dot-path `HashMap`
//!
//! ## Quick Start
//!
//! ```ignore
//! use tairitsu_web::i18n::{I18nProvider, Language, provide_i18n, use_locale, set_locale, loader::load_toml_flat};
//! use std::collections::HashMap;
//!
//! // Load translations for multiple locales
//! let mut translations: HashMap<Language, HashMap<String, String>> = HashMap::new();
//! translations.insert(Language::English, load_toml_flat(include_str!("en.toml")).unwrap());
//! translations.insert(Language::ChineseSimplified, load_toml_flat(include_str!("zh.toml")).unwrap());
//!
//! // Option A: Use I18nProvider
//! let provider = I18nProvider::new(translations.clone(), Language::English);
//! provider.provide();
//!
//! // Option B: Use shorthand
//! provide_i18n(Language::English, translations);
//!
//! // In any component:
//! //   let locale = use_locale();           // → Language::English
//! //   let text = t!("common.button.submit"); // → "Submit"
//! //   set_locale(Language::ChineseSimplified);
//! //   let text = t!("common.button.submit"); // → "提交"
//! ```

pub mod context;
pub mod keys;
pub mod language;
pub mod loader;
pub mod macros;

pub use context::{
    I18nProvider, I18nState, provide_i18n, set_locale, translate, translate_or_key, use_locale,
};
pub use keys::*;
pub use language::*;
