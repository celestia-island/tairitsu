# PLAN: i18n Language System Overhaul — Full W3C Coverage

## Status: COMPLETE

All tasks completed in commit `e32b798` on `dev` branch.

## What was done

Replaced the hand-written 9-variant `Language` enum with a `struct Language` backed by `iso639_enum` crate (~8000 ISO 639-1/639-3 languages).

### Key design decisions

1. **Newtype wrapper** — `iso639_enum::Language` doesn't derive `Hash`, so we wrap it in `struct Language { iso: IsoLang, script: ScriptVariant }` with manual `Hash` impl
2. **ScriptVariant** — Preserves Chinese Simplified vs Traditional distinction (both map to `Cmn` in ISO 639, but differ in BCP 47 script subtag)
3. **Constants** — `Language::ENGLISH`, `Language::CHINESE_SIMPLIFIED`, etc. provide backward-compatible named access
4. **`from_code()`** — Handles legacy codes (`zh-CHS`, `zh-CHT`) plus standard BCP47/ISO codes, plus any unknown ISO 639 code

### Files changed

| File | Change |
|------|--------|
| `packages/web/Cargo.toml` | Added `iso639_enum = "0.6"` (optional, gated by `i18n` feature) |
| `packages/web/src/i18n/language.rs` | Complete rewrite: struct with `Hash`, `from_code()`, `native_name()`, RTL detection, `common_languages()` |
| `packages/web/src/i18n/mod.rs` | Updated doc comment (9 → full ISO 639 coverage) |
| `packages/web/src/i18n/context.rs` | Updated tests to use `Language::ENGLISH` constants |
| `examples/website/Cargo.toml` | Added `i18n` feature to tairitsu-web dep |
| `examples/website/src/i18n.rs` | Replaced hand-written `Locale` enum with re-exported `Language` |
| `examples/website/src/pages/not_found.rs` | Updated `Locale::EnUs` → `Language::ENGLISH` |

### Verification

- All 19 i18n unit tests pass
- `cargo check -p tairitsu-web --features full` — clean
- `cargo check -p tairitsu-website` — clean
