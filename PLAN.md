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
| M8 | Hikari | `theme/styles/foundation.scss` vs 组件 CSS | **两套互不相连的 CSS 变量命名系统** — Foundation 用 `--hi-color-*`，组件用 `--hi-primary`，说明命名迁移不完整 |
| M9 | Hikari | `radio_group.rs:99` | ~~`Box::leak` 获取 `&'static str`~~ **已修复** — `RadioContext.name` 改为 `String` 类型，消除内存泄漏 |

### 🟢 轻微问题 (P3)

| # | 项目 | 位置 | 问题 |
|---|------|------|------|
| L1 | Tairitsu | `macros/src/rsx.rs:119` | ~~`syn::parse_str` 的 `unwrap()` 在遇到 Rust 关键字时会 panic~~ **已修复** — fallback 到 `syn::parse_quote!` |
| L2 | Tairitsu | `macros/src/scss.rs:234-260` | 类名提取器不处理 `url()` 中的点、`&.class` 嵌套、`.a.b.c` 链式选择器 |
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
| `cargo audit` / `cargo deny` | 均无 | 无法发现已知安全漏洞 |
| 依赖审查 (diff on PRs) | 均无 | 无法发现恶意依赖引入 |
| 许可证合规检查 | 均无 | 无法发现许可证冲突 |
| MSRV 检查 | 均无 | 无法保证最低 Rust 版本兼容性 |
| Visual regression | Hikari 无 | UI 组件库应比运行时更需要视觉回归 |

### 工程问题

| # | 项目 | 问题 |
|---|------|------|
| E1 | Hikari | `just fmt` 用 `cargo fmt --all`，但 CI 用 `cargo +nightly fmt --all -- --check --unstable-features` — 本地和 CI 格式化行为不一致 |
| E2 | Hikari | `justfile:25` 硬编码 `../tairitsu/packages/packager/Cargo.toml` — 绑死相对路径 |
| E3 | Hikari | CI 每个 PR 都 `python3 scripts/icons/fetch_mdi_icons.py` 从网络获取，未缓存 |
| E4 | Tairitsu | `justfile` 中两套 WIT 管线并存（`gen-wit-*` 和 `wit-gen-*`），使用不同的缓存目录 |
| E5 | Tairitsu | ~~`README.md:125-126` 两个文档链接指向不存在的目录~~ **已修复** → `docs/zhs/`、`docs/en/` |
| E6 | Tairitsu | ~~`README.md:603` "What's New in 0.3.0" 但当前版本是 0.4.5~~ **已修复** → 0.4.5 |

---

## 五、测试质量评估

| 项目 | 测试总数 | 通过率 | 问题 |
|------|----------|--------|------|
| **Tairitsu** | 909+ `#[test]` 标注 | **1 失败** (browser-wit-resolver) | diff.rs 仅 2 个测试；registry/container/image 核心模块零测试；wit.rs 只有一个空体 smoke test |
| **Hikari** | 817+ 通过 (25 binaries) | **0 失败** | basic_components_tests 的 15 个测试仅调用组件丢弃返回值（无断言）；无无障碍测试；animation 23 个 doctest 全被 ignore |

### Tairitsu 失败测试

```
packages/browser-wit-resolver/src/resolver.rs:259:
resolve_options_default_values:
  left:  "https://my-registry.com"     ← 测试期望
  right: "https://wit.tairitsu.dev"    ← 代码实际默认值
```

---

## 六、依赖问题

### 悬空/可能未使用的依赖

| 项目 | 依赖 | 理由 |
|------|------|------|
| Hikari | `once_cell` | Rust 2024 edition 已内置 `std::sync::OnceLock` |
| Hikari | `gloo`, `gloo-net` | Dioxus 时代的 `web-sys` 包装，迁移后可能不再需要 |
| Hikari | `chrono` + `unstable-locales` | chrono 社区维护堪忧 |
| Hikari | `tokio` features = `["full"]` | UI 组件库只需 `rt` + 基本 I/O |
| Tairitsu | `[patch.crates-io]` for `tairitsu-macros` | workspace member 已自动优先，可能残留 |

### Docker 问题 (Hikari)

