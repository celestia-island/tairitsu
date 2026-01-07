# Tairitsu Dynamic Advanced Invocation Example

## 🚀 Running the Example

```bash
# Navigate to example directory
cd examples/wit-dynamic-advanced

# Run the demo
cargo run --bin dynamic-advanced-demo
```

## 📋 Requirements

- Rust 1.70+ with the `wasm32-wasip1` and `wasm32-wasip2` targets
- Tairitsu runtime with `dynamic` feature enabled
- WASI adapter for WASM components

## 🎨 Scenarios Covered

### 1. RON Serialization

Demonstrates RON (Rust Object Notation) serialization for better Rust type compatibility compared to JSON:

```rust
#[derive(Serialize, Deserialize)]
struct CalculatorRequest {
    a: i32,
    b: i32,
}

let request = CalculatorRequest { a: 42, b: 58 };
let ron = RonBinding::params_to_ron(&request)?;
// Result: "(a: 42, b: 58)"
```

### 2. RonTool Registry

Type-safe tool wrapping using RON:

```rust
let tool = typed_ron_tool("reverse", |input: ReverseInput| -> ReverseOutput {
    ReverseOutput {
        result: input.text.chars().rev().collect(),
    }
});

let result = registry.invoke("reverse", "(text: \"Hello\")")?;
```

### 3. Host Import Registration

Register custom host functions for WASM components:

```rust
let mut registry = HostImportRegistry::new();
registry.register(
    "host-log".to_string(),
    vec![Type::String, Type::String],
    vec![Type::String],
    |args| {
        // Handle logging
        Ok(vec![Val::String("OK".to_string())])
    },
);
```

### 4. Runtime Discovery

Query available functions at runtime:

```rust
let exports = container.list_guest_exports()?;
for export in exports {
    println!("Function: {}", export.name);
    println!("  Params: {:?}", export.params);
    println!("  Results: {:?}", export.results);
}
```

### 5. API Usage Patterns

Shows common usage patterns for dynamic invocation:

```rust
// Pattern 1: RON-based dynamic call
let result = container.call_guest_raw_desc(
    "add",
    "(a: 42, b: 58)"
)?;

// Pattern 2: Binary high-performance call
use wasmtime::component::Val;
let args = vec![Val::S32(42), Val::S32(58)];
let results = container.call_guest_binary("add", &args)?;

// Pattern 3: Host import call
container.call_host_import_raw_desc(
    "host-log",
    r#"["info", "Message"]"#
)?;
```

## 🔌 API Comparison

### RON vs JSON

| Feature | JSON | RON |
| ------- | ---- | ---- |
| **Rust Enums** | `{"variant": "value"}` | `Variant(value)` ✓ |
| **Tuples** | `[1, 2]` (no semantics) | `(1, 2)` ✓ |
| **Unit Type** | `null` or `{}` | `()` ✓ |
| **Option** | `null` or wrapper | `Some(value)` / `None` ✓ |
| **Result** | Custom wrapper | `Ok(value)` / `Err(e)` ✓ |
| **Performance** | Good | Better ✓ |
| **Rust Integration** | Basic | Native ✓ |

### Dynamic Invocation Paths

| Path | Function | Use Case |
| ---- | -------- | ------- |
| **RON** | `call_guest_raw_desc()` | Convenient, human-readable, debugging |
| **Binary** | `call_guest_binary()` | High performance, zero-copy |
| **Static** | Existing type-safe API | Best performance, compile-time checks |

## 💡 Usage Guidelines

### When to Use Dynamic Invocation

**Choose RON Path (`call_guest_raw_desc`) when:**

- Building RPC/HTTP APIs
- Debugging and logging
- Cross-language communication
- Human-readable serialization needed

**Choose Binary Path (`call_guest_binary`) when:**

- Maximum performance required
- Hot loops and frequent calls
- Binary protocol compatibility
- Zero-copy operations

**Choose Static API (existing) when:**

- WIT interface known at compile time
- Best performance is critical
- Full type safety required
- IDE autocomplete needed

## 🧪 Testing

```bash
# Run the example
cargo run --bin dynamic-advanced-demo

# Run with logging
RUST_LOG=debug cargo run --bin dynamic-advanced-demo

# Run tests
cargo test --package tairitsu-example-wit-dynamic-advanced
```

## 📚 Related Examples

- **wit-dynamic** - JSON-based tool registry (simpler)
- **wit-runtime** - Runtime WIT loading
- **wit-compile-time** - Compile-time static binding

## 🔄 Migration Guide

### From JSON to RON

**Old (JSON):**

```rust
use tairitsu::JsonBinding;
let json = r#"{"a": 42, "b": 58}"#;
let result = registry.invoke("calculator", json)?;
```

**New (RON):**

```rust
use tairitsu::RonBinding;
let ron = r#"(a: 42, b: 58)"#;
let result = registry.invoke("calculator", ron)?;
```

### From Static to Dynamic

**Old (Static):**

```rust
let guest = container.guest().downcast_ref::<MyWit>()?;
let result = guest.my_function(&mut container.store)?;
```

**New (Dynamic):**

```rust
let result = container.call_guest_raw_desc(
    "my-function",
    r#"Request { input: "hello" }"#
)?;
```

## ⚠️ Limitations

### Current Implementation (0.2.2)

**Supported:**

- ✅ Basic types: Bool, Integers (U8-S64), String, Char
- ✅ Floating point types: Float32, Float64
- ✅ Complex types: List, Tuple, Record, Variant, Result, Option
- ✅ Guest export dynamic invocation
- ✅ Host import registration and invocation
- ✅ Runtime discovery for host imports
- ✅ Full bidirectional RON serialization/deserialization

**Not Yet Supported:**

- ⚠️ Runtime discovery for guest exports (placeholder implementation)
- ⚠️ Nested complex types (e.g., List<List<T>>, Record containing Lists, etc.)

**Planned for Future Versions:**

- 📋 Complete guest export discovery with type information
- 📋 Nested complex type support
- 📋 Performance optimizations for hot paths

## 🐛 Troubleshooting

### Common Issues

**Issue: "Dynamic instance not available"**

- Cause: Container built without dynamic instance initialization
- Fix: Ensure `dynamic` feature is enabled and guest initializer is provided

**Issue: "Type mismatch in RON conversion"**

- Cause: RON format doesn't match expected type
- Fix: Check RON syntax matches WIT function signature

**Issue: "Host import not found"**

- Cause: Import not registered in container
- Fix: Register imports before calling `call_host_import_raw_desc()`

## 📖 Additional Resources

- [Tairitsu Documentation](https://docs.rs/tairitsu)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [RON Format](https://github.com/ron-rs/ron)
- [Wasmtime Component Model](https://docs.rs/wasmtime)

## 🤝 Contributing

To improve this example:

1. Add more realistic use cases
2. Implement actual WASM guest component
3. Add performance benchmarks
4. Improve error messages

## 📄 License

This example is part of the Tairitsu project and follows the same license.
