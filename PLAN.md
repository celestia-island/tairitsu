# Browser Glue Connection Layer Implementation Plan

## Current State

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Browser (JS Runtime)                        │
│                                                                     │
│  ┌─────────────────────────┐     ┌─────────────────────────────┐  │
│  │ browser-glue (TS)       │     │ WASM Component              │  │
│  │ - domGlue.ts            │  ?  │ - WitPlatform               │  │
│  │ - eventsGlue.ts         │ ←── │ - wit_bindgen bindings      │  │
│  │ - fetchGlue.ts          │     │                             │  │
│  │ ...                     │     │                             │  │
│  └─────────────────────────┘     └─────────────────────────────┘  │
│                                                                     │
│                    MISSING: Connection Layer                        │
└─────────────────────────────────────────────────────────────────────┘
```

**What works:**
- WIT definitions: `packages/browser-worlds/wit/browser-full.wit`
- TypeScript glue: `packages/browser-glue/src/*.ts` (454 interfaces, 3974 functions)
- Rust bindings: `wit_bindgen::generate!()` in `wit_platform.rs`

**What's missing:**
- A runtime that connects TS glue exports to WASM Component imports
- Browser-compatible Component Model instantiation
- Event callback dispatch from TS to Rust

## Architecture Options

### Option A: jco (Bytecode Alliance JS Component Tools)

```
┌──────────────────────────────────────────────────────────────────┐
│ Browser                                                          │
│  ┌────────────┐    ┌─────────────┐    ┌──────────────────────┐  │
│  │ .wasm      │───→│ jco adapter │───→│ browser-glue exports │  │
│  │ component  │    │ (generated) │    │ (TS functions)       │  │
│  └────────────┘    └─────────────┘    └──────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**Pros:**
- Standard tooling from Bytecode Alliance
- Handles canonical ABI lift/lower automatically
- Supports async operations

**Cons:**
- Large bundle size (~2MB adapter code)
- Build complexity (requires jco CLI)
- May not support all WIT types we need

### Option B: Custom Lightweight Adapter

```
┌──────────────────────────────────────────────────────────────────┐
│ Browser                                                          │
│  ┌────────────┐    ┌───────────────┐    ┌────────────────────┐  │
│  │ .wasm      │───→│ custom-       │───→│ browser-glue       │  │
│  │ component  │    │ adapter.ts    │    │ exports            │  │
│  └────────────┘    └───────────────┘    └────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**Pros:**
- Minimal bundle size
- Full control over implementation
- Can optimize for our specific use case

**Cons:**
- Must implement canonical ABI manually
- More development effort
- Must handle all edge cases

### Option C: Hybrid - WASI Preview 2 + Custom Host

```
┌──────────────────────────────────────────────────────────────────┐
│ Browser                                                          │
│  ┌────────────┐    ┌─────────────┐    ┌──────────────────────┐  │
│  │ .wasm      │───→│ wasi-http   │───→│ fetch-style bridge   │  │
│  │ component  │    │ polyfill    │    │ to browser-glue      │  │
│  └────────────┘    └─────────────┘    └──────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**Pros:**
- Uses standard WASI interfaces where possible
- Easier browser integration

**Cons:**
- Limited to WASI-defined interfaces
- Doesn't support custom DOM interfaces

## Recommended Approach: Option B (Custom Adapter)

Given our specific requirements (DOM manipulation, event handling, minimal bundle size), we'll implement a custom adapter.

## Implementation Tasks

### Phase 1: Core Adapter Infrastructure

#### 1.1 Create adapter package structure
- [ ] Create `packages/browser-adapter/` directory
- [ ] Set up TypeScript project with proper configuration
- [ ] Add build scripts for both development and production

#### 1.2 Implement Component Model instantiation
- [ ] Create `component-loader.ts` - load and instantiate WASM component
- [ ] Implement canonical ABI value marshaling:
  - [ ] `lift_string.ts` - string → canonical ABI
  - [ ] `lower_string.ts` - canonical ABI → string
  - [ ] `lift_u64.ts` - number → u64 handle
  - [ ] `lower_u64.ts` - u64 handle → number
  - [ ] `lift_list.ts` - array → canonical ABI list
  - [ ] `lower_list.ts` - canonical ABI list → array
  - [ ] `lift_option.ts` - nullable → option<T>
  - [ ] `lower_option.ts` - option<T> → nullable
  - [ ] `lift_result.ts` - Result<T, E> handling
  - [ ] `lower_result.ts` - error handling

#### 1.3 Implement import satisfier
- [ ] Create `import-satisfier.ts` - maps WIT imports to browser-glue exports
- [ ] Generate import mapping from WIT definitions
- [ ] Handle interface versioning

### Phase 2: DOM Interface Binding

#### 2.1 Core DOM operations
- [ ] Bind `node` interface imports to `domGlue.ts` exports
- [ ] Bind `document` interface imports
- [ ] Bind `element` interface imports
- [ ] Bind `style` interface imports

#### 2.2 Handle management
- [ ] Implement handle table in adapter (maps u64 ↔ JS objects)
- [ ] Ensure handles are properly cleaned up on component drop
- [ ] Handle edge cases (detached nodes, moved nodes)

#### 2.3 Event handling
- [ ] Bind `event-target` interface
- [ ] Implement `event-callbacks` export interface:
  - [ ] `on_mouse_event(listener_id, event_handle, data)`
  - [ ] `on_keyboard_event(listener_id, event_handle, data)`
  - [ ] `on_focus_event(listener_id, event_handle, data)`
  - [ ] `on_input_event(listener_id, event_handle, data)`
  - [ ] `on_generic_event(listener_id, event_handle, event_type)`
- [ ] Implement `lifecycle` export interface:
  - [ ] `start()` → call component bootstrap

### Phase 3: Extended Browser APIs

#### 3.1 Fetch API
- [ ] Bind `fetch` interface
- [ ] Implement poll-handle pattern for async fetch
- [ ] Handle streaming responses

#### 3.2 Canvas API
- [ ] Bind `canvas2d` interface
- [ ] Map CanvasRenderingContext2D methods

#### 3.3 Other Phase A interfaces (22 domains)
- [ ] Generate adapter bindings for all generated WIT interfaces
- [ ] Create adapter code generator script

### Phase 4: Build Integration

#### 4.1 Packager integration
- [ ] Update `packages/packager` to support `--target component`
- [ ] Integrate browser-adapter into build output
- [ ] Generate HTML template that loads adapter

#### 4.2 Development server
- [ ] Add hot-reload support for component development
- [ ] Integrate with existing `tairitsu dev` command

#### 4.3 Production optimization
- [ ] Tree-shake unused glue code
- [ ] Minify adapter bundle
- [ ] Generate source maps

### Phase 5: Testing & Documentation

#### 5.1 E2E testing
- [ ] Add browser-adapter tests to `packages/e2e`
- [ ] Test all WIT import/export roundtrips
- [ ] Test event callback dispatch
- [ ] Test async operations (fetch, etc.)

#### 5.2 Documentation
- [ ] Document adapter architecture in `docs/`
- [ ] Add migration guide from wasm-bindgen
- [ ] Create troubleshooting guide

## File Structure

```
packages/
├── browser-adapter/
│   ├── src/
│   │   ├── index.ts              # Main entry point
│   │   ├── component-loader.ts   # WASM component instantiation
│   │   ├── canonical-abi/
│   │   │   ├── lift.ts           # JS → Canonical ABI
│   │   │   ├── lower.ts          # Canonical ABI → JS
│   │   │   ├── types.ts          # Type definitions
│   │   │   └── memory.ts         # Linear memory management
│   │   ├── import-satisfier.ts   # Maps imports to browser-glue
│   │   ├── export-handler.ts     # Handles component exports
│   │   ├── handle-table.ts       # u64 ↔ JS object mapping
│   │   └── generated/
│   │       └── bindings.ts       # Auto-generated from WIT
│   ├── package.json
│   ├── tsconfig.json
│   └── README.md
├── browser-glue/                 # (existing)
├── browser-worlds/               # (existing)
└── packager/                     # (existing, needs update)
```

## Key Technical Decisions

### 1. Handle Representation
```typescript
// Option 1: BigInt (current browser-glue approach)
type Handle = bigint;

// Option 2: Number with overflow handling
type Handle = number;

// Decision: Use BigInt for correctness, add fast-path for small handles
```

### 2. Memory Management
```typescript
// Component shares memory with adapter
// Must track allocation/deallocation carefully

interface ComponentMemory {
  buffer: ArrayBuffer;
  allocate(size: number): number;
  deallocate(ptr: number, size: number): void;
}
```

### 3. Event Dispatch Pattern
```typescript
// Component exports event-callbacks interface
// Adapter calls these when browser events fire

class EventDispatcher {
  private callbacks: Map<u64, EventCallback>;
  
  dispatch(listenerId: u64, event: Event): void {
    const callback = this.callbacks.get(listenerId);
    if (callback) {
      callback(event);
    }
  }
}
```

### 4. Async Operation Pattern
```typescript
// Poll-handle pattern for async operations
interface PollHandle<T> {
  requestId: u64;
  poll(): Option<Result<T, string>>;
}
```

## Dependencies

### Required
- `@bytecodealliance/jco` (for component transpilation, optional)
- TypeScript 5.x
- esbuild or rollup (for bundling)

### Optional
- `wasm-tools` (for component optimization)
- Source map support

## Success Criteria

1. ✅ `examples/website` builds and runs with `--target component`
2. ✅ All E2E tests pass in browser environment
3. ✅ Bundle size < 500KB (adapter + glue + component)
4. ✅ First paint time < 100ms
5. ✅ Event dispatch latency < 1ms

## Timeline Estimate

- Phase 1 (Core Infrastructure): 2-3 days
- Phase 2 (DOM Binding): 2-3 days
- Phase 3 (Extended APIs): 3-4 days
- Phase 4 (Build Integration): 1-2 days
- Phase 5 (Testing & Docs): 2-3 days

**Total: 10-15 days**

## References

- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [Canonical ABI Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)
- [jco - JS Component Tools](https://github.com/bytecodealliance/jco)
- [wit-bindgen](https://github.com/bytecodealliance/wit-bindgen)
- [WASI Preview 2](https://github.com/WebAssembly/wasi/tree/main/preview2)

## Notes

- The adapter must handle all canonical ABI type conversions
- Event callbacks require careful memory management (strings must be copied)
- Consider using SharedArrayBuffer for better async performance (requires COOP/COEP headers)
- May need to support both sync and async variants of some operations
