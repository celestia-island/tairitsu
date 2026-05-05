# Migration from web to wit-bindings

Use this guide when switching browser integration from wasm-bindgen/web-sys to Component Model WIT bindings.

## Minimal migration
1. Enable feature: web -> wit-bindings
2. Target: wasm32-unknown-unknown -> wasm32-wasip2
3. Platform: WebPlatform::new() -> WitPlatform::new()?
4. Ensure host imports from tairitsu-browser:full are provided

## Check command

```bash
cargo check -p tairitsu-web --features wit-bindings
```
