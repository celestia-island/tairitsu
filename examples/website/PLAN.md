# Tairitsu Website — Comprehensive Audit & Gap Analysis

## Status: Round 1 Complete (2026-04-12)

---

## 1. Architecture Overview

### What This Is
A WASM Component Model website (`wasm32-wasip2`) built with:
- **tairitsu_vdom** + **tairitsu_macros::rsx!** macro (NOT Dioxus)
- **tairitsu_hooks** (use_signal, use_state, etc.)
- **tairitsu-web** (browser + wit-bindings features)
- **hikari-components** + **hikari-palette** (design system CSS)
- **tairitsu-packager** (build tool, dev server)

### Build Output
- `target/tairitsu-dist/` — static files served by dev server
- WASM: ~514KB release / ~12MB debug
- Entry: `index.html` with inline SPA router JS → loads `tairitsu_website.wasm`

### File Structure
```
src/
  lib.rs              # Bootstrap: run() / tairitsu_component_bootstrap()
  app.rs              # Root VNode tree assembler
  i18n.rs             # Locale enum + SiteText (8 locales, NOT wired to UI)
  components/
    mod.rs            # top_nav(), sidebar(), aside_footer()
  pages/
    mod.rs            # Module declarations
    home.rs           # Home page (hero + cards)
    not_found.rs      # 404 page
    state_test.rs     # Reactive state test (use_signal, Rc<RefCell>)
    dom_ops_test.rs   # DOM ops test (set_style, get_bcr, set_attribute)
    guides/           # 5 guide pages (quick-start, workspace-map, etc.)
    system/           # 5 system pages (overview, runtime, wit-pipeline, etc.)
    packages/         # 2 pages (overview, list)
  styles/
    main.scss         # SCSS source → compiled to styles.css by packager
public/
  styles.css          # Dark theme overrides (compiled from main.scss)
  styles/bundle.css   # Hikari design system (~50KB minified)
  styles/spa.css     # SPA routing + markdown styles
  styles/animations.css # Animation preset classes
```

---

## 2. Route Table (16 routes + fallback)

| Path | Page ID | Source | Content |
|------|---------|--------|---------|
| `/` | `home` | home.rs | Hero + 2 card grids |
| `/guides` | `guides-overview` | guides/mod.rs | 4-card index |
| `/guides/quick-start` | `guides-quick-start` | quick_start.rs | 5 code blocks |
| `/guides/workspace-map` | `guides-workspace-map` | workspace_map.rs | Mermaid diagram |
| `/guides/build-test-release` | `guides-build-test-release` | build_test_release.rs | 3 code blocks |
| `/guides/migration` | `guides-migration` | migration.rs | 2 code blocks |
| `/guides/glossary` | `guides-glossary` | glossary.rs | Definition list |
| `/system` | `system-overview` | system/mod.rs | Card grid (**has duplicates**) |
| `/system/runtime` | `system-runtime` | runtime.rs | Code block + lists |
| `/system/wit-pipeline` | `system-wit-pipeline` | wit_pipeline.rs | WIT code block |
| `/system/web-backends` | `system-web-backends` | web_backends.rs | **HTML table** |
| `/system/versioning` | `system-versioning` | versioning.rs | TOML code block |
| `/packages` | `packages-overview` | packages/mod.rs | Layer list |
| `/packages/list` | `packages-list` | packages/mod.rs | 7 package desc |
| `/dom-ops-test` | `dom-ops-test` | dom_ops_test.rs | DOM op tests |
| *(fallback)* | `not-found` | not_found.rs | 404 |

---

## 3. Implementation Status Matrix

### 3.1 Layout Components (✅ ALL IMPLEMENTED)

| Class | Status | Location |
|-------|--------|---------|
| `.hi-layout` / `.hi-layout-dark` / `.hi-layout-has-sidebar` | ✅ | app.rs body_class |
| `.hi-background` | ✅ | app.rs |
| `.hi-header` / `.hi-header-sticky` / `.hi-header-md` | ✅ | top_nav() |
| `.hi-header-left` / `.hi-header-right` / `.hi-header-toggle` | ✅ | top_nav() |
| `.hi-header-brand` | ✅ | top_nav() |
| `.hi-aside` / `.hi-aside-drawer` / `.hi-aside-lg` | ✅ | sidebar() |
| `.hi-aside-content` / `.hi-aside-footer` | ✅ | sidebar(), aside_footer() |
| `.hi-layout-body` / `.hi-layout-overlay` | ✅ | app.rs |
| `.hi-layout-main` / `.hi-layout-content` | ✅ | app.rs |

### 3.2 Navigation Components (✅ STRUCTURE IMPLEMENTED)

