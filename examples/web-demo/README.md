# Tairitsu Web Demo

Simple web demo showcasing Tairitsu's Virtual DOM, Hooks, and rsx! macro.

## Quick Start

```bash
# Start development server with hot reload
just dev

# Or use trunk directly
cd examples/web-demo
trunk serve --open
```

The demo will be available at http://localhost:3000

## Available Examples

### Main Demo (WASM)
- **Location**: `index.html`
- **Description**: Full Tairitsu demo with Virtual DOM, Hooks, and rsx! macro
- **Technologies**: Rust, WASM, tairitsu-vdom, tairitsu-hooks, tairitsu-macros

### Simple HTML/CSS Examples

These are pure HTML/CSS/JavaScript examples for quick testing:

1. **Counter** (`examples/counter.html`)
   - Simple counter with increment/decrement/reset
   - JavaScript state management
   - Basic animations

2. **Hover Effects** (`examples/hover.html`)
   - CSS hover animations
   - Gradient backgrounds
   - Responsive grid layout

3. **Form Components** (`examples/form.html`)
   - Various form inputs
   - Text, email, select, textarea
   - Checkbox, radio, switch components

4. **Demo Index** (`examples/index.html`)
   - Overview of all available demos
   - Quick navigation

## Building for Production

```bash
# Build optimized WASM
just build-web

# Or manually
cd examples/web-demo
trunk build --release
```

Output will be in `examples/web-demo/dist/`

## Project Structure

```
examples/web-demo/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # Main WASM entry point
├── index.html          # Main demo HTML
├── examples/           # Simple HTML/CSS examples
│   ├── index.html      # Demo overview
│   ├── counter.html    # Counter example
│   ├── hover.html      # Hover effects
│   └── form.html       # Form components
└── README.md           # This file
```

## Features Demonstrated

### Main Demo
- ✅ Virtual DOM rendering
- ✅ Reactive state management (use_state)
- ✅ Effect hooks (use_effect)
- ✅ rsx! macro for declarative UI
- ✅ Dynamic styles
- ✅ Event handling

### Simple Examples
- ✅ Pure HTML/CSS/JavaScript
- ✅ No build step required
- ✅ Easy to understand and modify
- ✅ Good for testing and prototyping

## Development

### Prerequisites

- Rust (with wasm32-unknown-unknown target)
- trunk (WASM bundler)
- just (command runner)

```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install just
```

### Ports

- Development server: http://localhost:3000
- Production build server: http://localhost:3000

(Consistent with Hikari project)

### Hot Reload

The development server automatically reloads when you make changes to:
- Rust code (`.rs` files)
- HTML templates
- Assets (CSS, images, etc.)

## Browser Support

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+

## Troubleshooting

### "trunk not found"
```bash
cargo install trunk
```

### "wasm32-unknown-unknown target not found"
```bash
rustup target add wasm32-unknown-unknown
```

### Build errors
```bash
# Clean and rebuild
cargo clean
cargo build
```

## Related Documentation

- [Main README](../../README.md)
- [Examples README](../README.md)
- [PLAN.md](../../PLAN.md)
