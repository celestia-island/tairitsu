# 브라우저 글루 아키텍처

browser-glue 패키지는 `tairitsu-browser:full` WIT 인터페이스의 TypeScript 구현을 제공하여, WebAssembly 컴포넌트가 컴포넌트 모델을 통해 브라우저 API와 상호작용할 수 있도록 합니다.

## 아키텍처 개요

```
┌─────────────────────────────────────────────────────────────────────┐
│                         브라우저 (JS 런타임)                         │
│                                                                     │
│  ┌─────────────────────────────┐     ┌─────────────────────────┐  │
│  │ browser-glue (TS)           │     │ WASM 컴포넌트           │  │
│  │ - domGlue.ts                │ ←── │ - wit_bindgen 바인딩    │  │
│  │ - eventsGlue.ts             │     │ - WitPlatform           │  │
│  │ - fetchGlue.ts              │     │                         │  │
│  │ - 28개 도메인, 454개 인터페이스│     │                         │  │
│  └─────────────────────────────┘     └─────────────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ browser-glue/ (jco import 어댑터)                           │   │
│  │ - console.js, document.js, element.js, node.js, ...        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Import Map: tairitsu-browser:full/* → ./browser-glue/*            │
│  jco transpile: 적절한 imports로 컴포넌트 래퍼 생성                 │
└─────────────────────────────────────────────────────────────────────┘
```

## 주요 컴포넌트

### TypeScript 글루 (`src/*.ts`)

WIT 인터페이스의 자동 생성된 TypeScript 구현:

| 도메인 | 파일 | 인터페이스 | 함수 |
|--------|------|------------|-----------|
| DOM | `domGlue.ts` | 34 | ~300 |
| HTML | `htmlGlue.ts` | 182 | ~1500 |
| CSS | `cssGlue.ts` | 44 | ~400 |
| Canvas | `canvasGlue.ts` | 20 | ~200 |
| Fetch | `fetchGlue.ts` | 25 | ~150 |
| Events | `eventsGlue.ts` | 15 | ~100 |
| ... | ... | ... | ... |

### 타입 선언 (`dist/*.d.ts`)

IDE 지원 및 타입 검사를 위한 TypeScript 선언 파일.

### 인터페이스 래퍼 (`dist/browser-glue/*.js`)

jco 변환된 imports를 위한 최소한의 어댑터 파일:

- `console.js` - 로깅 인터페이스
- `document.js` - 문서 생성
- `element.js` - 요소 속성
- `node.js` - DOM 트리 작업
- `style.js` - CSS 스타일 속성
- `event-target.js` - 이벤트 리스너
- `non-element-parent-node.js` - getElementById
- `window.js` - 창 크기

## jco 통합

### Import Map 설정

```html
<script type="importmap">
{
  "imports": {
    "@bytecodealliance/preview2-shim/": "https://esm.sh/@bytecodealliance/preview2-shim/",
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

### 변환 프로세스

1. WASM 컴포넌트 빌드: `cargo build --target wasm32-wasip2 --lib --release`
2. jco로 변환: `jco transpile component.wasm -o output/`
3. jco가 `tairitsu-browser:full/*`에서 imports를 가져오는 래퍼 생성
4. Import map이 `./browser-glue/*` 어댑터로 해석

## 핸들 시스템

브라우저 객체는 불투명한 `u64` 핸들로 표현됩니다:

```typescript
// TypeScript 측
const element = document.createElement('div');
const handle = registerHandle(element); // bigint 반환

// Rust 측에서 u64 수신
let handle: u64 = bindings::document::create_element("div", None);
```

### 핸들 테이블 (`handles.ts`)

```typescript
const _handles = new Map<bigint, object>();
let _nextHandle = 1n;

export function registerHandle(obj: object): bigint {
  const handle = BigInt(_nextHandle++);
  _handles.set(handle, obj);
  return handle;
}

export function lookupHandle<T>(handle: bigint): T | null {
  return _handles.get(handle) as T ?? null;
}
```

## 빌드 프로세스

```bash
# WIT에서 글루 재생성
python3 scripts/generate_browser_glue.py

# 선언과 함께 빌드
cd packages/browser-glue && npm run build

# 축소와 함께 프로덕션 빌드
npm run build:production
```

## 패키지 구조

```
packages/browser-glue/
├── src/
│   ├── index.ts              # 메인 진입점
│   ├── handles.ts            # 핸들 관리
│   ├── async.ts              # 비동기 유틸리티
│   ├── consoleGlue.ts        # 콘솔 인터페이스
│   ├── styleGlue.ts          # 스타일 인터페이스
│   ├── eventTargetGlue.ts    # 이벤트 타겟
│   ├── domGlue.ts            # DOM 작업
│   ├── eventsGlue.ts         # 이벤트 타입
│   ├── fetchGlue.ts          # Fetch API
│   ├── canvasGlue.ts         # Canvas 2D
│   └── ... (28개 도메인)
├── dist/
│   ├── index.js              # 컴파일된 진입점
│   ├── *.d.ts                # 타입 선언
│   └── browser-glue/         # jco 어댑터
└── package.json
```