| Class | Status | Notes |
|-------|--------|-------|
| `.hi-menu` / `.hi-menu-vertical` / `.hi-menu-compact` | ✅ | Sidebar uses this |
| `.hi-menu-item` / `.hi-menu-item-active` / `.hi-menu-item-inner` | ✅ | Active via JS router |
| `.hi-menu-submenu` / `.hi-menu-submenu-list-open` | ✅ | All forced open |
| `.hi-menu-submenu-title` / `.hi-menu-submenu-title-inner` | ✅ | With arrow SVGs |
| `.hi-arrow` / `.hi-arrow-down` / `.hi-arrow-14` | ✅ | Chevron-right style |
| `.hi-glow-wrapper` / `.hi-glow-blur-medium` / `.hi-glow-subtle` | ✅ | Static (no mouse tracking) |
| `.hi-menu-height-compact` / `.hi-menu-item-wrapper` | ✅ | Proper sizing |

### 3.3 Buttons (✅ IMPLEMENTED)

| Class | Status | Used In |
|-------|--------|---------|
| `.hi-button-primary` / `.hi-button-secondary` | ✅ | home.rs CTA buttons |
| `.hi-button-lg` / `.hi-button-md` | ✅ | home.rs, not_found.rs |
| `.hi-button-borderless` / `.hi-icon-button` / `.hi-icon-button-40` | ✅ | aside_footer() |
| `.hi-button-width-auto` / `.hi-justify-center` | ✅ | home.rs |

### 3.4 Content Components (✅ IMPLEMENTED)

| Class | Status | Used In |
|-------|--------|---------|
| `.card-grid` / `.card` / `.card__title` / `.card__body` | ✅ | home.rs, all overviews |
| `.hi-code-block` (with language-* hints) | ✅ | All doc pages |
| `.hi-markdown-content` | ✅ | All doc pages |
| `.hi-container` / `.hi-container-md` | ✅ | home.rs, not_found.rs |
| `.hi-section` / `.hi-section-lg` / `.hi-section-body` | ✅ | home.rs |
| `.hi-text-sm` through `.hi-text-2xl` | ✅ | Multiple pages |
| `.hi-text-center` / `.hi-text-primary` / `.hi-text-secondary` | ✅ | Multiple pages |
| `.hi-p-4` / `.hi-mb-4` / `.hi-mb-6` / `.hi-row` / `.hi-row-gap-md` | ✅ | home.rs |

### 3.5 CSS Files (✅ CLEAN)

| File | Size | Purpose | Status |
|------|------|---------|--------|
| `bundle.css` | ~50KB | Hikari design system | ✅ Loaded first |
| `spa.css` | ~4.2KB | SPA routing + markdown | ✅ Cleaned (old sidebar/nav removed) |
| `styles.css` | ~3.9KB | Dark theme overrides | ✅ Fixed (was old bloat) |
| `animations.css` | exists | Animation presets | ✅ Present |

---

## 4. Gaps & Issues (Prioritized)

### P0 — Broken Interactivity (must fix)

| # | Issue | Current State | Fix Needed |
|---|-------|--------------|-----------|
| 1 | **Drawer toggle non-functional** | Hamburger button has no onclick | Add JS: toggle `.hi-aside-drawer-open` + `.hi-layout-overlay-open` |
| 2 | **Drawer overlay close missing** | Overlay div has no onclick | Add click handler to close drawer |
| 3 | **Theme toggle button dead** | Sun icon button, no handler | Add JS: swap `hi-layout-light/dark`, swap icon ☾/☀ |
| 4 | **Language switcher dead** | "A" button, no handler | Wire to i18n.rs locales, reload on change |
| 5 | **Glow effect is static** | `--glow-x/y` hardcoded to 50% | Add mousemove listener updating CSS vars on submenu items |
| 6 | **Header right section empty** | No nav links, no search, no GitHub | Add nav links matching sidebar structure |

### P1 — Missing Pages/Content

| # | Missing Item | Hikari Has It? | Priority |
|---|------------|---------------|----------|
| 7 | **Component showcase pages** (Layer 1/2/3) | Yes (30+ components demoed) | High |
| 8 | **System palette page** (color swatches) | Yes (11 colors × 3 themes) | Medium |
| 9 | **CSS utilities demo page** | Yes (layout/spacing/typography/colors) | Medium |
| 10 | **Icons showcase page** | Yes (MDI icons with sizes) | Low |
| 11 | **Animations live demo page** | Yes (21 presets with interactive toggles) | Medium |
| 12 | **Form demo page** | Yes (login form) | Low |
| 13 | **Dashboard page** | Yes (stats + charts + table) | Low |
| 14 | **Interactive/reactive demo page** | Yes (switches, counters, inputs) | Medium (state_test.rs is a partial start) |

### P2 — Bugs & Polish

