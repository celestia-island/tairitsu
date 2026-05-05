use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-debug-api", class: "ts-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Guides", "/guides"), ("Debug API", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Debug API" }
                div { class: "hi-markdown-content",

                    div { id: "lang-switcher", style: "display:flex;gap:6px;flex-wrap:wrap;margin-bottom:16px;",
                        button { class: "hi-button hi-button-primary hi-button-sm lang-btn", "data-lang": "zh-CN", "zh-CN" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "en", "en" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "ja", "ja" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "ko", "ko" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "fr", "fr" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "de", "de" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "es", "es" }
                        button { class: "hi-button hi-button-ghost hi-button-sm lang-btn", "data-lang": "pt", "pt" }
                    }

                    ..{ let mut v = Vec::new(); v.extend(lang_zh_cn()); v.extend(lang_en()); v.extend(lang_ja()); v.extend(lang_ko()); v.extend(lang_fr()); v.extend(lang_de()); v.extend(lang_es()); v.extend(lang_pt()); v.extend(shared_examples()); v }
                }
            }
        }
    }
}

fn lang_zh_cn() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "zh-CN",
        h3 { "概述" }
        p { "Tairitsu 内置调试 API，在开发模式下自动启动（添加 ", code { "--debug" }, " 参数），提供浏览器自动化、截图、DOM 检查、性能监控等功能。基于 wry (WebKitGTK) 引擎，无需安装外部浏览器。" }
        p { "默认地址：开发服务器 ", code { "http://localhost:3000" }, "，调试 API ", code { "http://localhost:3001" }, "。" }

        h3 { "快速开始" }
        div { class: "hi-code-block language-bash", pre { code {
            "# 启动开发服务器（含调试模式）
tairitsu dev --port 3000 --debug

# 检查页面是否就绪
curl http://localhost:3001/ready

# 截取像素级截图
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "API 端点一览" }
        p { "所有端点返回统一 JSON 格式：", code { "{\"ok\":true,\"data\":{...}}" }, "，出错时 ", code { "{\"ok\":false,\"error\":\"...\"}" }, "。" }

        h4 { "只读端点" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "方法" } th { "路径" } th { "说明" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "健康检查" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "服务信息（端口、PID、运行时间等）" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "页面就绪状态（WASM 加载、水合完成）" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "视口尺寸和设备像素比" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "JS 错误和未处理的 Promise 拒绝" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "控制台日志" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "DOM 查询（标签、文本、HTML、位置）" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "无障碍树" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "已加载的网络资源" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "性能指标（FCP、DOM 节点数、WASM 状态）" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "WebSocket 连接状态" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "清空控制台日志" } }
            }
        } }

        h4 { "操作端点" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "方法" } th { "路径" } th { "说明" } th { "参数" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "页面导航" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "截图" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "点击元素" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "输入文本" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "按键" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "滚动" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "执行 JS 表达式" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "获取计算样式" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "拖拽元素" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "批量操作" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "源码映射解析" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "截图" }
        p { "截图通过 Canvas 实现，跨平台兼容。通过 JS 将页面内容绘制到 Canvas 再导出为 PNG（1280x720）。" }

        h3 { "批量操作" }
        p { "使用 ", code { "/batch" }, " 端点可以一次执行多个操作，每个操作按顺序执行：", }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }
        p { "支持的操作类型：", code { "navigate" }, "、", code { "screenshot" }, "、", code { "click" }, "、", code { "type" }, "、", code { "press" }, "、", code { "scroll" }, "、", code { "evaluate" }, "、", code { "wait" }, "。" }

        h3 { "错误处理" }
        p { "当浏览器未启动或操作超时时，API 返回 503 或 504 状态码。操作超时默认为 10 秒。" }
    } }]
}

