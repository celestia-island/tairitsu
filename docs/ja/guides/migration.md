# web から wit-bindings への移行

1. Cargo feature を web から wit-bindings に切替
2. target を wasm32-unknown-unknown から wasm32-wasip2 に切替
3. WebPlatform::new() を WitPlatform::new()? に切替