| # | Issue | Details |
|---|-------|---------|
| 15 | **system/mod.rs duplicate cards** | Runtime/WIT/Web-backends each appear twice in overview grid |
| 16 | **not_found.rs hardcoded Chinese** | Should use i18n or at least English fallback |
| 17 | **state_test.rs input stubbed** | `oninput: move |_: InputEvent\| {}` is TODO |
| 18 | **state_test.rs remove buttons dead** | List items have × button but no onclick |
| 19 | **state_test.rs computed values don't react** | Number inputs lack oninput handlers |
| 20 | **Mermaid diagrams are raw text** | `div.mermaid` contains graph TD text, no mermaid.js init |
| 21 | **Home breadcrumb redundant** | Shows "Home / Home" (should just be "Home") |
| 22 | **No favicon.ico in public/** | Referenced in Cargo.toml but may not exist |

### P3 — Infrastructure Gaps

| # | Gap | Impact |
|---|-----|--------|
| 23 | **Dynamic doc loading missing** | All content compiled into WASM; can't load .md at runtime |
| 24 | **i18n.rs not wired to UI** | 8 locales defined but nothing renders them conditionally |
| 25 | **No keyboard navigation** | Arrow keys, Escape to close drawer, etc. |
| 26 | **No search functionality** | Neither client-side nor server-side |
| 27 | **Hot reload may have issues** | Dev daemon had serving problems during audit |

---

## 5. VDOM & Browser-Glue Capabilities (What Works)

### Working in Event Callbacks
- `set_style(element_handle, property, value)` ✅ (tested dom_ops_test.rs)
- `get_bounding_client_rect(element_handle)` → DomRect ✅
- `set_attribute(element_handle, name, value)` ✅
- Mouse events: `MouseEvent.target`, `.client_x/y`, `.button`, modifier keys ✅
- Input events: `InputEvent.data`, `.input_type` ✅
- Reactive signals: `use_signal` with `.get()` / `.set()` ✅

### VDOM Limitations Affecting Website
- No runtime classList.toggle (only add_if at render time)
- No spread syntax (`..children`) in rsx!
- No function calls inside rsx! macro
- No format!() macro in attribute values
- Dynamic content requires manual VNode::Element construction chain

---

## 6. Comparison: Tairitsu vs Hikari-Legacy

| Aspect | Hikari-Legacy | Tairitsu | Gap |
|--------|-------------|----------|-----|
| **Routes** | 70+ (with aliases) | 16 | -54 routes |
| **Pages** | 16 unique | 13 unique | -3 pages |
| **Components demoed** | 30+ | 0 showcase pages | All missing |
| **Theme toggle** | ✅ Working | ❌ Placeholder | Need JS |
| **Language switch** | ✅ 10 locales | ❌ Placeholder | Need JS |
| **Glow mouse-follow** | ✅ CSS vars + JS | ⚠️ CSS only (static) | Need JS |
| **Drawer toggle** | ✅ hamburger + overlay | ❌ Button exists, no handler | Need JS |
| **Header nav links** | ✅ Components/System/Demos/GitHub | ❌ Empty div | Need HTML+JS |
| **Animation demos** | ✅ Interactive preset page | ❌ CSS only | Need page+JS |
| **Reactive components** | ✅ Switch/Counter/Input | ⚠️ Partial (state_test) | Stub handlers |
| **i18n** | ✅ 9 locales rendered | ❌ Defined but unused | Need wiring |
| **Dynamic docs** | ✅ Lazy loading | ❌ Static only | Architecture limit |
| **Sidebar style** | `<details>/<summary>` BEM | `.hi-menu` system | Different but functional parity needed |
| **Body class** | `hi-layout-light` | `hi-layout-dark` | ✅ Correct (THEMED difference) |

---

## 7. Verification Rounds Log

### Round 1 (2026-04-12) — Structural Audit
- [x] Full source code traversal (25 Rust files)
- [x] Route table extraction (16 routes)
- [x] Component implementation matrix (66 hi-* classes used)
- [x] CSS file analysis (4 files, all clean)
- [x] JavaScript interactivity audit (router works, drawer/theme/glow broken)
- [x] VDOM/browser-glue capability check
- [x] Gap analysis vs hikari-legacy (27 gaps identified)
- [ ] Visual screenshot verification — **PENDING** (need working HTTP server)
- [ ] Interaction testing (drawer, glow, theme) — **PENDING**
- [ ] Cross-browser compatibility — **PENDING**

### Round 2 — [NOT STARTED]
### Round 3 — [NOT STARTED]

**Stop condition**: 3 consecutive complete rounds with ALL of:
- Visual screenshot verification (every page renders correctly)
- Source code scan (every file matches expected structure)
- Interaction testing (drawer opens/closes, glow follows mouse, theme toggles, routing works, all buttons clickable)
- Zero P0/P1 issues remaining

---

## 8. Next Actions (Priority Order)

1. **Fix drawer toggle JS** — hamburger button + overlay click-to-close
2. **Fix theme toggle JS** — sun/moon icon swap + layout class toggle
3. **Add header nav links** — Components/System/GitHub in header-right
4. **Implement glow mouse-follow** — mousemove listener on .hi-glow-wrapper elements
5. **Fix system/mod.rs duplicates** — Remove 2 duplicate cards
6. **Wire i18n to UI** — At minimum make not_found.rs use SiteText
7. **Fix state_test.rs stubs** — Add proper input/remove handlers
8. **Create component showcase page** — Layer 1 (buttons, inputs, switches)
9. **Visual verification round** — Screenshot every page, compare with hikari-legacy