fn lang_en() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "en", style: "display:none",
        h3 { "Overview" }
        p { "Tairitsu includes a built-in Debug API that starts automatically in dev mode (add ", code { "--debug" }, " flag). It provides browser automation, screenshots, DOM inspection, performance monitoring and more — powered by wry (WebKitGTK), no external browser required." }
        p { "Default: dev server ", code { "http://localhost:3000" }, ", debug API ", code { "http://localhost:3001" }, "." }

        h3 { "Quick Start" }
        div { class: "hi-code-block language-bash", pre { code {
            "# Start dev server with debug mode
tairitsu dev --port 3000 --debug

# Check if page is ready
curl http://localhost:3001/ready

# Take a pixel-perfect screenshot
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "API Endpoints" }
        p { "All endpoints return unified JSON: ", code { "{\"ok\":true,\"data\":{...}}" }, ", on error ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "Read Endpoints" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Method" } th { "Path" } th { "Description" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "Health check" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "Server info (ports, PID, uptime)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "Page ready state (WASM loaded, hydrated)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "Viewport size and device pixel ratio" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "JS errors and unhandled promise rejections" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "Console log entries" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "DOM query (tag, text, HTML, position)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "Accessibility tree" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "Loaded network resources" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "Performance metrics (FCP, DOM nodes, WASM state)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "WebSocket connection state" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "Clear console log" } }
            }
        } }

        h4 { "Action Endpoints" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Method" } th { "Path" } th { "Description" } th { "Parameters" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "Navigate to URL" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "Take screenshot" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "Click element" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "Type text" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "Press key" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "Scroll page" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "Evaluate JS expression" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "Get computed styles" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "Drag element" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "Batch operations" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "Source map resolution" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "Screenshots" }
        p { "Screenshots use Canvas for cross-platform compatibility. Page content is drawn to a Canvas via JS and exported as PNG (1280x720)." }

        h3 { "Batch Operations" }
        p { "Use ", code { "/batch" }, " to execute multiple operations sequentially:" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }
        p { "Supported operation types: ", code { "navigate" }, ", ", code { "screenshot" }, ", ", code { "click" }, ", ", code { "type" }, ", ", code { "press" }, ", ", code { "scroll" }, ", ", code { "evaluate" }, ", ", code { "wait" }, "." }

        h3 { "Error Handling" }
        p { "When the browser is not started or an operation times out, the API returns 503 or 504 status codes. Default operation timeout is 10 seconds." }
    } }]
}

fn lang_ja() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "ja", style: "display:none",
        h3 { "概要" }
        p { "Tairitsuは開発モードで自動起動する内蔵デバッグAPIを備えています（", code { "--debug" }, " フラグを追加）。ブラウザ自動化、スクリーンショット、DOM検査、パフォーマンス監視などの機能を提供します。wry (WebKitGTK) エンジンを使用し、外部ブラウザは不要です。" }
        p { "デフォルト：開発サーバー ", code { "http://localhost:3000" }, "、デバッグAPI ", code { "http://localhost:3001" }, "。" }

        h3 { "クイックスタート" }
        div { class: "hi-code-block language-bash", pre { code {
            "# デバッグモードで開発サーバーを起動
tairitsu dev --port 3000 --debug

# ページの準備完了を確認
curl http://localhost:3001/ready

# ピクセル単位のスクリーンショットを取得
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "APIエンドポイント一覧" }
        p { "すべてのエンドポイントは統一JSON形式で返します：", code { "{\"ok\":true,\"data\":{...}}" }, "、エラー時 ", code { "{\"ok\":false,\"error\":\"...\"}" }, "。" }

        h4 { "読み取り専用エンドポイント" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "メソッド" } th { "パス" } th { "説明" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "ヘルスチェック" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "サーバー情報（ポート、PID、稼働時間）" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "ページ準備状態（WASM読込、ハイドレーション完了）" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "ビューポートサイズとデバイスピクセル比" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "JSエラーと未処理のPromise拒否" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "コンソールログ" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "DOMクエリ（タグ、テキスト、HTML、位置）" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "アクセシビリティツリー" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "読み込み済みネットワークリソース" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "パフォーマンス指標（FCP、DOMノード数、WASM状態）" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "WebSocket接続状態" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "コンソールログをクリア" } }
            }
        } }

        h4 { "操作エンドポイント" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "メソッド" } th { "パス" } th { "説明" } th { "パラメータ" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "ページナビゲーション" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "スクリーンショット" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "要素をクリック" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "テキスト入力" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "キーを押す" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "ページスクロール" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "JS式を実行" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "計算スタイルを取得" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "要素をドラッグ" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "バッチ操作" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "ソースマップ解決" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "スクリーンショット" }
        p { "スクリーンショットはクロスプラットフォーム互換のCanvasを使用します。JSでページ内容をCanvasに描画し、PNG（1280x720）としてエクスポートします。" }

        h3 { "バッチ操作" }
        p { code { "/batch" }, " を使用して複数の操作を順次実行します：" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }
        p { "対応操作タイプ：", code { "navigate" }, "、", code { "screenshot" }, "、", code { "click" }, "、", code { "type" }, "、", code { "press" }, "、", code { "scroll" }, "、", code { "evaluate" }, "、", code { "wait" }, "。" }

        h3 { "エラー処理" }
        p { "ブラウザが起動していない場合や操作がタイムアウトした場合、APIは503または504ステータスコードを返します。デフォルトの操作タイムアウトは10秒です。" }
    } }]
}

