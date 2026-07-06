# Tairitsu 문서 허브 (한국어)

> WASM Component Model 기반의 풀스택 프레임워크

## 시작하기

| 문서 | 설명 |
|:--|:--|
| [시작하기 튜토리얼](getting-started.md) | 제로부터 풀스택 앱 구축. `tairitsu new`, 첫 컴포넌트, 서버 + 클라이언트 실행, 배포까지. |
| [빠른 시작](quick-start.md) | 5분 설치 및 확인. |
| [워크스페이스 맵](workspace-map.md) | 모노레포 구조 둘러보기. |
| [빌드, 테스트, 릴리스](build-test-release.md) | `just` 레시피를 사용한 개발 워크플로우. |

## 마이그레이션

| 문서 | 설명 |
|:--|:--|
| [web-sys에서 WIT 바인딩으로](migration.md) | `wasm-bindgen`/`web-sys`에서 Component Model WIT 바인딩으로 전환. |

## 참조

| 문서 | 설명 |
|:--|:--|
| [용어집](glossary.md) | 핵심 용어: WIT, Component Model, VNode, Signal, Platform, Container 등 |
| [문제 해결](troubleshooting.md) | 일반적인 문제와 해결 방법. |

## 아키텍처

| 문서 | 설명 |
|:--|:--|
| [시스템 개요](../system/overview.md) | 4계층 아키텍처: Interface → Runtime → Platform → Tooling |
| [런타임 & 컨테이너 모델](../system/runtime.md) | Image/Container/Registry 라이프사이클, WIT 바인딩, 동적 호출 |
| [VDOM & 렌더링](../system/vdom.md) | 가상 DOM diffing, patching, 이벤트 시스템, 반응형 스케줄러 |
| [W3C WebIDL → WIT 파이프라인](../system/wit-pipeline.md) | 50+ WebIDL 스펙이 WIT 인터페이스가 되는 과정 |
| [듀얼 웹 백엔드](../system/web-backends.md) | WitPlatform vs WebPlatform 전략 |
| [Browser Glue 아키텍처](../system/browser-glue.md) | WIT ABI와 DOM을 연결하는 TypeScript 계층 |
| [버저닝 전략](../system/versioning.md) | 멀티 Crate 워크스페이스의 시맨틱 버저닝 |

## 패키지 참조

| 문서 | 설명 |
|:--|:--|
| [계층별 패키지 개요](../components/index.md) | 4계층 Crate 계층 구조와 의존성 그래프 |
| [워크스페이스 패키지 목록](../components/packages.md) | 각 Crate에 대한 상세 설명 |
