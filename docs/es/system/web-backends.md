# Backend web: un solo camino — contrato WIT + implementación del host en JS

`packages/web` tiene exactamente **un** camino de backend web. El «doble backend» que describían
los documentos anteriores — `web` (`wasm-bindgen` / `web-sys`) junto a `wit-bindings` — ya no
existe: el lado `web` no tenía consumidores, y tanto sus dependencias como sus ramas `cfg` están
retiradas.

## Qué es este camino

| Elemento | Valor |
| --- | --- |
| Objetivo de compilación | `wasm32-wasip2` (WASI 0.2 / 0.3, Component Model) |
| Contrato | Mundos WIT `tairitsu-browser:*` (`packages/browser-worlds/wit/`) |
| Bindings del lado guest | `wit-bindgen` (feature `wit-bindings`) |
| Implementación del lado navegador | Host JS (`packages/browser-glue/`) |
| Capacidades de navegador del guest | Host de Component Model, vía imports inyectados por el host |

El punto de entrada es `WitPlatform` (`packages/web/src/wit_platform.rs`, feature `wit-bindings`).
`BrowserPlatform` (`packages/web/src/browser.rs`) es una implementación de relleno — callbacks
inmediatos, rects por defecto — y no es un backend web-sys.

## Por qué solo queda este camino

1. **El Component Model cambia cómo se entregan las «capacidades».** `wasm32-unknown-unknown` +
   `web-sys` compila las capacidades del navegador directamente en la sección de imports del guest,
   de modo que no se puede declarar *quién* es el host ni *qué* capacidades se concedieron. Un
   contrato WIT expone la superficie de capacidades como interfaces de imports revisables y
   versionables, y el host las autoriza instancia por instancia.
2. **Transpilación en tiempo de compilación, no autoalojamiento en ejecución.** jco transpila un
   component a core wasm más un runtime JS de ABI canónica durante la compilación; los navegadores
   **no** tienen `WebAssembly.Component` nativo. Por tanto «contrato en WIT, implementación en JS»
   es la única forma viable hoy — no una concesión.
3. **Las responsabilidades del host se vuelven explícitas.** La instanciación, la autorización de
   capacidades, la planificación asíncrona y el ciclo de vida (incluida la terminación forzosa)
   pertenecen al host de tairitsu; el guest solo describe lo que necesita.

## Niveles de mundos

Los mundos WIT se dividen en paquetes llamados `tairitsu-browser:{domain}` bajo
`packages/browser-worlds/wit/` (`browser-full.wit` más los paquetes por dominio generados en
`generated/`). Ese directorio es la referencia para la lista de paquetes.

## Compilación y empaquetado

La CLI `tairitsu build` de `packages/packager` admite **solo el objetivo `component`**
(`wasm32-wasip2` + ruta browser-glue). Cualquier otro objetivo falla con
`Unknown build target … Only 'component' is supported.`

Pasos de compilación de un component:

1. Verificar que el toolchain `wasm32-wasip2` está instalado
2. `cargo build --target wasm32-wasip2 --lib`
3. Copiar el component `.wasm` a `output_dir`
4. Copiar el runtime de browser-glue a `output_dir/browser-glue/`
5. Generar el `index.html` del host (carga el component mediante browser-glue)

> **Requisito previo**: ejecutar primero `npm run build` en `packages/browser-glue/` para compilar
> el código TypeScript del glue.

## Recomendaciones

- Escribe los proyectos nuevos directamente contra `wit-bindings` y los mundos WIT; no existe una
  ruta heredada que obligue a «seguir en `web` primero y migrar después».
- Cuando necesites una nueva capacidad del navegador, declara primero la interfaz en el lado WIT y
  luego impleméntala en el host JS — no metas una biblioteca de bindings del DOM en el guest.
- La forma de los contratos de API asíncrona sigue las declaraciones actuales en
  `packages/browser-worlds/wit/`; esta página no repite firmas individuales.