# Changes since v0.3.0

Generated on 2026-04-01

## Features

- feat: add comprehensive E2E test coverage (`f98aa77e`)

- feat: 添加 WIT 同步工作流，支持自动化生成和版本管理 (`4895eeae`)

- feat: 增强守护进程功能，支持传递参数并优化日志文件处理 (`095e9cef`)

- feat: 更新配置和依赖，增强 WASM 支持及优化包管理 (`095934cf`)

- feat: 添加守护进程支持，增强构建工具的后台服务能力 (`c5289d6c`)

- feat: 添加媒体查询相关方法到 MockPlatform 和测试中 (`15e37dfd`)

- feat: 实现 MediaQueryList matchMedia API 和事件监听支持 (`6ac19868`)

- feat: add Clone/Copy derives to DomRect (`eb47bda1`)

- feat: 实现 CSS 表达式解析、Suspense 资源追踪、E2E 测试增强 (`e21380a0`)

- feat: 完成开发功能整合到 SSR 包 (`9482d8eb`)

- feat: 完成 router、i18n → web 包整合 (`572b582d`)

- feat: 完成 css-values → style 包整合 (`9aecd23e`)

- feat: 完成包架构重整 - web-next 重命名为 web 并整合 (`66ade95b`)

- feat: 初始化 web-next 包架构重整 (`c6f9bfc0`)

- feat: 完善 WIT 接口以替代 web-sys/wasm-bindgen，新增动画帧、媒体查询和 DOM 几何接口 (`05bc1777`)

- feat: 完成 Phase 5 - 开发体验工具 (`16742075`)

- feat: 完成 Phase 2 - 异步资源加载 (`99e11f36`)

- feat: 实现文件系统路由 Router (`5f8bfc6c`)

- feat: 实现全局状态管理 Store (`cf298301`)

- feat: 完成 Phase 1 - CSS 值类型系统 (`518a9c0c`)

- feat: 添加 CSS 自定义属性的方法以支持 --custom-properties (`14130884`)

- feat: 重构 website demo 路由和样式系统 (`7a4e4f1e`)

- feat: 升级 Rust edition 至 2024 (`96a4aa3b`)

- feat: 统一 Cargo.toml 元数据配置，使用工作区继承 (`33544bec`)

- feat: 将多个 Cargo.toml 文件中的作者信息更新为工作区配置 (`b2f68213`)

- feat: 添加作者信息到多个 Cargo.toml 文件 (`27666ae2`)

- feat: 更新多个模块以使用工作区中的 Tokio 依赖 (`70f8b7e8`)

- feat: 添加多个新模块以支持 CSSStyleDeclaration 和事件目标功能 (`29ce7480`)

- feat: 更新 WIT 接口生成路径并添加 CSSStyleDeclaration 和事件目标相关的实现 (`31b9b9a4`)

- feat: Complete WIT interface migration to W3C standards (`9d2c9b8f`)

- feat: Complete PLAN.md tasks 1-6 - Core animation and reactivity system (`2bd8d63f`)

- feat: 补充 MouseEvent 字段和 builder 方法 (`3cb5840b`)

- feat: implement Phase 4 - Advanced event types (`5dcc242c`)

- feat: implement Phase 1.3, 2.1, 3.1, 3.2 - complete animation infrastructure (`2fd2da4f`)

- feat: implement Phase 0 and Phase 1 animation infrastructure (`990f616e`)

- **ssr**:  use bindgen to call lifecycle::start export (`bfe44ffd`)

- **ssr**:  add full HTML document rendering support (`67bc8d5b`)

- **wit**:  add platform-helpers interface and fix WIT type definitions (`91043c26`)

- **ssr**:  add resize-observer interfaces and fix WIT type mapping (`4a1f4b64`)

- **ssr**:  add get-element-by-id and query-selector to document interface (`64073d4f`)

- **ssr**:  complete core WIT interface implementations for node, element, style, and platform-helpers (`d2e9d623`)

