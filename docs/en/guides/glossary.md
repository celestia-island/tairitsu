# Glossary

- WIT: WebAssembly Interface Types
- Component Model: ABI and linking model for composable wasm components
- Host import: function provided by host to guest component
- Guest export: function exposed by component to host
- web backend: wasm-bindgen/web-sys implementation path
- wit-bindings backend: wit-bindgen implementation path for wasm32-wasip2
- browser-glue: TypeScript host runtime implementing WIT imports in the browser
- setInterval / use_interval: recurring timer hook; fires callback every N milliseconds
- wit_plugin: project-level WIT extension mechanism for third-party JS library integration
- dev-wasm: cargo build profile optimized for debug WASM components (LTO, size optimization)
