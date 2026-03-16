use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    CHS,
    CHT,
    En,
    Ja,
    Ko,
    Fr,
    Es,
    Ru,
    Ar,
}

impl Default for Language {
    fn default() -> Self {
        Self::En
    }
}

impl Language {
    fn from_system_locale(raw: &str) -> Self {
        let norm = raw.to_ascii_lowercase().replace('_', "-");

        if norm.starts_with("zh") {
            if norm.contains("hant")
                || norm.contains("tw")
                || norm.contains("hk")
                || norm.contains("mo")
            {
                return Self::CHT;
            }
            return Self::CHS;
        }

        if norm.starts_with("ja") {
            return Self::Ja;
        }
        if norm.starts_with("ko") {
            return Self::Ko;
        }
        if norm.starts_with("fr") {
            return Self::Fr;
        }
        if norm.starts_with("es") {
            return Self::Es;
        }
        if norm.starts_with("ru") {
            return Self::Ru;
        }
        if norm.starts_with("ar") {
            return Self::Ar;
        }

        Self::En
    }
}

yuuka::derive_struct!(
    #[derive(Serialize, Deserialize)]
    pub Translations {
        cli: Cli {
            starting_dev_server: String,
            building_for: String,
            native_not_implemented: String,
            unknown_target: String,
        },
        dev: Dev {
            local: String,
            serving: String,
            watch: String,
            port_switched: String,
            opening_browser: String,
            open_browser_failed: String,
            press_ctrl_c_to_stop: String,
            watch_error: String,
            source_changed: String,
            files_changed: String,
            rebuilt: String,
            manual_rebuild_triggered: String,
            opening_url: String,
            build_idle: String,
            build_rebuilding: String,
            check_ready: String,
            check_building: String,
            check_no_errors: String,
            check_compile_failed: String,
            shortcuts_full: String,
            shortcuts_compact: String,
        }
    }
);

const EN_CLI: &str = include_str!("../../res/locales/en/cli.toml");
const EN_DEV: &str = include_str!("../../res/locales/en/dev.toml");
const CHS_CLI: &str = include_str!("../../res/locales/chs/cli.toml");
const CHS_DEV: &str = include_str!("../../res/locales/chs/dev.toml");
const CHT_CLI: &str = include_str!("../../res/locales/cht/cli.toml");
const CHT_DEV: &str = include_str!("../../res/locales/cht/dev.toml");

fn toml_for(lang: Language) -> String {
    match lang {
        Language::CHS => [CHS_CLI, CHS_DEV].join("\n"),
        Language::CHT => [CHT_CLI, CHT_DEV].join("\n"),
        _ => [EN_CLI, EN_DEV].join("\n"),
    }
}

fn detect_system_language() -> Language {
    if let Some(locale) = sys_locale::get_locale() {
        return Language::from_system_locale(&locale);
    }
    Language::En
}

fn load_translations(language: Language) -> Translations {
    let primary = toml_for(language);
    match toml::from_str(&primary) {
        Ok(v) => v,
        Err(_) => {
            let en = toml_for(Language::En);
            toml::from_str(&en).expect("invalid en locale")
        }
    }
}

static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

pub fn translations() -> &'static Translations {
    TRANSLATIONS.get_or_init(|| load_translations(detect_system_language()))
}