- **ssr**:  add PartialEq derive to SsrNode for testing (`10c44b5f`)

- **ssr**:  add stub implementations for browser interfaces (`da88a42a`)

- **ssr**:  add SSR support to packager CLI and complete Phase 2-4 (`a5cea3f2`)

- **ssr**:  implement Phase 0-1 of SSR with in-memory DOM and core WIT interfaces (`106c545c`)

- **ssr**:  implement Phase 0-3 of SSR functionality (`38fec37a`)

- **i18n**:  add internationalization module with language definitions, context, and TOML loader (`93c6cc99`)

- feat: enhance SCSS compilation with load path resolution and remove extractor usage (`edcced47`)

- feat: add mutation record and resize observer interfaces, update import paths to use @tairitsu-glue (`d4f1b19c`)

- feat: add Canvas and Observer APIs to Platform trait (`b1c731a3`)

- feat: add manifest_dir to Config and update output directory handling in WASM build process (`bd9d1884`)

- feat: update runtime and index files to enhance global handle management and improve function definitions (`eaa8d346`)

- feat: update runtime.ts to enhance event handling and add new interfaces (`0d014c9c`)

- feat: update browser glue paths to use relative imports and add runtime bundle generation (`c5a73648`)

- feat: add production optimization, event latency tests, and documentation (`ff76602f`)

- feat: add style and event-target interface support (`22174374`)

- feat: add console interface support (`1b54775a`)

- feat: 完成 browser-glue 架构增强 Phase 3 (`8b77346f`)

- **worlds**:  embed all 27 WIT packages in browser-worlds (`d76c88ee`)

- **glue**:  expand WIT parser support and domains (`6250deb3`)

- **glue**:  expand WIT parser and improve glue generation (`aede21e3`)

- **glue**:  implement WIT → TypeScript glue code generator (`971cdf70`)

- **style**:  expand CSS properties to 403 and implement UtilityClass trait (`df950bc9`)

- **style**:  implement CSS property auto-generation system with build.rs (`a34a6ec8`)

- **macros**:  add define_props attribute macro for cleaner Props DSL (`a731980d`)

- **macros**:  add allow attributes to generated component code (`95fdb671`)

- **packager**:  add flexible SCSS configuration support (`e2f03014`)

- **packager**:  embed browser-glue for standalone operation (`863a6143`)

- **macros**:  add file: syntax to scss! macro **BREAKING** (`9f1c8215`)

  **BREAKING CHANGE**


- **macros**:  integrate svg! macro with resource index **BREAKING** (`39f7e586`)

  **BREAKING CHANGE**


- **macros**:  add svg! macro for compile-time SVG embedding **BREAKING** (`e195a329`)

  **BREAKING CHANGE**


- **vdom,packager**:  add SafeSvg and resource indexing support (`7e690293`)

- **packager**:  add icon support module with CLI commands (`cec3956a`)

- **packager**:  添加安装 tairitsu CLI 的功能和相关本地化文本 (`43f50a36`)

- **wasm**:  添加 Ctrl+C 信号处理以实现优雅关闭 (`370854b8`)

- **hooks**:  add From<Memo<String>> for Style and Classes (`69e6222b`)

- **vdom**:  添加文件输入元素的 FileData 结构及相关方法，增强表单数据处理 (`458808a0`)

- **wasm**:  使用 JSON 消息格式增强构建过程的输出和进度显示 (`cbbd2b75`)

- **home**:  添加主页组件，包含英雄部分和导航卡片 (`6c2adcaf`)

- feat: Enhance vdom library and add comprehensive documentation for Tairitsu website (`fa4c96b1`)

- **vdom**:  add Signal::write() returning mutable RefMut (`d3b3a3b0`)

- **memo**:  enhance use_memo API for Dioxus compatibility and improve children handling in rsx macro (`456ecf31`)

- **vdom**:  extend IntoAttrValue implementations (`40f139c4`)

- **hooks**:  re-export GenericEvent from vdom (`9856fac0`)

