# Tairitsu + Hikari 体系健康检查报告

> 审查时间: 2026-05-11
> 审查范围: tairitsu (v0.4.5) + hikari (v0.1.8) 作为完整技术栈
> hikari 侧的精简条目见 `hikari/PLAN.md`

---

## 一、体系架构概览

```
┌──────────────────────────────────────────────────┐
│                    Tairitsu                       │
│  WASM Component Runtime + VDOM + CLI + MCP       │
│  ┌──────────┐ ┌──────┐ ┌──────────┐             │
│  │ Runtime  │ │ VDOM  │ │ Packager │             │
│  │ (Docker  │ │(React-│ │ (Dev Srv,│             │
│  │  模型)   │ │  like)│ │  Build)  │             │
│  └──────────┘ └──────┘ └──────────┘             │
│  ┌──────────┐ ┌──────┐ ┌──────────┐             │
│  │  Macros  │ │ Hooks │ │  Style   │             │
│  │ (rsx!…)  │ │(Solid │ │ (SCSS,   │             │
│  │          │ │ -like) │ │  CSS)    │             │
│  └──────────┘ └──────┘ └──────────┘             │
├──────────────────────────────────────────────────┤
│            Hikari (构建在 Tairitsu 之上)           │
│  ┌──────────┐ ┌──────┐ ┌──────────┐             │
│  │Palette   │ │Theme │ │Animation │             │
│  │(500+色彩)│ │(CSS  │ │(GSAP风格)│             │
│  │          │ │ 变量)│ │          │             │
│  └──────────┘ └──────┘ └──────────┘             │
│  ┌──────────┐ ┌──────┐ ┌──────────┐             │
│  │Components│ │ Icons│ │  Extra   │             │
│  │  (40+ UI │ │ (MDI)│ │ Components│            │
│  │   组件)  │ │      │ │ (NodeGraph│            │
│  │          │ │      │ │ , RTE)    │             │
│  └──────────┘ └──────┘ └──────────┘             │
└──────────────────────────────────────────────────┘
```

Tairitsu 提供基础设施层 — WASM Component Model 运行时、虚拟 DOM 引擎、proc-macro、CLI 工具链。
Hikari 在其上构建 UI 组件库（40+ 组件、500 色版、动画引擎、主题系统），通过 `path = "../tairitsu/packages/..."` 硬链接依赖 Tairitsu。

---

## 二、技术优势（做得好）

### 架构层面

| 领域 | 优势 | 详情 |
|------|------|------|
| WASM 运行时选型 | 前沿且正确 | 使用 wasmtime + Component Model + WIT，抛弃传统的 wasm-bindgen/web-sys 路径。WASI preview 2 (`wasm32-wasip2`) 原生支持，类型安全且不依赖 JavaScript glue |
| Docker 模型抽象 | 直观且可扩展 | `Image → Container → Registry` 三层抽象，用户心智模型清晰 |
| WIT 自动流水线 | 端到端自动化 | WebIDL 抓取 → WIT 生成 → TypeScript glue 生成 → npm 打包，整个流水线自动化（含每周 CI 自动同步） |
| Hikari 三层架构 | 清晰且可组合 | Foundation（palette/theme/animation）→ Core（components/icons）→ Extra（node-graph），feature flag 支持按需引入 |

### 代码层面

| 领域 | 优势 | 详情 |
|------|------|------|
| RSX 宏 | 语法设计精良 | proc-macro 解析器处理了嵌套组件、条件渲染、循环、字符串插值等复杂语法，接近 JSX 表现力 |
| 响应式系统 | 设计合理 | signals + memo + effect 模式，类似 Solid.js。`use_effect` 自动追踪依赖，`use_memo` 惰性派生 |
| SCSS 集成 | 类型安全 | `include_scss!` 宏在编译期提取类名并生成枚举，消除 class name 字符串的运行时错误 |
| 色彩系统 | 文化价值高 | 500+ 中国传统色作为类型安全常量，中文名（石青、朱砂）保证文化真实性，英文 API 保证互操作性 |
| 动画引擎 | 功能完整 | 30+ 缓动函数、状态机、timeline、tween、`prefers-reduced-motion` 无障碍支持 |

### 工程层面

