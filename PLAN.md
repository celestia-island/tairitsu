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

### Bug 2 (FIXED): Missing `[dependencies]` header after `[target.'cfg(unix)'.dependencies]`

**File**: `packages/packager/Cargo.toml`

The `[target.'cfg(unix)'.dependencies]` section (containing only `daemonize`) was placed in the middle of the `[dependencies]` block. In TOML, all key-value pairs after a `[section]` header belong to that section until the next `[section]` header. This meant **every dependency after `daemonize`** (`serde`, `tracing`, `tokio`, `axum`, `indicatif`, `chrono`, `clap`, `toml`, `grass`, `walkdir`, etc.) was silently placed under `[target.'cfg(unix)'.dependencies]`.

**Symptoms**: On Windows, 318 "use of unresolved crate" errors. `cargo tree --depth 1` showed only 5 direct dependencies (the ones before the target section). `cargo metadata` revealed almost all deps had `"target": "cfg(unix)"`.

**Fix**: Moved `[target.'cfg(unix)'.dependencies]` to the end of `Cargo.toml`, after all `[dependencies]` entries. Also changed `just install-packager` from `cargo install --path` to `cargo build --release --package` + copy binary (more robust, avoids workspace isolation issues).

### Note: `cargo install --path` workspace isolation

`cargo install --path <subdir>` compiles the crate in **isolation**, not as part of the workspace. The `just install-packager` recipe now uses `cargo build --release --package` + binary copy to avoid this.
