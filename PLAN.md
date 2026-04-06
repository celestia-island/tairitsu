# Tairitsu Framework - Implementation Status

## Current State

All planned features have been implemented and verified:

- `cargo check --workspace` — zero errors
- `cargo clippy --workspace` — zero warnings (core packages)
- `cargo test --workspace` — all tests passing

## i18n System

Reactive internationalization system in `packages/web/src/i18n/` (feature: `i18n`):

| Component | API | Status |
|-----------|-----|--------|
| `I18nProvider` | `new(translations, locale)`, `provide()`, `set_locale()` | Done |
| `I18nState` | `t(key)`, `t_or_key(key)`, `with_fallback(locale)` | Done |
| `Language` | 9 languages: EN, ZH-CHS, ZH-CHT, FR, RU, ES, AR, JA, KO | Done |
| `t!("key.subkey")` | Dot-path translation with key fallback + interpolation | Done |
| `tr!("key.subkey")` | Dot-path translation returning `Option<String>` | Done |
| `use_locale()` | Returns current `Language` from context | Done |
| `set_locale(lang)` | Switches locale at runtime (shared `Rc<RefCell>` state) | Done |
| `load_toml_flat()` | Flattens nested TOML to dot-path `HashMap<String, String>` | Done |
| `provide_i18n()` | Shorthand for `I18nProvider::new().provide()` | Done |
| `I18nKeys` | Typed struct for compile-time TOML deserialization | Done |

---

## BUG: `cargo install --path packages/packager` fails

**Severity**: High — blocks downstream projects from installing the `tairitsu` CLI globally.

### Bug 1: `include_str!` references `.js` but file is `.ts`

**File**: `packages/packager/src/wasm/mod.rs:478`

```rust
let loader = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/wasm/component-wrapper-loader.template.js"  // <-- .js
))
```

**Actual file**: `packages/packager/src/wasm/component-wrapper-loader.template.ts`

The file was renamed from `.js` to `.ts` in commit `5417bfc`, briefly deleted, then re-added as `.ts` in commit `a75b485`. But the `include_str!` path was never updated.

**Fix**: Change `.js` to `.ts` in the `include_str!` call.

### Bug 2: `cargo install` cannot resolve workspace dependencies

`cargo install --path <subdir>` compiles the crate in **isolation**, not as part of the workspace. Several dependencies in `packages/packager/Cargo.toml` use `workspace = true` (e.g. `tokio`, `tracing`, `serde`, `serde_json`, `toml`, `chrono`, `clap`, etc.), which requires workspace context.

**Symptoms**: 100+ "use of unresolved crate" errors for `tracing`, `tokio`, `serde`, `clap`, `axum`, `indicatif`, `grass`, `regex`, `walkdir`, etc.

**Workaround**: Use `cargo run --package tairitsu-packager -- <args>` from the workspace root instead of `cargo install`.

**Fix options**:
1. Replace `workspace = true` with explicit version paths in `packages/packager/Cargo.toml` for direct dependencies (not workspace member crates).
2. OR document that `cargo install` is not supported and users should use `cargo run --package tairitsu-packager` from a clone of the repo.
3. OR add a `[profile.release.package.tairitsu-packager]` section and publish to crates.io (then `cargo install tairitsu-packager` would work from the published crate which has resolved versions).