- **vdom,web**:  add GenericEvent type and fix on_generic_event (`76bcfa28`)

- **macros,vdom,hooks**:  add for loop, Option<T> attrs, and Event type (`1bc66df5`)

- feat: add Dioxus compatibility features for migration (`bd81b29d`)

- **hooks,vdom**:  add use_memo, use_callback hooks and Callback<T> type (`8cb70d8a`)

- feat: implement Hikari migration support features (`9d5f5f51`)

- feat: 更新构建配置，移除外部工具依赖并调整输出目录 (`22121b15`)

- feat: 完善构建文档，移除外部工具依赖并更新测试用例 (`a75b4858`)

- feat: update build process to component-only model and remove legacy dependencies (`5417bfce`)

- **website**:  enhance app structure with new sections and localization support (`9f6ae203`)

- **i18n**:  add localization support with multiple languages and text resources (`6b543601`)

- **website**:  wire demo sections and render code snippets (`e9b39864`)

- **website**:  bootstrap visible docs-driven demo page (`b934bb41`)

- **i18n**:  update localization keys and improve build status messages (`09f55220`)

- **packager**:  make top frame dividers follow terminal width (`d1683373`)

- **packager**:  add new localization keys for build status and timing information (`4e0097db`)

- **packager**:  resize-aware bottom bar and locale-aware CLI i18n (`71257750`)

- **packager**:  pipe cargo stderr through MultiProgress for bottom-anchored bars (`fee961db`)

- **packager**:  keyboard shortcuts, check bar, and select! watch loop **BREAKING** (`1992826a`)

  **BREAKING CHANGE**


- **packager**:  add persistent watch dashboard for dev CLI (`405dc733`)

- **dev-server**:  add watch mode for automatic rebuilds on file changes (`aec3f7a1`)

- **packager**:  add component wrapper loader and prevent caching in dev mode (`12e9e880`)

- **component-start**:  add lifecycle.start export and invoke it from loader (`82d70d23`)

- **wrapper**:  rewrite preview2 shim imports to esm URLs (`5892a3d7`)

- **component-fallback**:  add dynamic wrapper fallback for browsers (`8c9b327c`)

- **wit**:  extend browser worlds and add component build path (`bfa6fd8f`)

- **docs**:  complete multilingual docs sets and finalize plan (`5609c83d`)

- **web**:  Phase 4 — wit-bindings feature, migration & versioning docs (`0d023ab1`)

- **browser-wit-resolver**:  add embedded WIT fallback and complete Phase 1 (`84ea6c19`)

- feat: W3C WebIDL→WIT pipeline — scripts, generated WIT (422 interfaces), justfile integration (`dd271310`)

- feat: automated W3C WebIDL→WIT pipeline — scripts, justfile, generated WIT, PLAN.md (`d90b1e3b`)

- feat: WIT-first browser interface architecture Phase 0 scaffolding (`15ec14a0`)

- **styles**:  add SCSS compiler, extractor, and injector for CSS generation and optimization (`bd165b01`)

- feat: add tairitsu-style package with StyleBuilder and ClassesBuilder (`354c97f3`)

- feat: implement Portal system for Modal/Toast/Tooltip support (`1d114d71`)

- feat: implement #[component] macro for automatic Props generation (`4799ce3d`)

- feat: implement Phase B hooks (`fdffc739`)

- feat: update hikari_compat example with new features (`18e10cbc`)

- feat: implement event parameter system and dynamic children support (`c655acb8`)

- feat: update justfile to use new website demo (`be36babd`)

- feat: create tairitsu website demo with basic structure (`194bc44f`)

- **packager**:  implement real dev server with static file serving (`6b19d36f`)

- feat: implement tairitsu-package core functionality (`eb4ee67a`)

- feat: add tairitsu-package E2E tests and update examples (`1b9fbffc`)

- feat: add web demo with simple HTML/CSS examples (`72f6b3a2`)

- feat: add cargo config and Hikari compatibility example (`d98d8e74`)

