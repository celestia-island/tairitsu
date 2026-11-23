# Migration from web to wit-bindings

Background reading: this page describes the switch away from wasm-bindgen/web-sys. That switch is
complete — there is no `web` backend left to migrate from — so keep it as history, not as a task.

## Minimal migration
1. Enable feature: web -> wit-bindings
2. Target: wasm32-unknown-unknown -> wasm32-wasip2
3. Platform: `WebPlatform` no longer exists — construct `WitPlatform` directly
4. Ensure host imports from tairitsu-browser:full are provided

## Check command

```bash
cargo check -p tairitsu-web --features wit-bindings
```