| 领域 | 优势 | 详情 |
|------|------|------|
| CI 覆盖 | Tairitsu 6 workflows | 测试/格式/clippy/发布/视觉回归/WIT同步，覆盖全面 |
| 测试框架 | 多层次 | 单元测试、集成测试、浏览器测试(CDP)、E2E(WebDriver)、视觉回归(Python)，层次分明 |
| 多语言文档 | 10 语言 | 含阿拉伯语的 RTL 支持，中文优先的国际化策略 |
| 代码整洁度 | Clippy 零警告 | 两个项目均通过 `cargo clippy --workspace -- -D warnings` |

---

## 三、技术劣势（需改进）

### 🔴 严重问题 (P0)

| # | 项目 | 位置 | 问题 | 影响 |
|---|------|------|------|------|
| C1 | Hikari | `docs/**/*.md` (66处) | ~~**文档严重过时**~~ **已确认无问题** — 迁移指南中 Dioxus 引用是有意保留的，Lucide 引用已不存在 |
| C2 | Tairitsu | `vdom/src/diff.rs:109-140` | ~~**key-based reconciliation 未实现**~~ **已修复** — 实现了 LIS 启发式 key 匹配算法，包含 MoveChild/ReorderChildren 补丁类型，支持有 key/无 key 混合子节点 |
| C3 | Hikari | `tabs.rs:270`, `switch.rs:188-214`, `select.rs` | ~~**无障碍形同虚设**~~ **已修复** — Tabs/Switch/Menu 已有键盘处理器，Select 已添加 onkeydown (Escape/ArrowDown) + tabindex + ConditionalGlow 重构 |

### 🟡 中等问题 (P1-P2)

| # | 项目 | 位置 | 问题 |
|---|------|------|------|
| M1 | Tairitsu | `macros/src/svg.rs:241` vs `vdom/src/svg.rs:124` | ~~**两个不同的 SVG 净化器**~~ **已修复** — macros 版本从 regex 替换为与 vdom 相同的三阶段解析器策略（remove_script_tags + remove_event_handlers + sanitize_urls） |
| M2 | Tairitsu | `runtime/src/container.rs:65` | ~~`HostState::default()` 通过 `.expect()` panic~~ **已修复** — 直接构建 WasiCtx 而不调用可能失败的 `inherit_stdio()`/`inherit_network()` |
| M3 | Tairitsu | `runtime/src/registry.rs` (8处) | ~~Mutex `unwrap()` 在 poisoning 时会 panic~~ **已修复** — 全部改为 `.expect("registry ... lock poisoned")` |
| M4 | Hikari | `button.rs:142-171`, `input.rs:133-162` (5处) | ~~**CSS 变量覆盖代码大量重复**~~ **已修复** — 提取了 `build_css_vars_style()` + `CssVarEntry` 共享工具 |
| M5 | Hikari | `button.rs:203-227`, `select.rs` (5处) | ~~**Glow 条件包装重复**~~ **已修复** — 封装了 `ConditionalGlow` 组件，Select 已迁移 |
| M6 | Hikari | `animation/src/easing.rs` (55个函数) | ~~**55+ 个浪费的单行包装函数**~~ **已修复** — 替换为有实质逻辑的 `custom`/`bezier`/`steps` 等工具函数 |
| M7 | Hikari | `extra-components/src/node_graph/node.rs` | ~~**`NodeState` 和 `Node` 近似重复**~~ **已修复** — 重命名为 `NodePlacement`（最小状态）和 `NodeView`（完整渲染数据），语义明确 |
| M8 | Hikari | `theme/styles/foundation.scss` vs 组件 CSS | ~~**两套互不相连的 CSS 变量命名系统**~~ **已修复** — `index.scss` 的 `:root` 和 `[data-theme="tairitsu"]` 均改为以 `--hi-color-*` 为规范名、`--hi-*` 为别名，与 ThemeProvider `base.scss` 保持一致 |
| M9 | Hikari | `radio_group.rs:99` | ~~`Box::leak` 获取 `&'static str`~~ **已修复** — `RadioContext.name` 改为 `String` 类型，消除内存泄漏 |

### 🟢 轻微问题 (P3)

