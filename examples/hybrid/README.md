# Tairitsu Hybrid Example

This example demonstrates a hybrid architecture where a single crate can be compiled to both:
- **Native** (host side) - Runs the Tairitsu runtime and manages WASM modules
- **WASM** (guest side) - Runs inside the Tairitsu runtime as a guest module

Both sides share the same WIT interface definitions, enabling type-safe bidirectional communication.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Host (Native)                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Tairitsu Registry (like Docker daemon)           │  │
│  │  - Manages Images and Containers                  │  │
│  └───────────────────────────────────────────────────┘  │
│                          │                               │
│  ┌───────────────────────▼───────────────────────────┐  │
│  │  Container (running instance)                     │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │  Guest (WASM Module)                        │  │  │
│  │  │                                             │  │  │
│  │  │  Exports: guest-api                         │  │  │
│  │  │  - init()                                   │  │  │
│  │  │  - handle-command()                         │  │  │
│  │  │                                             │  │  │
│  │  │  Imports: host-api                          │  │  │
│  │  │  - execute() ◄─┐                            │  │  │
│  │  │  - log()       │ Bidirectional              │  │  │
│  │  └────────────────┼─────────────────────────────┘  │  │
│  │                   │ Communication                  │  │
│  │                   │ via WIT                        │  │
│  │  Host provides:   │                                │  │
│  │  - execute() ◄────┘                                │  │
│  │  - log()                                           │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Building and Running

### 1. Build the WASM module (guest side)

```bash
cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid
```

### 2. Run the host side

```bash
cargo run --package tairitsu-example-hybrid --bin host
```

Or use the shortcut:

```bash
just run
```

## How It Works

### Shared WIT Interface (`wit/world.wit`)

Both host and guest use the same WIT definitions:

- **guest-api**: Functions exported by the WASM guest
  - `init()`: Initialize the guest module
  - `handle-command()`: Process commands from the host

- **host-api**: Functions provided by the host
  - `execute()`: Execute commands on the host
  - `log()`: Send log messages to the host

### Guest Side (`src/lib.rs`)

Compiled to WASM, this implements:
- The `guest-api` interface (exports)
- Calls to `host-api` functions (imports)

### Host Side (`src/host.rs`)

Runs natively and:
- Loads the WASM module as an Image
- Creates a Container to run the WASM
- Implements handlers for `host-api` functions
- Calls guest functions via the `guest-api`

## Key Features Demonstrated

1. **Docker-like Image/Container Model**: Load WASM as images, run as containers
2. **Bidirectional Communication**: Both host and guest can call each other
3. **Type-Safe WIT Interfaces**: Compile-time guarantees for cross-boundary calls
4. **Single Crate, Dual Targets**: Same code, different compilation targets
5. **Registry Pattern**: Manage multiple images and containers
