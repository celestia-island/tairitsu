# Tairitsu — Centro de documentación (Español)

> Framework full-stack basado en el modelo de componentes WASM

## Primeros pasos

| Documento | Descripción |
|:--|:--|
| [Tutorial de inicio](getting-started.md) | Desde cero hasta una aplicación full-stack funcional. Cubre `tairitsu new`, tu primer componente, ejecución servidor + cliente y despliegue. |
| [Inicio rápido](quick-start.md) | Configuración y verificación en 5 minutos. |
| [Mapa del workspace](workspace-map.md) | Tour de la estructura del monorepo. |
| [Construir, probar y publicar](build-test-release.md) | Cómo usar las recetas de `just` para el flujo de desarrollo. |

## Migración

| Documento | Descripción |
|:--|:--|
| [De web-sys a enlaces WIT](migration.md) | Transición de `wasm-bindgen`/`web-sys` a enlaces WIT del Component Model. |

## Referencia

| Documento | Descripción |
|:--|:--|
| [Glosario](glossary.md) | Términos clave: WIT, Component Model, VNode, Signal, Platform, Container, etc. |
| [Solución de problemas](troubleshooting.md) | Problemas comunes y soluciones. |

## Arquitectura

| Documento | Descripción |
|:--|:--|
| [Resumen del sistema](../system/overview.md) | Arquitectura de cuatro capas: Interface → Runtime → Platform → Tooling |
| [Runtime y modelo de contenedor](../system/runtime.md) | Ciclo de vida Image/Container/Registry, enlace WIT, invocación dinámica |
| [VDOM y renderizado](../system/vdom.md) | Diferenciación del DOM virtual, parcheo, sistema de eventos, planificador reactivo |
| [Pipeline W3C WebIDL → WIT](../system/wit-pipeline.md) | Cómo 50+ especificaciones WebIDL se convierten en interfaces WIT |
| [Backends web duales](../system/web-backends.md) | Estrategia WitPlatform vs WebPlatform |
| [Arquitectura Browser Glue](../system/browser-glue.md) | Capa TypeScript que conecta WIT ABI con el DOM |
| [Estrategia de versionado](../system/versioning.md) | Versionado semántico en el workspace multi-crate |

## Referencia de paquetes

| Documento | Descripción |
|:--|:--|
| [Resumen de paquetes por capas](../components/index.md) | Jerarquía de Crates en cuatro capas con grafo de dependencias |
| [Lista de paquetes del workspace](../components/packages.md) | Descripción detallada de cada Crate |