| # | 项目 | 位置 | 问题 |
|---|------|------|------|
| L1 | Tairitsu | `macros/src/rsx.rs:119` | ~~`syn::parse_str` 的 `unwrap()` 在遇到 Rust 关键字时会 panic~~ **已修复** — fallback 到 `syn::parse_quote!` |
| L2 | Tairitsu | `macros/src/scss.rs:234-260` | ~~类名提取器不处理 `url()` 中的点、`&.class` 嵌套、`.a.b.c` 链式选择器~~ **已修复** — 增加 `url()` 跳过、`&` 父选择器透传、链式选择器分隔、逗号/子选择器边界检测 |
| L3 | Tairitsu | `runtime/src/dynamic/deserialize.rs` (10处) | ~~整数截断 via `as u8` 没有范围检查~~ **已修复** — 改用 `u8::try_from(v)` 等带范围检查的转换 |
| L4 | Hikari | `avatar.rs:80` | ~~唯一使用 `#[props()]` 语法~~ **已确认使用 `#[define_props]`** — 无需修改 |
| L5 | Hikari | `icons/src/lib.rs:41` | ~~回退 SVG 缺少 `fill="currentColor"`~~ **已确认包含** — `DEFAULT_SVG` 已有 `fill="currentColor"` |
| L6 | Hikari | `components/src/lib.rs:14` | ~~`pub mod hooks` 声明空目录~~ **已修复** — 移除了不存在的模块声明 |
| L7 | Hikari | `extra-components/src/node.rs:137` | ~~`NodePlugin::handle_input` 空实现~~ **已修复** — 改为带默认空实现的 trait 方法 |

---

## 四、CI / 工程基础设施问题

### CI Bug

| # | 项目 | 文件 | 问题 |
|---|------|------|------|
| B1 | Tairitsu | `fmt.yml:6` | ~~路径过滤器引用为 `clippy.yml` 而非 `fmt.yml`~~ **已修复** → `fmt.yml` |
| B2 | Tairitsu | `visual-regression.yml:12` | ~~push 触发用 `[main, dev]`~~ **已修复** → `[master, dev]` |
| B3 | Tairitsu | `wit-sync.yml:47` | ~~Cache key 用 `${{ github.run_id }}`~~ **已修复** → 基于脚本文件 hash |
| B4 | Hikari | `fmt.yml:24` | ~~在安装 Rust toolchain **之前**运行 `cargo build`~~ **已修复** — 调整步骤顺序 |

### CI 缺失

| 缺失项 | 两个项目 | 影响 |
|--------|-----------|------|
| `cargo audit` / `cargo deny` | ~~均无~~ **已添加** | Tairitsu: `audit.yml`; Hikari: `security.yml` |
| 依赖审查 (diff on PRs) | 均无 | 无法发现恶意依赖引入 |
| 许可证合规检查 | 均无 | 无法发现许可证冲突 |
| MSRV 检查 | 均无 | 无法保证最低 Rust 版本兼容性 |
| Visual regression | Hikari 无 | UI 组件库应比运行时更需要视觉回归 |

### 工程问题

| # | 项目 | 问题 |
|---|------|------|
| E1 | Hikari | ~~`just fmt` 用 `cargo fmt --all`，但 CI 用 `cargo +nightly fmt --all -- --check --unstable-features`~~ **已修复** — justfile 已使用 `cargo +nightly fmt --all -- --unstable-features` |
| E2 | Hikari | ~~`justfile:25` 硬编码 `../tairitsu/packages/packager/Cargo.toml`~~ **已修复** — 改用 `env_var_or_default("TAIRITSU_PACKAGER", justfile_directory() / ..)` |
| E3 | Hikari | ~~CI 每个 PR 都 `python3 scripts/icons/fetch_mdi_icons.py` 从网络获取，未缓存~~ **已修复** — fmt/clippy/test/publish 均添加 `actions/cache` 步骤 |
| E4 | Tairitsu | ~~`justfile` 中两套 WIT 管线并存（`gen-wit-*` 和 `wit-gen-*`），使用不同的缓存目录~~ **已修复** — `gen-wit-*` 改为 deprecated 别名指向 `wit-*`，`clean-idl-cache` 清理两个缓存目录 |
| E5 | Tairitsu | ~~`README.md:125-126` 两个文档链接指向不存在的目录~~ **已修复** → `docs/zhs/`、`docs/en/` |
| E6 | Tairitsu | ~~`README.md:603` "What's New in 0.3.0" 但当前版本是 0.4.5~~ **已修复** → 0.4.5 |

---

## 五、测试质量评估

