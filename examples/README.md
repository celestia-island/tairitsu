# Tairitsu Examples

This directory contains various usage examples for the Tairitsu WASM runtime.

## 📚 Example List

### 1. wit-compile-time - Compile-time WIT Binding

Demonstrates how to use WIT definitions and wasmtime bindgen at compile time for complete static type safety.

**Features:**

- ✅ Complete compile-time type checking
- ✅ Zero runtime overhead
- ✅ IDE autocomplete support
- ✅ Best performance

**Run the example:**

```bash
cd examples/wit-compile-time
cargo run --bin compile-time-demo
```

**Use cases:**

- WIT interfaces known at compile time
- Best performance required
- Static type safety needed
- Stable development environment

---

### 2. wit-runtime - Runtime WIT Loading

Demonstrates how to dynamically load WIT definitions at runtime for interface discovery and capability checking.

**Features:**

- 🔍 Runtime dynamic WIT loading
- 🔍 Interface discovery and validation
- 🔍 Plugin system support
- 🔍 Flexible capability negotiation

**Run the example:**

```bash
cd examples/wit-runtime
cargo run --bin runtime-demo
```

**Use cases:**

- Plugin systems
- Microservices architecture
- Hot updates
- Multi-tenant systems

---

### 3. wit-dynamic - Dynamic JSON Invocation

Demonstrates how to use JSON serialization layer for fully dynamic WIT function calls.

**Features:**

- 🔄 Fully dynamic function calls
- 🔄 JSON serialization/deserialization
- 🔄 Type-safe tool wrapping
- 🔄 Flexible registry management

**Run the example:**

```bash
cd examples/wit-dynamic
cargo run --bin dynamic-demo
```

**Use cases:**

- HTTP API servers
- Message queue processing
- Command-line tools
- Cross-language communication

---

### 4. wit-dynamic-advanced - Dynamic WASM Component Invocation (NEW in 0.2.2)

Demonstrates the new dynamic invocation features for runtime WASM Component function calls with both RON and binary canonical ABI support.

**Features:**

- 🚀 Runtime dynamic guest export calls (RON + Binary)
- 📥 Host import registration and invocation
- 🔍 Runtime function discovery
- 🔤 RON serialization for Rust-friendly types
- ⚡ Binary canonical ABI for high performance

**Run the example:**

```bash
cd examples/wit-dynamic-advanced
cargo run --bin dynamic-advanced-demo
```

**Use cases:**

- Plugin systems with dynamic loading
- RPC servers with flexible protocols
- Cross-language WASM communication
- Hot-reloadable component systems
- Performance-critical dynamic invocation

**Key Differences from wit-dynamic:**

- Uses actual WASM Components (not just tool registry)
- Supports both RON and binary calling paths
- Guest export discovery and invocation
- Host import management for bidirectional communication

---

### 5. wit-native-simple - Simple Trait Implementation

Demonstrates basic trait-based WIT interface implementation.

**Run the example:**

```bash
cd examples/wit-native-simple

# Run demo
cargo run --bin simple-demo

# Run host
cargo run --bin simple-host

# Run WASM host
cargo run --bin simple-wasm-host
```

---

### 5. wit-native-macro - Macro-Assisted Trait Implementation

Demonstrates how to use macros to simplify WIT interface definitions.

**Run the example:**

```bash
cd examples/wit-native-macro

# Run demo
cargo run --bin macro-demo

# Run host
cargo run --bin macro-host
```

---

## 🎯 How to Choose the Right Example

### Compile-time vs Runtime vs Dynamic

| Feature | Compile-time | Runtime | Dynamic JSON | Dynamic WASM |
| ------- | ----------- | ------- | ------------ | ------------ |
| Type Safety | Full | Partial | Runtime | Runtime |
| Performance | Best | Good | Medium | Best (Binary) |
| Flexibility | Low | High | Highest | Highest |
| Complexity | Low | Medium | Medium | High |
| Debug Difficulty | Low | Medium | Medium | Medium |
| WASM Components | No | No | No | **Yes** |
| Bidirectional Calls | No | No | No | **Yes** |

### Recommended Use Cases

**Compile-time binding (`wit-compile-time`)**

- Core business logic
- High performance requirements
- Fixed WIT interfaces
- Best type safety needed

**Runtime binding (`wit-runtime`)**

- Plugin systems
- Hot update requirements
- Multiple versions coexistence
- Microservices architecture

**Dynamic JSON (`wit-dynamic`)**

- HTTP APIs
- Message queues
- Cross-language calls
- External integrations

**Dynamic WASM (`wit-dynamic-advanced`)** ⭐ **NEW in 0.2.2**

- Plugin systems with WASM sandboxing
- Hot-reloadable components
- Performance-critical dynamic calls
- Bidirectional guest-host communication
- Multi-language WASM ecosystems

---

## 📖 Learning Path

1. **Beginners**: Start with `wit-compile-time`
   - Understand basic WIT concepts
   - Learn compile-time binding

2. **Intermediate**: Learn `wit-runtime`
   - Understand runtime loading
   - Master interface discovery

3. **Advanced**: Explore `wit-dynamic`
   - Implement dynamic invocation
   - Build flexible systems

4. **Expert**: Master `wit-dynamic-advanced` ⭐ **NEW**
   - Dynamic WASM component invocation
   - RON and binary calling paths
   - Host import management
   - Runtime function discovery

5. **Production**: Reference `wit-native-*`
   - Complete WASM integration
   - Production environment examples

---

## 🔧 Build All Examples

```bash
# Build all examples
cargo build --workspace

# Run all examples
cargo test --workspace

# Release build
cargo build --workspace --release
```

---

## 📝 Example Structure

Each example contains:

- `Cargo.toml` - Project configuration
- `src/` - Source code
- `README.md` - Detailed documentation (if available)

---

## 🤝 Contributing

New examples are welcome! Suggested example topics:

- **Dynamic WASM**: Real-world plugin system examples
- **Performance**: Benchmarking RON vs JSON vs Binary paths
- **Complete web services**: HTTP + WASM integration
- **Database integration**: WASM components with database access
- **Filesystem operations**: Secure sandboxed file I/O
- **Network communication**: WASM networking examples
- **Cryptography and security**: Secure enclaves with WASM
- **Multi-language**: Guest components in different languages (C++, Rust, Go)

---

## ✨ What's New in 0.2.2

- **New Example**: `wit-dynamic-advanced` showcasing dynamic WASM component invocation
- **RON Support**: Rust-friendly serialization replacing JSON for better type compatibility
- **Binary Path**: High-performance canonical ABI calling
- **Host Imports**: Bidirectional guest-host communication
- **Runtime Discovery**: Query available functions at runtime

---

## 📚 Related Documentation

- [Main README](../../README.md)
- [API Documentation](https://docs.rs/tairitsu)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
