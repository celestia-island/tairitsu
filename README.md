<div align="center"><img src="./docs/logo.png" /></div>
<h1 align="center">Tairitsu</h1>
<div align="center">
 <strong>
   A WIT-based Bidirectional Communication Framework for WebAssembly
 </strong>
</div>

<br />

<div align="center">
  <!-- CI status -->
  <a href="https://github.com/celestia-island/tairitsu/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/celestia-island/tairitsu/ci.yml?branch=main"
      alt="CI Status" />
  </a>
  <!-- Built with -->
  <a href="https://sagiegurari.github.io/cargo-make">
    <img src="https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg" alt="Built with cargo-make">
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://celestia.world">
      Website
    </a>
    <span> | </span>
    <a href="./examples/hybrid">
      Example
    </a>
  </h3>
</div>

<br/>

## What is Tairitsu?

**Tairitsu** (対立) carries a dual meaning:

1. **From Arcaea**: Named after the character Tairitsu from the rhythm game Arcaea, representing the "Conflict" side
2. **Opposition & Duality**: Reflects the architectural concept of this framework - the inherent duality and opposition between the WASM virtual machine (guest) and the host environment, connected through WIT (WebAssembly Interface Types)

## Features

- 🐳 **Docker-like Architecture**: Image/Container model for managing WASM modules
- 🔄 **Bidirectional Communication**: Type-safe communication between host and guest via WIT
- 🎯 **Registration Framework**: Manage multiple WASM components like a container runtime
- 🔌 **Flexible Interface Control**: Host can selectively expose or lock down APIs
- 🦀 **Pure Rust**: Built on Wasmtime with the Component Model
- 📦 **Hybrid Compilation**: Same crate can target both native and WASM

## Quick Start

### Build the Example

```bash
# Build the WASM guest module
cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid

# Run the native host
cargo run --package tairitsu-example-hybrid --bin host
```

Or use cargo-make:

```bash
cargo make run
```

### Basic Usage

```rust,no_run
use tairitsu::{Registry, Container};
use bytes::Bytes;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create a registry (like Docker daemon)
let registry = Registry::new();

// Register a WASM module as an image
let wasm_binary = Bytes::from(std::fs::read("module.wasm")?);
registry.register_image("my-module:v1", wasm_binary)?;

// Get the image and create a container
let image = registry.get_image("my-module:v1").unwrap();
let mut container = Container::new(&image)?;

// Set up bidirectional communication handlers
container
    .on_execute(|cmd: String, payload: String| {
        // Handle guest requests to the host
        Ok(format!("Host processed: {}", cmd))
    })
    .on_log(|level: String, msg: String| {
        println!("[Guest][{}] {}", level, msg);
    });

// Initialize and communicate with the guest
container.init()?;
let response = container.handle_command("greet", "Hello")?;
# Ok(())
# }
```

## Architecture

Tairitsu implements a Docker-inspired architecture for WebAssembly:

```
┌─────────────────────────────────┐
│         Registry                │  Manages images and containers
├─────────────────────────────────┤
│  Image  │  Image  │  Image      │  Compiled WASM components
├─────────────────────────────────┤
│  Container  │  Container        │  Running instances
│  ┌──────────────────────────┐   │
│  │     Guest (WASM)         │   │  Bidirectional
│  │  ↕ WIT Interface ↕       │◄──┼─ Communication
│  │        Host              │   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘
```

### Core Concepts

- **Registry**: Manages WASM images and running containers (like Docker daemon)
- **Image**: A compiled WASM component that can be instantiated (like Docker image)
- **Container**: A running instance of an Image with its own state (like Docker container)
- **WIT Interface**: Type-safe communication layer between host and guest

## WIT Interface

The framework uses WebAssembly Interface Types (WIT) for defining communication:

```wit
// Guest exports (callable from host)
interface guest-api {
    init: func() -> result<_, string>;
    handle-command: func(command: string, payload: string) -> result<string, string>;
}

// Host provides (callable from guest)
interface host-api {
    execute: func(command: string, payload: string) -> result<string, string>;
    log: func(level: string, message: string);
}
```

## Project Structure

```
tairitsu/
├── wit/                    # WIT interface definitions
│   └── world.wit          # Core Tairitsu world
├── packages/
│   └── vm/                # Core framework (renamed to tairitsu)
│       ├── src/
│       │   ├── image.rs      # Image management
│       │   ├── container.rs  # Container runtime
│       │   └── registry.rs   # Registry system
└── examples/
    └── hybrid/            # Hybrid example (native + WASM)
        ├── src/
        │   ├── lib.rs     # WASM guest side
        │   └── host.rs    # Native host side
        └── README.md
```

## Philosophy

Tairitsu embraces the duality between WASM guests and native hosts. Rather than seeing them as separate, the framework treats them as two sides of the same system, connected through well-defined WIT interfaces. The host maintains control over what capabilities to expose, while guests can safely request services, creating a balanced "opposition" that enables powerful yet secure applications.

## License

See [LICENSE](LICENSE) for details.
