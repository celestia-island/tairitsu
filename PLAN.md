# Tairitsu 模块化 WASM Component CDN 分发计划

> **目标**: 利用 WASM Component Model 的动态链接特性，将框架拆分为可独立缓存、CDN 加速的 npm 模块，
> 使业务代码只需引用一份胶水 + 一份基于 tairitsu 编译的 wasm 即可快速加载。
> hikari 也可按同样模式分模块上传和缓存。

---

## 现状分析

### 当前架构 (单体打包)

```
Rust 业务代码
    ↓ cargo build --target wasm32-wasip2
单体 .wasm (~1-5MB)
    ↓ jco transpile (构建时)
component-wrapper/*.js
    ↓ + browser-glue (嵌入在 packager 编译时)
dist/
  ├── index.html
  ├── __tairitsu_glue__.js          ← 运行时胶水 (IIFE, 嵌入在 Rust binary 中)
  ├── browser-glue/                  ← 完整 glue dist 副本
  ├── component-wrapper/             ← jco 生成的 wrapper
  └── app.wasm                       ← 单体组件
```

**问题**:
1. 每次业务代码变更，整个 .wasm 必须重新下载
2. browser-glue 胶水代码嵌入在 packager Rust binary 中，无法独立更新/缓存
3. hikari 组件库被静态链接进 .wasm，无法共享缓存
4. 无 npm 分发基础设施，无法 CDN 加速

### 目标架构 (模块化动态链接)

```
npm CDN (@celestia scope)
├── @celestia/tairitsu-runtime        ← 运行时加载器 + import map 注册 (JS)
├── @celestia/tairitsu-glue-core      ← 核心胶水: handles, async, registry (JS)
├── @celestia/tairitsu-glue-dom       ← DOM 域胶水 (JS)
├── @celestia/tairitsu-glue-events    ← Events 域胶水 (JS)
├── @celestia/tairitsu-glue-html      ← HTML 域胶水 (JS)
├── ... (每个 WIT domain 一个包)
├── @celestia/tairitsu-glue-full      ← 聚合包: 引用上述所有域
├── @celestia/tairitsu-wit-adapter    ← WASI preview2 adapter .wasm (CDN 可缓存)
├── @celestia/tairitsu-component-loader ← jco 转译后的通用加载器 (JS)
└── @celestia/hikari-components       ← hikari UI 组件 (独立 .wasm)

业务页面引用:
├── <script type="importmap">{ ... }</script>
├── <script type="module">
│     import { boot } from '@celestia/tairitsu-runtime';
│     boot({ componentUrl: '/app.wasm' });
│   </script>
└── app.wasm  ← 仅包含业务逻辑 (极小, ~50-200KB)
```

---

## Phase 1: npm 发布基础设施

### 1.1 修正 browser-glue package.json 发布元数据

- [x] 添加 `files` 白名单 (仅 `dist/`, `README.md`)
- [x] 添加 `repository`, `publishConfig` 字段
- [x] 将 scope 从 `@tairitsu` 迁移到 `@celestia`

### 1.2 创建 @celestia/tairitsu-runtime 包

新建 `packages/npm/runtime/` 目录:
- `package.json`: @celestia/tairitsu-runtime
- `src/index.ts`: 运行时加载器 (从 browser-glue/src/runtime 提取)
- 构建: esbuild → ESM + IIFE 双格式

### 1.3 justfile 添加 publish recipe

```just
# 发布所有 npm 包到 @celestia scope (需要 NPM_TOKEN 环境变量)
publish:
    npm publish --access public packages/npm/runtime
    npm publish --access public packages/browser-glue
```

---

## Phase 2: 拆分 browser-glue 为独立域模块

### 2.1 域级包结构

每个 WIT domain 对应一个独立的 npm 包:

