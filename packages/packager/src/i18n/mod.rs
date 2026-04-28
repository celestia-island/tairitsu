use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    Ar,
    #[default]
    En,
    Es,
    Fr,
    Ja,
    Ko,
    Ru,
    Zhs,
    Zht,
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
                return Self::Zht;
            }
            return Self::Zhs;
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
        build: Build {
            check_target: String,
            compile_wasm: String,
            bundle_assets: String,
            component_wrapper: String,
            generate_html: String,
            build_complete: String,
            initial_build_succeeded: String,
            building_component: String,
            creating_output_dir: String,
            copying_wasm: String,
            copying_browser_glue: String,
            copying_static_assets: String,
            compiling_scss: String,
            package_bytes: String,
            inlined_css: String,
            generated_route_files: String,
            compilation_failed: String,
            compiled_with: String,
            error_count: String,
            warning_count: String,
            dev_server_watch: String,
            dev_server: String,
            workspace_label: String,
            package_label: String,
            output_label: String,
            daemon_already_running: String,
            use_daemon_hint: String,
            port_in_use_daemon: String,
            port_in_use_foreign_daemon: String,
            port_in_use_other: String,
            force_killing: String,
            use_different_port: String,
            force_takeover: String,
            executable_label: String,
            unknown_target: String,
            rebuild_panicked: String,
            failed_bind: String,
            failed_bind_range: String,
            hint_daemon_running: String,
            hint_port_held: String,
            failed_create_watcher: String,
            cannot_watch: String,
            failed_flatten_barrel: String,
            browser_glue_missing: String,
            extra_public_missing: String,
            css_not_found: String,
            wrapper_transpile_succeeded_no_output: String,
            wrapper_transpile_failed: String,
            wrapper_execute_failed: String,
            wrapper_not_found: String,
            no_output: String,
        },
        icons: Icons {
            fetching: String,
            cached: String,
            building: String,
            generated: String,
            listing: String,
            found: String,
            source_not_implemented: String,
        }
    }
);

const EN_CLI: &str = include_str!("../../res/i18n/packager/en/cli.toml");
const EN_DEV: &str = include_str!("../../res/i18n/packager/en/dev.toml");
const EN_BUILD: &str = include_str!("../../res/i18n/packager/en/build.toml");
const EN_ICONS: &str = include_str!("../../res/i18n/packager/en/icons.toml");
const ZHS_CLI: &str = include_str!("../../res/i18n/packager/zhs/cli.toml");
const ZHS_DEV: &str = include_str!("../../res/i18n/packager/zhs/dev.toml");
const ZHS_BUILD: &str = include_str!("../../res/i18n/packager/zhs/build.toml");
const ZHS_ICONS: &str = include_str!("../../res/i18n/packager/zhs/icons.toml");
const ZHT_CLI: &str = include_str!("../../res/i18n/packager/zht/cli.toml");
const ZHT_DEV: &str = include_str!("../../res/i18n/packager/zht/dev.toml");
const ZHT_BUILD: &str = include_str!("../../res/i18n/packager/zht/build.toml");
const JA_CLI: &str = include_str!("../../res/i18n/packager/ja/cli.toml");
const JA_DEV: &str = include_str!("../../res/i18n/packager/ja/dev.toml");
const JA_BUILD: &str = include_str!("../../res/i18n/packager/ja/build.toml");
const KO_CLI: &str = include_str!("../../res/i18n/packager/ko/cli.toml");
const KO_DEV: &str = include_str!("../../res/i18n/packager/ko/dev.toml");
const KO_BUILD: &str = include_str!("../../res/i18n/packager/ko/build.toml");
const FR_CLI: &str = include_str!("../../res/i18n/packager/fr/cli.toml");
const FR_DEV: &str = include_str!("../../res/i18n/packager/fr/dev.toml");
const FR_BUILD: &str = include_str!("../../res/i18n/packager/fr/build.toml");
const ES_CLI: &str = include_str!("../../res/i18n/packager/es/cli.toml");
const ES_DEV: &str = include_str!("../../res/i18n/packager/es/dev.toml");
const ES_BUILD: &str = include_str!("../../res/i18n/packager/es/build.toml");
const RU_CLI: &str = include_str!("../../res/i18n/packager/ru/cli.toml");
const RU_DEV: &str = include_str!("../../res/i18n/packager/ru/dev.toml");
const RU_BUILD: &str = include_str!("../../res/i18n/packager/ru/build.toml");
const AR_CLI: &str = include_str!("../../res/i18n/packager/ar/cli.toml");
const AR_DEV: &str = include_str!("../../res/i18n/packager/ar/dev.toml");
const AR_BUILD: &str = include_str!("../../res/i18n/packager/ar/build.toml");

fn toml_for(lang: Language) -> String {
    match lang {
        Language::Zhs => [ZHS_CLI, ZHS_DEV, ZHS_BUILD, ZHS_ICONS].join("\n"),
        Language::Zht => [ZHT_CLI, ZHT_DEV, ZHT_BUILD, EN_ICONS].join("\n"),
        Language::Ja => [JA_CLI, JA_DEV, JA_BUILD, EN_ICONS].join("\n"),
        Language::Ko => [KO_CLI, KO_DEV, KO_BUILD, EN_ICONS].join("\n"),
        Language::Fr => [FR_CLI, FR_DEV, FR_BUILD, EN_ICONS].join("\n"),
        Language::Es => [ES_CLI, ES_DEV, ES_BUILD, EN_ICONS].join("\n"),
        Language::Ru => [RU_CLI, RU_DEV, RU_BUILD, EN_ICONS].join("\n"),
        Language::Ar => [AR_CLI, AR_DEV, AR_BUILD, EN_ICONS].join("\n"),
        _ => [EN_CLI, EN_DEV, EN_BUILD, EN_ICONS].join("\n"),
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