fn lang_ko() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "ko", style: "display:none",
        h3 { "개요" }
        p { "Tairitsu는 개발 모드에서 자동으로 시작되는 내장 디버그 API를 제공합니다（", code { "--debug" }, " 플래그 추가). 브라우저 자동화, 스크린샷, DOM 검사, 성능 모니터링 등의 기능을 제공합니다. wry (WebKitGTK) 엔진 기반으로 외부 브라우저가 필요 없습니다." }
        p { "기본값: 개발 서버 ", code { "http://localhost:3000" }, ", 디버그 API ", code { "http://localhost:3001" }, "." }

        h3 { "빠른 시작" }
        div { class: "hi-code-block language-bash", pre { code {
            "# 디버그 모드로 개발 서버 시작
tairitsu dev --port 3000 --debug

# 페이지 준비 상태 확인
curl http://localhost:3001/ready

# 픽셀 단위 스크린샷 캡처
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "API 엔드포인트" }
        p { "모든 엔드포인트는 통일된 JSON 형식을 반환합니다: ", code { "{\"ok\":true,\"data\":{...}}" }, ", 오류 시 ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "읽기 엔드포인트" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "메서드" } th { "경로" } th { "설명" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "상태 확인" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "서버 정보 (포트, PID, 가동 시간)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "페이지 준비 상태 (WASM 로드, 하이드레이션)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "뷰포트 크기 및 기기 픽셀 비율" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "JS 오류 및 처리되지 않은 Promise 거부" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "콘솔 로그" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "DOM 쿼리 (태그, 텍스트, HTML, 위치)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "접근성 트리" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "로드된 네트워크 리소스" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "성능 지표 (FCP, DOM 노드 수, WASM 상태)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "WebSocket 연결 상태" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "콘솔 로그 지우기" } }
            }
        } }

        h4 { "작업 엔드포인트" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "메서드" } th { "경로" } th { "설명" } th { "매개변수" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "페이지 탐색" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "스크린샷" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "요소 클릭" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "텍스트 입력" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "키 누르기" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "페이지 스크롤" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "JS 표현식 실행" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "계산된 스타일 가져오기" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "요소 드래그" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "배치 작업" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "소스 맵 해석" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "스크린샷" }
        p { "스크린샷은 크로스 플랫폼 호환을 위해 Canvas를 사용합니다. JS로 페이지 내용을 Canvas에 그린 후 PNG(1280x720)로 내보냅니다." }

        h3 { "배치 작업" }
        p { code { "/batch" }, " 를 사용하여 여러 작업을 순차적으로 실행합니다：" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }

        h3 { "오류 처리" }
        p { "브라우저가 시작되지 않았거나 작업이 시간 초과된 경우 503 또는 504 상태 코드가 반환됩니다. 기본 작업 시간 초과는 10초입니다." }
    } }]
}

