use std::hash::{Hash, Hasher};
use std::sync::LazyLock;

use iso639_enum::{IsoCompat, Language as IsoLang};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

impl Default for TextDirection {
    fn default() -> Self {
        TextDirection::Ltr
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptVariant {
    Default,
    Hans,
    Hant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    iso: IsoLang,
    script: ScriptVariant,
}

impl Hash for Language {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.iso as i32).hash(state);
        self.script.hash(state);
    }
}

impl Language {
    pub const ENGLISH: Self = Self { iso: IsoLang::Eng, script: ScriptVariant::Default };
    pub const CHINESE_SIMPLIFIED: Self = Self { iso: IsoLang::Cmn, script: ScriptVariant::Hans };
    pub const CHINESE_TRADITIONAL: Self = Self { iso: IsoLang::Cmn, script: ScriptVariant::Hant };
    pub const FRENCH: Self = Self { iso: IsoLang::Fra, script: ScriptVariant::Default };
    pub const RUSSIAN: Self = Self { iso: IsoLang::Rus, script: ScriptVariant::Default };
    pub const SPANISH: Self = Self { iso: IsoLang::Spa, script: ScriptVariant::Default };
    pub const ARABIC: Self = Self { iso: IsoLang::Ara, script: ScriptVariant::Default };
    pub const JAPANESE: Self = Self { iso: IsoLang::Jpn, script: ScriptVariant::Default };
    pub const KOREAN: Self = Self { iso: IsoLang::Kor, script: ScriptVariant::Default };

    pub fn new(iso: IsoLang) -> Self {
        Self { iso, script: ScriptVariant::Default }
    }

    pub fn with_script(iso: IsoLang, script: ScriptVariant) -> Self {
        Self { iso, script }
    }

    pub fn iso(&self) -> IsoLang {
        self.iso
    }

    pub fn script(&self) -> ScriptVariant {
        self.script
    }

    pub fn code(&self) -> String {
        let lang_code = self.iso.iso639_1().unwrap_or(self.iso.iso639_3());
        let region = self.default_region();
        format!("{}-{}", lang_code.to_uppercase(), region)
    }

    pub fn url_prefix(&self) -> String {
        match self.script {
            ScriptVariant::Hans => "zh-chs".to_string(),
            ScriptVariant::Hant => "zh-cht".to_string(),
            _ => self.iso.iso639_1().unwrap_or(self.iso.iso639_3()).to_lowercase(),
        }
    }

    pub fn native_name(&self) -> &str {
        match self.script {
            ScriptVariant::Hans => "简体中文",
            ScriptVariant::Hant => "繁體中文",
            _ => self.iso.autonym().unwrap_or(self.iso.name()),
        }
    }

