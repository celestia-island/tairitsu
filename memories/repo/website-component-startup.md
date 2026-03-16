# Tairitsu 构建闭环记录

## 外部工具清除（2026-03-16）

已从整个代码库彻底移除以下外部工具链依赖：
- legacy 打包器链路（历史文档提及）
- legacy JS glue CLI 依赖（packager 与 web 包已移除）
- legacy browser sys 依赖（web 包已移除）
- legacy wasm unknown target 构建路径（已完全移除）

## 当前唯一构建目标
wasm32-wasip2 (WIT Component Model)

## 构建工具链（全部使用项目自有设施）
- tairitsu-packager：一站式打包/dev server
- tairitsu-web + wit-bindings feature：Rust 侧 WIT 绑定（via wit-bindgen）
- packages/browser-glue TypeScript：浏览器 DOM/事件/Canvas 侧 WIT 实现
- jco / npx @bytecodealliance/jco：wasm component → JS transpile（packager 自动调用）

## CLI 默认
- tairitsu build 默认 --target component
- Cargo.toml [package.metadata.tairitsu.build] target 默认 "component"

## 关键修改文件
- packages/packager/Cargo.toml: 无 legacy JS glue CLI 依赖，default feature 无 wasm
- packages/web/Cargo.toml: 无 legacy browser sys / glue 依赖，无 web feature
- packages/packager/build.rs: 不再需要 Node.js/tsc
- packages/packager/src/wasm/component-wrapper-loader.template.js: 纯 JS，include_str! 直接引用
- packages/packager/src/wasm/mod.rs: 仅保留 component 路径
- packages/web/src/platform.rs: 纯存根（无 cfg web 块）
- packages/web/src/portal.rs: 纯存根