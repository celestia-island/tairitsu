# Approach A: Proc-Macro Generated WIT Commands

This approach uses procedural macros to automatically generate type-safe command enums from WIT interface definitions.

## Overview

Approach A automatically parses WIT definitions and generates Rust enums with full type safety, eliminating boilerplate code.

## Planned Features

### 1. Automatic Enum Generation
```rust
// Given a WIT interface:
// interface filesystem {
//     read: func(path: string) -> result<list<u8>, string>;
//     write: func(path: string, data: list<u8>) -> result<_, string>;
// }

// The macro generates:
#[wit_commands(interface = "filesystem")]
pub enum FileSystemCommands {} // Auto-populated

// Expands to:
pub enum FileSystemCommands {
    Read { path: String },
    Write { path: String, data: Vec<u8> },
}
```

### 2. Composable Enums
```rust
// Combine multiple interfaces
#[wit_compose]
pub enum AllCommands {
    #[from(filesystem_basic)]
    FileSystem(FileSystemBasicCommands),
    
    #[from(filesystem_advanced)]
    FileSystemAdvanced(FileSystemAdvancedCommands),
}
```

### 3. Automatic Handler Trait Implementation
```rust
// The macro also generates the handler trait
impl WitCommandHandler<FileSystemCommands> for MyHandler {
    // Auto-generated dispatch logic
}
```

## Architecture

```
┌──────────────────────────────────────────────┐
│          WIT Definition (world.wit)           │
└──────────────────────────────────────────────┘
                    ↓
        ┌───────────────────────┐
        │   wit_bindgen parse   │
        └───────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│     Procedural Macro (tairitsu-macros)        │
│  • Parse wit_bindgen output                  │
│  • Generate command enums                    │
│  • Generate handler traits                   │
│  • Generate composition helpers              │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│        Type-Safe Rust Enums + Traits         │
│  • Zero runtime overhead                     │
│  • Full IDE support                          │
│  • Compile-time verification                 │
└──────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Basic Macro
- [ ] Create `tairitsu-macros` crate
- [ ] Parse WIT interface definitions
- [ ] Generate basic command enums
- [ ] Generate WitCommand trait implementations

### Phase 2: Composition
- [ ] Support multiple interfaces
- [ ] Generate composite enums
- [ ] Implement enum flattening/nesting strategies

### Phase 3: Advanced Features
- [ ] Custom derive macros for handlers
- [ ] Automatic dispatcher generation
- [ ] Error handling improvements

## Usage Example (When Implemented)

```rust
use tairitsu_macros::wit_commands;

// Automatically generate from WIT
#[wit_commands(
    wit_file = "../../wit/world.wit",
    interface = "guest-api"
)]
pub enum GuestCommands {}

// The macro expands this to:
// pub enum GuestCommands {
//     Init,
//     HandleCommand { command: String, payload: String },
// }

// With automatic trait implementations:
impl WitCommand for GuestCommands {
    // Generated implementation
}
```

## Benefits

✅ **Zero boilerplate** - Macros generate everything from WIT
✅ **Type safety** - Rust compiler checks everything
✅ **IDE support** - Full autocomplete and navigation
✅ **WIT as source of truth** - Changes propagate automatically
✅ **Composable** - Multiple interfaces combine seamlessly
✅ **No serialization** - Direct type mappings

## Comparison with Approach B

| Feature | Approach A (This) | Approach B (Trait-Based) |
|---------|-------------------|--------------------------|
| Code Generation | Automatic | Manual |
| Boilerplate | None | Some |
| Flexibility | Very High | High |
| Setup Complexity | Medium | Low |
| WIT Integration | Native | Separate |
| Composition | Enum-based | Trait-based |

## Current Status

⚠️ **This is a design document for future implementation.**

Approach A requires creating a procedural macro crate which is a larger undertaking. The design is documented here for reference. See Approach B for a working implementation that achieves similar goals with a different trade-off.

To implement Approach A, we would need to:
1. Create `packages/tairitsu-macros` crate
2. Implement WIT parser (or use wit-parser crate)
3. Generate enum definitions from parsed WIT
4. Generate trait implementations
5. Implement composition macros

## Next Steps

If you want to implement Approach A:
1. Review the design in this document
2. Create the `tairitsu-macros` crate structure
3. Implement the proc-macro in phases as outlined above
4. Add integration tests
5. Update examples to use the new macros

For now, **use Approach B** which provides similar functionality with manual enum definitions and trait-based composition.
