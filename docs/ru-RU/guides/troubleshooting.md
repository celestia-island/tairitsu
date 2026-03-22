# Руководство по устранению неполадок

Типичные проблемы и их решения при работе с Tairitsu browser-glue и Component Model.

## Ошибки сборки

### Целевая платформа wasm32-wasip2 не найдена

**Ошибка:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**Решение:**
```bash
rustup target add wasm32-wasip2
```

### Несовпадение версии wit-bindgen

**Ошибка:**
```
error: failed to select a version for `wit-bindgen`
```

**Решение:**
Убедитесь, что версия `wit-bindgen` совпадает в `Cargo.toml`:
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### Ошибки компиляции TypeScript

**Ошибка:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**Решение:**
Регенерируйте glue и пересоберите:
```bash
cd packages/browser-glue
npm run build
```

## Ошибки времени выполнения

### Отсутствуют хост-импорты

**Ошибка:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**Решение:**
1. Убедитесь, что import map настроен:
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. Проверьте наличие файлов browser-glue в выходной директории.

### Ошибка инициализации компонента

**Ошибка:**
```
Error: Component instantiation failed: undefined import
```

**Решение:**
Проверьте, что для всех необходимых WIT-импортов есть соответствующие реализации в browser-glue.

### Ошибки jco transpile

**Ошибка:**
```
Error: Failed to transpile component
```

**Решение:**
1. Убедитесь, что jco установлен:
```bash
npm install -g @bytecodealliance/jco
```

2. Проверьте валидность WASM-компонента:
```bash
wasm-tools print component.wasm
```

## Методы отладки

### Включить отладочные логи

В консоли браузера:
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### Просмотр WIT-биндингов

Просмотр сгенерированных биндингов:
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### Инструменты разработчика браузера

1. Откройте DevTools (F12)
2. Проверьте Console на наличие ошибок
3. Вкладка Network для неудачных загрузок модулей
4. Вкладка Sources для отладки

### Валидация компонента

```bash
# Проверка структуры компонента
wasm-tools validate component.wasm

# Вывод содержимого компонента
wasm-tools print component.wasm
```

## Типичные проблемы

### Дескриптор не найден

**Симптом:** `null` возвращается из DOM-операций

**Причина:** Дескриптор был собран сборщиком мусора или не зарегистрирован

**Решение:** Убедитесь, что элементы остаются доступными в JavaScript

### События не срабатывают

**Симптом:** Обработчики событий не вызываются

**Причина:** Несовпадение ID слушателя или неправильный тип события

**Решение:** Проверьте, что `addEventListener` возвращает корректный ID слушателя

### Утечки памяти

**Симптом:** Постоянный рост использования памяти

**Причина:** Дескрипторы не освобождаются после использования

**Решение:** Вызывайте `dropHandle()` после завершения работы с объектами

## Проблемы производительности

### Медленная загрузка компонента

**Решения:**
1. Используйте release-сборку: `cargo build --release`
2. Включите LTO в `Cargo.toml`:
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### Высокая задержка событий

**Решения:**
1. Избегайте синхронных операций в обработчиках
2. Используйте `requestAnimationFrame` для визуальных обновлений
3. Делайте debounce частых событий

## Получение помощи

1. Проверьте существующие issues: https://github.com/anomalyco/opencode/issues
2. Изучите документацию в директории `docs/`
3. Изучите примеры кода в `examples/website/`
