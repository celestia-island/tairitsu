# 從 web 遷移到 wit-bindings

1. 將 Cargo feature 由 web 改為 wit-bindings
2. target 由 wasm32-unknown-unknown 改為 wasm32-wasip2
3. WebPlatform::new() 改為 WitPlatform::new()?
