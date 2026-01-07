//! Wasmtime Val to RON serialization
//!
//! This module provides conversion from Wasmtime Component Model `Val` types
//! to RON (Rust Object Notation), with full support for nested complex types.

use anyhow::{bail, Result};
use wasmtime::component::Val;

/// Convert Wasmtime Val to RON string
///
/// # Supported Types
///
/// ## Basic Types
/// - Bool, Integers (U8-S64), Floats (F32, F64), Char, String
///
/// ## Complex Types (with full nesting support)
/// - List<T> including List<List<T>>
/// - Tuple<T1, T2, ...> including nested tuples
/// - Record { fields } including nested records
/// - Variant cases with optional payloads
/// - Result<T, E>
/// - Option<T>
///
/// # Examples
///
/// ```ignore
/// use wasmtime::component::Val;
///
/// // Basic type
/// let ron = val_to_ron(&Val::U32(42))?;
/// assert_eq!(ron, "42");
///
/// // Nested list
/// let val = Val::List(vec![
///     Val::List(vec![Val::U32(1), Val::U32(2)]),
///     Val::List(vec![Val::U32(3), Val::U32(4)]),
/// ]);
/// let ron = val_to_ron(&val)?;
/// assert_eq!(ron, "[[1, 2], [3, 4]]");
/// ```
pub fn val_to_ron(val: &Val) -> Result<String> {
    match val {
        // Delegate to type-specific handlers
        Val::Bool(b) => basic::serialize_bool(*b),
        Val::U8(n) => basic::serialize_u8(*n),
        Val::U16(n) => basic::serialize_u16(*n),
        Val::U32(n) => basic::serialize_u32(*n),
        Val::U64(n) => basic::serialize_u64(*n),
        Val::S8(n) => basic::serialize_s8(*n),
        Val::S16(n) => basic::serialize_s16(*n),
        Val::S32(n) => basic::serialize_s32(*n),
        Val::S64(n) => basic::serialize_s64(*n),
        Val::Float32(f) => basic::serialize_f32(*f),
        Val::Float64(f) => basic::serialize_f64(*f),
        Val::Char(c) => basic::serialize_char(*c),
        Val::String(s) => basic::serialize_string(s.as_str()),

        // Complex types - delegate to complex module
        Val::List(items) => complex::serialize_list(items, val_to_ron),
        Val::Tuple(items) => complex::serialize_tuple(items, val_to_ron),
        Val::Record(fields) => complex::serialize_record(fields, val_to_ron),
        Val::Variant(case_name, val) => complex::serialize_variant(case_name, val, val_to_ron),
        Val::Result(r) => complex::serialize_result(r, val_to_ron),
        Val::Option(o) => complex::serialize_option(o, val_to_ron),

        _ => bail!("Unsupported Val type for RON conversion: {:?}", val),
    }
}

// Basic type serializers
mod basic {
    use super::*;

    pub fn serialize_bool(b: bool) -> Result<String> {
        Ok(format!("{}", b))
    }

    pub fn serialize_u8(n: u8) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_u16(n: u16) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_u32(n: u32) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_u64(n: u64) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_s8(n: i8) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_s16(n: i16) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_s32(n: i32) -> Result<String> {
        Ok(format!("{}", n))
    }

    pub fn serialize_s64(n: i64) -> Result<String> {
        Ok(format!("{}", n))
    }

    /// Serialize Float32 using scientific notation to avoid precision loss
    pub fn serialize_f32(f: f32) -> Result<String> {
        Ok(format!("{:e}", f))
    }

    /// Serialize Float64 using scientific notation to avoid precision loss
    pub fn serialize_f64(f: f64) -> Result<String> {
        Ok(format!("{:e}", f))
    }

    pub fn serialize_char(c: char) -> Result<String> {
        Ok(format!("'{}'", c.escape_default()))
    }

    pub fn serialize_string(s: &str) -> Result<String> {
        Ok(format!("{:?}", s))
    }
}

// Complex type serializers
mod complex {
    use super::*;

    /// Type alias for recursive serializer function
    pub type Serializer = fn(&Val) -> Result<String>;

    /// Serialize List<T> including nested lists
    ///
    /// # Examples
    /// - [1, 2, 3] -> "[1, 2, 3]"
    /// - [[1, 2], [3, 4]] -> "[[1, 2], [3, 4]]"
    pub fn serialize_list(items: &[Val], serialize: Serializer) -> Result<String> {
        let items: Result<Vec<_>> = items.iter().map(serialize).collect();
        Ok(format!("[{}]", items?.join(", ")))
    }

    /// Serialize Tuple<T1, T2, ...> including nested tuples
    ///
    /// # Examples
    /// - (1, "hello") -> "(1, \"hello\")"
    /// - ("name", [1, 2, 3]) -> "(\"name\", [1, 2, 3])"
    pub fn serialize_tuple(items: &[Val], serialize: Serializer) -> Result<String> {
        let items: Result<Vec<_>> = items.iter().map(serialize).collect();
        Ok(format!("({})", items?.join(", ")))
    }