| 项目 | 测试总数 | 通过率 | 问题 |
|------|----------|--------|------|
| **Tairitsu** | 940+ `#[test]` 标注 | **0 失败** | diff.rs 20 个测试；registry 11 个测试；container 6 个测试 |
| **Hikari** | 836+ 通过 (25 binaries) | **0 失败** | basic_components_tests 升级为结构化断言（find_elements_by_tag + 属性验证） |

### Tairitsu 失败测试

~~`resolve_options_default_values` 测试已通过~~ — 测试期望值与代码默认值一致（`https://wit.tairitsu.dev`），之前报告的失败可能来自中间状态。

---

## 六、依赖问题

### 悬空/可能未使用的依赖

| 项目 | 依赖 | 理由 |
|------|------|------|
| Hikari | ~~`once_cell`~~ | **已移除** — 替换为 `std::sync::OnceLock` |
| Hikari | `gloo`, `gloo-net` | Dioxus 时代的 `web-sys` 包装，迁移后可能不再需要 |
| Hikari | `chrono` + `unstable-locales` | chrono 社区维护堪忧 |
| Hikari | `tokio` features = `["full"]` | UI 组件库只需 `rt` + 基本 I/O |
| Tairitsu | `[patch.crates-io]` for `tairitsu-macros` | workspace member 已自动优先，可能残留 |

### Docker 问题 (Hikari) — **全部已修复**

| # | 问题 | 状态 |
|---|------|------|
| D1 | ~~`docker-compose.yml:5` 硬编码绝对路径~~ | ✅ 改为相对 `context: ..` |
| D2 | ~~`website.Dockerfile:25` 从 GitHub clone tairitsu~~ | ✅ 改为 COPY 本地仓库 |
| D3 | ~~`website.Dockerfile:28` `pip install --break-system-packages`~~ | ✅ 改为 venv |
| D4 | ~~`base-selenium.Dockerfile` 和 `screenshot-selenium.Dockerfile` 90% 重复~~ | ✅ 合并为单一 Dockerfile |
| D5 | ~~Selenium 容器以 root 运行~~ | ✅ 改为 `USER seluser` |
| D6 | ~~缺少 `.dockerignore`~~ | ✅ 已存在 |

---

## 七、改进建议（按优先级排序）

### P0 — 立即修复（阻塞用户）

- [x] **[Hikari] 重写全部文档** — 已确认无问题：Dioxus 引用仅存在于迁移指南中（有意保留），Lucide 引用已不存在
- [x] **[Tairitsu] 实现 key-based reconciliation** — 在 `diff.rs:diff_children` 中使用 `VElement.key` 进行高效子节点匹配（LIS 启发式算法），含 MoveChild/ReorderChildren 补丁类型
- [x] **[Hikari] 补齐键盘无障碍** — Switch、Tabs、Menu、Select 均已添加 Enter/Space/Arrow 键盘事件处理器 + tabindex

### P1 — 高优先级（影响可靠性）

- [x] **[Tairitsu] 统一 SVG 净化器** — 用手写解析器替换 `macros/src/svg.rs` 中的 regex 版本，与 `vdom/src/svg.rs` 采用相同的解析策略（remove_script_tags + remove_event_handlers + sanitize_urls 三阶段）
- [x] **[Tairitsu] 修复 `HostState::default()` 的 panic** — 改为直接构建 `WasiCtxBuilder::new().build()` 而不调用 `.expect()`
- [x] **[Hikari] 消除 CSS 变量重复代码** — 抽离 `build_css_vars_style()` + `CssVarEntry` 共享工具
- [x] **[Hikari] 封装 ConditionalGlow 组件** — 替换 button/input/icon_button/select 中的重复
- [x] **[Both] CI 添加 `cargo audit` + `cargo deny`** — Tairitsu: `.github/workflows/audit.yml`; Hikari: `.github/workflows/security.yml`

### P2 — 中优先级（改善开发体验）

- [x] **[Tairitsu] 清理 `unwrap()` on mutex** — `registry.rs` 的 8 处 `lock().unwrap()` 改为 `lock().expect("registry ... lock poisoned")`，提供诊断信息
- [x] **[Hikari] 删除 55 个冗余 easing 包装函数**
- [x] **[Hikari] 统一 Avatar 的 props 定义方式**
- [x] **[Hikari] 合并 `NodeState` 和 `Node`** — 重命名为 `NodePlacement`/`NodeView`，语义明确
- [x] **[Hikari] 统一 CSS 变量命名** — `index.scss` `:root` 和 `[data-theme]` 均改为以 `--hi-color-*` 为规范名、`--hi-*` 为别名，与 ThemeProvider `base.scss` 一致