    pub fn short_name(&self) -> &'static str {
        if let Some(a2) = self.iso.iso639_1() {
            match (a2, self.script) {
                ("en", _) => "EN",
                ("zh", ScriptVariant::Hans) => "简",
                ("zh", ScriptVariant::Hant) => "繁",
                ("zh", _) => "中",
                ("ja", _) => "日",
                ("ko", _) => "한",
                ("de", _) => "DE",
                ("fr", _) => "FR",
                ("es", _) => "ES",
                ("ar", _) => "ع",
                ("ru", _) => "РУ",
                (other, _) => match other {
                    "pt" => "PT",
                    "it" => "IT",
                    "nl" => "NL",
                    "pl" => "PL",
                    "tr" => "TR",
                    "vi" => "VI",
                    "th" => "TH",
                    "id" => "ID",
                    "hi" => "HI",
                    _ => "??",
                },
            }
        } else {
            "??"
        }
    }

    pub fn is_rtl(&self) -> bool {
        static RTL_ALPHA2: &[&str] = &[
            "ar", "he", "fa", "ur", "yi", "syr", "diq", "ckb", "ps", "ug", "ku", "dv", "sd",
        ];
        static RTL_ALPHA3: &[&str] = &[
            "adp", "afb", "ajp", "apc", "arb", "arz", "haz", "mhr", "phr", "shi", "sux", "tmh",
            "uzn", "ydd", "yud",
        ];
        if let Some(a2) = self.iso.iso639_1() {
            return RTL_ALPHA2.contains(&a2);
        }
        RTL_ALPHA3.contains(&self.iso.iso639_3())
    }

    pub fn direction(&self) -> TextDirection {
        if self.is_rtl() {
            TextDirection::Rtl
        } else {
            TextDirection::Ltr
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        match code {
            "en-US" | "en-us" | "en" => return Some(Self::ENGLISH),
            "zh-CHS" | "zh-chs" | "zh-Hans" | "zh-CN" | "zh-cn" => {
                return Some(Self::CHINESE_SIMPLIFIED)
            }
            "zh-CHT" | "zh-cht" | "zh-Hant" | "zh-TW" | "zh-tw" => {
                return Some(Self::CHINESE_TRADITIONAL)
            }
            "fr-FR" | "fr-fr" | "fr" => return Some(Self::FRENCH),
            "ru-RU" | "ru-ru" | "ru" => return Some(Self::RUSSIAN),
            "es-ES" | "es-es" | "es" => return Some(Self::SPANISH),
            "ar-SA" | "ar-sa" | "ar" => return Some(Self::ARABIC),
            "ja-JP" | "ja-jp" | "ja" => return Some(Self::JAPANESE),
            "ko-KR" | "ko-kr" | "ko" => return Some(Self::KOREAN),
            _ => {}
        }

        if let Some((lang, _region)) = code.split_once('-') {
            if let Ok(iso) = IsoLang::from_iso639_1(lang) {
                return Some(Self::new(iso));
            }
            if let Ok(iso) = IsoLang::from_iso639_3(lang) {
                return Some(Self::new(iso));
            }
        }

        if let Ok(iso) = IsoLang::from_iso639_1(code) {
            return Some(Self::new(iso));
        }
        if let Ok(iso) = IsoLang::from_iso639_3(code) {
            return Some(Self::new(iso));
        }

        None
    }

    pub fn from_url_prefix(prefix: &str) -> Option<Self> {
        Self::from_code(prefix)
    }

    pub fn from_iso(iso: IsoLang) -> Self {
        Self::new(iso)
    }

    fn default_region(&self) -> &'static str {
        if let Some(a2) = self.iso.iso639_1() {
            match a2 {
                "en" => "US",
                "zh" => match self.script {
                    ScriptVariant::Hant => "TW",
                    _ => "CN",
                },
                "pt" => "BR",
                "fr" => "FR",
                "de" => "DE",
                "es" => "ES",
                "ar" => "SA",
                "ja" => "JP",
                "ko" => "KR",
                "it" => "IT",
                "nl" => "NL",
                "pl" => "PL",
                "tr" => "TR",
                "vi" => "VN",
                "th" => "TH",
                "id" => "ID",
                "ms" => "MY",
                "hi" => "IN",
                "bn" => "BD",
                "ta" => "IN",
                "te" => "IN",
                "mr" => "IN",
                "gu" => "IN",
                "kn" => "IN",
                "ml" => "IN",
                "or" => "IN",
                "pa" => "IN",
                "my" => "MM",
                "km" => "KH",
                "lo" => "LA",
                "si" => "LK",
                "ne" => "NP",
                "fa" => "IR",
                "ur" => "PK",
                "he" => "IL",
                "sw" => "KE",
                "af" => "ZA",
                "ro" => "RO",
                "uk" => "UA",
                "cs" => "CZ",
                "sk" => "SK",
                "hu" => "HU",
                "bg" => "BG",
                "el" => "GR",
                "da" => "DK",
                "no" => "NO",
                "sv" => "SE",
                "fi" => "FI",
                "et" => "EE",
                "lv" => "LV",
                "lt" => "LT",
                "sr" => "RS",
                "hr" => "HR",
                "sl" => "SI",
                "ca" => "ES",
                "eu" => "ES",
                "gl" => "ES",
                "ru" => "RU",
                _ => "XX",
            }
        } else {
            "XX"
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Self::ENGLISH,
            Self::CHINESE_SIMPLIFIED,
            Self::CHINESE_TRADITIONAL,
            Self::FRENCH,
            Self::RUSSIAN,
            Self::SPANISH,
            Self::ARABIC,
            Self::JAPANESE,
            Self::KOREAN,
        ]
    }

    pub fn un_official_languages() -> &'static [Language] {
        &[
            Self::ARABIC,
            Self::CHINESE_SIMPLIFIED,
            Self::ENGLISH,
            Self::FRENCH,
            Self::RUSSIAN,
            Self::SPANISH,
        ]
    }

    pub fn east_asian_languages() -> &'static [Language] {
        &[
            Self::CHINESE_SIMPLIFIED,
            Self::CHINESE_TRADITIONAL,
            Self::JAPANESE,
            Self::KOREAN,
        ]
    }

    pub fn rtl_languages() -> &'static [Language] {
        &[Self::ARABIC]
    }

    pub fn default_lang() -> Self {
        Self::CHINESE_SIMPLIFIED
    }
}

static COMMON_ALPHA3: &[&str] = &[
    "eng", "zho", "cmn", "spa", "fra", "deu", "jpn", "kor", "ara", "por", "rus", "ita", "nld",
    "pol", "tur", "vie", "tha", "ind", "msa", "fil", "hin", "ben", "tam", "tel", "mar", "guj",
    "kan", "mal", "ori", "pan", "bur", "khm", "lao", "sin", "nep", "fas", "urd", "heb", "swa",
    "afr", "ron", "ukr", "ces", "slk", "hun", "bul", "ell", "dan", "nor", "swe", "fin", "est",
    "lav", "lit", "srp", "hrv", "slv", "cat", "eus", "glg",
];

static COMMON_LANGUAGES: LazyLock<Vec<Language>> = LazyLock::new(|| {
    COMMON_ALPHA3
        .iter()
        .filter_map(|&code| IsoLang::from_iso639_3(code).ok().map(Language::new))
        .collect()
});

pub fn common_languages() -> &'static [Language] {
    COMMON_LANGUAGES.as_slice()
}