    /// Serialize Record { field: Type, ... }
    ///
    /// # Examples
    /// - {x: 1, y: 2} -> "{x: 1, y: 2}"
    /// - {data: [1, 2, 3]} -> "{data: [1, 2, 3]}"
    pub fn serialize_record(fields: &[(String, Val)], serialize: Serializer) -> Result<String> {
        let fields: Result<Vec<_>> = fields
            .iter()
            .map(|(k, v)| Ok(format!("{}: {}", k, serialize(v)?)))
            .collect();
        Ok(format!("{{{}}}", fields?.join(", ")))
    }

    /// Serialize Variant with optional payload
    ///
    /// # Examples
    /// - Some(42) -> "Some(42)"
    /// - None -> "None" (unit variant)
    pub fn serialize_variant(
        case_name: &str,
        val: &Option<Box<Val>>,
        serialize: Serializer,
    ) -> Result<String> {
        match val {
            Some(v) => Ok(format!("{}({})", case_name, serialize(v)?)),
            None => Ok(case_name.to_string()),
        }
    }

    /// Serialize Result<T, E>
    ///
    /// # Examples
    /// - Ok(42) -> "Ok(42)"
    /// - Err("error") -> "Err(\"error\")"
    /// - Ok(()) -> "Ok(())" (unit type)
    pub fn serialize_result(
        r: &Result<Option<Box<Val>>, Option<Box<Val>>>,
        serialize: Serializer,
    ) -> Result<String> {
        match r {
            Ok(Some(v)) => Ok(format!("Ok({})", serialize(v)?)),
            Err(Some(e)) => Ok(format!("Err({})", serialize(e)?)),
            Ok(None) => Ok("Ok(())".to_string()),
            Err(None) => Ok("Err(())".to_string()),
        }
    }

    /// Serialize Option<T>
    ///
    /// # Examples
    /// - Some(42) -> "Some(42)"
    /// - None -> "None"
    pub fn serialize_option(o: &Option<Box<Val>>, serialize: Serializer) -> Result<String> {
        match o {
            Some(v) => Ok(format!("Some({})", serialize(v)?)),
            None => Ok("None".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic_types() {
        assert_eq!(val_to_ron(&Val::Bool(true)).unwrap(), "true");
        assert_eq!(val_to_ron(&Val::U32(42)).unwrap(), "42");
        assert_eq!(
            val_to_ron(&Val::String("hello".to_string())).unwrap(),
            "\"hello\""
        );
    }

    #[test]
    fn test_serialize_float_types() {
        let ron = val_to_ron(&Val::Float32(std::f32::consts::PI)).unwrap();
        assert!(ron.contains("e")); // Scientific notation

        let ron = val_to_ron(&Val::Float64(std::f64::consts::E)).unwrap();
        assert!(ron.contains("e"));
    }

    #[test]
    fn test_serialize_nested_list() {
        // Create [[1, 2], [3, 4, 5]]
        let val = Val::List(vec![
            Val::List(vec![Val::U32(1), Val::U32(2)]),
            Val::List(vec![Val::U32(3), Val::U32(4), Val::U32(5)]),
        ]);

        let ron = val_to_ron(&val).unwrap();
        assert_eq!(ron, "[[1, 2], [3, 4, 5]]");
    }

    #[test]
    fn test_serialize_tuple_with_list() {
        // Create ("numbers", [10, 20, 30])
        let val = Val::Tuple(vec![
            Val::String("numbers".to_string()),
            Val::List(vec![Val::U32(10), Val::U32(20), Val::U32(30)]),
        ]);

        let ron = val_to_ron(&val).unwrap();
        assert_eq!(ron, "(\"numbers\", [10, 20, 30])");
    }

    #[test]
    fn test_serialize_option_list() {
        // Create Some([100, 200, 300])
        let list = Val::List(vec![Val::U32(100), Val::U32(200), Val::U32(300)]);
        let option = Val::Option(Some(Box::new(list)));

        let ron = val_to_ron(&option).unwrap();
        assert_eq!(ron, "Some([100, 200, 300])");

        // Create None
        let none = Val::Option(None);
        let ron = val_to_ron(&none).unwrap();
        assert_eq!(ron, "None");
    }

    #[test]
    fn test_serialize_result() {
        // Create Ok(200)
        let ok = Val::Result(Ok(Some(Box::new(Val::U32(200)))));
        let ron = val_to_ron(&ok).unwrap();
        assert_eq!(ron, "Ok(200)");

        // Create Err("error")
        let err = Val::Result(Err(Some(Box::new(Val::String("error".to_string())))));
        let ron = val_to_ron(&err).unwrap();
        assert_eq!(ron, "Err(\"error\")");
    }
}