fn lang_fr() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "fr", style: "display:none",
        h3 { "Aperçu" }
        p { "Tairitsu inclut une API de débogage intégrée qui démarre automatiquement en mode développement (ajoutez le flag ", code { "--debug" }, "). Elle offre l'automatisation du navigateur, les captures d'écran, l'inspection DOM, la surveillance des performances et plus encore — propulsée par wry (WebKitGTK), aucun navigateur externe requis." }
        p { "Par défaut : serveur de développement ", code { "http://localhost:3000" }, ", API de débogage ", code { "http://localhost:3001" }, "." }

        h3 { "Démarrage rapide" }
        div { class: "hi-code-block language-bash", pre { code {
            "# Démarrer le serveur en mode débogage
tairitsu dev --port 3000 --debug

# Vérifier si la page est prête
curl http://localhost:3001/ready

# Capturer une capture d'écran pixel-parfaite
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "Endpoints de l'API" }
        p { "Tous les endpoints retournent un JSON unifié : ", code { "{\"ok\":true,\"data\":{...}}" }, ", en cas d'erreur ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "Endpoints de lecture" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Méthode" } th { "Chemin" } th { "Description" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "Vérification de santé" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "Info serveur (ports, PID, temps de fonctionnement)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "État de préparation (WASM chargé, hydratation)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "Taille de la fenêtre et ratio de pixels" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "Erreurs JS et rejets de Promise non gérés" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "Journal de la console" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "Requête DOM (balise, texte, HTML, position)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "Arbre d'accessibilité" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "Ressources réseau chargées" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "Métriques de performance (FCP, nœuds DOM, état WASM)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "État des connexions WebSocket" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "Vider le journal de la console" } }
            }
        } }

        h4 { "Endpoints d'action" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Méthode" } th { "Chemin" } th { "Description" } th { "Paramètres" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "Navigation de page" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "Capture d'écran" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "Cliquer sur un élément" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "Saisir du texte" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "Appuyer sur une touche" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "Faire défiler la page" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "Évaluer une expression JS" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "Styles calculés" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "Glisser un élément" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "Opérations par lot" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "Résolution de carte source" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "Captures d'écran" }
        p { "Les captures d'écran utilisent Canvas pour la compatibilité multiplateforme. Le contenu de la page est dessiné sur un Canvas via JS et exporté en PNG (1280x720)." }

        h3 { "Opérations par lot" }
        p { "Utilisez ", code { "/batch" }, " pour exécuter plusieurs opérations séquentiellement :" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }

        h3 { "Gestion des erreurs" }
        p { "Lorsque le navigateur n'est pas démarré ou qu'une opération expire, l'API renvoie les codes d'état 503 ou 504. Le délai d'expiration par défaut est de 10 secondes." }
    } }]
}

