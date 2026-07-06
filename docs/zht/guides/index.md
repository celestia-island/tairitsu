# Tairitsu 文檔中心（繁體中文）

> 基於 WASM Component Model 的全端框架

## 入門

| 文件 | 說明 |
|:--|:--|
| [入門教學](getting-started.md) | 從零開始建構全端應用程式。涵蓋 `tairitsu new`、第一個元件、伺服器與瀏覽器執行、部署。 |
| [快速開始](quick-start.md) | 5 分鐘安裝與驗證。 |
| [工作區導覽](workspace-map.md) | Monorepo 結構導覽。 |
| [建置、測試與發佈](build-test-release.md) | 如何使用 `just` 進行開發工作流程。 |

## 遷移

| 文件 | 說明 |
|:--|:--|
| [從 web-sys 遷移到 WIT 繫結](migration.md) | 從 `wasm-bindgen`/`web-sys` 遷移到 Component Model WIT 繫結。 |

## 參考

| 文件 | 說明 |
|:--|:--|
| [術語表](glossary.md) | 核心術語：WIT、Component Model、VNode、Signal、Platform、Container 等 |
| [疑難排解](troubleshooting.md) | 常見問題與解決方案。 |

## 架構

| 文件 | 說明 |
|:--|:--|
| [系統總覽](../system/overview.md) | 四層架構：Interface → Runtime → Platform → Tooling |
| [執行階段與容器模型](../system/runtime.md) | Image/Container/Registry 生命週期、WIT 繫結、動態呼叫 |
| [VDOM 與渲染](../system/vdom.md) | 虛擬 DOM 差分比對、修補、事件系統、響應式排程器 |
| [W3C WebIDL → WIT 管線](../system/wit-pipeline.md) | 50+ WebIDL 規格如何轉換為 WIT 介面 |
| [Web 雙後端](../system/web-backends.md) | WitPlatform 與 WebPlatform 策略 |
| [Browser Glue 架構](../system/browser-glue.md) | 橋接 WIT ABI 與 DOM 的 TypeScript 層 |
| [版本策略](../system/versioning.md) | 多 Crate 工作區的語意化版本管理 |

## 套件參考

| 文件 | 說明 |
|:--|:--|
| [分層套件總覽](../components/index.md) | 四層 Crate 階層與依賴圖 |
| [工作區套件清單](../components/packages.md) | 各 Crate 的詳細說明 |