- feat: add comprehensive rsx! macro examples **BREAKING** (`5d22f1b6`)

  **BREAKING CHANGE**


- feat: implement complete rsx! macro parser **BREAKING** (`6ecb806c`)

  **BREAKING CHANGE**


- feat: implement E2E testing framework (`8d985d65`)

- feat: add rsx! macro and use_style hook **BREAKING** (`063acb44`)

  **BREAKING CHANGE**


- feat: implement Phase 1-4 core packages (`e0f61047`)


## Bug Fixes

- fix: 修复 WebIDL 解析边界情况和代码质量问题 (`c2fd61c6`)

- fix: 修复 WIT 类型定义 - i32 应为 s32 (`029d137f`)

- fix: resolve clippy warnings and update PLAN.md (`c9064052`)

- fix: 修复 clippy 警告 (`267fd151`)

- fix: 恢复 PLAN.md (意外删除) (`9510bfed`)

- fix: 修复包架构重整后的编译和测试问题 (`da1f5c9f`)

- fix: 修复 vdom runtime 测试 (`01a37c73`)

- fix: 自动修复 clippy 警告 (`1cbb87fb`)

- fix: 修复 FetchError 测试用例顺序，确保所有错误类型均被验证 refactor: 优化 lib.rs 中的导入顺序 (`61773101`)

- fix: 适配 Rust 2024 edition 的语法变更 (`278cc153`)

- fix: 移除无效的 tokio patch 配置 (`2e1c6021`)

- fix: 移除重复的 CssStyleDeclaration 特性 (`e3160c68`)

- fix: 配置 wasm32 兼容的 tokio 特性 (`90a45f21`)

- fix: Correctly merge import map in browser-glue (`15b70357`)

- fix: Add WASI import map to generated HTML for jco transpiled wrappers (`5415d793`)

- fix: 修复 SSR platform-helpers 接口的 record 类型处理 (`13b1a9e4`)

- fix: resolve clippy warnings and improve code style (`553e44e8`)

- fix: code style improvements and clippy warnings (`c40e1c4b`)

- **ssr**:  resolve resize-observer type marshalling issue with bindgen (`0a0dbaaf`)

- fix: 修复 tairitsu-ssr 中的多个类型映射问题 (`fd9077c8`)

- fix: 修复 SSR 中的 WIT 路径和重复 map entry 错误 (`9c58b855`)

- **ssr**:  fix WIT type signatures and build script for linker registration (`b1861f11`)

- fix: update generated timestamp and improve code formatting for consistency (`33cc070a`)

- **wit**:  move dom-rect record into types interface (`1ffebad3`)

- fix: update function export syntax to ensure completeness for shorthand methods (`9af42c97`)

- fix: update file extension from .ts to .js for component wrapper loader template (`f567cfb7`)

- fix: update import paths in generate_interface_wrappers.py to use relative paths (`ff40e182`)

- fix: correct interface wrapper paths and add missing interfaces (`8af7436e`)

- fix: correct WIT type handling for union types and innerHTML/outerHTML (`d9b4f3e7`)

- fix: resolve all TypeScript compilation errors in browser-glue (`fe1a4eea`)

- fix: resolve 179 TypeScript compilation errors in browser-glue (`413a0159`)

- fix: update auto-generated file paths in glue files and add browser-full.wit (`cfdc0414`)

- fix: update auto-generated file paths in glue files (`2933f17a`)

- fix: update auto-generated file paths in glue files (`7024eea3`)

- fix: resolve all TypeScript compilation errors (`42d1b614`)

- fix: continue reducing TS2322 and TS2345 errors (`aca5733a`)

- fix: eliminate TS2552 errors, continue reducing TS2322 and TS2345 (`432d6d71`)

- fix: eliminate TS2339, reduce TS2322 and TS2345 (`af517d14`)

- fix: reduce TS2322, TS2345, and misc errors (`0a4b6836`)

- fix: eliminate TS2552 and TS2339 errors, reduce TS2322 and TS2345 (`6ef7239c`)