### P3 — 低优先级（长期优化）

- [x] **[Hikari] E2E 视觉回归加入 CI** — `.github/workflows/visual-regression.yml` + Playwright capture/compare 脚本 + baseline 截图集
- [x] **[Tairitsu] 修复 CI bugs** — `fmt.yml` 路径过滤器→`fmt.yml`、`visual-regression.yml` 分支名→`master`、`wit-sync.yml` 缓存 key→基于脚本 hash
- [x] **[Tairitsu] 合并 justfile 两套 WIT 管线** — `gen-wit-*` 改为 deprecated 别名指向 `wit-*`，`clean-idl-cache` 清理两个缓存目录
- [x] **[Hikari] 清理悬空依赖** — `once_cell` 已移除（替换为 `std::sync::OnceLock`）、`gloo`/`gloo-net` 已移除、`chrono` 保留（date-picker 使用）
- [x] **[Tairitsu] Upstream `load_toml_flat`** — `tairitsu-web/src/i18n/loader.rs` 已有完整实现并导出 `pub use`。Hikari website 内联副本待下个版本发布后删除。
- [x] **[Tairitsu] 增加核心模块测试覆盖率**（registry、container、image、diff）— 37 个新测试：registry (11), container (6), diff (20)
- [x] **[Hikari] 组件测试升级** — basic_components_tests 从 16 个 panic-only 测试升级为 19 个结构化断言（find_elements_by_tag 辅助函数 + 属性/类名验证）；background 已升级
- [x] **[Tairitsu] 修复 `resolve_options_default_values` 过期断言** — 测试已通过，期望值与代码一致
- [x] **[Hikari] MDI 图标获取加入 CI 缓存** — fmt/clippy/test/publish 均添加 `actions/cache` 步骤
- [x] **[Hikari] 删除空模块 `components/src/hooks/` 声明**
- [x] **[Tairitsu] `README.md` 更新死链接和版本号** — 修复 `docs/zh-CHS/` → `docs/zhs/`、`docs/en-US/` → `docs/en/`、版本号 0.3.0 → 0.4.5

---

## 八、总结

### 整体评分

| 维度 | Tairitsu (运行时+工具链) | Hikari (UI 组件库) |
|------|--------------------------|---------------------|
| 架构设计 | ★★★★☆ | ★★★★☆ |
| 代码质量 | ★★★★☆ | ★★★☆☆ |
| 测试覆盖 | ★★★☆☆ | ★★★☆☆ |
| 文档质量 | ★★★★☆ | ★☆☆☆☆ |
| CI/工程化 | ★★★★☆ | ★★★★☆ |
| 安全性 | ★★★★☆ | ★★★★☆ |
| 无障碍访问 | N/A | ★☆☆☆☆ |

### 核心判断

**Tairitsu** 作为基础设施层是合格的 — 架构前沿（WASM Component Model）、virtual DOM 和 proc-macro 设计精良、CI 覆盖全面、核心模块测试覆盖改善。主要短板在部分模块测试仍不足。

**Hikari** 作为 UI 组件库已大幅改善 — 代码重复问题已清理、CSS 变量命名已统一、Docker 基础设施已现代化、`once_cell` 等悬空依赖已清除。剩余短板在 E2E 视觉回归测试和 `chrono` 依赖。

**两个项目作为一个体系**，最大的优势是技术方向正确（WASM Component Model + 纯 Rust UI），最大的风险是文档与代码的错位可能劝退新用户。建议优先推进文档重写和无障碍修复。

---

## 九、实际项目集成中发现的待改进项

> 来源: 氢能走廊数智平台项目 (h2-corridor-panel) 迁移实战, 2026-05-13

### P0 — 阻塞性问题

- [x] **[Packager] browser-glue runtime 分发机制** — `resolve_glue_runtime_bundle()` 现支持：`TAIRITSU_RUNTIME_BUNDLE` 环境变量覆盖、`DEP_TAIRITSU_BROWSER_WORLDS_WIT_DIR` 自动发现、`node_modules/` 搜索、改进的错误消息
  - `packages/packager/src/wasm/mod.rs` — 添加 env var 覆盖 + browser-worlds 定位 + node_modules 搜索