fn lang_de() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "de", style: "display:none",
        h3 { "Übersicht" }
        p { "Tairitsu enthält eine integrierte Debug-API, die im Entwicklungsmodus automatisch startet (", code { "--debug" }, " Flag hinzufügen). Sie bietet Browser-Automatisierung, Screenshots, DOM-Inspektion, Leistungsüberwachung und mehr — basierend auf wry (WebKitGTK), kein externer Browser erforderlich." }
        p { "Standard: Entwicklungsserver ", code { "http://localhost:3000" }, ", Debug-API ", code { "http://localhost:3001" }, "." }

        h3 { "Schnellstart" }
        div { class: "hi-code-block language-bash", pre { code {
            "# Entwicklungsserver im Debug-Modus starten
tairitsu dev --port 3000 --debug

# Prüfen ob die Seite bereit ist
curl http://localhost:3001/ready

# Pixelgenauen Screenshot erstellen
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "API-Endpunkte" }
        p { "Alle Endpunkte geben einheitliches JSON zurück: ", code { "{\"ok\":true,\"data\":{...}}" }, ", bei Fehler ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "Lese-Endpunkte" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Methode" } th { "Pfad" } th { "Beschreibung" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "Gesundheitsprüfung" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "Server-Info (Ports, PID, Betriebszeit)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "Seitenbereitschaft (WASM geladen, Hydriert)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "Viewport-Größe und Gerätepixelverhältnis" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "JS-Fehler und unbehandelte Promise-Ablehnungen" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "Konsolenprotokoll" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "DOM-Abfrage (Tag, Text, HTML, Position)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "Barrierefreiheitsbaum" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "Geladene Netzwerkressourcen" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "Leistungsmetriken (FCP, DOM-Knoten, WASM-Status)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "WebSocket-Verbindungsstatus" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "Konsole leeren" } }
            }
        } }

        h4 { "Aktions-Endpunkte" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Methode" } th { "Pfad" } th { "Beschreibung" } th { "Parameter" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "Seitennavigation" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "Screenshot" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "Element anklicken" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "Text eingeben" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "Taste drücken" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "Seite scrollen" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "JS-Ausdruck ausführen" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "Berechnete Stile" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "Element ziehen" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "Stapelverarbeitung" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "Quellcode-Zuordnung" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "Screenshots" }
        p { "Screenshots verwenden Canvas für Plattformübergreifende Kompatibilität. Seiteninhalte werden über JS auf ein Canvas gezeichnet und als PNG (1280x720) exportiert." }

        h3 { "Stapelverarbeitung" }
        p { "Verwenden Sie ", code { "/batch" }, " für die sequenzielle Ausführung mehrerer Operationen:" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }

        h3 { "Fehlerbehandlung" }
        p { "Wenn der Browser nicht gestartet ist oder eine Operation abläuft, gibt die API 503- oder 504-Statuscodes zurück. Standard-Operationstimeout: 10 Sekunden." }
    } }]
}

fn lang_es() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "es", style: "display:none",
        h3 { "Resumen" }
        p { "Tairitsu incluye una API de depuración integrada que se inicia automáticamente en modo de desarrollo (añada el flag ", code { "--debug" }, "). Proporciona automatización del navegador, capturas de pantalla, inspección DOM, monitoreo de rendimiento y más — basada en wry (WebKitGTK), sin navegador externo necesario." }
        p { "Por defecto: servidor de desarrollo ", code { "http://localhost:3000" }, ", API de depuración ", code { "http://localhost:3001" }, "." }

        h3 { "Inicio rápido" }
        div { class: "hi-code-block language-bash", pre { code {
            "# Iniciar servidor con modo de depuración
tairitsu dev --port 3000 --debug

# Verificar si la página está lista
curl http://localhost:3001/ready

# Capturar pantalla pixel-perfecta
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "Endpoints de la API" }
        p { "Todos los endpoints devuelven JSON unificado: ", code { "{\"ok\":true,\"data\":{...}}" }, ", en error ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "Endpoints de lectura" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Método" } th { "Ruta" } th { "Descripción" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "Comprobación de salud" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "Info del servidor (puertos, PID, tiempo activo)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "Estado de preparación (WASM cargado, hidratado)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "Tamaño del viewport y ratio de píxeles" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "Errores JS y rechazos de Promise no manejados" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "Registro de consola" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "Consulta DOM (etiqueta, texto, HTML, posición)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "Árbol de accesibilidad" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "Recursos de red cargados" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "Métricas de rendimiento (FCP, nodos DOM, estado WASM)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "Estado de conexiones WebSocket" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "Limpiar registro de consola" } }
            }
        } }

        h4 { "Endpoints de acción" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Método" } th { "Ruta" } th { "Descripción" } th { "Parámetros" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "Navegación de página" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "Captura de pantalla" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "Hacer clic en elemento" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "Escribir texto" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "Pulsar tecla" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "Desplazar página" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "Evaluar expresión JS" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "Estilos calculados" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "Arrastrar elemento" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "Operaciones por lotes" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "Resolución de mapa de origen" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "Capturas de pantalla" }
        p { "Las capturas de pantalla usan Canvas para compatibilidad multiplataforma. El contenido de la página se dibuja en un Canvas vía JS y se exporta como PNG (1280x720)." }

        h3 { "Operaciones por lotes" }
        p { "Use ", code { "/batch" }, " para ejecutar múltiples operaciones secuencialmente:" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }

        h3 { "Manejo de errores" }
        p { "Cuando el navegador no está iniciado o una operación expira, la API devuelve códigos de estado 503 o 504. Tiempo de espera predeterminado: 10 segundos." }
    } }]
}

