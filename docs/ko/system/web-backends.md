# Web 백엔드: 유일한 경로 = WIT 계약 + JS 호스트 구현

`packages/web`의 Web 백엔드 경로는 **하나뿐**입니다. 이전 문서가 설명하던 "이중 백엔드" —
`web`(`wasm-bindgen` / `web-sys`)과 `wit-bindings` — 는 더 이상 존재하지 않습니다. `web` 쪽에는
소비자가 없었고, 그 의존성과 `cfg` 분기는 이미 철거되었습니다.

## 이 경로는 무엇인가

| 항목 | 값 |
| --- | --- |
| 컴파일 타깃 | `wasm32-wasip2`(WASI 0.2 / 0.3 컴포넌트 모델) |
| 계약 | WIT 월드 `tairitsu-browser:*`(`packages/browser-worlds/wit/`) |
| 게스트 측 바인딩 | `wit-bindgen`(`wit-bindings` feature) |
| 브라우저 측 구현 | JS 호스트(`packages/browser-glue/`) |
| 게스트의 브라우저 기능 | Component Model host(호스트가 주입하는 imports 경유) |

진입점은 `WitPlatform`(`packages/web/src/wit_platform.rs`, `wit-bindings` feature)입니다.
`BrowserPlatform`(`packages/web/src/browser.rs`)은 자리표시자 구현 — 즉시 콜백, 기본 rect —
이며 web-sys 백엔드가 아닙니다.

## 왜 이 경로만 남았는가

1. **컴포넌트 모델은 "기능"을 전달하는 방식을 바꿉니다.** `wasm32-unknown-unknown` +
   `web-sys`는 브라우저 기능을 게스트의 import 섹션에 직접 넣기 때문에, 호스트가 누구인지,
   어떤 권한이 주어졌는지 선언할 수 없습니다. WIT 계약은 기능 면을 검토 가능하고 버전
   관리 가능한 import 인터페이스로 서술하며, 호스트가 인스턴스별로 권한을 부여합니다.
2. **빌드 시점 변환이지 런타임 자기 호스팅이 아닙니다.** jco는 빌드 시점에 component를
   core wasm + JS canonical ABI 런타임으로 변환합니다. 브라우저에는 네이티브
   `WebAssembly.Component`가 **없습니다**. 따라서 "계약은 WIT, 구현은 JS"는 현재 유일하게
   성립하는 형태이며, 트레이드오프가 아닙니다.
3. **호스트의 책임이 명시됩니다.** 인스턴스화, 기능 권한 부여, 비동기 스케줄링, 수명 주기
   (강제 종료 포함)는 tairitsu 호스트가 맡고, 게스트는 필요한 것만 기술합니다.

## 월드 계층

WIT 월드는 `packages/browser-worlds/wit/` 아래에 `tairitsu-browser:{domain}` 이름으로
나뉘어 있습니다(`browser-full.wit` 와 `generated/` 에 생성되는 도메인별 패키지). 패키지
목록은 이 디렉터리를 기준으로 합니다.

## 빌드와 패키징

`packages/packager`의 `tairitsu build` CLI가 지원하는 target은 **`component` 하나뿐**입니다
(`wasm32-wasip2` + browser-glue 경로). 다른 target은
`Unknown build target … Only 'component' is supported.` 로 실패합니다.

컴포넌트 빌드 절차:

1. `wasm32-wasip2` toolchain이 설치되어 있는지 확인
2. `cargo build --target wasm32-wasip2 --lib`
3. `.wasm` 컴포넌트를 `output_dir` 로 복사
4. browser-glue 런타임을 `output_dir/browser-glue/` 로 복사
5. 호스트용 `index.html` 생성(browser-glue를 통해 컴포넌트 로드)

> **전제**: TypeScript glue를 컴파일하려면 먼저 `packages/browser-glue/` 에서
> `npm run build` 를 실행하십시오.

## 권장 사항

- 새 프로젝트는 `wit-bindings` 와 WIT 월드에 맞춰 바로 작성하십시오. "먼저 `web` 으로 두고
  나중에 이전"해야 하는 기존 경로는 없습니다.
- 새 브라우저 기능이 필요하면 먼저 WIT 쪽에서 인터페이스를 선언하고 JS 호스트에서
  구현하십시오. 게스트에 DOM 바인딩 라이브러리를 직접 넣지 마십시오.
- 비동기 API의 계약 형태는 `packages/browser-worlds/wit/` 의 현재 선언이 기준입니다.
  이 문서는 개별 함수 시그니처를 다시 적지 않습니다.