```
packages/npm/
├── runtime/                     ← @celestia/tairitsu-runtime
│   ├── package.json
│   ├── src/index.ts
│   └── tsconfig.json
├── glue-core/                   ← @celestia/tairitsu-glue-core (handles + async + registry)
│   ├── package.json
│   ├── src/
│   │   ├── handles.ts
│   │   ├── async.ts
│   │   └── registry.ts
│   └── tsconfig.json
├── glue-dom/                    ← @celestia/tairitsu-glue-dom
│   ├── package.json
│   └── src/index.ts             ← re-export from browser-glue/src/glue/dom.ts
├── glue-events/                 ← @celestia/tairitsu-glue-events
├── glue-html/
├── glue-css/
├── glue-fetch/
├── glue-canvas/
├── ... (每个 WIT domain)
└── glue-full/                   ← @celestia/tairitsu-glue-full (聚合所有域)
    ├── package.json
    └── src/index.ts             ← import + re-export 所有子包
```

### 2.2 依赖关系

```
@celestia/tairitsu-glue-dom
  └── @celestia/tairitsu-glue-core (handles)

@celestia/tairitsu-glue-events
  └── @celestia/tairitsu-glue-core

@celestia/tairitsu-glue-full
  ├── @celestia/tairitsu-glue-core
  ├── @celestia/tairitsu-glue-dom
  ├── @celestia/tairitsu-glue-events
  └── ... (所有域)

@celestia/tairitsu-runtime
  └── @celestia/tairitsu-glue-core
```

---

## Phase 3: WASM Component 动态链接支持

### 3.1 浏览器端 Component Model 支持现状

| 浏览器 | WebAssembly.Component | 状态 |
|--------|----------------------|------|
| Chrome 125+ | ✅ 支持 | 需要 origin trial 或 flag |
| Firefox | ❌ 不支持 | 需要 polyfill (jco) |
| Safari | ❌ 不支持 | 需要 polyfill (jco) |

**当前方案**: jco transpile 在构建时将 .wasm component 转为 .js + .wasm (core module)，
运行时用纯 JS 实现 component model 的链接语义。

**改进方向**: 
- 短期: 继续用 jco，但将 jco 的产物也作为 npm 包发布
- 中期: 当浏览器原生支持后，直接用 `WebAssembly.Component` API

### 3.2 分模块编译与链接

当前: 所有代码编译为一个 .wasm component
```
cargo build --target wasm32-wasip2
→ app.wasm (包含 runtime + vdom + hooks + hikari + 业务代码)
```

目标: 框架层与业务代码分离
```
cargo build --target wasm32-wasip2 -p hikari-components
→ hikari.wasm (独立组件库 component)

cargo build --target wasm32-wasip2 -p tairitsu-vdom
→ tairitsu-vdom.wasm (虚拟 DOM component)

cargo build --target wasm32-wasip2 -p my-app
→ app.wasm (仅业务代码, import tairitsu-vdom 和 hikari)
```

**需要的 Rust 侧改动**:
1. `tairitsu-vdom` 需要能作为独立 wasm32-wasip2 crate 编译
2. `hikari-components` 同理
3. 业务代码通过 WIT import 引用这些共享 component 的导出
4. packager 需要支持多 component 链接模式

### 3.3 packager 改造: 多 component 构建模式

当前 `build_component()` 只处理单个 .wasm，需要扩展为:

```
tairitsu build --release
  → 编译 app.wasm
  → 检测 WIT imports 中引用的已知框架 component
  → 从 npm CDN 或本地缓存获取 pre-built framework components
  → 生成 HTML 时包含多个 component 的加载和链接逻辑
```

---

## Phase 4: import map CDN 分发

### 4.1 运行时加载策略

业务页面 HTML 只需要:

```html
<script type="importmap">
{
  "imports": {
    "@celestia/tairitsu-runtime": "https://esm.sh/@celestia/tairitsu-runtime@0.1.0",
    "@celestia/tairitsu-glue-dom": "https://esm.sh/@celestia/tairitsu-glue-dom@0.1.0",
    "@celestia/tairitsu-glue-events": "https://esm.sh/@celestia/tairitsu-glue-events@0.1.0",
    "@celestia/tairitsu-glue-html": "https://esm.sh/@celestia/tairitsu-glue-html@0.1.0"
  }
}
</script>
<script type="module">
  import { boot } from '@celestia/tairitsu-runtime';
  await boot({ componentUrl: '/app.wasm' });
</script>
```

### 4.2 框架 component 的 CDN 分发