fn lang_pt() -> Vec<VNode> {
    vec![rsx! { div { "data-lang": "pt", style: "display:none",
        h3 { "Visão geral" }
        p { "Tairitsu inclui uma API de depuração integrada que inicia automaticamente no modo de desenvolvimento (adicione o flag ", code { "--debug" }, "). Fornece automação de navegador, capturas de tela, inspeção DOM, monitoramento de desempenho e mais — baseada em wry (WebKitGTK), sem navegador externo necessário." }
        p { "Padrão: servidor de desenvolvimento ", code { "http://localhost:3000" }, ", API de depuração ", code { "http://localhost:3001" }, "." }

        h3 { "Início rápido" }
        div { class: "hi-code-block language-bash", pre { code {
            "# Iniciar servidor com modo de depuração
tairitsu dev --port 3000 --debug

# Verificar se a página está pronta
curl http://localhost:3001/ready

# Capturar tela pixel-perfeita
curl -X POST http://localhost:3001/screenshot \\
  -H 'Content-Type: application/json' \\
  -d '{}'"
        } } }

        h3 { "Endpoints da API" }
        p { "Todos os endpoints retornam JSON unificado: ", code { "{\"ok\":true,\"data\":{...}}" }, ", em caso de erro ", code { "{\"ok\":false,\"error\":\"...\"}" }, "." }

        h4 { "Endpoints de leitura" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Método" } th { "Caminho" } th { "Descrição" } } }
            tbody {
                tr { td { code { "GET" } } td { code { "/health" } } td { "Verificação de saúde" } }
                tr { td { code { "GET" } } td { code { "/info" } } td { "Info do servidor (portas, PID, tempo de atividade)" } }
                tr { td { code { "GET" } } td { code { "/ready" } } td { "Estado de prontidão (WASM carregado, hidratado)" } }
                tr { td { code { "GET" } } td { code { "/viewport" } } td { "Tamanho do viewport e proporção de pixels" } }
                tr { td { code { "GET" } } td { code { "/errors" } } td { "Erros JS e rejeições de Promise não tratadas" } }
                tr { td { code { "GET" } } td { code { "/console" } } td { "Registro do console" } }
                tr { td { code { "GET" } } td { code { "/dom?selector=..." } } td { "Consulta DOM (tag, texto, HTML, posição)" } }
                tr { td { code { "GET" } } td { code { "/a11y" } } td { "Árvore de acessibilidade" } }
                tr { td { code { "GET" } } td { code { "/network" } } td { "Recursos de rede carregados" } }
                tr { td { code { "GET" } } td { code { "/performance" } } td { "Métricas de desempenho (FCP, nós DOM, estado WASM)" } }
                tr { td { code { "GET" } } td { code { "/websocket" } } td { "Estado de conexões WebSocket" } }
                tr { td { code { "DELETE" } } td { code { "/console" } } td { "Limpar registro do console" } }
            }
        } }

        h4 { "Endpoints de ação" }
        div { class: "table-responsive", table { class: "hi-table",
            thead { tr { th { "Método" } th { "Caminho" } th { "Descrição" } th { "Parâmetros" } } }
            tbody {
                tr { td { code { "POST" } } td { code { "/navigate" } } td { "Navegação de página" } td { code { "{\"url\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/screenshot" } } td { "Captura de tela" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/click" } } td { "Clicar no elemento" } td { code { "{\"selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/type" } } td { "Digitar texto" } td { code { "{\"selector\":\"...\",\"text\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/press" } } td { "Pressionar tecla" } td { code { "{\"key\":\"Enter|Escape|Tab...\"}" } } }
                tr { td { code { "POST" } } td { code { "/scroll" } } td { "Rolar página" } td { code { "{\"direction\":\"up|down\",\"amount\":100}" } } }
                tr { td { code { "POST" } } td { code { "/evaluate" } } td { "Executar expressão JS" } td { code { "{\"expression\":\"1+1\"}" } } }
                tr { td { code { "POST" } } td { code { "/dom/computed" } } td { "Estilos computados" } td { code { "{\"selector\":\"...\",\"properties\":[\"color\"]}" } } }
                tr { td { code { "POST" } } td { code { "/drag" } } td { "Arrastar elemento" } td { code { "{\"from_selector\":\"...\",\"to_selector\":\"...\"}" } } }
                tr { td { code { "POST" } } td { code { "/batch" } } td { "Operações em lote" } td { code { "{\"operations\":[...]}" } } }
                tr { td { code { "POST" } } td { code { "/source-map" } } td { "Resolução de mapa de fonte" } td { code { "{\"stack\":\"Error at...\"}" } } }
            }
        } }

        h3 { "Capturas de tela" }
        p { "As capturas de tela usam Canvas para compatibilidade multiplataforma. O conteúdo da página é desenhado em um Canvas via JS e exportado como PNG (1280x720)." }

        h3 { "Operações em lote" }
        p { "Use ", code { "/batch" }, " para executar múltiplas operações sequencialmente:" }
        div { class: "hi-code-block language-bash", pre { code {
            "curl -X POST http://localhost:3001/batch \\
  -H 'Content-Type: application/json' \\
  -d '{
    \"operations\": [
      {\"type\": \"navigate\", \"url\": \"http://localhost:3000/\"},
      {\"type\": \"evaluate\", \"expression\": \"document.title\"},
      {\"type\": \"screenshot\"},
      {\"type\": \"click\", \"selector\": \"#my-button\"}
    ]
  }'"
        } } }

        h3 { "Tratamento de erros" }
        p { "Quando o navegador não está iniciado ou uma operação expira, a API retorna códigos de status 503 ou 504. Tempo limite padrão: 10 segundos." }
    } }]
}

fn shared_examples() -> Vec<VNode> {
    vec![
        rsx! { script { r#"
(function(){
    var init=false;
    function setup(){
        if(init)return;init=true;
        var el=document.getElementById('lang-switcher');
        if(!el)return;
        el.addEventListener('click',function(e){
            var btn=e.target.closest('.lang-btn');
            if(!btn)return;
            var lang=btn.getAttribute('data-lang');
            el.querySelectorAll('.lang-btn').forEach(function(b){
                b.className='hi-button hi-button-ghost hi-button-sm lang-btn';
            });
            btn.className='hi-button hi-button-primary hi-button-sm lang-btn';
            var page=document.getElementById('page-guides-debug-api');
            if(!page)return;
            page.querySelectorAll('[data-lang]').forEach(function(d){
                if(d.tagName==='BUTTON')return;
                d.style.display=d.getAttribute('data-lang')===lang?'block':'none';
            });
        });
    }
    if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',setup);
    else setup();
})();
        "#} },
    ]
}
