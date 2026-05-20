# Tairitsu ドキュメントハブ（日本語）

> WASM Component Model 基盤のフルスタックフレームワーク

## 入門

| ドキュメント | 説明 |
|:--|:--|
| [入門チュートリアル](getting-started.md) | ゼロからフルスタックアプリを構築。`tairitsu new`、初めてのコンポーネント、サーバー＋ブラウザ実行、デプロイまで。 |
| [クイックスタート](quick-start.md) | 5分でセットアップと動作確認。 |
| [ワークスペース構成](workspace-map.md) | モノレポ構造のツアー。 |
| [ビルド・テスト・リリース](build-test-release.md) | `just` レシピを使った開発ワークフロー。 |

## マイグレーション

| ドキュメント | 説明 |
|:--|:--|
| [web-sys から WIT バインディングへ](migration.md) | `wasm-bindgen`/`web-sys` から Component Model WIT バインディングへの移行。 |

## リファレンス

| ドキュメント | 説明 |
|:--|:--|
| [用語集](glossary.md) | 主要用語: WIT, Component Model, VNode, Signal, Platform, Container など |
| [トラブルシューティング](troubleshooting.md) | よくある問題と解決方法。 |

## アーキテクチャ

| ドキュメント | 説明 |
|:--|:--|
| [システム概要](../system/overview.md) | 4層アーキテクチャ: Interface → Runtime → Platform → Tooling |
| [ランタイムとコンテナモデル](../system/runtime.md) | Image/Container/Registry ライフサイクル、WIT バインディング、動的呼び出し |
| [VDOM とレンダリング](../system/vdom.md) | 仮想 DOM の差分、パッチ、イベントシステム、リアクティブスケジューラ |
| [W3C WebIDL → WIT パイプライン](../system/wit-pipeline.md) | 50+ の WebIDL 仕様がどのように WIT インターフェースになるか |
| [Web バックエンド二系統](../system/web-backends.md) | WitPlatform と WebPlatform の戦略 |
| [Browser Glue アーキテクチャ](../system/browser-glue.md) | WIT ABI と DOM を繋ぐ TypeScript 層 |
| [バージョニング戦略](../system/versioning.md) | マルチ Crate ワークスペースのセマンティックバージョニング |

## パッケージリファレンス

| ドキュメント | 説明 |
|:--|:--|
| [レイヤー別パッケージ概要](../components/index.md) | 4層 Crate 階層と依存グラフ |
| [ワークスペースパッケージ一覧](../components/packages.md) | 各 Crate の詳細説明 |

## 上級

| ドキュメント | 説明 |
|:--|:--|
| [エンタープライズサポート](../enterprise/support.md) | 商用サポートオプション |
