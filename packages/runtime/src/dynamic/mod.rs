//! Dynamic value conversion layer for WASM Component Model
//!
//! This module provides conversion between Wasmtime Component Model `Val` types
//! and RON (Rust Object Notation) for dynamic function invocation.
//!
//! Note: This is a simplified implementation that supports basic types.
//! Full support for all Component Model types (lists, records, variants, etc.)
//! will be added in future updates.

pub mod host_imports;

use anyhow::{Context, Result, bail};

use wasmtime::component::Val;

/// Convert Wasmtime Val to RON string
///
/// This is a simplified implementation supporting basic types.
pub fn val_to_ron(val: &Val) -> Result<String> {
    match val {
        // Basic types - direct serialization
        Val::Bool(b) => Ok(format!("{}", b)),
        Val::U8(n) => Ok(format!("{}", n)),
        Val::U16(n) => Ok(format!("{}", n)),
        Val::U32(n) => Ok(format!("{}", n)),
        Val::U64(n) => Ok(format!("{}", n)),
        Val::S8(n) => Ok(format!("{}", n)),
        Val::S16(n) => Ok(format!("{}", n)),
        Val::S32(n) => Ok(format!("{}", n)),
        Val::S64(n) => Ok(format!("{}", n)),
        Val::Char(c) => Ok(format!("'{}'", c.escape_default())),
        Val::String(s) => Ok(format!("{:?}", s)),

        // TODO: Add support for complex types (List, Tuple, Record, Variant, Result, Option)
        // For now, return an error for unsupported types
        _ => bail!("Unsupported Val type for RON conversion: {:?}", val),
    }
}

/// Convert RON string to Wasmtime Val (requires type information)
///
/// This is a simplified implementation supporting basic types.
pub fn ron_to_val(ron: &str, target_type: &wasmtime::component::Type) -> Result<Val> {
    use ron::Value as RonValue;

    // Parse RON to generic value first
    let ron_value: RonValue = ron::from_str(ron)
        .context("Failed to parse RON")?;

    match (ron_value, target_type) {
        // Basic types
        (RonValue::Bool(b), wasmtime::component::Type::Bool) => Ok(Val::Bool(b)),
        (RonValue::Number(n), wasmtime::component::Type::U8) => {
            Ok(Val::U8(n.as_i64().context("U8 expected")? as u8))
        }
        (RonValue::Number(n), wasmtime::component::Type::U16) => {
            Ok(Val::U16(n.as_i64().context("U16 expected")? as u16))
        }
        (RonValue::Number(n), wasmtime::component::Type::U32) => {
            Ok(Val::U32(n.as_i64().context("U32 expected")? as u32))
        }
        (RonValue::Number(n), wasmtime::component::Type::U64) => {
            Ok(Val::U64(n.as_i64().context("U64 expected")? as u64))
        }
        (RonValue::Number(n), wasmtime::component::Type::S8) => {
            Ok(Val::S8(n.as_i64().context("S8 expected")? as i8))
        }
        (RonValue::Number(n), wasmtime::component::Type::S16) => {
            Ok(Val::S16(n.as_i64().context("S16 expected")? as i16))
        }
        (RonValue::Number(n), wasmtime::component::Type::S32) => {
            Ok(Val::S32(n.as_i64().context("S32 expected")? as i32))
        }
        (RonValue::Number(n), wasmtime::component::Type::S64) => {
            Ok(Val::S64(n.as_i64().context("S64 expected")?))
        }
        (RonValue::String(s), wasmtime::component::Type::String) => Ok(Val::String(s)),
        (RonValue::Char(c), wasmtime::component::Type::Char) => Ok(Val::Char(c)),

        // TODO: Add support for complex types (List, Tuple, Record, Variant, Result, Option)
        _ => bail!("Type mismatch or unsupported type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_val_to_ron_basic_types() {
        assert_eq!(val_to_ron(&Val::Bool(true)).unwrap(), "true");
        assert_eq!(val_to_ron(&Val::U32(42)).unwrap(), "42");
        assert_eq!(val_to_ron(&Val::String("hello".to_string())).unwrap(), "\"hello\"");
    }

    #[test]
    fn test_ron_to_val_basic_types() {
        use wasmtime::component::Type;
        assert!(matches!(
            ron_to_val("true", &Type::Bool).unwrap(),
            Val::Bool(true)
        ));
        assert!(matches!(
            ron_to_val("42", &Type::U32).unwrap(),
            Val::U32(42)
        ));
    }
}