- [x] **[Web] crates.io 发布缺少 WIT 文件** — `tairitsu-web` 的 `build.rs` 现通过 `DEP_TAIRITSU_BROWSER_WORLDS_WIT_COMPOSED_DIR` 定位 WIT 文件并生成 `wit_bindings_generated.rs`，使用 `include!()` 引入，不再依赖相对路径
  - `packages/browser-worlds/Cargo.toml` — 添加 `links = "tairitsu-browser-worlds"` 以触发 `DEP_*` 环境变量
  - `packages/browser-worlds/build.rs` — 新增，输出 `cargo:wit_dir` 和 `cargo:wit_composed_dir`
  - `packages/web/Cargo.toml` — 添加 `tairitsu-browser-worlds` 作为 build-dependency
  - `packages/web/build.rs` — 新增 WIT 路径解析逻辑
  - `packages/web/src/wit_platform.rs` — 从 `wit_bindgen::generate!({ path: "..." })` 改为 `include!(concat!(env!("OUT_DIR"), "/wit_bindings_generated.rs"))`

- [x] **[Packager] `dev-wasm` profile 要求** — `tairitsu init` 模板现已包含 `[profile.dev-wasm]` 定义
  - `packages/packager/src/utils/init.rs` — Cargo.toml 模板添加 dev-wasm profile

### P1 — 功能缺口

- [x] **[Packager] 自定义 WIT 扩展接口支持** — 新增 `wit_plugin` 模块，支持项目级 `wit/` 目录下放置自定义 .wit 文件，自动扫描和合并到 composed WIT 目录
  - `packages/packager/src/wit_plugin.rs` — `PluginWitRegistry` 扫描项目 `wit/` 目录，`merge_into_composed_dir()` 合并到构建输出
  - `packages/packager/src/lib.rs` — 导出 `PluginWitRegistry`

- [x] **[Web] setInterval 缺失** — WIT `platform-helpers` 新增 `set-interval`/`clear-interval`，`timer-callbacks` 新增 `on-interval`，WitPlatform 完整实现
  - `packages/browser-worlds/wit/handwritten/platform-helpers.wit` — 添加 `set-interval`/`clear-interval`
  - `packages/browser-worlds/wit/handwritten/callbacks.wit` — `timer-callbacks` 添加 `on-interval`
  - `packages/browser-worlds/wit/browser-full.wit` — 同步更新
  - `packages/browser-worlds/wit/composed/_handwritten.wit` — 同步更新
  - `packages/vdom/src/platform/trait.rs` — Platform trait 添加 `set_interval`/`clear_interval`
  - `packages/vdom/src/dom_ops.rs` — DomFuncs 添加 `set_interval_fn`/`clear_interval_fn` + 全局函数
  - `packages/web/src/wit_platform.rs` — 完整实现（INTERVAL_CALLBACKS + Guest impl + Platform impl）
  - `packages/browser-glue/src/runtime/platformHelpers.ts` — JS 端实现 setInterval/clearInterval
  - Mock platforms (integration_tests.rs, scheduler.rs, reactive_render_test.rs) — 同步实现

- [x] **[Hooks] 缺少 `use_interval` hook** — 新增 `use_interval()` 返回 `IntervalHandle`，支持 `start(platform, callback, ms)` / `clear(platform)` / `is_active()`，自动清理
  - `packages/hooks/src/interval.rs` — 新文件
  - `packages/hooks/src/lib.rs` — 导出 `IntervalHandle` 和 `use_interval`

### P2 — 开发体验改善

- [x] **[Packager] Windows 兼容性** — 已确认所有 `std::os::unix` 引用均在 `#[cfg(unix)]` 块内，daemon 有完整的 `#[cfg(windows)]` 替代实现（`SetStdHandle`），MCP 有 `get_ppid_windows()` 替代，vtty 有 `pty_win.rs` 替代

- [x] **[Packager] `tairitsu init` 模板过时** — 更新为正确 API：`WitPlatform::new()?.mount_vnode_to_app(vnode)` 替代不存在的 `WitPlatform::mount(|| { ... })`
  - `packages/packager/src/utils/init.rs` — lib.rs 模板使用 `WitPlatform::new()` + `mount_vnode_to_app()`
