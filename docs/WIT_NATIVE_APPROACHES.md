# WIT Native Command Systems - Comparison

This document compares the two approaches for implementing type-safe WIT command systems without runtime serialization.

## Overview

Both approaches eliminate JSON/serde serialization overhead and provide compile-time type safety, but use different implementation strategies.

## Approach A: Procedural Macro Generation

### Concept
Automatically generate type-safe Rust enums directly from WIT interface definitions using procedural macros.

### Status
**⚠️ Design Document Only** - Not yet implemented. Requires creating a `tairitsu-macros` crate.

### Example Usage
```rust
// Automatically generate from WIT
#[wit_commands(
    wit_file = "wit/world.wit",
    interface = "guest-api"
)]
pub enum GuestCommands {} // Auto-populated by macro

// Expands to:
// pub enum GuestCommands {
//     Init,
//     HandleCommand { command: String, payload: String },
// }
```

### Pros
- ✅ Zero boilerplate - everything generated from WIT
- ✅ WIT is the single source of truth
- ✅ Changes to WIT automatically propagate
- ✅ Ideal for large, evolving interfaces

### Cons
- ❌ Requires proc-macro crate (~500-800 LOC)
- ❌ More complex setup
- ❌ Longer compile times (macro expansion)
- ❌ Requires WIT parser integration

### Implementation Effort
- **Time**: 2-3 days
- **Complexity**: High
- **Dependencies**: `syn`, `quote`, `wit-parser`

## Approach B: Trait-Based Composition

### Concept
Manual enum definitions with trait-based composition for multiple interfaces.

### Status
**✅ Fully Implemented** - Working example in `examples/wit-native-b`

### Example Usage
```rust
// Define command enum
#[derive(Debug, Clone)]
pub enum FileSystemCommands {
    Read { path: String },
    Write { path: String, data: Vec<u8> },
}

// Implement WitCommand trait
impl WitCommand for FileSystemCommands {
    type Response = Result<Vec<u8>, String>;
    
    fn command_name(&self) -> &'static str {
        match self {
            Self::Read { .. } => "fs_read",
            Self::Write { .. } => "fs_write",
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Compose multiple interfaces
let mut composite = CompositeWitInterface::new();
composite.add_interface(Box::new(FileSystemBasicInterface));
composite.add_interface(Box::new(FileSystemAdvancedInterface));
```

### Pros
- ✅ No macro complexity
- ✅ Works today - no additional crates needed
- ✅ Easy to understand and debug
- ✅ Flexible trait-based composition

### Cons
- ❌ Some boilerplate (enum + trait impl)
- ❌ Manual sync with WIT definitions
- ❌ More verbose than Approach A

### Implementation Effort
- **Time**: Already done
- **Complexity**: Medium
- **Dependencies**: None (uses std library)

## Feature Comparison

| Feature | Approach A (Proc-Macro) | Approach B (Trait-Based) |
|---------|------------------------|-------------------------|
| **Serialization** | None | None |
| **Type Safety** | ✅ Compile-time | ✅ Compile-time |
| **Boilerplate** | None | Some |
| **WIT Sync** | Automatic | Manual |
| **Composability** | Enum-based | Trait-based |
| **Setup Cost** | High | Low |
| **Runtime Cost** | Zero | Zero |
| **IDE Support** | ✅ Full | ✅ Full |
| **Learning Curve** | High | Medium |
| **Flexibility** | Very High | High |
| **Status** | Design Only | ✅ Working |

## Composability Comparison

### Approach A - Enum Composition
```rust
#[wit_compose]
pub enum AllCommands {
    #[from(filesystem_basic)]
    FileSystem(FileSystemBasicCommands),
    
    #[from(filesystem_advanced)]
    FileSystemAdvanced(FileSystemAdvancedCommands),
}

// Auto-generates dispatch logic
```

### Approach B - Trait Composition
```rust
let mut composite = CompositeWitInterface::new();
composite.add_interface(Box::new(FileSystemBasicInterface));
composite.add_interface(Box::new(FileSystemAdvancedInterface));

// Dynamic dispatch via trait objects
```

## Performance

Both approaches have **zero runtime serialization overhead**:
- Direct function calls
- Compile-time type checking
- Static dispatch (approach A) or vtable dispatch (approach B)
- No JSON parsing/generation

Performance difference is negligible (~1-2 CPU cycles for vtable lookup in B).

## Recommendation

### Use Approach B if:
- ✅ You need a solution **now**
- ✅ You prefer **simplicity** over automation
- ✅ You have **few interfaces** (< 5)
- ✅ Interfaces **change infrequently**
- ✅ You want to **avoid proc-macros**

### Use Approach A if:
- ✅ You have **many interfaces** (> 5)
- ✅ WIT definitions **change frequently**
- ✅ You want **zero boilerplate**
- ✅ You're willing to **invest** in macro development
- ✅ WIT is your **primary** source of truth

## Migration Path

1. **Start with Approach B** - Get working code quickly
2. **Measure complexity** - Track how many interfaces you have
3. **If complexity grows** - Consider implementing Approach A
4. **Gradual migration** - Both can coexist in same codebase

## Running Examples

### Approach B (Working)
```bash
# Run the demo
cargo run --package tairitsu-example-wit-native-b --bin approach-b-demo

# Expected output:
# ✓ No runtime serialization overhead
# ✓ Compile-time type safety
# ✓ Composable interfaces via traits
```

### Approach A (Design Only)
See `examples/wit-native-a/README.md` for implementation plan.

## Conclusion

**Approach B is implemented and ready to use.** It provides all the key benefits (no serialization, type safety, composability) with a straightforward implementation.

**Approach A is designed but not implemented.** It would be ideal for larger projects with many evolving WIT interfaces, but requires significant upfront investment in macro development.

For most use cases, **start with Approach B** and migrate to A if/when complexity justifies it.