- fix: eliminate TS2304 errors by adding synthetic handle types (`f2d2d101`)

- fix: eliminate TS2554 and TS2355 errors, reduce TS2322 and TS2345 (`eb23c037`)

- fix: eliminate TS2304 and TS4104 errors, reduce TS2322 and TS2345 (`de571f8a`)

- fix: eliminate TS2769 errors, reduce TS2322 and TS2345 (`c4c1b75b`)

- fix: eliminate TS2300, TS2304, TS2349 errors (`39da64d0`)

- fix: reduce TS2322, TS2345, TS2554, TS2551 errors (`d118d5f4`)

- fix: eliminate TS18046 errors, reduce TS2322 and TS2345 (`3665359d`)

- fix: add more NUMBER_TO_BIGINT_PROPERTIES and HANDLE_RETURNING_FUNCTIONS (`fefea3e2`)

- fix: reduce TS2322 and TS2345 errors (`01d070c8`)

- fix: eliminate TS2393 duplicate function errors, reduce TS2322 (`5a773cc2`)

- fix: eliminate TS2693 and TS2339 errors, reduce TS2322 (`89a72590`)

- fix: eliminate all TS2304 errors by adding synthetic handle types (`4861ade6`)

- fix: reduce TypeScript errors in browser-glue generated code (`78a752ae`)

- fix: 修复生成脚本 - 解决所有 TypeScript 类型错误 (`5fa31a02`)

- fix: 修复所有生成文件的 TypeScript 类型错误 (`155acb1b`)

- fix: 完成 browser-glue 架构重构 - Phase 1 和 2 (`24b6c1e5`)

- **glue**:  resolve WIT parser issues with keywords and trailing commas (`1a8455f2`)

- **style**:  apply clippy suggestions (`b94bdb42`)

- **style**:  apply cargo fmt formatting (`a3e709e5`)

- **macros**:  improve type detection in define_props for nested generics (`0051a065`)

- fix: resolve all clippy warnings (`953b5ae4`)

- **packager**:  add debug logging for SCSS processing (`d356acf3`)

- **packager**:  create parent directory for SCSS output (`2eeb7cc4`)

- **tests**:  更新 use_memo 相关测试以使用新函数签名 refactor(macros): 移除未使用的 TokenStream2 导入 feat(macros): 添加 Fragment 支持以处理多个根元素 refactor(style): 修正导入路径以匹配模块结构 (`ec477230`)

- **macros**:  improve rsx! macro event handling **BREAKING** (`6350ae38`)

  **BREAKING CHANGE**


- **macros**:  handle children prop and add format string support (`30418ad8`)

- fix: resolve clippy warnings and TypeScript errors (`1812fb30`)

- **hooks**:  resolve clippy redundant_closure warnings (`693c2e55`)

- **browser-glue**:  add missing eventHandle parameter to callback types (`94489a9d`)

- **macros,packager**:  fix compiler warnings and test failures (`807a21fd`)

- fix: component macro default props and rsx if-else (`8c3f9130`)

- **packager**:  suppress unused_mut warning for platform-specific code (`b3e32f89`)

- fix: suppress unused variable warning in wit_platform.rs (`24c8ab14`)

- fix: resolve remaining clippy warnings (`e43e7f3d`)

- **packager**:  update shortcut messages for better readability in UI (`eaf6e5be`)

- **packager**:  update progress bar styles for better visibility during build and watch processes (`6321710a`)

- **justfile**:  update PowerShell init script for Node dependencies and enhance command execution (`f79ce428`)

- **justfile**:  make windows init recipe parse and execute correctly (`4bb08f5c`)

- **web**:  mount VNode after lifecycle start and silence wrapper 404 probe (`39d3aa32`)

- **packager**:  enhance TypeScript compilation error handling and improve type definitions (`8d30b296`)

- **wasm**:  improve error logging for failed commands in gen_wit_from_webidl.py (`ae95d9a4`)

