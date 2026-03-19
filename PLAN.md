# Browser-Glue 架构重构计划

## 问题分析

### 当前问题

1. **命名混乱**
   - 手写文件和生成文件都带 `-glue` 后缀
   - `generated/` 下使用 kebab-case (如 `auth-glue.ts`)
   - 手写文件在 `src/` 下也带 `-glue` 后缀

2. **重复文件**
   - `src/events-glue.ts` (手写) vs `generated/events-glue.ts` (生成)
   - `src/dom-glue.ts` (手写) vs `generated/dom-glue.ts` (生成)
   - `src/fetch-glue.ts` (手写) vs `generated/fetch-glue.ts` (生成)
   - `src/canvas-glue.ts` (手写) vs `generated/canvas-glue.ts` (生成)

3. **TS 引用 JS**
   - 当前 import 语句使用 `.js` 扩展名（如 `from "./dom-glue.js"`）
   - 这在 TypeScript 中是不规范的

4. **架构定位不清**
   - 手写文件和生成文件职责重叠

### 目标架构

```
browser-glue/
├── src/
│   ├── index.ts              # 包入口
│   ├── handles.ts            # 通用句柄管理（替代 js-sys 部分）
│   ├── dom.ts                # DOM 工具库（替代 gloo/dom）
│   ├── events.ts             # 事件系统（替代 gloo/events）
│   ├── http.ts               # 网络请求（替代 gloo/net）
│   ├── canvas.ts             # Canvas 工具（替代 gloo/canvas）
│   ├── async.ts              # 异步调度（替代 gloo/futures）
│   └── generated/
│       ├── index.ts          # 生成文件入口
│       ├── authGlue.ts       # 驼峰命名
│       ├── cryptoGlue.ts
│       ├── domGlue.ts        # 纯 WIT 接口映射
│       └── ... (26 domains)
```

### 职责划分

| 层级 | 替代目标 | 职责 |
|------|---------|------|
| `generated/*.ts` | web-sys | 纯 WIT 接口映射，无业务逻辑 |
| `src/handles.ts` | js-sys | 通用句柄管理、类型转换 |
| `src/dom.ts` | gloo/dom | DOM 操作工具、批量操作、模板 |
| `src/events.ts` | gloo/events | 事件系统、事件委托、回调管理 |
| `src/http.ts` | gloo/net | HTTP 客户端、请求构建、响应解析 |
| `src/canvas.ts` | gloo/canvas | Canvas 工具、渲染循环、离屏渲染 |
| `src/async.ts` | gloo/futures | 异步调度、Promise 桥接、超时处理 |

---

## 执行步骤

### Phase 1: 文件重命名

#### 1.1 重命名手写文件
```bash
# src/
mv src/dom-glue.ts src/dom.ts
mv src/events-glue.ts src/events.ts
mv src/fetch-glue.ts src/http.ts
mv src/canvas-glue.ts src/canvas.ts
mv src/handle-table.ts src/handles.ts
mv src/generated-index.ts src/generated/index.ts
```

#### 1.2 重命名生成文件 (kebab → camelCase)
```
generated/auth-glue.ts → generated/authGlue.ts
generated/crypto-glue.ts → generated/cryptoGlue.ts
generated/css-glue.ts → generated/cssGlue.ts
... (全部 26 个文件)
```

#### 1.3 更新生成器脚本
- `scripts/generate_browser_glue.py`: 输出文件名改为驼峰式

### Phase 2: 修复 import 语句

#### 2.1 移除 `.js` 扩展名
```typescript
// Before
import { getElement } from "./handle-table.js";

// After
import { getElement } from "./handles";
```

#### 2.2 更新所有引用
- `src/index.ts`
- `src/dom.ts`
- `src/events.ts`
- `src/http.ts`
- `src/canvas.ts`
- `src/handles.ts`
- `src/generated/index.ts`

### Phase 3: 架构增强

#### 3.1 增强 handles.ts
- 添加类型安全的句柄注册
- 添加批量操作支持
- 添加垃圾回收钩子

#### 3.2 增强 dom.ts
- 添加批量 DOM 操作
- 添加模板字符串支持
- 添加选择器缓存

#### 3.3 增强 events.ts
- 添加事件委托
- 添加防抖/节流
- 添加一次性事件

#### 3.4 增强 http.ts
- 添加请求拦截器
- 添加响应缓存
- 添加重试逻辑

#### 3.5 新增 async.ts
- Promise 桥接到 WIT
- setTimeout/setInterval 封装
- async/iterator 支持

### Phase 4: 构建验证

1. 运行 `npm run build` 验证编译
2. 运行 `cargo test -p tairitsu-browser-worlds` 验证 WIT
3. 运行 `cargo build --all` 验证整体构建

---

## 文件映射表

### 手写文件

| 旧路径 | 新路径 | 新职责 |
|--------|--------|--------|
| `src/dom-glue.ts` | `src/dom.ts` | DOM 工具库 |
| `src/events-glue.ts` | `src/events.ts` | 事件系统 |
| `src/fetch-glue.ts` | `src/http.ts` | HTTP 客户端 |
| `src/canvas-glue.ts` | `src/canvas.ts` | Canvas 工具 |
| `src/handle-table.ts` | `src/handles.ts` | 句柄管理 |
| `src/generated-index.ts` | `src/generated/index.ts` | 生成文件入口 |

### 生成文件

| 旧命名 | 新命名 |
|--------|--------|
| `auth-glue.ts` | `authGlue.ts` |
| `canvas-glue.ts` | `canvasGlue.ts` |
| `crypto-glue.ts` | `cryptoGlue.ts` |
| `css-glue.ts` | `cssGlue.ts` |
| `device-glue.ts` | `deviceGlue.ts` |
| `dom-glue.ts` | `domGlue.ts` |
| `events-glue.ts` | `eventsGlue.ts` |
| `fetch-glue.ts` | `fetchGlue.ts` |
| `file-api-glue.ts` | `fileApiGlue.ts` |
| `geolocation-glue.ts` | `geolocationGlue.ts` |
| `html-glue.ts` | `htmlGlue.ts` |
| `indexed-db-glue.ts` | `indexedDbGlue.ts` |
| `media-glue.ts` | `mediaGlue.ts` |
| `notifications-glue.ts` | `notificationsGlue.ts` |
| `observers-glue.ts` | `observersGlue.ts` |
| `payments-glue.ts` | `paymentsGlue.ts` |
| `performance-glue.ts` | `performanceGlue.ts` |
| `permissions-glue.ts` | `permissionsGlue.ts` |
| `resize-observer-glue.ts` | `resizeObserverGlue.ts` |
| `service-workers-glue.ts` | `serviceWorkersGlue.ts` |
| `storage-glue.ts` | `storageGlue.ts` |
| `streams-glue.ts` | `streamsGlue.ts` |
| `url-glue.ts` | `urlGlue.ts` |
| `wasm-glue.ts` | `wasmGlue.ts` |
| `web-animations-glue.ts` | `webAnimationsGlue.ts` |
| `webrtc-glue.ts` | `webrtcGlue.ts` |
| `websocket-glue.ts` | `websocketGlue.ts` |
| `websockets-glue.ts` | `websocketsGlue.ts` |
| `workers-glue.ts` | `workersGlue.ts` |

---

## 风险评估

1. **破坏性变更**: import 路径变化会影响所有依赖方
2. **测试覆盖**: 需要确保所有功能在重命名后仍然正常
3. **文档更新**: 需要更新所有相关文档

## 回滚方案

如果出现问题，可以通过 git revert 回滚所有更改。
