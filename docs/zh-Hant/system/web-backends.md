# Web 後端：唯一路徑 = WIT 契約 + JS 宿主實作

`packages/web` 只有**一條** Web 後端路徑。早期文件描述的「雙後端」——`web`
（`wasm-bindgen` / `web-sys`）與 `wit-bindings`——已不存在：`web` 側沒有消費者，
其依賴與 `cfg` 分支均已退役。

## 這條路徑是什麼

| 項目 | 取值 |
| --- | --- |
| 編譯目標 | `wasm32-wasip2`（WASI 0.2 / 0.3 組件模型） |
| 契約 | WIT 世界 `tairitsu-browser:*`（`packages/browser-worlds/wit/`） |
| Guest 側綁定 | `wit-bindgen`（`wit-bindings` feature） |
| 瀏覽器側實作 | JS 宿主（`packages/browser-glue/`） |
| Guest 瀏覽器能力 | Component Model host，經宿主注入的 imports |

承載入口是 `WitPlatform`（`packages/web/src/wit_platform.rs`，`wit-bindings` feature）。
`BrowserPlatform`（`packages/web/src/browser.rs`）是佔位實作——立即回呼、預設 rect——
不是 web-sys 後端。

## 為什麼只剩這一條

1. **組件模型改變了「能力」的交付方式**。`wasm32-unknown-unknown` + `web-sys` 把瀏覽器能力
   直接編進 guest 的 import 段，宿主是誰、授予了什麼能力都無從宣告。WIT 契約把能力面
   寫成可審閱、可版本化的 import 介面，宿主按實例授權。
2. **建置期轉譯，不是執行期自舉**。jco 在建置期把 component 轉譯成 core wasm + JS
   canonical ABI 執行期；瀏覽器**沒有**原生 `WebAssembly.Component`。所以工程上要求的
   「契約在 WIT、實作在 JS」是當前唯一可行形態，不是取捨。
3. **宿主職責被顯式化**。實例化、能力授權、非同步排程、生命週期（含強制終止）歸 tairitsu
   宿主；guest 只描述它需要什麼。

## 世界層級

WIT 世界按 `tairitsu-browser:{domain}` 分包，位於 `packages/browser-worlds/wit/`
（`browser-full.wit` 與 `generated/` 下的按域包）。具體包清單以該目錄為準。

## 建置與打包

`packages/packager` 提供 `tairitsu build` CLI，**只支援 `component` 一個 target**
（`wasm32-wasip2` + browser-glue 路徑）。傳入其它 target 會以
`Unknown build target … Only 'component' is supported.` 失敗。

組件建置流程：

1. 驗證 `wasm32-wasip2` toolchain 已安裝
2. `cargo build --target wasm32-wasip2 --lib`
3. 將 `.wasm` 組件複製到 `output_dir`
4. 將 browser-glue 執行期複製到 `output_dir/browser-glue/`
5. 產生宿主 `index.html`（透過 browser-glue 載入組件）

> **前提**：在 `packages/browser-glue/` 目錄先執行 `npm run build` 以編譯 TypeScript glue 程式碼。

## 使用建議

- 新工程直接按 `wit-bindings` + WIT 世界撰寫；不存在需要「先保持 `web` 再遷移」的存量路徑。
- 需要新的瀏覽器能力時，先在 WIT 側宣告介面，再在 JS 宿主實作——不要在 guest 裡
  直接引用 DOM 綁定庫。
- 非同步 API 的契約形狀以 WIT 側目前宣告為準（見 `packages/browser-worlds/wit/`），
  本文不複述具體函式簽名。