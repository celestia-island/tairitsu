# Approach A: Proc-Macro Based WIT Interface Generation

**Status**: ✅ **Fully Implemented**

This example demonstrates automatic generation of type-safe WIT command enums using procedural macros, eliminating all manual boilerplate while maintaining compile-time type safety.

## Overview

Approach A uses procedural macros to automatically generate command enums and response types from WIT-like interface definitions. This provides:

- **Zero boilerplate**: Write WIT interface once, get enums automatically
- **Single source of truth**: WIT definition drives all code generation
- **Compile-time type safety**: Full Rust type system enforcement
- **No runtime overhead**: Zero serialization, direct function calls
- **IDE integration**: Full autocomplete, type hints, and refactoring support

## Architecture

### Macro System (`packages/tairitsu-macros`)

Two main procedural macros:

1. **`#[derive(WitCommand)]`**: Automatically implements `WitCommand` trait for enums
   - Generates `command_name()` mapping from enum variants
   - Supports custom response types via `#[wit_response(Type)]`
   - Automatic kebab-case conversion (e.g., `HttpGet` → `http-get`)

2. **`wit_interface!`**: Generates complete command/response enums from WIT syntax
   - Parses WIT-like function definitions
   - Generates Commands enum with all variants
   - Generates Response enum with associated types
   - Automatically implements `WitCommand` trait

### Example Usage

```rust
use tairitsu_macros::wit_interface;

// Define interface using WIT-like syntax
wit_interface! {
    interface filesystem {
        read: func(path: String) -> Result<Vec<u8>, String>;
        write: func(path: String, data: Vec<u8>) -> Result<(), String>;
        delete: func(path: String) -> Result<(), String>;
        list: func(directory: String) -> Result<Vec<String>, String>;
    }
}

// Macro automatically generates:
// - FileSystemCommands enum with Read, Write, Delete, List variants
// - FileSystemResponse enum with corresponding response types
// - WitCommand trait implementation
// - Command name mapping
```

## Running the Examples

### Demo - Basic Interface Examples

Demonstrates automatic enum generation for FileSystem and Network interfaces:

```bash
cargo run --package tairitsu-example-wit-native-a --bin approach-a-demo
```

### Host - Full Integration Example

Demonstrates composition of multiple macro-generated interfaces (FileSystem, Network, Database):

```bash
cargo run --package tairitsu-example-wit-native-a --bin approach-a-host
```

## Benefits Over Manual Implementation

| Feature | Manual (Approach B) | Macro (Approach A) |
|---------|--------------------|--------------------|
| Boilerplate | ~50 lines per interface | 0 lines |
| Type Safety | ✅ | ✅ |
| Serialization | ❌ None | ❌ None |
| Maintainability | Good | Excellent |
| Single Source | No | Yes (WIT) |
| Refactoring | Manual | Automatic |

## Comparison with Approach B

| Aspect | Approach A (Macros) | Approach B (Traits) |
|--------|---------------------|---------------------|
| **Boilerplate** | None | Some |
| **Setup Cost** | Medium (macro crate) | Low |
| **Runtime Cost** | Zero | Zero |
| **Type Safety** | ✅ Compile-time | ✅ Compile-time |
| **Composability** | ✅ Via traits | ✅ Via traits |
| **IDE Support** | ✅ Full | ✅ Full |
| **Maintainability** | ✅ Excellent | ✅ Good |
| **Learning Curve** | Medium | Low |

## When to Use Approach A

Use Approach A when:

- ✅ You have many WIT interfaces (> 5)
- ✅ WIT definitions change frequently
- ✅ You want zero boilerplate
- ✅ Single source of truth is important
- ✅ You're building a plugin system with extensible interfaces
