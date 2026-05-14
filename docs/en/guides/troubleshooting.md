# Troubleshooting Guide

Common issues and solutions when working with Tairitsu browser-glue and Component Model.

## Build Errors

### wasm32-wasip2 target not found

**Error:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**Solution:**
```bash
rustup target add wasm32-wasip2
```

### wit-bindgen version mismatch

**Error:**
```
error: failed to select a version for `wit-bindgen`
```

**Solution:**
Ensure `wit-bindgen` version matches in `Cargo.toml`:
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### dev-wasm profile not defined

**Error:**
```
error: profile `dev-wasm` is not defined
```

**Solution:**
Add the profile to your workspace `Cargo.toml`:
```toml
[profile.dev-wasm]
inherits = "release"
lto = true
opt-level = 'z'
codegen-units = 1
panic = "abort"
```
Projects created with `tairitsu init` already include this profile.

### browser-glue runtime bundle not found

**Error:**
```
browser-glue runtime bundle (dist/runtime.js) not found
```

**Solution:**
1. Run `npm run build` in `packages/browser-glue/`
2. Or set the environment variable: `TAIRITSU_RUNTIME_BUNDLE=/path/to/browser-glue/dist/runtime.js`
3. Or install via npm: `npm install tairitsu-browser-glue`

### TypeScript compilation errors

**Error:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**Solution:**
Regenerate glue and rebuild:
```bash
cd packages/browser-glue
npm run build
```

## Runtime Errors

### Missing host imports

**Error:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**Solution:**
1. Ensure import map is configured:
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. Verify browser-glue files exist in output directory.

### Component initialization failure

**Error:**
```
Error: Component instantiation failed: undefined import
```

**Solution:**
Check that all required WIT imports have corresponding implementations in browser-glue.

### jco transpile errors

**Error:**
```
Error: Failed to transpile component
```

**Solution:**
1. Ensure jco is installed:
```bash
npm install -g @bytecodealliance/jco
```

2. Verify WASM component is valid:
```bash
wasm-tools print component.wasm
```

## Debug Techniques

### Enable debug logs

In browser console:
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### Inspect WIT bindings

View generated bindings:
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### Browser DevTools

1. Open DevTools (F12)
2. Check Console for errors
3. Network tab for failed module loads
4. Sources tab for debugging

### Component validation

```bash
# Validate component structure
wasm-tools validate component.wasm

# Print component contents
wasm-tools print component.wasm
```

## Common Issues

### Handle not found

**Symptom:** `null` returned from DOM operations

**Cause:** Handle was garbage collected or not registered

**Solution:** Ensure elements remain referenced in JavaScript

### Event not firing

**Symptom:** Event handlers not called

**Cause:** Listener ID mismatch or event type incorrect

**Solution:** Check `addEventListener` returns valid listener ID

### Memory leaks

**Symptom:** Increasing memory usage over time

**Cause:** Handles not released after use

**Solution:** Call `dropHandle()` when done with objects

## Performance Issues

### Slow component load

**Solutions:**
1. Use release build: `cargo build --release`
2. Enable LTO in `Cargo.toml`:
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### High event latency

**Solutions:**
1. Avoid synchronous operations in handlers
2. Use `requestAnimationFrame` for visual updates
3. Debounce rapid events

## Getting Help

1. Check existing issues: https://github.com/anomalyco/opencode/issues
2. Review documentation in `docs/` directory
3. Examine example code in `examples/website/`
