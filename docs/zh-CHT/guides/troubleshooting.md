# 疑難排解指南

使用 Tairitsu browser-glue 與元件模型時的常見問題與解決方案。

## 建置錯誤

### 找不到 wasm32-wasip2 目標

**錯誤訊息：**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**解決方案：**
```bash
rustup target add wasm32-wasip2
```

### wit-bindgen 版本不符

**錯誤訊息：**
```
error: failed to select a version for `wit-bindgen`
```

**解決方案：**
確保 `Cargo.toml` 中的 `wit-bindgen` 版本正確：
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### TypeScript 編譯錯誤

**錯誤訊息：**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**解決方案：**
重新產生 glue 並重新建置：
```bash
cd packages/browser-glue
npm run build
```

## 執行階段錯誤

### 缺少主機匯入

**錯誤訊息：**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**解決方案：**
1. 確保已設定匯入映射：
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. 確認 browser-glue 檔案存在於輸出目錄中。

### 元件初始化失敗

**錯誤訊息：**
```
Error: Component instantiation failed: undefined import
```

**解決方案：**
檢查所有 WIT 匯入是否在 browser-glue 中都有對應的實作。

### jco 轉譯錯誤

**錯誤訊息：**
```
Error: Failed to transpile component
```

**解決方案：**
1. 確保已安裝 jco：
```bash
npm install -g @bytecodealliance/jco
```

2. 驗證 WASM 元件是否有效：
```bash
wasm-tools print component.wasm
```

## 除錯技巧

### 啟用除錯日誌

在瀏覽器主控台中執行：
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### 檢查 WIT 繫結

檢視產生的繫結：
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### 瀏覽器開發者工具

1. 開啟開發者工具（F12）
2. 檢查主控台的錯誤訊息
3. 使用網路面分頁檢查失敗的模組載入
4. 使用來源面分頁進行除錯

### 元件驗證

```bash
# 驗證元件結構
wasm-tools validate component.wasm

# 印出元件內容
wasm-tools print component.wasm
```

## 常見問題

### 找不到控制代碼

**症狀：** DOM 操作回傳 `null`

**原因：** 控制代碼被垃圾回收或未註冊

**解決方案：** 確保元素在 JavaScript 中保持被參照

### 事件未觸發

**症狀：** 事件處理器未被呼叫

**原因：** 監聽器 ID 不符或事件類型錯誤

**解決方案：** 檢查 `addEventListener` 是否回傳有效的監聽器 ID

### 記憶體流失

**症狀：** 記憶體使用量隨時間增加

**原因：** 控制代碼使用後未釋放

**解決方案：** 物件使用完畢後呼叫 `dropHandle()`

## 效能問題

### 元件載入緩慢

**解決方案：**
1. 使用 release 建置：`cargo build --release`
2. 在 `Cargo.toml` 中啟用 LTO：
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### 事件延遲過高

**解決方案：**
1. 避免在處理器中執行同步操作
2. 使用 `requestAnimationFrame` 進行視覺更新
3. 對快速觸發的事件進行防抖處理

## 取得協助

1. 查看現有議題：https://github.com/anomalyco/opencode/issues
2. 參閱 `docs/` 目錄中的文件
3. 研究 `examples/website/` 中的範例程式碼