- **loader-state**:  avoid persistent Loading when no boot export exists (`21e19741`)

- **loader**:  remove illegal top-level return in module script (`447126c3`)

- **component-loader**:  detect wasm component header in browser loader (`4d3be0a3`)

- **browser-glue**:  inline loader in HTML, remove instantiate from index.js (`fdd32e38`)

- **dev**:  disable auto-open and fix browser-glue instantiate/favicon (`866d42ab`)

- **dev**:  honor component target and migrate website off wasm-bindgen path (`ce33e858`)

- fix: implement with_host_linker, real wit_world! macro, and remove all placeholder text **BREAKING** (`7ac11f5b`)

  **BREAKING CHANGE**


- fix: resolve all clippy warnings and remove stub/mock markers (`4c17ab3a`)

- **browser-glue**:  fix TypeScript type error in fetch-glue (`6cd80b1d`)

- **packager**:  update output directory and enhance wasm build process (`0dca26bf`)

- **macros**:  fix hex encoding in scss! macro **BREAKING** (`d296bca2`)

  **BREAKING CHANGE**


- fix: resolve web_sys error handling in PortalRenderer (`496d25ea`)

- fix: resolve all clippy warnings in tairitsu-website (`484c512b`)

- **web**:  remove placeholder modules and implement proper event listener management (`a1b10fd7`)

- fix: resolve all clippy warnings (`e10e56cc`)


## Performance

- perf: add batch DOM operations for reduced WIT round-trips (`9fffeab0`)

- perf: add opaque handle caching for style operations (`d4c704ce`)


## Refactoring

- refactor: remove deprecated wit_bindings.rs placeholder (`ecd27729`)

- refactor: 调整 CSS 值类型和实用工具的导入顺序 (`38dcac53`)

- refactor: remove doctor diagnostic tool (`70e5da01`)

- refactor: remove web-sys/wasm-bindgen dependencies, use WIT-only (`9c9d10a4`)

- refactor: 优化页面组件的 JSX 语法，简化代码结构 (`eb8723d8`)

- refactor: 移除控制台和样式相关的模块及其接口定义 (`8743b164`)

- refactor: 移除不再使用的事件目标相关代码 (`3d5e54d9`)

- **ssr**:  update WIT file path and clean up stub registration in linker (`117a3565`)

- **browser-glue**:  rename files and restructure architecture (`35d836ca`)

- refactor: remove browser-full.wit and clean up redundancy (`7ae1f325`)

- **worlds**:  consolidate browser-full.wit into browser-full.wit (`2b9223f1`)

- **packager**:  simplify icon fetching with optional HTTP feature (`157b0428`)

- **macros**:  优化 rsx! 宏属性解析，支持简写语法 **BREAKING** (`119231d8`)

  **BREAKING CHANGE**


- **macros**:  rename has_attribute to has_props_attribute for clarity refactor(vdom): update comment for blanket implementation of IntoAttrValue (`e4a5e1f0`)

- **packager**:  move watch status into top panel and remove bottom bars (`213c404b`)

- **web-demo**:  remove trunk dependency and use tairitsu-packager (`3cdd040d`)

- refactor: rename tairitsu-package to tairitsu-packager (`f0eee697`)


## Documentation

- docs: update PLAN.md - mark Phase 2 and 3 complete (`ae619ca7`)

- docs: add comprehensive package README documentation (`10dc7aaa`)

- docs: 更新 PLAN.md 进度 - Phase 1.2 和 1.3 已完成 (`8cb9de9f`)

- docs: 更新 PLAN.md 进度 - 标记 Phase 1.1 测试任务已完成 (`564d5f45`)

- docs: 更新 PLAN.md 为实际执行计划 (`2500b859`)

- docs: 添加 wasm-bindgen/web-sys 与 W3C 标准同步机制对比与改进报告 (`cea4c984`)

- docs: 更新 PLAN.md - 标记 matchMedia 和 MediaQueryList 事件监听为已完成 (`b6659bf7`)