```
npm publish → @celestia/tairitsu-vdom-wasm (包含 .wasm + wrapper .js)
           → @celestia/hikari-components-wasm (包含 .wasm + wrapper .js)
```

CDN 引用:
```
https://esm.sh/@celestia/tairitsu-vdom-wasm@0.1.0/tairitsu_vdom.wasm
https://esm.sh/@celestia/tairitsu-vdom-wasm@0.1.0/wrapper.js
```

### 4.3 版本缓存策略

- 每个 npm 包独立 semver
- wasm 文件使用 content hash 文件名 (cache busting)
- import map 中的版本号由 packager 生成时自动注入

---

## Phase 5: hikari 模块化

### 5.1 hikari 组件库独立 wasm component

hikari 当前被静态链接到业务 .wasm 中。改为:

```
hikari-components
  ↓ cargo build --target wasm32-wasip2 --lib
hikari.wasm (独立 component)
  ↓ jco transpile
hikari-wrapper.js
  ↓ npm publish
@celestia/hikari-components
```

### 5.2 业务代码引用 hikari

```rust
// 业务代码中通过 WIT import 引用 hikari 组件
// wit:
//   import hikari:components/button;
//   import hikari:components/card;
```

---

## 实施优先级

| 阶段 | 优先级 | 依赖 | 预估工作量 |
|------|--------|------|-----------|
| Phase 1 | P0 | 无 | 2-3h |
| Phase 2 | P1 | Phase 1 | 4-6h |
| Phase 3 | P2 | Phase 1 | 8-12h |
| Phase 4 | P3 | Phase 2 + 3 | 4-6h |
| Phase 5 | P4 | Phase 3 | 4-6h |

---

## 技术风险与缓解

| 风险 | 影响 | 缓解方案 |
|------|------|---------|
| 浏览器不支持原生 Component Model | jco 产物体积大 | 短期继续 jco，中期用 WebAssembly.Component |
| 多 component 动态链接性能 | 加载时间增加 | HTTP/2 multiplexing + 预加载 |
| WIT 接口跨 component 版本不兼容 | 运行时链接失败 | 严格 semver + 构建时版本检查 |
| npm 包体积 | CDN 加载慢 | tree-shaking + 按域拆分 |

---

## 当前已完成

1. ✅ PLAN.md 撰写
2. ✅ browser-glue package.json 更新 (publish 元数据, @celestia scope)
3. ✅ @celestia/tairitsu-runtime (1.6KB) — WASM component 加载器
4. ✅ justfile 添加 `publish` / `publish-live` / `npm-build-glue` / `npm-build-wasm` recipes
5. ✅ 34 个 per-domain npm 包全部生成并构建完成 (总计 309.8KB, minified)
6. ✅ @celestia/tairitsu-glue-core (4.4KB) — handles + helpers + async
7. ✅ 7 个 runtime domain 包 (dom/events/css/html/observers/resize-observer/platform)
8. ✅ 25 个 auto-generated stub 包 (canvas/fetch/media/svg/webrtc/workers 等)
9. ✅ @celestia/tairitsu-glue-full (2.1KB) — 聚合包
10. ✅ Rust → wasm → npm 构建脚本 (scripts/build_wasm_packages.py)
11. ✅ glue 包自动生成脚本 (scripts/build_npm_glue_packages.py)

### 包大小分布 (esbuild minified)

| 包 | 大小 | 说明 |
|---|------|------|
| glue-canvas | 48.6 KB | Canvas 2D API |
| glue-media | 46 KB | Media APIs |
| glue-svg | 41.3 KB | SVG 操作 |
| glue-fetch | 30.7 KB | Fetch API |
| glue-webrtc | 25.4 KB | WebRTC |
| glue-platform | 14.9 KB | 平台助手 (setTimeout, rAF 等) |
| glue-storage | 15.1 KB | Storage APIs |
| glue-core | 4.4 KB | 共享 handles + helpers |
| glue-dom | 2.5 KB | DOM 操作 |
| glue-events | 4 KB | 事件系统 |
| glue-full | 2.1 KB | 聚合入口 |
| runtime | 1.6 KB | 加载器 |
| **总计** | **~310 KB** | **~80-90 KB gzip** |
