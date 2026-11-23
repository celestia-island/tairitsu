# Web バックエンド：唯一の経路 = WIT 契約 + JS ホスト実装

`packages/web` の Web バックエンド経路は**一つだけ**です。以前のドキュメントが説明していた
「二系統」——`web`（`wasm-bindgen` / `web-sys`）と `wit-bindings`——はもう存在しません。
`web` 側には利用者がおらず、依存関係と `cfg` 分岐は撤去済みです。

## この経路とは

| 項目 | 値 |
| --- | --- |
| コンパイルターゲット | `wasm32-wasip2`（WASI 0.2 / 0.3 コンポーネントモデル） |
| 契約 | WIT ワールド `tairitsu-browser:*`（`packages/browser-worlds/wit/`） |
| ゲスト側バインディング | `wit-bindgen`（`wit-bindings` feature） |
| ブラウザ側実装 | JS ホスト（`packages/browser-glue/`） |
| ゲストのブラウザ機能 | Component Model host（ホストが注入する imports 経由） |

入口は `WitPlatform`（`packages/web/src/wit_platform.rs`、`wit-bindings` feature）です。
`BrowserPlatform`（`packages/web/src/browser.rs`）はプレースホルダ実装——即時コールバック、
既定の rect——であり、web-sys バックエンドではありません。

## なぜ一本だけになったか

1. **コンポーネントモデルは「機能」の受け渡し方を変えます**。`wasm32-unknown-unknown` +
   `web-sys` はブラウザ機能をゲストの import セクションに直接埋め込むため、ホストが誰か、
   何が許可されたかを宣言できません。WIT 契約は機能面をレビュー可能でバージョン管理可能な
   import インターフェースとして表し、ホストがインスタンスごとに許可します。
2. **ビルド時のトランスパイルであり、実行時の自己ホストではありません**。jco はビルド時に
   component を core wasm + JS canonical ABI ランタイムへ変換します。ブラウザにはネイティブの
   `WebAssembly.Component` は**ありません**。したがって「契約は WIT、実装は JS」は現時点で
   唯一成立する形であり、トレードオフではありません。
3. **ホストの責務が明示されます**。インスタンス化、機能の許可、非同期スケジューリング、
   ライフサイクル（強制終了を含む）は tairitsu ホストが担い、ゲストは必要なものを記述するだけです。

## ワールドの階層

WIT ワールドは `tairitsu-browser:{domain}` という名前で `packages/browser-worlds/wit/` 配下に
分割されています（`browser-full.wit` と、`generated/` に生成されるドメイン別パッケージ）。
パッケージ一覧はこのディレクトリが正とします。

## ビルドとパッケージング

`packages/packager` の `tairitsu build` CLI が対応する target は **`component` のみ**です
（`wasm32-wasip2` + browser-glue 経路）。それ以外の target は
`Unknown build target … Only 'component' is supported.` で失敗します。

コンポーネントのビルド手順：

1. `wasm32-wasip2` toolchain が入っていることを確認
2. `cargo build --target wasm32-wasip2 --lib`
3. `.wasm` コンポーネントを `output_dir` にコピー
4. browser-glue ランタイムを `output_dir/browser-glue/` にコピー
5. ホスト用 `index.html` を生成（browser-glue 経由でコンポーネントを読み込む）

> **前提**：TypeScript glue をコンパイルするため、先に `packages/browser-glue/` で
> `npm run build` を実行してください。

## 推奨事項

- 新しいプロジェクトは `wit-bindings` と WIT ワールドに直接合わせて書いてください。
  「まず `web` のままで、後で移行する」という既存経路はありません。
- 新しいブラウザ機能が必要なときは、まず WIT 側でインターフェースを宣言し、JS ホスト側で
  実装します。ゲストに DOM バインディングのライブラリを直接入れないでください。
- 非同期 API の契約形状は `packages/browser-worlds/wit/` の現在の宣言が正です。
  本ページは個々の関数シグネチャを再掲しません。