# Документация Tairitsu

Tairitsu — полнофункциональный фреймворк на базе модели компонентов WASM. Напишите компоненты один раз и запускайте их где угодно — сервер, браузер, edge. Вся коммуникация типизирована через WIT.

## Выберите свой путь

| Я хочу... | Начните здесь |
|:--|:--|
| Попробовать за 5 минут | [Быстрый старт](guides/quick-start.md) |
| Учиться с нуля | [Руководство для начинающих](guides/getting-started.md) |
| Понять архитектуру | [Обзор системы](system/overview.md) |
| Посмотреть все пакеты | [Карта пакетов](components/index.md) |
| Мигрировать с Dioxus | [Руководство по миграции](guides/migration/dioxus-to-tairitsu.md) |
| Решить проблему | [Устранение неполадок](guides/troubleshooting.md) |
| Просмотреть workspace | [Карта workspace](guides/workspace-map.md) |
| Посмотреть термины | [Глоссарий](guides/glossary.md) |

## Структура документации

```mermaid
graph TD
    ROOT["docs/"] --> GUIDES["guides/ — Учебники, руководства, миграция"]
    ROOT --> SYSTEM["system/ — Архитектура в деталях"]
    ROOT --> COMPONENTS["components/ — Справочник Crate"]
    GUIDES --> GS["getting-started.md"]
    GUIDES --> QS["quick-start.md"]
    GUIDES --> WM["workspace-map.md"]
    GUIDES --> BTR["build-test-release.md"]
    GUIDES --> MIG["migration/"]
    GUIDES --> TS["troubleshooting.md"]
    GUIDES --> GL["glossary.md"]
    SYSTEM --> OV["overview.md"]
    SYSTEM --> RT["runtime.md"]
    SYSTEM --> VD["vdom.md"]
    SYSTEM --> WP["wit-pipeline.md"]
    SYSTEM --> WB["web-backends.md"]
    SYSTEM --> BG["browser-glue.md"]
    SYSTEM --> VER["versioning.md"]
    COMPONENTS --> CI["index.md"]
    COMPONENTS --> PKG["packages.md"]
```

## Другие языки

- [English](../en/index.md)
- [简体中文](../zhs/index.md)
- [繁體中文](../zht/index.md)
- [日本語](../ja/index.md)
- [한국어](../ko/index.md)
- [Español](../es/index.md)
- [Français](../fr/index.md)
- [العربية](../ar/index.md)
