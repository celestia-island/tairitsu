# トラブルシューティングガイド

Tairitsu browser-glueとコンポーネントモデルを使用する際の一般的な問題と解決策。

## ビルドエラー

### wasm32-wasip2ターゲットが見つからない

**エラー:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**解決策:**
```bash
rustup target add wasm32-wasip2
```

### wit-bindgenバージョンの不一致

**エラー:**
```
error: failed to select a version for `wit-bindgen`
```

**解決策:**
`Cargo.toml`で `wit-bindgen` バージョンが一致していることを確認：
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### TypeScriptコンパイルエラー

**エラー:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**解決策:**
グルーを再生成してリビルド：
```bash
cd packages/browser-glue
npm run build
```

## ランタイムエラー

### ホストインポートが見つからない

**エラー:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**解決策:**
1. Import Mapが設定されていることを確認：
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. browser-glueファイルが出力ディレクトリに存在することを確認。

### コンポーネント初期化失敗

**エラー:**
```
Error: Component instantiation failed: undefined import
```

**解決策:**
必要なすべてのWITインポートに対応する実装がbrowser-glueにあることを確認。

### jcoトランスパイルエラー

**エラー:**
```
Error: Failed to transpile component
```

**解決策:**
1. jcoがインストールされていることを確認：
```bash
npm install -g @bytecodealliance/jco
```

2. WASMコンポーネントが有効であることを確認：
```bash
wasm-tools print component.wasm
```

## デバッグ手法

### デバッグログの有効化

ブラウザコンソールで：
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### WITバインディングの確認

生成されたバインディングを表示：
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### ブラウザDevTools

1. DevToolsを開く (F12)
2. Consoleでエラーを確認
3. Networkタブでモジュール読み込み失敗を確認
4. Sourcesタブでデバッグ

### コンポーネント検証

```bash
# コンポーネント構造を検証
wasm-tools validate component.wasm

# コンポーネントの内容を表示
wasm-tools print component.wasm
```

## 一般的な問題

### ハンドルが見つからない

**症状:** DOM操作から `null` が返される

**原因:** ハンドルがガベージコレクションされた、または登録されていない

**解決策:** 要素がJavaScriptで参照され続けていることを確認

### イベントが発火しない

**症状:** イベントハンドラが呼び出されない

**原因:** リスナーIDの不一致、またはイベントタイプが正しくない

**解決策:** `addEventListener` が有効なリスナーIDを返すことを確認

### メモリリーク

**症状:** 時間の経過とともにメモリ使用量が増加

**原因:** ハンドルが使用後に解放されていない

**解決策:** オブジェクトの使用後に `dropHandle()` を呼び出す

## パフォーマンスの問題

### コンポーネントの読み込みが遅い

**解決策:**
1. リリースビルドを使用: `cargo build --release`
2. `Cargo.toml` でLTOを有効化：
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### イベントレイテンシが高い

**解決策:**
1. ハンドラ内で同期操作を避ける
2. 視覚的更新に `requestAnimationFrame` を使用
3. 高速なイベントをデバウンス

## ヘルプを得る

1. 既存のIssueを確認: https://github.com/anomalyco/opencode/issues
2. `docs/` ディレクトリのドキュメントを確認
3. `examples/website/` のサンプルコードを確認
