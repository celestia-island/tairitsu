# Tairitsu Framework Website Demo

A comprehensive demonstration of the Tairitsu framework's core features, inspired by Hikari's website demo structure.

## Features

This demo showcases:

- **rsx! Macro**: Declarative UI syntax for building components
- **Builder System**: StyleBuilder, ClassesBuilder, and AnimationBuilder
- **Reactive System**: Signal, Effect, and state management hooks
- **Platform Abstraction**: Cross-platform WebPlatform implementation

## Project Structure

```
examples/website/
├── Cargo.toml              # Project configuration
├── README.md               # This file
├── public/                 # Static assets
│   └── favicon.ico
├── src/
│   ├── lib.rs              # Main library entry
│   ├── app.rs              # Main application component
│   ├── components/         # Reusable components
│   │   ├── mod.rs
│   │   ├── layout.rs       # Layout components
│   │   ├── navigation.rs   # Navigation components
│   │   └── code_block.rs   # Code display component
│   ├── pages/              # Page components
│   │   ├── mod.rs
│   │   ├── home.rs         # Homepage
│   │   ├── rsx_demo.rs     # rsx! macro demos
│   │   ├── builders.rs     # Builder system demos
│   │   └── reactive.rs     # Reactive system demos
│   └── styles/             # CSS styles
│       └── main.css
└── dist/                   # Build output (generated)
```

## Running the Demo

### Development Mode

```bash
# Using tairitsu-packager (recommended)
cd examples/website
cargo run --package tairitsu-packager -- dev --open

# Or using just
just dev
```

### Production Build

```bash
cargo run --package tairitsu-packager -- build --release
```

### Serve Production Build

```bash
cd dist
python3 -m http.server 3000
```

## Demo Sections

### 1. Home Page
- Framework introduction
- Feature overview
- Quick start guide

### 2. rsx! Macro Demo
- Basic elements and attributes
- Dynamic content and expressions
- Conditional rendering
- Loops and iteration
- Event handling

### 3. Builder System Demo
- **StyleBuilder**: CSS style construction
- **ClassesBuilder**: Dynamic class management
- **AnimationBuilder**: Animation configuration
- Integration with reactive system

### 4. Reactive System Demo
- **use_state**: Local state management
- **use_signal**: Reactive signals
- **use_effect**: Side effects
- **use_style**: Dynamic styling
- Performance optimization with batch updates

## Multi-language Support

All documentation and examples are available in multiple languages:

- 🇨🇳 Chinese (zh-CN)
- 🇺🇸 English (en-US)
- 🇯🇵 Japanese (ja-JP) - Coming soon

## Architecture

Built with:
- **Frontend**: Tairitsu vdom + hooks + rsx! macro
- **Build Tool**: tairitsu-packager (replaces trunk)
- **Configuration**: Cargo.toml metadata (no HTML templates)

## Comparison with Hikari Website

| Aspect | Hikari Website | Tairitsu Website |
|--------|----------------|------------------|
| Framework | Dioxus | Tairitsu (custom vdom) |
| UI Syntax | rsx! | rsx! |
| Styling | StyleBuilder | StyleBuilder (shared) |
| State | Dioxus signals | Custom reactive system |
| Build Tool | trunk | tairitsu-packager |
| Focus | Component library | Framework mechanisms |

## Development Status

- ✅ Project structure
- ✅ Basic configuration
- 🚧 Component implementation
- 🚧 Page content
- 🚧 Multi-language support
- 🚧 E2E testing

## Contributing

This demo serves as both documentation and testing ground for Tairitsu framework features. Contributions are welcome!

## License

MIT
