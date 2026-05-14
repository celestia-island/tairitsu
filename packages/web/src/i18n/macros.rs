//! # I18n Macros
//!
//! Convenience macros for i18n translation lookup.
//!
//! ## `t!` — translate with key-or-key fallback
//!
//! ```ignore
//! let text = t!("common.button.submit"); // → "Submit" (or key itself if not found)
//! let missing = t!("nonexistent"); // → "nonexistent"
//! ```
//!
//! ## `tr!` — translate returning `Option<String>`
//!
//! ```ignore
//! let text = tr!("common.button.submit"); // → Some("Submit")
//! let missing = tr!("nonexistent"); // → None
//! ```

/// Translate a key using the current locale.
///
/// Returns `String` resolved from the i18n context. If the key is not found,
/// the key string itself is returned as a fallback.
///
/// # Key format
///
/// Dot-separated path: `t!("common.button.submit")`
///
/// # With interpolation
///
/// `t!("greeting", name = "World")` — after lookup, replaces `{name}` placeholders.
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        $crate::i18n::context::translate_or_key($key)
    }};
    ($key:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let template = $crate::i18n::context::translate_or_key($key);
        let mut result = template;
        $(
            result = result.replace(&format!("{{{}}}", stringify!($field)), &$value.to_string());
        )*
        result
    }};
}

/// Translate a key, returning `Option<String>`.
///
/// Returns `None` if the key is not found in any locale (including fallback).
///
/// # Key format
///
/// Dot-separated path: `tr!("common.button.submit")`
#[macro_export]
macro_rules! tr {
    ($key:expr) => {{
        $crate::i18n::context::translate($key)
    }};
}
