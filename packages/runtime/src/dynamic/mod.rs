//! Dynamic value conversion layer for WASM Component Model
//!
//! This module provides conversion between Wasmtime Component Model `Val` types
//! and RON (Rust Object Notation) for dynamic function invocation.
//!
//! # Supported Types
//!
//! ## Basic Types
//! - Bool
//! - Integers: U8, U16, U32, U64, S8, S16, S32, S64
//! - Floats: Float32, Float64
//! - Char, String
//!
//! ## Complex Types
//! - List (homogeneous arrays)
//! - Tuple (fixed-length heterogeneous arrays)
//! - Record (named fields with types)
//! - Variant (enum-like types with optional payloads)
//! - Result (Ok/Err with optional payloads)
//! - Option (Some/None)
//!
//! # Architecture
//!
//! This module is organized into submodules for better maintainability:
//! - [`serialize`] - Val → RON conversion
//! - [`deserialize`] - RON → Val conversion with full nesting support
//! - [`host_imports`] - Host import function registry

pub mod host_imports;

// Re-export modular serialization/deserialization
pub mod deserialize;
pub mod serialize;

// Public API
pub use deserialize::{ron_to_val, ron_value_to_val};
pub use serialize::val_to_ron;

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::component::Val;

    #[test]
    fn test_val_to_ron_basic_types() {
        assert_eq!(val_to_ron(&Val::Bool(true)).unwrap(), "true");
        assert_eq!(val_to_ron(&Val::U32(42)).unwrap(), "42");
        assert_eq!(
            val_to_ron(&Val::String("hello".to_string())).unwrap(),
            "\"hello\""
        );
    }

    #[test]
    fn test_val_to_ron_float_types() {
        let ron = val_to_ron(&Val::Float32(std::f32::consts::FRAC_PI_4)).unwrap();
        assert!(ron.contains("e")); // Scientific notation
        let ron = val_to_ron(&Val::Float64(std::f64::consts::LN_2)).unwrap();
        assert!(ron.contains("e"));
    }

    #[test]
    fn test_val_to_ron_option() {
        assert_eq!(val_to_ron(&Val::Option(None)).unwrap(), "None");
        assert_eq!(
            val_to_ron(&Val::Option(Some(Box::new(Val::U32(42))))).unwrap(),
            "Some(42)"
        );
    }

    #[test]
    fn test_val_to_ron_result() {
        assert_eq!(val_to_ron(&Val::Result(Ok(None))).unwrap(), "Ok(())");
        assert_eq!(val_to_ron(&Val::Result(Err(None))).unwrap(), "Err(())");
        assert_eq!(
            val_to_ron(&Val::Result(Ok(Some(Box::new(Val::U32(42)))))).unwrap(),
            "Ok(42)"
        );
    }

    #[test]
    fn test_val_to_ron_tuple() {
        let tuple = Val::Tuple(vec![Val::U32(1), Val::U32(2)]);
        assert_eq!(val_to_ron(&tuple).unwrap(), "(1, 2)");
    }

    #[test]
    fn test_val_to_ron_list() {
        let list = Val::List(vec![Val::U32(1), Val::U32(2), Val::U32(3)]);
        assert_eq!(val_to_ron(&list).unwrap(), "[1, 2, 3]");
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

    #[test]
    fn test_ron_to_val_float_types() {
        use wasmtime::component::Type;
        let result = ron_to_val("3.14", &Type::Float32).unwrap();
        assert!(matches!(result, Val::Float32(_)));

        let result = ron_to_val("2.718", &Type::Float64).unwrap();
        assert!(matches!(result, Val::Float64(_)));
    }

    // Note: Tests for complex types (list, tuple, option, result with nesting)
    // are in integration_test.rs where we can extract types from actual
    // WASM component functions. The wasmtime 40 API doesn't provide
    // public constructors for List, Tuple, OptionType, ResultType.
}