- docs: 添加 PLAN.md，记录来自 hikari-animation 的 WIT 接口需求 (`4fb42179`)

- docs: cleanup PLAN.md - remove all completed items (`0039acd9`)

- docs: update PLAN.md - remove completed items, clarify E2E status (`0be08350`)

- docs: 完成 PLAN.md 架构重整计划 (`9f20eefc`)

- docs: 更新 PLAN.md - 将架构重整标记为可选优化 (`35139c4d`)

- docs: 清理 PLAN.md - WASM 组件浏览器接口补全已完成 (`3b8538b2`)

- docs: Update PLAN.md - all tasks completed (`49f83bd1`)

- docs: Update PLAN.md - all tasks completed (`a9f06036`)

- docs: clean up PLAN.md - all tasks completed (`67d43362`)

- docs: update PLAN.md with all completed tasks (`2e5e91b1`)

- docs: update PLAN.md with completed tasks and remove resolved issue file (`e2e5d501`)

- docs: update resize-observer issue analysis and PLAN.md (`17a5bbbc`)

- docs: update PLAN.md with resize-observer issue and P2/P3 completion (`fc135bc6`)

- docs: mark P2 task as completed in PLAN.md (`1021e059`)

- docs: update PLAN.md with current progress and findings (`47351ffc`)

- **ssr**:  add progress report summarizing Phase 0-3 completion (`93d08135`)

- docs: add browser-glue and troubleshooting docs in all languages (`6fc88984`)

- docs: update PLAN.md to reflect all tasks completed (`b0f0b969`)

- docs: update CONTEXT.md with complete architecture diagram (`b1411357`)

- docs: update PLAN.md to reflect completed implementation status (`d4c7ec2a`)

- docs: update CONTEXT.md to reflect completed status (`6aab6371`)

- docs: update PLAN.md with completed tasks (`ca8c5bb3`)

- docs: update PLAN.md with completed tasks (`0573904a`)

- docs: update PLAN.md with progress and remaining issues (`996b8d08`)

- docs: update PLAN.md - all TypeScript errors resolved (`034d8c98`)

- docs: update PLAN.md with current progress (`1f7672ef`)

- docs: 清理 PLAN.md - 移除所有已完成任务 (`4541c870`)

- docs: 更新 PLAN.md - 标记 Phase 3 已完成 (`512e982e`)

- docs: 更新 PLAN.md - 标记 Phase 1 和 2 已完成，Phase 3 和 4 状态 (`37644984`)

- docs: update PLAN.md with completed tairitsu-style tasks (`443edbcb`)

- docs: add tairitsu-style package plan for CSS infrastructure (`7eced2d8`)

- docs: add Props DSL macro enhancement plan (`5ee41a0f`)

- docs: remove outdated PLAN.md for macro warning elimination (`8d183d96`)

- **plan**:  mark macro warning elimination complete (`30ab3f58`)

- docs: update PLAN.md progress (`af050d3a`)

- **plan**:  mark Phase 6 complete - SCSS configuration flexibility (`e4b638ef`)

- docs: add Phase 6 SCSS configuration flexibility tasks (`edd11fbc`)

- **plan**:  mark Phase 5 complete - browser-glue embedding (`6bc10022`)

- docs: add task for embedding browser-glue in packager (`83971501`)

- docs: add Safe SVG and resource documentation (`e06d8990`)

- docs: mark icon support plan as implemented (`977f914e`)

- docs: add icon support plan for tairitsu-packager (`7d697cfd`)

- **PLAN**:  update WIT platform implementation status (`84b72979`)

- **PLAN**:  update Dioxus compatibility status - all core features done (`bed72df4`)

- docs: mark PLAN.md as complete - all core tasks done (`f8d588a4`)

- docs: clean up PLAN.md - remove completed task details (`c22d9b11`)

- docs: update PLAN.md - mark completed tasks (`023ee093`)

- docs: add PLAN.md for hikari migration collaboration (`ebb7ab10`)

