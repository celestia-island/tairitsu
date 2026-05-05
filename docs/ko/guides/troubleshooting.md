# 문제 해결 가이드

Tairitsu browser-glue 및 컴포넌트 모델 작업 시 발생하는 일반적인 문제와 해결 방법.

## 빌드 오류

### wasm32-wasip2 타겟을 찾을 수 없음

**오류:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**해결 방법:**
```bash
rustup target add wasm32-wasip2
```

### wit-bindgen 버전 불일치

**오류:**
```
error: failed to select a version for `wit-bindgen`
```

**해결 방법:**
`Cargo.toml`에서 `wit-bindgen` 버전이 일치하는지 확인:
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### TypeScript 컴파일 오류

**오류:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**해결 방법:**
글루 재생성 및 재빌드:
```bash
cd packages/browser-glue
npm run build
```

## 런타임 오류

### 호스트 imports 누락

**오류:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**해결 방법:**
1. Import map이 설정되어 있는지 확인:
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. 출력 디렉토리에 browser-glue 파일이 있는지 확인.

### 컴포넌트 초기화 실패

**오류:**
```
Error: Component instantiation failed: undefined import
```

**해결 방법:**
모든 필수 WIT imports에 해당하는 구현이 browser-glue에 있는지 확인.

### jco 변환 오류

**오류:**
```
Error: Failed to transpile component
```

**해결 방법:**
1. jco가 설치되어 있는지 확인:
```bash
npm install -g @bytecodealliance/jco
```

2. WASM 컴포넌트가 유효한지 확인:
```bash
wasm-tools print component.wasm
```

## 디버그 기법

### 디버그 로그 활성화

브라우저 콘솔에서:
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### WIT 바인딩 검사

생성된 바인딩 보기:
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### 브라우저 개발자 도구

1. 개발자 도구 열기 (F12)
2. 콘솔에서 오류 확인
3. 네트워크 탭에서 실패한 모듈 로드 확인
4. 소스 탭에서 디버깅

### 컴포넌트 검증

```bash
# 컴포넌트 구조 검증
wasm-tools validate component.wasm

# 컴포넌트 내용 출력
wasm-tools print component.wasm
```

## 일반적인 문제

### 핸들을 찾을 수 없음

**증상:** DOM 작업에서 `null` 반환

**원인:** 핸들이 가비지 컬렉션되었거나 등록되지 않음

**해결 방법:** JavaScript에서 요소가 참조된 상태로 유지되는지 확인

### 이벤트가 발생하지 않음

**증상:** 이벤트 핸들러가 호출되지 않음

**원인:** 리스너 ID 불일치 또는 이벤트 타입이 잘못됨

**해결 방법:** `addEventListener`가 유효한 리스너 ID를 반환하는지 확인

### 메모리 누수

**증상:** 시간이 지남에 따라 메모리 사용량 증가

**원인:** 사용 후 핸들이 해제되지 않음

**해결 방법:** 객체 사용이 끝나면 `dropHandle()` 호출

## 성능 문제

### 느린 컴포넌트 로드

**해결 방법:**
1. 릴리스 빌드 사용: `cargo build --release`
2. `Cargo.toml`에서 LTO 활성화:
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### 높은 이벤트 지연 시간

**해결 방법:**
1. 핸들러에서 동기 작업 피하기
2. 시각적 업데이트에 `requestAnimationFrame` 사용
3. 빈번한 이벤트 디바운스

## 도움 받기

1. 기존 이슈 확인: https://github.com/anomalyco/opencode/issues
2. `docs/` 디렉토리의 문서 검토
3. `examples/website/`의 예제 코드 검사
