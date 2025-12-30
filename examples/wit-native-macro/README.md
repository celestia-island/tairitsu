# Proc-Macro Based WIT Interface Generation

**Status**: ✅ **Fully Implemented**

This example demonstrates automatic generation of type-safe WIT command enums using procedural macros, eliminating all manual boilerplate while maintaining compile-time type safety.

## Overview

This approach uses procedural macros to automatically generate command enums and response types from WIT-like interface definitions. This provides:

- **Zero boilerplate**: Write WIT interface once, get enums automatically
- **Single source of truth**: WIT definition drives all code generation
- **Compile-time type safety**: Full Rust type system enforcement
- **No runtime overhead**: Zero serialization, direct function calls
- **IDE integration**: Full autocomplete, type hints, and refactoring support

## Architecture

### Macro System (`packages/macros`)

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
// - FilesystemCommands enum with Read, Write, Delete, List variants
// - FilesystemResponse enum with corresponding response types
// - WitCommand trait implementation
// - Command name mapping
```

## Running the Examples

### Demo - Basic Interface Examples

Demonstrates automatic enum generation for Filesystem and Network interfaces:

```bash
just run-macro-demo
```

### Host - Full Integration Example

Demonstrates composition of multiple macro-generated interfaces (Filesystem, Network, Database):

```bash
just run-macro-host
```

### WASM Integration

Demonstrates real bidirectional communication with WASM:

```bash
just run-macro-wasm
```

## Benefits

| Feature | Manual | Macro (This Approach) |
|---------|--------|-----------------------|
| Boilerplate | ~50 lines per interface | 0 lines |
| Type Safety | ✅ | ✅ |
| Serialization | ❌ None | ❌ None |
| Maintainability | Good | Excellent |
| Single Source | No | Yes (WIT) |
| Refactoring | Manual | Automatic |

## When to Use This Approach

Use macro-based generation when:

- ✅ You have many WIT interfaces (> 5)
- ✅ WIT definitions change frequently
- ✅ You want zero boilerplate
- ✅ Single source of truth is important
- ✅ You're building a plugin system with extensible interfaces
