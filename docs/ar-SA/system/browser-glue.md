# بنية Browser Glue

توفر حزمة browser-glue تطبيقات TypeScript لواجهات WIT الخاصة بـ `tairitsu-browser:full`، مما يمكّن مكونات WebAssembly من التفاعل مع واجهات برمجة تطبيقات المتصفح من خلال نموذج المكونات.

## نظرة عامة على البنية

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

## المكونات الرئيسية

### TypeScript Glue (`src/*.ts`)

تطبيقات TypeScript مُولّدة تلقائياً لواجهات WIT:

| النطاق | الملف | الواجهات | الدوال |
|--------|-------|----------|--------|
| DOM | `domGlue.ts` | 34 | ~300 |
| HTML | `htmlGlue.ts` | 182 | ~1500 |
| CSS | `cssGlue.ts` | 44 | ~400 |
| Canvas | `canvasGlue.ts` | 20 | ~200 |
| Fetch | `fetchGlue.ts` | 25 | ~150 |
| Events | `eventsGlue.ts` | 15 | ~100 |
| ... | ... | ... | ... |

### ملفات تعريف الأنواع (`dist/*.d.ts`)

ملفات تعريف TypeScript لدعم IDE وفحص الأنواع.

### غلافات الواجهات (`dist/browser-glue/*.js`)

ملفات محولات بسيطة لاستيرادات jco المحوّلة:

- `console.js` - واجهة التسجيل
- `document.js` - إنشاء المستند
- `element.js` - سمات العناصر
- `node.js` - عمليات شجرة DOM
- `style.js` - خصائص نمط CSS
- `event-target.js` - مستمعي الأحداث
- `non-element-parent-node.js` - getElementById
- `window.js` - أبعاد النافذة

## التكامل مع jco

### تكوين خريطة الاستيراد

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

### عملية التحويل (Transpile)

1. بناء مكون WASM: `cargo build --target wasm32-wasip2 --lib --release`
2. التحويل باستخدام jco: `jco transpile component.wasm -o output/`
3. jco يُنشئ غلافاً مع الاستيرادات من `tairitsu-browser:full/*`
4. خريطة الاستيراد تحل إلى محولات `./browser-glue/*`

## نظام المقابض (Handle System)

تُمثَّل كائنات المتصفح كمقابض معتمة `u64`:

```typescript
// جانب TypeScript
const element = document.createElement('div');
const handle = registerHandle(element); // يُرجع bigint

// جانب Rust يستقبل u64
let handle: u64 = bindings::document::create_element("div", None);
```

### جدول المقابض (`handles.ts`)

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

## عملية البناء

```bash
# إعادة توليد الـ glue من WIT
python3 scripts/generate_browser_glue.py

# البناء مع التعريفات
cd packages/browser-glue && npm run build

# بناء الإنتاج مع التصغير
npm run build:production
```

## هيكل الحزمة

```
packages/browser-glue/
├── src/
│   ├── index.ts              # نقطة الدخول الرئيسية
│   ├── handles.ts            # إدارة المقابض
│   ├── async.ts              # أدوات غير متزامنة
│   ├── consoleGlue.ts        # واجهة الطرفية
│   ├── styleGlue.ts          # واجهة النمط
│   ├── eventTargetGlue.ts    # هدف الحدث
│   ├── domGlue.ts            # عمليات DOM
│   ├── eventsGlue.ts         # أنواع الأحداث
│   ├── fetchGlue.ts          # Fetch API
│   ├── canvasGlue.ts         # Canvas 2D
│   └── ... (28 نطاقاً)
├── dist/
│   ├── index.js              # نقطة الدخول المُجمّعة
│   ├── *.d.ts                # تعريفات الأنواع
│   └── browser-glue/         # محولات jco
└── package.json
```
