# Guía de Solución de Problemas

Problemas comunes y soluciones al trabajar con Tairitsu browser-glue y Component Model.

## Errores de Compilación

### Target wasm32-wasip2 no encontrado

**Error:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**Solución:**
```bash
rustup target add wasm32-wasip2
```

### Incompatibilidad de versión wit-bindgen

**Error:**
```
error: failed to select a version for `wit-bindgen`
```

**Solución:**
Asegurar que la versión de `wit-bindgen` coincida en `Cargo.toml`:
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### Errores de compilación TypeScript

**Error:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**Solución:**
Regenerar glue y recompilar:
```bash
cd packages/browser-glue
npm run build
```

## Errores en Tiempo de Ejecución

### Imports de host faltantes

**Error:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**Solución:**
1. Asegurar que el import map esté configurado:
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. Verificar que los archivos browser-glue existan en el directorio de salida.

### Fallo de inicialización del componente

**Error:**
```
Error: Component instantiation failed: undefined import
```

**Solución:**
Verificar que todos los imports WIT requeridos tengan implementaciones correspondientes en browser-glue.

### Errores de transpilación jco

**Error:**
```
Error: Failed to transpile component
```

**Solución:**
1. Asegurar que jco esté instalado:
```bash
npm install -g @bytecodealliance/jco
```

2. Verificar que el componente WASM sea válido:
```bash
wasm-tools print component.wasm
```

## Técnicas de Depuración

### Habilitar logs de depuración

En la consola del navegador:
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### Inspeccionar bindings WIT

Ver bindings generados:
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### DevTools del Navegador

1. Abrir DevTools (F12)
2. Revisar Consola para errores
3. Pestaña Network para cargas de módulos fallidas
4. Pestaña Sources para depuración

### Validación de componentes

```bash
# Validar estructura del componente
wasm-tools validate component.wasm

# Imprimir contenido del componente
wasm-tools print component.wasm
```

## Problemas Comunes

### Handle no encontrado

**Síntoma:** Se retorna `null` de operaciones DOM

**Causa:** El handle fue recolectado por garbage collector o no fue registrado

**Solución:** Asegurar que los elementos permanezcan referenciados en JavaScript

### Evento no disparando

**Síntoma:** Handlers de eventos no llamados

**Causa:** ID de listener no coincide o tipo de evento incorrecto

**Solución:** Verificar que `addEventListener` retorna un ID de listener válido

### Fugas de memoria

**Síntoma:** Uso de memoria creciente en el tiempo

**Causa:** Handles no liberados después de uso

**Solución:** Llamar `dropHandle()` cuando se termine de usar objetos

## Problemas de Rendimiento

### Carga lenta del componente

**Soluciones:**
1. Usar build de release: `cargo build --release`
2. Habilitar LTO en `Cargo.toml`:
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### Alta latencia de eventos

**Soluciones:**
1. Evitar operaciones síncronas en handlers
2. Usar `requestAnimationFrame` para actualizaciones visuales
3. Debounce eventos rápidos

## Obtener Ayuda

1. Revisar issues existentes: https://github.com/anomalyco/opencode/issues
2. Consultar documentación en el directorio `docs/`
3. Examinar código de ejemplo en `examples/website/`
