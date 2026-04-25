# Packager: SSR-Style CSS Inline Injection

## Background

The current CSS loading pipeline relies on external `<link rel="stylesheet">` tags in `<head>`:

```
Browser loads index.html
  → <link href="/styles/bundle.css"> (network request)
  → <link href="/styles/spa.css"> (network request)
  → <link href="/styles/animations.css"> (network request)
  → ... WASM boots, renders VDOM into #app ...
  → stylesReady() polls document.styleSheets
  → removes [data-booting] guard, content becomes visible
```

**Problem**: If any CSS file returns 404 (wrong output path, SCSS compilation failure, or dev server misconfiguration), the page renders as **unstyled/blank** — all component styles are missing.

**Root causes observed**:
1. Output directory path resolution may differ between build steps and dev server ServeDir root
2. SCSS compilation can fail silently for certain @use/@import chains
3. Race condition: WASM may render before `<link>`-fetched CSS is parsed on slow networks

## Goal

Inline compiled CSS directly into the generated `index.html` so that **zero network requests are needed for initial render**. External `<link>` tags become optional fallbacks for cache warming.

## Approach: Use Existing StyleInjector

The codebase already has a fully-tested `StyleInjector` (`packages/packager/src/styles/injector.rs`) that is **not connected to the main build pipeline**:

```rust
// Existing, unused API:
StyleInjector::generate_injection_code(css)   // → <script> creating <style> block
StyleInjector::inject_into_html(html, css)     // → inserts <script> before </head>
StyleInjector::generate_cssom_injection(css)   // → rule-by-rule sheet.insertRule()
```

### Preferred Strategy: `<style>` Block Injection (not `<script>`)

Instead of the current JS-based injection (`document.createElement('style')`), inject CSS as a **static `<style>` element** in `<head>`. This is simpler, faster (no JS execution needed), and works even with JS disabled:

```html
<head>
    <!-- Injected inline — zero network dependency -->
    <style>/* bundle.css contents (~15KB minified) */</style>
    <style>/* spa.css contents */</style>
    <style>/* animations.css contents */</style>

    <!-- Optional: keep <link> for browser preload/cache -->
    <!-- <link rel="stylesheet" href="/styles/bundle.css"> -->
</head>
```

---

## Implementation Plan

### Step 1: Add inline-CSS method to StyleInjector

**File**: `packages/packager/src/styles/injector.rs`

Add a new method that generates a static `<style>` block (not a `<script>`):

```rust
pub fn generate_style_block(&self, css: &str) -> String {
    format!("<style>\n{}\n</style>", css)
}

pub fn inject_style_into_html(&self, html: &str, css: &str) -> Result<String> {
    let style_block = self.generate_style_block(css);
    if let Some(head_pos) = html.find("</head>") {
        let mut result = String::with_capacity(html.len() + style_block.len());
        result.push_str(&html[..head_pos]);
        result.push_str(&style_block);
        result.push_str(&html[head_pos..]);
        Ok(result)
    } else {
        Ok(html.to_string())
    }
}
```

### Step 2: Hook StyleInjector into HTML generation

**File**: `packages/packager/src/wasm/mod.rs`
**Function**: `generate_component_html_with_output_dir()` (line 1114)

After generating the HTML string (line 1141), read each compiled CSS file from the output directory and inject it:

```rust
// After line 1149 (let html = format!(...)):
use crate::styles::injector::StyleInjector;

let injector = StyleInjector::new();

// Read compiled CSS files from output_dir
let css_files = ["styles/bundle.css", "styles/spa.css", "styles/animations.css"];

for css_path in &css_files {
    let full_path = output_dir.join(css_path);
    if let Ok(css) = std::fs::read_to_string(&full_path) {
        html = injector.inject_style_into_html(&html, &css)?;
    } else {
        log::warn!("CSS file not found for inline injection: {}", css_path);
    }
}
```

**Important**: SCSS compilation (Step 3 of `build_component()`) runs **before** HTML generation (Step 5). So CSS files are guaranteed to exist on disk when HTML is generated.

### Step 3: Make inline-CSS configurable via Cargo.toml

**File**: `packages/packager/src/config/mod.rs`

Add a new config option:

```toml
[package.metadata.tairitsu.html]
inline_css = true          # default: false (backward compatible)
# head = "..."             # existing: external <link> tags
```

When `inline_css = true`:
- CSS is inlined as `<style>` blocks in generated `index.html`
- External `<link>` tags from `head` are still included (for preload hints / cache warming)

When `inline_css = false` (default, current behavior):
- No changes — uses existing `<link>` tag approach

### Step 4: Update website Cargo.toml to enable inline CSS

**File**: `hikari/examples/website/Cargo.toml`

```toml
[package.metadata.tairitsu.html]
inline_css = true
title = "Hikari UI"
head = """
<link rel="stylesheet" href="/styles/bundle.css">
<link rel="stylesheet" href="/styles/spa.css">
<link rel="stylesheet" href="/styles/animations.css">
<!-- SPA router script ... -->
"""
```

### Step 5: Handle dev-mode hot-reload

When SCSS files change during `just dev --daemon`:
1. SCSS recompiles → writes new `.css` files to output dir
2. Hot-reload triggers page refresh
3. HTML regeneration picks up new CSS content
4. Fresh `<style>` blocks contain updated CSS

**No additional work needed** — the existing hot-reload flow already handles this.

---

## Alternative Approaches Considered

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Static `<style>` block** (recommended) | Zero JS needed; works without JS; simplest | Increases HTML size by ~20KB | Best for our use case |
| JS `createElement('style')` (existing) | Already implemented | Requires JS execution; async FOUC risk | Keep as fallback |
| CSSOM `sheet.insertRule()` | Per-rule error handling | Slower for large CSS; complex | Overkill |
| Data-URI in `<link>` | Cacheable | Size overhead (base33%); browser limits | Not worth it |
| HTTP/2 push | No HTML changes | Complex server config; not always supported | Depends on deployment |

---

## File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `packages/packager/src/styles/injector.rs` | Modify | Add `generate_style_block()` + `inject_style_into_html()` |
| `packages/packager/src/config/mod.rs` | Modify | Add `html.inline_css: bool` config field |
| `packages/packager/src/wasm/mod.rs` | Modify | Call StyleInjector after HTML template generation |
| `hikari/examples/website/Cargo.toml` | Modify | Set `inline_css = true` |

## Testing Checklist

- [ ] `cargo test -p tairitsu-packager` passes (add unit tests for new injector methods)
- [ ] `just dev` serves page with correctly styled components
- [ ] Dev hot-reload updates inline CSS on SCSS change
- [ ] Production build (`just build`) includes inlined CSS
- [ ] Page renders correctly even when `/styles/bundle.css` returns 404
- [ ] No regression when `inline_css = false` (default behavior unchanged)
