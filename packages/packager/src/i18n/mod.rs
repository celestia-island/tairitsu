use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    CHS,
    CHT,
    #[default]
    En,
    Ja,
    Ko,
    Fr,
    Es,
    Ru,
    Ar,
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
            packaging_for: String,
            packaging_not_implemented: String,
            preview_starting: String,
            preview_not_implemented: String,
            autofix_not_implemented: String,
            init_starting: String,
            init_creating_dir: String,
            init_writing_cargo: String,
            init_writing_lib: String,
            init_project_created: String,
            init_next_steps: String,
        },
        dev: Dev {
            local: String,
            serving: String,
            watch: String,
            port_switched: String,
            opening_browser: String,
            open_browser_failed: String,
            press_ctrl_c_to_stop: String,
            watching_for_changes: String,
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
            stopping: String,
        },
        doctor: Doctor {
            running: String,
            check_complete: String,
            project_healthy: String,
            project_has_issues: String,
            summary: String,
            checks_passed: String,
            warnings_count: String,
            errors_count: String,
            report_header: String,
            category_dependencies: String,
            category_environment: String,
            category_configuration: String,
            category_build: String,
            category_migration: String,
        }
    }
);

const EN_CLI: &str = include_str!("../../res/locales/en/cli.toml");
const EN_DEV: &str = include_str!("../../res/locales/en/dev.toml");
const EN_DOCTOR: &str = include_str!("../../res/locales/en/doctor.toml");
const CHS_CLI: &str = include_str!("../../res/locales/chs/cli.toml");
const CHS_DEV: &str = include_str!("../../res/locales/chs/dev.toml");
const CHS_DOCTOR: &str = include_str!("../../res/locales/chs/doctor.toml");
const CHT_CLI: &str = include_str!("../../res/locales/cht/cli.toml");
const CHT_DEV: &str = include_str!("../../res/locales/cht/dev.toml");
const CHT_DOCTOR: &str = include_str!("../../res/locales/cht/doctor.toml");
const JA_CLI: &str = include_str!("../../res/locales/ja/cli.toml");
const JA_DEV: &str = include_str!("../../res/locales/ja/dev.toml");
const JA_DOCTOR: &str = include_str!("../../res/locales/ja/doctor.toml");
const KO_CLI: &str = include_str!("../../res/locales/ko/cli.toml");
const KO_DEV: &str = include_str!("../../res/locales/ko/dev.toml");
const KO_DOCTOR: &str = include_str!("../../res/locales/ko/doctor.toml");
const FR_CLI: &str = include_str!("../../res/locales/fr/cli.toml");
const FR_DEV: &str = include_str!("../../res/locales/fr/dev.toml");
const FR_DOCTOR: &str = include_str!("../../res/locales/fr/doctor.toml");
const ES_CLI: &str = include_str!("../../res/locales/es/cli.toml");
const ES_DEV: &str = include_str!("../../res/locales/es/dev.toml");
const ES_DOCTOR: &str = include_str!("../../res/locales/es/doctor.toml");
const RU_CLI: &str = include_str!("../../res/locales/ru/cli.toml");
const RU_DEV: &str = include_str!("../../res/locales/ru/dev.toml");
const RU_DOCTOR: &str = include_str!("../../res/locales/ru/doctor.toml");
const AR_CLI: &str = include_str!("../../res/locales/ar/cli.toml");
const AR_DEV: &str = include_str!("../../res/locales/ar/dev.toml");
const AR_DOCTOR: &str = include_str!("../../res/locales/ar/doctor.toml");

fn toml_for(lang: Language) -> String {
    match lang {
        Language::CHS => [CHS_CLI, CHS_DEV, CHS_DOCTOR].join("\n"),
        Language::CHT => [CHT_CLI, CHT_DEV, CHT_DOCTOR].join("\n"),
        Language::Ja => [JA_CLI, JA_DEV, JA_DOCTOR].join("\n"),
        Language::Ko => [KO_CLI, KO_DEV, KO_DOCTOR].join("\n"),
        Language::Fr => [FR_CLI, FR_DEV, FR_DOCTOR].join("\n"),
        Language::Es => [ES_CLI, ES_DEV, ES_DOCTOR].join("\n"),
        Language::Ru => [RU_CLI, RU_DEV, RU_DOCTOR].join("\n"),
        Language::Ar => [AR_CLI, AR_DEV, AR_DOCTOR].join("\n"),
        _ => [EN_CLI, EN_DEV, EN_DOCTOR].join("\n"),
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