| # | 问题 |
|---|------|
| D1 | `docker-compose.yml:5` 硬编码绝对路径 `/mnt/sdb1/hikari`，仅在一台机器工作 |
| D2 | `website.Dockerfile:25` 从 GitHub clone tairitsu，无版本锁定 |
| D3 | `website.Dockerfile:28` `pip install --break-system-packages` |
| D4 | `base-selenium.Dockerfile` 和 `screenshot-selenium.Dockerfile` 90% 重复 |
| D5 | Selenium 容器以 root 运行 |
| D6 | 缺少 `.dockerignore` |

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
- [ ] **[Both] CI 添加 `cargo audit` + `cargo deny`**

### P2 — 中优先级（改善开发体验）

- [x] **[Tairitsu] 清理 `unwrap()` on mutex** — `registry.rs` 的 8 处 `lock().unwrap()` 改为 `lock().expect("registry ... lock poisoned")`，提供诊断信息
- [x] **[Hikari] 删除 55 个冗余 easing 包装函数**
- [x] **[Hikari] 统一 Avatar 的 props 定义方式**
- [x] **[Hikari] 合并 `NodeState` 和 `Node`** — 重命名为 `NodePlacement`/`NodeView`，语义明确
- [ ] **[Hikari] 统一 CSS 变量命名** — 全量迁移到 `--hi-` 或 `--hi-color-`

### P3 — 低优先级（长期优化）

- [ ] **[Hikari] E2E 视觉回归加入 CI**
- [x] **[Tairitsu] 修复 CI bugs** — `fmt.yml` 路径过滤器→`fmt.yml`、`visual-regression.yml` 分支名→`master`、`wit-sync.yml` 缓存 key→基于脚本 hash
- [ ] **[Tairitsu] 合并 justfile 两套 WIT 管线**
- [ ] **[Hikari] 清理悬空依赖**（`once_cell`、`gloo`、`gloo-net`、`chrono`）
- [ ] **[Hikari] Docker 重构** — 多阶段构建、相对路径、非 root 用户
- [ ] **[Hikari] `just fmt` 与 CI 统一**
- [ ] **[Tairitsu] 增加核心模块测试覆盖率**（registry、container、image、diff）
- [ ] **[Hikari] 组件测试升级** — 从"仅验证不 panic"到"验证输出结构和属性"
- [ ] **[Tairitsu] 修复 `resolve_options_default_values` 过期断言**
- [ ] **[Hikari] MDI 图标获取加入 CI 缓存**
- [ ] **[Hikari] 删除空模块 `components/src/hooks/` 声明**
- [x] **[Tairitsu] `README.md` 更新死链接和版本号** — 修复 `docs/zh-CHS/` → `docs/zhs/`、`docs/en-US/` → `docs/en/`、版本号 0.3.0 → 0.4.5

---

## 八、总结

### 整体评分

| 维度 | Tairitsu (运行时+工具链) | Hikari (UI 组件库) |
|------|--------------------------|---------------------|
| 架构设计 | ★★★★☆ | ★★★★☆ |
| 代码质量 | ★★★★☆ | ★★★☆☆ |
| 测试覆盖 | ★★☆☆☆ | ★★★☆☆ |
| 文档质量 | ★★★★☆ | ★☆☆☆☆ |
| CI/工程化 | ★★★★☆ | ★★★☆☆ |
| 安全性 | ★★★☆☆ | ★★★☆☆ |
| 无障碍访问 | N/A | ★☆☆☆☆ |

### 核心判断

**Tairitsu** 作为基础设施层是合格的 — 架构前沿（WASM Component Model）、virtual DOM 和 proc-macro 设计精良、CI 覆盖全面。主要短板在测试覆盖率和 VDOM diff 的关键优化缺失。

**Hikari** 作为 UI 组件库有巨大的文档债务 — 从 Dioxus 迁移到 Tairitsu 后，66 处代码示例和架构文档仍然引用已废弃的依赖。代码层面 40+ 组件功能完整，但存在大量剪贴板式代码重复和无障碍关键缺失。

**两个项目作为一个体系**，最大的优势是技术方向正确（WASM Component Model + 纯 Rust UI），最大的风险是文档与代码的错位可能劝退新用户。建议优先推进文档重写和无障碍修复。
