# Tairitsu Documentation Hub

> Full-stack framework powered by the WASM Component Model

## Getting Started

| Document | Description |
|:--|:--|
| [Getting Started Tutorial](getting-started.md) | From zero to a working full-stack app. Covers `tairitsu new`, your first component, server + client execution, and deployment. |
| [Quick Start](quick-start.md) | 5-minute setup and verification. |
| [Workspace Map](workspace-map.md) | Tour of the monorepo structure. |
| [Build, Test, and Release](build-test-release.md) | How to use `just` recipes for development workflow. |

## Migration

| Document | Description |
|:--|:--|
| [From Dioxus to Tairitsu](migration/dioxus-to-tairitsu.md) | API comparison: components, hooks, events, routing, state management, conditional rendering. |
| [From web-sys to WIT bindings](migration.md) | Transitioning from `wasm-bindgen`/`web-sys` to Component Model WIT bindings. |

## Reference

| Document | Description |
|:--|:--|
| [Glossary](glossary.md) | Key terms: WIT, Component Model, VNode, Signal, Platform, Container, etc. |
| [Troubleshooting](troubleshooting.md) | Common problems and solutions. |

## Architecture

| Document | Description |
|:--|:--|
| [System Overview](../system/overview.md) | Four-layer architecture: Interface → Runtime → Platform → Tooling |
| [Runtime & Container Model](../system/runtime.md) | Image/Container/Registry lifecycle, WIT binding, dynamic invocation |
| [VDOM & Rendering](../system/vdom.md) | Virtual DOM diffing, patching, event system, reactive scheduler |
| [W3C WebIDL → WIT Pipeline](../system/wit-pipeline.md) | How 50+ WebIDL specs become WIT interfaces |
| [Dual Web Backends](../system/web-backends.md) | WitPlatform vs WebPlatform strategy |
| [Browser Glue Architecture](../system/browser-glue.md) | TypeScript layer bridging WIT ABI to DOM |
| [Versioning Strategy](../system/versioning.md) | Semantic versioning across the multi-crate workspace |

## Package Reference

| Document | Description |
|:--|:--|
| [Layered Package Overview](../components/index.md) | Four-layer crate hierarchy with dependency graph |
| [Workspace Package List](../components/packages.md) | Detailed description of each crate |

## Advanced

| Document | Description |
|:--|:--|
| [Debug Agent](../skills/debug-agent.md) | Using the MCP server for AI-assisted debugging |
| [Enterprise Support](../enterprise/support.md) | Commercial support options |
