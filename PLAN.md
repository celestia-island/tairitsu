# tairitsu — 项目状态与计划 (PLAN)

> 本文件由自动化扫描于 **2026-07-04** 生成，记录项目当前状态、近期进展与后续计划。

## 1. 项目概述

- **名称**：`tairitsu`
- **简介**：基于 WASM Component Model 的全栈 Web 框架（monorepo）。
- **远程仓库**：https://github.com/celestia-island/tairitsu
- **技术栈**：Rust / Node/TypeScript / just
- **类别**：platform

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：有未提交改动
  - 修改 31（31 项）
- **最近提交时间**：2026-07-03
- **最近提交**：fix(mcp): system-fonts-first + install rustls crypto provider before reqwest
- **分支对比**：`dev` 领先 `master` 1618 个提交

## 3. 未提交改动明细

```
M packages/browser-glue/src/glue/auth.ts
 M packages/browser-glue/src/glue/canvas.ts
 M packages/browser-glue/src/glue/crypto.ts
 M packages/browser-glue/src/glue/css.ts
 M packages/browser-glue/src/glue/device.ts
 M packages/browser-glue/src/glue/dom.ts
 M packages/browser-glue/src/glue/events.ts
 M packages/browser-glue/src/glue/fetch.ts
 M packages/browser-glue/src/glue/fileApi.ts
 M packages/browser-glue/src/glue/geolocation.ts
 M packages/browser-glue/src/glue/html.ts
 M packages/browser-glue/src/glue/index.ts
 M packages/browser-glue/src/glue/indexedDb.ts
 M packages/browser-glue/src/glue/media.ts
 M packages/browser-glue/src/glue/misc.ts
 M packages/browser-glue/src/glue/notifications.ts
 M packages/browser-glue/src/glue/observers.ts
 M packages/browser-glue/src/glue/payments.ts
 M packages/browser-glue/src/glue/performance.ts
 M packages/browser-glue/src/glue/permissions.ts
 M packages/browser-glue/src/glue/resizeObserver.ts
 M packages/browser-glue/src/glue/serviceWorkers.ts
 M packages/browser-glue/src/glue/storage.ts
 M packages/browser-glue/src/glue/svg.ts
 M packages/browser-glue/src/glue/url.ts
 M packages/browser-glue/src/glue/wasm.ts
 M packages/browser-glue/src/glue/webAnimations.ts
 M packages/browser-glue/src/glue/webrtc.ts
 M packages/browser-glue/src/glue/websocket.ts
 M packages/browser-glue/src/glue/websockets.ts
 M packages/browser-glue/src/glue/workers.ts
```

## 4. 近期进展（最近提交）

- fix(mcp): system-fonts-first + install rustls crypto provider before reqwest
- chore: accumulative health-check fixes across workspace
- fix(mcp): scale fonts to supersampled resolution + bump output to desktop size
- fix(mcp): re-enable kou font-fetch — async load path is now runtime-safe
- style(mcp): wrap render_png call to satisfy rustfmt
- feat(mcp): wire vtty_screenshot theme through to kou renderer

## 5. 后续计划

1. 整理并提交当前未提交改动（共 31 项：修改 31）。
2. 推进核心功能里程碑，收敛历次审计（R3/R4/安全审计）遗留项。
3. 保持 workspace 内各 crate 一致（Cargo.lock、rust-toolchain、deny.toml）。
4. 定期刷新本 PLAN.md 以反映最新状态。

