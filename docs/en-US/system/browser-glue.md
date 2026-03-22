# Browser Glue Architecture

The browser-glue package provides TypeScript implementations of the `tairitsu-browser:full` WIT interfaces, enabling WebAssembly components to interact with browser APIs through the Component Model.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Browser (JS Runtime)                        │
│                                                                     │
│  ┌─────────────────────────────┐     ┌─────────────────────────┐  │
│  │ browser-glue (TS)           │     │ WASM Component          │  │
│  │ - domGlue.ts                │ ←── │ - wit_bindgen bindings  │  │
│  │ - eventsGlue.ts             │     │ - WitPlatform           │  │
│  │ - fetchGlue.ts              │     │                         │  │
│  │ - 28 domains, 454 interfaces│     │                         │  │
│  └─────────────────────────────┘     └─────────────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ browser-glue/ (jco import adapters)                         │   │
│  │ - console.js, document.js, element.js, node.js, ...        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Import Map: tairitsu-browser:full/* → ./browser-glue/*            │
│  jco transpile: generates component wrapper with proper imports    │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Components

### TypeScript Glue (`src/*.ts`)

Auto-generated TypeScript implementations of WIT interfaces:

| Domain | File | Interfaces | Functions |
|--------|------|------------|-----------|
| DOM | `domGlue.ts` | 34 | ~300 |
| HTML | `htmlGlue.ts` | 182 | ~1500 |
| CSS | `cssGlue.ts` | 44 | ~400 |
| Canvas | `canvasGlue.ts` | 20 | ~200 |
| Fetch | `fetchGlue.ts` | 25 | ~150 |
| Events | `eventsGlue.ts` | 15 | ~100 |
| ... | ... | ... | ... |

### Type Declarations (`dist/*.d.ts`)

TypeScript declaration files for IDE support and type checking.

### Interface Wrappers (`dist/browser-glue/*.js`)

Minimal adapter files for jco transpiled imports:

- `console.js` - Logging interface
- `document.js` - Document creation
- `element.js` - Element attributes
- `node.js` - DOM tree operations
- `style.js` - CSS style properties
- `event-target.js` - Event listeners
- `non-element-parent-node.js` - getElementById
- `window.js` - Window dimensions

## jco Integration

### Import Map Configuration

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

### Transpile Process

1. Build WASM component: `cargo build --target wasm32-wasip2 --lib --release`
2. Transpile with jco: `jco transpile component.wasm -o output/`
3. jco generates wrapper with imports from `tairitsu-browser:full/*`
4. Import map resolves to `./browser-glue/*` adapters

## Handle System

Browser objects are represented as opaque `u64` handles:

```typescript
// TypeScript side
const element = document.createElement('div');
const handle = registerHandle(element); // Returns bigint

// Rust side receives u64
let handle: u64 = bindings::document::create_element("div", None);
```

### Handle Table (`handles.ts`)

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

## Build Process

```bash
# Regenerate glue from WIT
python3 scripts/generate_browser_glue.py

# Build with declarations
cd packages/browser-glue && npm run build

# Production build with minification
npm run build:production
```

## Package Layout

```
packages/browser-glue/
├── src/
│   ├── index.ts              # Main entry
│   ├── handles.ts            # Handle management
│   ├── async.ts              # Async utilities
│   ├── consoleGlue.ts        # Console interface
│   ├── styleGlue.ts          # Style interface
│   ├── eventTargetGlue.ts    # Event target
│   ├── domGlue.ts            # DOM operations
│   ├── eventsGlue.ts         # Event types
│   ├── fetchGlue.ts          # Fetch API
│   ├── canvasGlue.ts         # Canvas 2D
│   └── ... (28 domains)
├── dist/
│   ├── index.js              # Compiled entry
│   ├── *.d.ts                # Type declarations
│   └── browser-glue/         # jco adapters
└── package.json
```
