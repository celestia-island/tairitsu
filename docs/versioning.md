# Versioning strategy

This document describes how Tairitsu coordinates version numbers across the
Rust crates, the WIT packages, the TypeScript browser-glue package, and the
generated binding artefacts.

---

## 1. Rust crates

All crates in `packages/` follow [Semantic Versioning 2.0](https://semver.org/).

The workspace `Cargo.toml` defines a single `version` field used as the
canonical release version.  Individual crates track this version manually
(no workspace-level `version.workspace = true` key is in use at this time).

| Crate | Current version | Notes |
|---|---|---|
| `tairitsu-vdom` | 0.1.0 | Core trait definitions (Platform, EventData, …) |
| `tairitsu-web` | 0.1.0 | `web` + `wit-bindings` backends |
| `tairitsu-hooks` | 0.1.0 | Reactive hooks |
| `tairitsu-macros` | 0.1.0 | `rsx!` + `wit_world!` proc-macros |
| `tairitsu-runtime` | 0.1.0 | Wasmtime host for WIT components |
| `tairitsu-packager` | 0.1.0 | CLI build tool |
| `tairitsu-style` | 0.1.0 | CSS-in-Rust utilities |
| `tairitsu-e2e` | 0.1.0 | End-to-end test helpers |

**Breaking-change policy**: A change is **breaking** if it alters a `pub`
trait signature (`Platform`, `ElementHandle`, `EventHandle`, `EventData`),
modifies a WIT world or interface (see §2), or removes a public item.  All
breaking changes increment the **minor** version while the crate is pre-1.0
(per semver spec §4) and will increment the **major** version once 1.0 is
reached.

---

## 2. WIT packages

WIT packages live in `packages/browser-worlds/wit/`.  They are versioned
independently of the Rust crates.

### Current WIT packages

| Package | Version | Status | Location |
|---|---|---|---|
| `tairitsu-browser:full` | 0.1.0 | Hand-authored baseline | `packages/browser-worlds/wit/browser-full.wit` |
| `tairitsu-browser:dom` | 0.1.0 | Hand-authored | `packages/browser-worlds/wit/dom.wit` |
| `tairitsu-browser:events` | 0.1.0 | Hand-authored | `packages/browser-worlds/wit/events.wit` |

### Planned WIT packages

| Package | Target version | Status | Notes |
|---|---|---|---|
| `tairitsu-browser:full` | 0.2.0 | Planned | Generated from W3C WebIDL via `scripts/generate_browser_wit.py` |

### WIT versioning rules

1. **Patch** (`0.1.x → 0.1.x+1`): bug fixes that do not change the ABI.
   Adding an optional record field is **not** considered breaking in WIT.
2. **Minor** (`0.1.x → 0.2.0`): adding new interfaces or functions.
   Existing import / export signatures remain unchanged.
3. **Major** (`0.1.x → 1.0.0` or `1.x.y → 2.0.0`): removing or renaming
   existing interfaces, functions, or record fields.

WIT version changes must be accompanied by:
- An update to the corresponding `wit-bindgen::generate!` calls in all
  consuming crates.
- A changelog entry in the relevant crate's `CHANGELOG.md` (if present).

---

## 3. WIT registry override (`TAIRITSU_WIT_REGISTRY`)

The build system supports overriding the WIT package resolution path via the
`TAIRITSU_WIT_REGISTRY` environment variable.  This is useful for:

- Testing a local fork of `browser-worlds` before publishing.
- Pin-pointing a specific version in a monorepo CI pipeline.
- Experimenting with an alternative WIT package tree.

```sh
# Use a custom local WIT directory
TAIRITSU_WIT_REGISTRY=/path/to/my-wit cargo build --target wasm32-wasip2

# Use a specific published package version (future – requires registry support)
TAIRITSU_WIT_REGISTRY=tairitsu-registry:0.2.0 cargo build --target wasm32-wasip2
```

When unset, the default path is `packages/browser-worlds/wit` relative to the
workspace root.

### WIT dependency cache

`wit-bindgen::generate!()` respects the standard Cargo incremental build cache.
The generated Rust code is regenerated only when:

| Trigger | Behaviour |
|---|---|
| `.wit` file content changes | Full regeneration |
| `TAIRITSU_WIT_REGISTRY` value changes | Full regeneration |
| Rust source changes (no WIT change) | Cached – no regeneration |
| `cargo clean` | Cache cleared, full regeneration on next build |

Build scripts in crates that use generated bindings should emit:
```
cargo:rerun-if-changed=../browser-worlds/wit
cargo:rerun-if-env-changed=TAIRITSU_WIT_REGISTRY
```

---

## 4. TypeScript browser-glue package

The TypeScript package (`packages/browser-glue`) is versioned in its
`package.json`.  Its version roughly mirrors the WIT `tairitsu-browser:full`
package it implements.

| npm package | Version | Implements WIT |
|---|---|---|
| `@tairitsu/browser-glue` | 0.1.0 | `tairitsu-browser:full@0.1.0` |

A browser-glue release is required whenever a new WIT minor version introduces
new interfaces that the host must satisfy.

---

## 5. Versioning independence from `wasm-bindgen`

The `wit-bindings` backend (`WitPlatform`) has **no dependency** on
`wasm-bindgen` or `web-sys`.  This means:

- WIT interface versions evolve on their own schedule.
- `wasm-bindgen` API changes do not affect WIT-based consumers.
- The two backends (`web` and `wit-bindings`) can be bumped independently.

The `web` feature continues to track `wasm-bindgen ^0.2` and `web-sys ^0.3`
independently of any WIT package changes.

---

## 6. Pre-1.0 stability guarantees

Until all crates reach `1.0.0`:

- Public API changes may occur in any release.
- SemVer compatibility guarantees are best-effort for patch releases.
- Breaking changes will be documented in commit messages and, where a crate has
  one, in its `CHANGELOG.md`.

Users are encouraged to pin to a specific minor version during the pre-1.0 phase:
```toml
tairitsu-web = "=0.1.0"
```
