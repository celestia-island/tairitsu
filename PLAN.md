# PLAN: i18n Language System Overhaul — Full W3C Coverage

## Problem

`Language` enum in `packages/web/src/i18n/language.rs` is hand-rolled with only 9 variants.
This means every new language requires manual enum changes across 4+ files.
The website demo already supports 10 languages (including German) but tairitsu can't represent German.

## Goal

Replace the hand-written `Language` enum with **full W3C/ISO 639 coverage** via third-party crate,
so that tairitsu can represent ANY human language without code changes.

## Solution: `iso639_enum` crate

**Crate:** [`iso639_enum`](https://crates.io/crates/iso639_enum) v0.6.0 (MIT)
**Repo:** https://github.com/nickzana/rust-iso639

Provides:
- **~8000 variants** covering all of ISO 639-1 + ISO 639-3
- `.to_name()` — English name (`"German"`, `"Japanese"`)
- `.to_autonym()` — Native name (`"Deutsch"`, `"日本語"`)
- `.to_alpha3()` / `.to_alpha2()` — ISO 639-3/639-1 codes (`"deu"`, `"de"`)
- `Language::from(alpha3)` / `Language::from(alpha2)` — lookup by code
- `serde` support built-in
- `phf`-backed O(1) lookups

### Why this crate over alternatives:

| Crate | Coverage | Autonyms | License | Notes |
|-------|----------|----------|---------|-------|
| `iso639_enum` | **ISO 639-1 + 639-3 (~8000)** | Yes | MIT | Best fit |
| `iso639-1` | ISO 639-1 only (184) | No | MIT | Too limited |
| `language-tag` | BCP47 parsing | No | MIT | Parser only, no enum |
| `unic-langid` | Unicode CLDR | Partial | Unicode | Heavy dependency |

## Files to Change

### 1. `packages/web/Cargo.toml` — Add dependency

```toml
[dependencies]
# ... existing deps ...
iso639_enum = "0.6"
```

### 2. `packages/web/src/i18n/language.rs` — Complete rewrite

**Current state:** 159 lines, 9 hand-written variants, match arms for everything.

**New design:**

```rust
//! Language definitions backed by iso639_enum (full ISO 639-1/639-3 coverage).

pub use iso639_enum::Language as IsoLanguage;

use std::sync::LazyLock;

/// Text direction for layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}

/// RTL language alpha2/alpha3 codes (ISO 639).
/// Covers Arabic, Hebrew, Persian, Urdu, Yiddish, Syriac, etc.
static RTL_CODES: LazyLock<std::collections::HashSet<&'static str>> = LazyLock(|| {
    let mut set = std::collections::HashSet::new();
    // ISO 639-1 RTL codes
    for &code in &["ar", "he", "fa", "ur", "yi", "syr", "diq", "ckb", "ps", "otk",
                     "sam", "arc", "ae", "ug", "ku", "dv", "sd"] {
        set.insert(code);
    }
    // ISO 639-3 RTL codes (additional)
    for &code in &["adp", "afb", "ajp", "apc", "arb", "arz", "haz", "mhr",
                     "phr", "shi", "sux", "tmh", "uzn", "ydd", "yud"] {
        set.insert(code);
    }
    set
};

/// Common/well-known languages subset for UI display (language picker, etc.)
/// This is a curated list — NOT the full set of supported languages.
/// Any IsoLanguage can be used; this just filters for common UI choices.
pub fn common_languages() -> &'static [IsoLanguage] {
    // Map from alpha3 codes to commonly-used languages
    // Use a static lazy vec or const array once stable
    COMMON_LANGUAGES.as_slice()
}

// We need a way to store specific IsoLanguage variants statically.
// Option A: Use alpha3 string lookup at runtime (simplest)
// Option B: Const-eval array (requires nightly or const-expr-in-patterns)
//
// Going with Option A for stability:
static COMMON_ALPHA3: &[&str] = &[
    "eng", "zho", "cmn", "zht", "spa", "fra", "deu", "jpn", "kor",
    "ara", "por", "rus", "ita", "nld", "pol", "tur", "vie", "tha",
    "ind", "msa", "fil", "hin", "ben", "tam", "tel", "mar", "guj",
    "kan", "mal", "ori", "pan", "bur", "khm", "lao", "sin", "nep",
    "fas", "urd", "heb", "swa", "afr", "ron", "ukr", "ces", "slk",
    "hun", "bul", "grc", "ell", "dan", "nor", "swe", "fin", "est",
    "lav", "lit", "srp", "hrv", "slv", "cat", "eus", "glg", "fin",
];

static COMMON_LANGUAGES: LazyLock<Vec<IsoLanguage>> = LazyLock(|| {
    COMMON_ALPHA3.iter()
        .filter_map(|&code| IsoLanguage::from(code).ok())
        .collect()
});

impl IsoLanguageExt for IsoLanguage {
    /// BCP47-style locale tag (e.g., "en-US", "zh-CN", "ja-JP")
    fn bcp47(&self) -> String {
        format!("{}-{}", self.to_alpha2().unwrap_or_else(|| self.to_alpha3()), self.default_region())
    }

    /// Short URL-friendly prefix (e.g., "en", "zh-cn", "ja")
    fn url_prefix(&self) -> String {
        self.to_alpha2().unwrap_or_else(|| self.to_alpha3()).to_lowercase()
    }

    /// Native autonym name (e.g., "English", "简体中文", "Deutsch")
    fn display_name(&self) -> &str {
        self.to_autonym()
    }

    /// English name fallback when autonym unavailable
    fn english_name(&self) -> &str {
        self.to_name()
    }

    /// Short display code (e.g., "EN", "简", "DE")
    fn short_name(&self) -> &str {
        // For common languages, provide abbreviated native names
        match self.to_alpha2() {
            Some("en") => "EN",
            Some("zh") => "简",
            Some("ja") => "日",
            Some("ko") => "한",
            Some("de") => "DE",
            Some("fr") => "FR",
            Some("es") => "ES",
            Some("ar") => "ع",
            Some("ru") => "РУ",
            _ => &self.to_alpha2().unwrap_or(self.to_alpha3()).to_uppercase()[..2.min(
                self.to_alpha2().unwrap_or(self.to_alpha3()).len()
            )],
        }
    }

    fn is_rtl(&self) -> bool {
        let alpha2 = self.to_alpha2();
        if let Some(a2) = alpha2 {
            return RTL_CODES.contains(a2);
        }
        RTL_CODES.contains(self.to_alpha3())
    }

    fn direction(&self) -> TextDirection {
        if self.is_rtl() { TextDirection::Rtl } else { TextDirection::Ltr }
    }

    /// Default region/subtag for common languages
    fn default_region(&self) -> &str {
        match self.to_alpha2() {
            Some("en") => "US",
            Some("zh") => "CN",
            Some("pt") => "BR",
            Some("fr") => "FR",
            Some("de") => "DE",
            Some("es") => "ES",
            Some("ar") => "SA",
            Some("ja") => "JP",
            Some("ko") => "KR",
            Some("it") => "IT",
            Some("nl") => "NL",
            Some("pl") => "PL",
            Some("tr") => "TR",
            Some("vi") => "VN",
            Some("th") => "TH",
            Some("id") => "ID",
            Some("ms") => "MY",
            Some("hi") => "IN",
            Some("bn") => "BD",
            Some("ta") => "IN",
            Some("te") => "IN",
            Some("mr") => "IN",
            Some("gu") => "IN",
            Some("kn") => "IN",
            Some("ml") => "IN",
            Some("or") => "IN",
            Some("pa") => "IN",
            Some("my") => "MM",
            Some("km") => "KH",
            Some("lo") => "LA",
            Some("si") => "LK",
            Some("ne") => "NP",
            Some("fa") => "IR",
            Some("ur") => "PK",
            Some("he") => "IL",
            Some("sw") => "KE",
            Some("af") => "ZA",
            Some("ro") => "RO",
            Some("uk") => "UA",
            Some("cs") => "CZ",
            Some("sk") => "SK",
            Some("hu") => "HU",
            Some("bg") => "BG",
            Some("el") => "GR",
            Some("da") => "DK",
            Some("no") => "NO",
            Some("sv") => "SE",
            Some("fi") => "FI",
            Some("et") => "EE",
            Some("lv") => "LV",
            Some("lt") => "LT",
            Some("sr") => "RS",
            Some("hr") => "HR",
            Some("sl") => "SI",
            Some("ca") => "ES",
            Some("eu") => "ES",
            Some("gl") => "ES",
            _ => "XX",
        }
    }

    /// Parse from any common code format (BCP47, alpha2, alpha3, legacy)
    fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();

        // Try BCP47 style first (e.g., "en-US", "zh-CN", "zh-Hans")
        if let Some((lang, _region)) = code.split_once('-') {
            if let Ok(l) = IsoLanguage::from(lang) {
                return Some(l);
            }
        }

        // Try direct alpha2/alpha3 lookup
        if let Ok(l) = IsoLanguage::from(code) {
            return Some(l);
        }

        // Legacy mappings for backward compat
        match code {
            "zh-CHS" | "zh-chs" | "zh-Hans" => IsoLanguage::from("zho").ok(),
            "zh-CHT" | "zh-cht" | "zh-Hant" => IsoLanguage::from("zho").ok(), // TODO: distinguish zho variants
            _ => None,
        }
    }
}

pub trait IsoLanguageExt {
    fn bcp47(&self) -> String;
    fn url_prefix(&self) -> String;
    fn display_name(&self) -> &str;
    fn english_name(&self) -> &str;
    fn short_name(&self) -> &str;
    fn is_rtl(&self) -> bool;
    fn direction(&self) -> TextDirection;
    fn default_region(&self) -> &str;
}
```

### 3. `packages/web/src/i18n/mod.rs` — Update docs + re-exports

- Update doc comment: `"9 languages"` → `"all ISO 639 languages"`
- Re-export `IsoLanguageExt` trait
- Keep backward-compatible type alias: `pub type Language = IsoLanguage;`

### 4. `packages/web/src/i18n/context.rs` — Minimal changes

- All `HashMap<Language, ...>` still works because `IsoLanguage` implements `Hash + Eq`
- Tests that use `Language::English` → change to `IsoLanguage::from("eng").unwrap()`
- Or keep convenience alias: `pub use iso639_enum::Language;`

### 5. `packages/web/src/i18n/macros.rs` — Likely no changes

The `t!()` macro uses string-based key lookup, not Language variants directly.

### 6. `packages/web/src/i18n/loader.rs` — No structural changes

TOML loader returns `HashMap<String, String>`, language-agnostic.

### 7. `examples/website/src/hooks.rs` — Simplify dramatically

**Current state:** Hand-written `Language` enum with 10 variants, duplicate data.

**After:** Delegate entirely to tairitsu's `IsoLanguage`:
```rust
pub use tairitsu_web::i18n::IsoLanguage as Language;
// detect_language() stays same logic but returns IsoLanguage
// supported_languages() returns common_languages() subset
```

### 8. `examples/website/src/components/aside_footer.rs` — Remove `map_to_tairitsu_lang()`

**Before:**
```rust
fn map_to_tairitsu_lang(lang: &hooks::Language) -> TairitsuLanguage { ... }
```

**After:** Direct usage — no mapping needed since both use same type.

### 9. `examples/website/src/i18n_init.rs` — Update HashMap keys

Change from old `Language::*` variants to `IsoLanguage::from("eng").unwrap()` etc.

## Migration Path (Backward Compatibility)

```rust
// In mod.rs, provide aliases for existing consumers:
pub use iso639_enum::Language;

// Convenience re-exports so existing code doesn't break immediately:
#[deprecated(note = "Use IsoLanguage::from(\"eng\") instead")]
pub const English: IsoLanguage = unsafe { std::mem::transmute(0u8) }; // NO — can't do this

// Better approach: just let people use .from() and update call sites.
// The API surface is small enough (internal crate) that we can update all callers.
```

Actually, since `Language` variants are used in **match arms** throughout the codebase,
and the new enum has ~8000 variants, exhaustive matches are impractical.
Solution: all consumer code should use **method calls** (`lang.code()`, `lang.native_name()`)
not pattern matching on variants. The current codebase already does this correctly
— pattern matches are only inside `language.rs` itself.

## What Does NOT Change

- `I18nState` struct — still holds `HashMap<Language, HashMap<String, String>>`
- `I18nProvider` API — `new()`, `provide()`, `set_locale()`, `locale()`, `t()` unchanged
- `t!()` / `tr!()` macros — string-key based, language-agnostic
- TOML translation file format — unchanged
- Portal / VDOM / hooks systems — completely unrelated

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| `iso639_enum` adds compile time (8000 variant enum) | Only affects `i18n` feature; phf lookups are O(1) at runtime |
| Existing `Language::English` pattern matches break | Limited to `language.rs` internals + tests; update all call sites |
| Autonym quality varies for obscure languages | Acceptable — obscure languages rarely used in UI pickers |
| Serde representation changes (variant names differ) | Translation files use string keys, not serialized Language enums |
| `Hash + Eq` behavior on IsoLanguage | Already implemented by the crate |

## Execution Order

1. Add `iso639_enum = "0.6"` to `Cargo.toml`
2. Rewrite `language.rs` with `IsoLanguageExt` trait
3. Update `mod.rs` re-exports
4. Fix test compilations in `context.rs`
5. Update `examples/website/src/hooks.rs` to use `IsoLanguage`
6. Remove `map_to_tairitsu_lang()` from `aside_footer.rs`
7. Update `i18n_init.rs` call sites
8. Run full test suite
9. Verify website demo language switching works end-to-end
