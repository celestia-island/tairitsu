//! # I18n Macros
//!
//! Convenience macros for accessing i18n text.

/// Macro to access i18n text from context
///
/// ## Usage
///
/// ```rust,no_run
/// use tairitsu_i18n::use_i18n;
///
/// fn component() {
///     let ctx = use_i18n();
///     let text = &ctx.keys.common.button.submit;
///     // text is now a &str containing the translated submit button text
/// }
/// ```
///
/// Note: This macro is a convenience wrapper. For direct access, use `use_i18n()` instead.
#[macro_export]
macro_rules! t {
    ($key_path:expr) => {{
        let ctx = $crate::context::use_i18n();
        $key_path
    }};
}

/// Macro to access i18n text with a custom context
///
/// ## Usage
///
/// ```rust,no_run
/// use tairitsu_i18n::I18nContext;
///
/// fn get_text(i18n_ctx: &I18nContext) -> &str {
///     &i18n_ctx.keys.common.button.submit
/// }
/// ```
#[macro_export]
macro_rules! t_with {
    ($context:expr, $key_path:expr) => {
        &$key_path
    };
}
