//! # I18n Context
//!
//! Reactive i18n context using Tairitsu hooks for dependency injection.
//!
//! ## Architecture
//!
//! `I18nState` stores all translations and the current locale.
//! `I18nProvider` wraps a `Context<I18nState>` — since `Context<T>` is
//! `Rc<RefCell<T>>`, all consumers share the same state and see locale
//! changes immediately.
//!
//! ## Usage
//!
//! ```ignore
//! use tairitsu_web::i18n::{I18nProvider, Language, loader::load_toml_flat};
//! use std::collections::HashMap;
//!
//! // Load translations for multiple locales
//! let mut translations: HashMap<Language, HashMap<String, String>> = HashMap::new();
//! translations.insert(Language::English, load_toml_flat(include_str!("en.toml")).unwrap());
//! translations.insert(Language::ChineseSimplified, load_toml_flat(include_str!("zh.toml")).unwrap());
//!
//! // Provide i18n to the component tree
//! let provider = I18nProvider::new(translations, Language::English);
//! provider.provide();
//!
//! // In any component:
//! //   let locale = tairitsu_i18n::use_locale();
//! //   let text = tairitsu_i18n::set_locale(Language::ChineseSimplified);
//! //   let msg = t!("common.button.submit");  // dot-path lookup
//! ```

use std::collections::HashMap;

use tairitsu_hooks::{consume_context, provide_context, use_context, Context};

use crate::i18n::language::Language;

/// The core i18n state stored in context.
///
/// Holds all translations and the currently active locale.
/// Shared across all consumers via `Rc<RefCell<>>`.
#[derive(Clone, Debug)]
pub struct I18nState {
    /// Currently active locale.
    pub current: Language,
    /// All loaded translations: locale → (dot-path key → translated string).
    pub translations: HashMap<Language, HashMap<String, String>>,
    /// Fallback locale used when a key is missing in the current locale.
    pub fallback: Language,
}

impl I18nState {
    pub fn new(
        translations: HashMap<Language, HashMap<String, String>>,
        default_locale: Language,
    ) -> Self {
        let fallback = default_locale;
        Self {
            current: default_locale,
            translations,
            fallback,
        }
    }

    pub fn with_fallback(mut self, fallback: Language) -> Self {
        self.fallback = fallback;
        self
    }

    /// Look up a translation key in the current locale, falling back if needed.
    ///
    /// Key format: dot-separated path, e.g. `"common.button.submit"`.
    /// Returns `None` if the key is not found in either current or fallback locale.
    pub fn t(&self, key: &str) -> Option<&str> {
        if let Some(map) = self.translations.get(&self.current) {
            if let Some(value) = map.get(key) {
                return Some(value.as_str());
            }
        }
        if self.current != self.fallback {
            if let Some(map) = self.translations.get(&self.fallback) {
                if let Some(value) = map.get(key) {
                    return Some(value.as_str());
                }
            }
        }
        None
    }

    /// Look up a translation key, returning the key itself as fallback.
    pub fn t_or_key<'a>(&'a self, key: &'a str) -> &'a str {
        self.t(key).unwrap_or(key)
    }
}

/// Reactive i18n provider.
///
/// Wraps a `Context<I18nState>` and provides methods for locale management.
/// All consumers share the same underlying state, so locale changes are
/// immediately visible to all components.
pub struct I18nProvider {
    context: Context<I18nState>,
}

impl I18nProvider {
    /// Create a new i18n provider.
    ///
    /// # Arguments
    ///
    /// * `translations` — Map of locale → (key → value) for all supported locales
    /// * `default_locale` — The initial active locale
    pub fn new(
        translations: HashMap<Language, HashMap<String, String>>,
        default_locale: Language,
    ) -> Self {
        let state = I18nState::new(translations, default_locale);
        Self {
            context: Context::new(state),
        }
    }

    /// Provide the i18n state to the component tree.
    ///
    /// Call this once at the app root. After calling, `use_locale()`,
    /// `set_locale()`, and `t!()` become available in all child components.
    pub fn provide(&self) {
        provide_context(self.context.get().clone());
    }

    /// Switch the active locale at runtime.
    ///
    /// All components sharing this context will see the new locale
    /// on their next read.
    pub fn set_locale(&self, locale: Language) {
        let mut state = self.context.get_mut();
        state.current = locale;
    }

    /// Get the current active locale.
    pub fn locale(&self) -> Language {
        self.context.get().current
    }

    /// Translate a key using the current locale.
    pub fn t(&self, key: &str) -> Option<String> {
        self.context.get().t(key).map(|s| s.to_string())
    }
}

/// Provide i18n context to the component tree (convenience function).
///
/// This is a shorthand for `I18nProvider::new(...).provide()`.
pub fn provide_i18n(language: Language, translations: HashMap<Language, HashMap<String, String>>) {
    let provider = I18nProvider::new(translations, language);
    provider.provide();
}

/// Get the current locale from the i18n context.
///
/// Panics if `provide_i18n` has not been called.
pub fn use_locale() -> Language {
    consume_context::<I18nState>().current
}

/// Switch the active locale at runtime.
///
/// Updates the shared context so all future `use_locale()` and `t!()` calls
/// reflect the new locale.
///
/// Panics if `provide_i18n` has not been called.
pub fn set_locale(locale: Language) {
    let ctx =
        use_context::<I18nState>().expect("I18n context not found. Call provide_i18n() first.");
    let mut state = ctx.get_mut();
    state.current = locale;
}