- **plan**:  mark plan fully completed with validated checks (`f1e42ce4`)

- **plan**:  mark Phase 3 complete; update status to all phases done (`b7adc37d`)

- **plan**:  update architecture status markers to match verified state (`4454e358`)

- docs: finalize PLAN.md - all core phases complete (`644960ff`)

- docs: update PLAN.md to reflect actual completion status (`c3654ec7`)

- docs: mark project as fully complete with all core features (`4c1554a8`)

- docs: clean up PLAN.md and mark project as complete (`8655163d`)

- docs: update PLAN.md with Phase C completion status (`68085d07`)

- docs: add Phase C plan - complete ecosystem enhancement (`b3ef6918`)

- docs: mark Phase B as completed in PLAN.md (`3fe017e3`)

- docs: update PLAN.md - Phase A completed (`a63f6e3a`)

- docs: add Hikari integration requirements to PLAN.md (`e93c2593`)

- docs: update PLAN.md with packager dev server implementation (`845d3736`)

- docs: update PLAN.md with latest improvements (`3ce156eb`)

- docs: update PLAN.md Phase 7 progress (`e25fb073`)

- docs: add tairitsu-package design to PLAN.md (`c5942d9e`)

- docs: simplify PLAN.md by removing completed details (`09cf8032`)

- docs: update PLAN.md with completion status (`e7accf4c`)

- docs: mark project as production-ready (`76a84096`)

- docs: add implementation summary and progress update (`b238838d`)


## Tests

- test: 补充单元测试覆盖 (`a571cbd8`)

- test: 修复 WIT 生成测试用例，匹配实际行为 (`35b2585d`)

- test: Add end-to-end integration tests (Task 7) (`a37a9976`)

- **ssr**:  add hikari website integration test (`6bb7e94d`)

- **ssr**:  add E2E tests for SSR functionality (`d9366cba`)

- **e2e**:  add SVG safety E2E test suite (`c8d60932`)


## Chores

- chore: 清理未使用的依赖 (`af7ab58e`)

- chore: 删除 testing 包（已不在 workspace 中） (`f5bf1224`)

- chore: 更新架构重整计划文档 (`fa17e985`)

- chore: 应用 clippy 修复并更新 PLAN.md (`f96739eb`)

- chore: 删除临时实现文档 (`87b70f82`)

- chore: 更新 PLAN.md - 标记 Phase 3-4 完成 (`a088128c`)

- chore: 删除过时的计划和研究文档 (`8888bdd7`)

- chore: 删除过时的迁移协同计划文档 (`643bd9f2`)

- **website**:  ignore public/css assets and update app (`cf5c5b41`)

- **dev**:  switch default port to 3001 and align python log style (`5551e671`)

- **plan**:  close remaining items and harden web wit build (`6e98f36b`)

- **docs**:  remove non-locale files from docs root (`92039bc5`)

- chore: 删除过时的计划文档并优化示例页面代码结构 (`b732779d`)

- **docs**:  stage multilingual docs baseline and verification checkpoint (`77ec9d11`)

- chore: checkpoint before fixing placeholder implementations (`e5ddb22d`)

- chore: checkpoint before plan cleanup and verification (`43cd2423`)

- chore: update .gitignore to exclude Node.js build artifacts (`c8932665`)

- chore: merge + cleanup — keep comprehensive wit-* pipeline, add Python cache to .gitignore (`9c15b3e2`)

- chore: add Python bytecode cache to .gitignore (`d04e6e60`)

- **examples**:  remove deprecated web demo example and related files (`6085a714`)

- chore: fix clippy warnings in tairitsu-package (`f72f87ec`)

- chore: add web demo dist to gitignore (`9a84a1b1`)

- chore: change default port to 3000 (`c1ee2efa`)


## Style

- style: 修复代码格式 (`f6040247`)

- style: improve parser.rs code formatting for clippy compliance (`238b64b8`)

- **build**:  format code for better readability in build.rs (`cdd23246`)


---

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).