# Архитектура Browser Glue

Пакет browser-glue предоставляет TypeScript-реализации WIT-интерфейсов `tairitsu-browser:full`, позволяя компонентам WebAssembly взаимодействовать с браузерными API через Component Model.

## Обзор архитектуры

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Браузер (JS Runtime)                        │
│                                                                     │
│  ┌─────────────────────────────┐     ┌─────────────────────────┐  │
│  │ browser-glue (TS)           │     │ WASM Component          │  │
│  │ - domGlue.ts                │ ←── │ - wit_bindgen bindings  │  │
│  │ - eventsGlue.ts             │     │ - WitPlatform           │  │
│  │ - fetchGlue.ts              │     │                         │  │
│  │ - 28 доменов, 454 интерфейса│     │                         │  │
│  └─────────────────────────────┘     └─────────────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ browser-glue/ (jco import adapters)                         │   │
│  │ - console.js, document.js, element.js, node.js, ...        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Import Map: tairitsu-browser:full/* → ./browser-glue/*            │
│  jco transpile: генерирует обёртку компонента с нужными импортами  │
└─────────────────────────────────────────────────────────────────────┘
```

## Ключевые компоненты

### TypeScript Glue (`src/*.ts`)

Автоматически сгенерированные TypeScript-реализации WIT-интерфейсов:

| Домен | Файл | Интерфейсы | Функции |
|-------|------|------------|---------|
| DOM | `domGlue.ts` | 34 | ~300 |
| HTML | `htmlGlue.ts` | 182 | ~1500 |
| CSS | `cssGlue.ts` | 44 | ~400 |
| Canvas | `canvasGlue.ts` | 20 | ~200 |
| Fetch | `fetchGlue.ts` | 25 | ~150 |
| Events | `eventsGlue.ts` | 15 | ~100 |
| ... | ... | ... | ... |

### Декларации типов (`dist/*.d.ts`)

Файлы деклараций TypeScript для поддержки IDE и проверки типов.

### Обёртки интерфейсов (`dist/browser-glue/*.js`)

Минимальные файлы-адаптеры для импортов jco transpiled:

- `console.js` - Интерфейс логирования
- `document.js` - Создание документа
- `element.js` - Атрибуты элементов
- `node.js` - Операции с DOM-деревом
- `style.js` - CSS-свойства стилей
- `event-target.js` - Слушатели событий
- `non-element-parent-node.js` - getElementById
- `window.js` - Размеры окна

## Интеграция с jco

### Конфигурация Import Map

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

### Процесс транспиляции

1. Сборка WASM-компонента: `cargo build --target wasm32-wasip2 --lib --release`
2. Транспиляция через jco: `jco transpile component.wasm -o output/`
3. jco генерирует обёртку с импортами из `tairitsu-browser:full/*`
4. Import Map резолвит в `./browser-glue/*` адаптеры

## Система дескрипторов

Браузерные объекты представлены как непрозрачные дескрипторы `u64`:

```typescript
// На стороне TypeScript
const element = document.createElement('div');
const handle = registerHandle(element); // Возвращает bigint

// На стороне Rust получает u64
let handle: u64 = bindings::document::create_element("div", None);
```

### Таблица дескрипторов (`handles.ts`)

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

## Процесс сборки

```bash
# Регенерация glue из WIT
python3 scripts/generate_browser_glue.py

# Сборка с декларациями
cd packages/browser-glue && npm run build

# Продакшн-сборка с минификацией
npm run build:production
```

## Структура пакета

```
packages/browser-glue/
├── src/
│   ├── index.ts              # Главный вход
│   ├── handles.ts            # Управление дескрипторами
│   ├── async.ts              # Утилиты async
│   ├── consoleGlue.ts        # Интерфейс консоли
│   ├── styleGlue.ts          # Интерфейс стилей
│   ├── eventTargetGlue.ts    # Цель событий
│   ├── domGlue.ts            # DOM-операции
│   ├── eventsGlue.ts         # Типы событий
│   ├── fetchGlue.ts          # Fetch API
│   ├── canvasGlue.ts         # Canvas 2D
│   └── ... (28 доменов)
├── dist/
│   ├── index.js              # Скомпилированный вход
│   ├── *.d.ts                # Декларации типов
│   └── browser-glue/         # jco адаптеры
└── package.json
```