/// Translate a key using the current locale from context.
///
/// Key format: dot-separated path, e.g. `"common.button.submit"`.
/// Returns `None` if the key is not found in any locale.
///
/// Panics if `provide_i18n` has not been called.
pub fn translate(key: &str) -> Option<String> {
    let state = consume_context::<I18nState>();
    state.t(key).map(|s| s.to_string())
}

/// Translate a key, returning the key itself as fallback if not found.
pub fn translate_or_key(key: &str) -> String {
    let state = consume_context::<I18nState>();
    state.t_or_key(key).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_translations() -> HashMap<Language, HashMap<String, String>> {
        let mut en: HashMap<String, String> = HashMap::new();
        en.insert("common.button.submit".to_string(), "Submit".to_string());
        en.insert("common.button.cancel".to_string(), "Cancel".to_string());
        en.insert("greeting".to_string(), "Hello".to_string());

        let mut zh: HashMap<String, String> = HashMap::new();
        zh.insert("common.button.submit".to_string(), "提交".to_string());
        zh.insert("common.button.cancel".to_string(), "取消".to_string());
        zh.insert("greeting".to_string(), "你好".to_string());

        let mut translations = HashMap::new();
        translations.insert(Language::English, en);
        translations.insert(Language::ChineseSimplified, zh);
        translations
    }

    #[test]
    fn test_i18n_state_new() {
        let state = I18nState::new(make_translations(), Language::English);
        assert_eq!(state.current, Language::English);
        assert_eq!(state.fallback, Language::English);
    }

    #[test]
    fn test_i18n_state_translate() {
        let state = I18nState::new(make_translations(), Language::English);
        assert_eq!(state.t("common.button.submit"), Some("Submit"));
        assert_eq!(state.t("common.button.cancel"), Some("Cancel"));
        assert_eq!(state.t("greeting"), Some("Hello"));
        assert_eq!(state.t("nonexistent.key"), None);
    }

    #[test]
    fn test_i18n_state_fallback() {
        let mut translations = make_translations();
        let mut ja: HashMap<String, String> = HashMap::new();
        ja.insert("greeting".to_string(), "こんにちは".to_string());
        translations.insert(Language::Japanese, ja);

        let state =
            I18nState::new(translations, Language::Japanese).with_fallback(Language::English);
        assert_eq!(state.t("greeting"), Some("こんにちは"));
        assert_eq!(state.t("common.button.submit"), Some("Submit"));
    }

    #[test]
    fn test_i18n_state_t_or_key() {
        let state = I18nState::new(make_translations(), Language::English);
        assert_eq!(state.t_or_key("common.button.submit"), "Submit");
        assert_eq!(state.t_or_key("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_i18n_provider_new_and_locale() {
        let provider = I18nProvider::new(make_translations(), Language::English);
        assert_eq!(provider.locale(), Language::English);
    }

    #[test]
    fn test_i18n_provider_set_locale() {
        let provider = I18nProvider::new(make_translations(), Language::English);
        provider.set_locale(Language::ChineseSimplified);
        assert_eq!(provider.locale(), Language::ChineseSimplified);
    }

    #[test]
    fn test_i18n_provider_translate() {
        let provider = I18nProvider::new(make_translations(), Language::English);
        assert_eq!(
            provider.t("common.button.submit"),
            Some("Submit".to_string())
        );
        assert_eq!(provider.t("nonexistent"), None);
    }

    #[test]
    fn test_i18n_provider_translate_after_locale_change() {
        let provider = I18nProvider::new(make_translations(), Language::English);
        assert_eq!(
            provider.t("common.button.submit"),
            Some("Submit".to_string())
        );
        provider.set_locale(Language::ChineseSimplified);
        assert_eq!(provider.t("common.button.submit"), Some("提交".to_string()));
    }

    #[test]
    fn test_provide_and_use_locale() {
        provide_i18n(Language::English, make_translations());
        assert_eq!(use_locale(), Language::English);
    }

    #[test]
    fn test_set_locale_via_function() {
        provide_i18n(Language::English, make_translations());
        assert_eq!(use_locale(), Language::English);
        set_locale(Language::ChineseSimplified);
        assert_eq!(use_locale(), Language::ChineseSimplified);
    }

    #[test]
    fn test_translate_via_function() {
        provide_i18n(Language::English, make_translations());
        assert_eq!(
            translate("common.button.submit"),
            Some("Submit".to_string())
        );
        set_locale(Language::ChineseSimplified);
        assert_eq!(translate("common.button.submit"), Some("提交".to_string()));
    }

    #[test]
    fn test_translate_or_key_via_function() {
        provide_i18n(Language::English, make_translations());
        assert_eq!(translate_or_key("common.button.submit"), "Submit");
        assert_eq!(translate_or_key("missing.key"), "missing.key");
    }

    #[test]
    fn test_context_shared_between_consumers() {
        let provider = I18nProvider::new(make_translations(), Language::English);
        provider.provide();

        assert_eq!(use_locale(), Language::English);

        set_locale(Language::ChineseSimplified);
        assert_eq!(use_locale(), Language::ChineseSimplified);
        assert_eq!(translate("common.button.cancel"), Some("取消".to_string()));
    }
}
