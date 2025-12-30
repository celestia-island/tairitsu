# Approach B: Trait-Based Composable WIT Interfaces

This example demonstrates a trait-based approach to composing multiple WIT interfaces without runtime serialization overhead.

## Key Concepts

### 1. WitCommand Trait
Defines the interface for type-safe commands that can be dispatched:
```rust
pub trait WitCommand: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
    fn command_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}
```

### 2. WitCommandHandler Trait
Handlers implement this trait to execute commands:
```rust
pub trait WitCommandHandler<C: WitCommand>: Send + Sync {
    fn execute(&mut self, command: &C) -> Result<C::Response, String>;
}
```

### 3. Composable Interfaces
Multiple WIT interfaces can be combined:
```rust
let mut composite = CompositeWitInterface::new();
composite.add_interface(Box::new(FileSystemBasicInterface));
composite.add_interface(Box::new(FileSystemAdvancedInterface));
```

## Benefits

✅ **No serialization overhead** - Direct function calls, no JSON/serde
✅ **Compile-time type safety** - Rust type system enforces correctness
✅ **Composable** - Multiple interfaces can be combined
✅ **Extensible** - New interfaces can build on existing ones  
✅ **Zero-cost abstractions** - Trait dispatch is optimized away

## Running the Example

```bash
cargo run --package tairitsu-example-wit-native-b --bin approach-b-demo
```

## Example Output

```
=== Approach B: Trait-Based Composable WIT Interfaces ===

✓ Added basic filesystem interface
✓ Added advanced filesystem interface
Registered handlers for: filesystem-basic
Registered handlers for: filesystem-advanced

=== Demonstrating Type-Safe Commands ===

Command: Write { path: "/test.txt", data: [72, 101, 108, 108, 111, 44, 32, 87, 73, 84, 33] }
✓ Write successful

Command: Read { path: "/test.txt" }
✓ Read successful: Hello, WIT!

Command: List { directory: "/dir" }
✓ List successful: ["/dir/file1.txt", "/dir/file2.txt"]

Command: Copy { from: "/dir/file1.txt", to: "/dir/file1_copy.txt" }
✓ Copy successful: ["/dir/file1_copy.txt"]

=== Key Benefits ===
✓ No runtime serialization overhead
✓ Compile-time type safety
✓ Composable interfaces via traits
✓ Each interface can extend/build on others
✓ Zero-cost abstractions
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│         CompositeWitInterface                    │
│  ┌──────────────────┐  ┌──────────────────┐   │
│  │ FileSystem Basic │  │FileSystem Advanced│   │
│  │   Interface      │  │    Interface      │   │
│  └──────────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────┘
                    ↓
        ┌───────────────────────┐
        │  WitCommandDispatcher  │
        │  ┌─────────────────┐  │
        │  │  Trait Objects  │  │
        │  │  (dyn Handler)  │  │
        │  └─────────────────┘  │
        └───────────────────────┘
                    ↓
         Direct function calls
          (no serialization)
```

## Composability Example

```rust
// Basic interface provides fundamental operations
enum FileSystemBasicCommands {
    Read { path: String },
    Write { path: String, data: Vec<u8> },
}

// Advanced interface builds on basic operations
enum FileSystemAdvancedCommands {
    List { directory: String },      // Uses storage from basic
    Copy { from: String, to: String }, // Delegates to basic Read/Write
}
```

## Comparison with Approach A

| Feature | Approach B (This) | Approach A (Proc-Macro) |
|---------|-------------------|-------------------------|
| Serialization | None | None |
| Code Generation | Manual | Automatic via macro |
| Composability | Trait objects | Enum composition |
| Learning Curve | Medium | Higher |
| Flexibility | High | Very High |
| Boilerplate | Some | Minimal |
