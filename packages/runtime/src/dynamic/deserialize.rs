//! RON to Wasmtime Val deserialization
//!
//! This module provides efficient conversion from RON (Rust Object Notation)
//! to Wasmtime Component Model `Val` types, with full support for nested
//! complex types.

use anyhow::{anyhow, bail, Context, Result};
use ron::Value as RonValue;
use wasmtime::component::{Type, Val};

/// Convert RON string to Wasmtime Val (requires type information)
///
/// # Supported Types
///
/// ## Basic Types
/// - Bool, Integers (U8-S64), Floats (F32, F64), Char, String
///
/// ## Complex Types (with full nesting support)
/// - List<T> including List<List<T>>
/// - Tuple<T1, T2, ...> including Tuple with nested types
/// - Record { fields } including nested fields
/// - Variant cases with optional payloads
/// - Result<T, E>
/// - Option<T>
///
/// # Examples
///
/// ```ignore
/// use wasmtime::component::{Type, List};
///
/// // Basic type
/// let val = ron_to_val("42", &Type::U32)?;
///
/// // Nested list: list<list<u32>>
/// let list_type = Type::List(List::new(Type::U32));
/// let nested_type = Type::List(List::new(list_type));
/// let val = ron_to_val("[[1, 2], [3, 4]]", &nested_type)?;
/// ```
pub fn ron_to_val(ron: &str, target_type: &Type) -> Result<Val> {
    // Parse RON to generic value first
    let ron_value: RonValue = ron::from_str(ron).context("Failed to parse RON")?;

    ron_value_to_val(ron_value, target_type)
}

/// Convert RON Value to Wasmtime Val (direct, without string conversion)
///
/// This is the core deserialization function that handles all types,
/// including nested complex types.
pub fn ron_value_to_val(ron_value: RonValue, target_type: &Type) -> Result<Val> {
    match (ron_value, target_type) {
        // Delegate to type-specific handlers
        (ron, Type::Bool) => basic::deserialize_bool(ron),
        (ron, Type::U8) => basic::deserialize_u8(ron),
        (ron, Type::U16) => basic::deserialize_u16(ron),
        (ron, Type::U32) => basic::deserialize_u32(ron),
        (ron, Type::U64) => basic::deserialize_u64(ron),
        (ron, Type::S8) => basic::deserialize_s8(ron),
        (ron, Type::S16) => basic::deserialize_s16(ron),
        (ron, Type::S32) => basic::deserialize_s32(ron),
        (ron, Type::S64) => basic::deserialize_s64(ron),
        (ron, Type::Float32) => basic::deserialize_f32(ron),
        (ron, Type::Float64) => basic::deserialize_f64(ron),
        (ron, Type::Char) => basic::deserialize_char(ron),
        (ron, Type::String) => basic::deserialize_string(ron),

        // Complex types - delegate to complex module
        (ron, Type::List(_)) => complex::deserialize_list(ron, target_type, ron_value_to_val),
        (ron, Type::Tuple(_)) => complex::deserialize_tuple(ron, target_type, ron_value_to_val),
        (ron, Type::Record(_)) => complex::deserialize_record(ron, target_type, ron_value_to_val),
        (ron, Type::Variant(_)) => complex::deserialize_variant(ron, target_type, ron_value_to_val),
        (ron, Type::Result(_)) => complex::deserialize_result(ron, target_type, ron_value_to_val),
        (ron, Type::Option(_)) => complex::deserialize_option(ron, target_type, ron_value_to_val),

        // Fallback - capture types before moving
        (ron, ty) => bail!(
            "Type mismatch or unsupported: ron_value={:?}, target_type={:?}",
            ron,
            ty
        ),
    }
}

// Basic type handlers
mod basic {
    use super::*;
    use ron::Value as RonValue;

    pub fn deserialize_bool(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Bool(b) => Ok(Val::Bool(b)),
            _ => bail!("Expected bool, got {:?}", ron),
        }
    }

    pub fn deserialize_u8(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::U8(n.as_i64().context("U8 expected")? as u8)),
            _ => bail!("Expected number for u8, got {:?}", ron),
        }
    }

    pub fn deserialize_u16(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::U16(n.as_i64().context("U16 expected")? as u16)),
            _ => bail!("Expected number for u16, got {:?}", ron),
        }
    }

    pub fn deserialize_u32(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::U32(n.as_i64().context("U32 expected")? as u32)),
            _ => bail!("Expected number for u32, got {:?}", ron),
        }
    }

    pub fn deserialize_u64(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::U64(n.as_i64().context("U64 expected")? as u64)),
            _ => bail!("Expected number for u64, got {:?}", ron),
        }
    }

    pub fn deserialize_s8(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::S8(n.as_i64().context("S8 expected")? as i8)),
            _ => bail!("Expected number for s8, got {:?}", ron),
        }
    }

    pub fn deserialize_s16(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::S16(n.as_i64().context("S16 expected")? as i16)),
            _ => bail!("Expected number for s16, got {:?}", ron),
        }
    }

    pub fn deserialize_s32(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::S32(n.as_i64().context("S32 expected")? as i32)),
            _ => bail!("Expected number for s32, got {:?}", ron),
        }
    }

    pub fn deserialize_s64(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::S64(n.as_i64().context("S64 expected")?)),
            _ => bail!("Expected number for s64, got {:?}", ron),
        }
    }

    pub fn deserialize_f32(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::Float32(n.as_f64().context("Float32 expected")? as f32)),
            _ => bail!("Expected number for f32, got {:?}", ron),
        }
    }

    pub fn deserialize_f64(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Number(n) => Ok(Val::Float64(n.as_f64().context("Float64 expected")?)),
            _ => bail!("Expected number for f64, got {:?}", ron),
        }
    }

    pub fn deserialize_char(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::Char(c) => Ok(Val::Char(c)),
            _ => bail!("Expected char, got {:?}", ron),
        }
    }

    pub fn deserialize_string(ron: RonValue) -> Result<Val> {
        match ron {
            RonValue::String(s) => Ok(Val::String(s)),
            _ => bail!("Expected string, got {:?}", ron),
        }
    }
}

// Complex type handlers
mod complex {
    use super::*;
    use ron::Value as RonValue;

    /// Type alias for recursive deserializer function
    pub type Deserializer = fn(RonValue, &Type) -> Result<Val>;

    /// Deserialize List<T> including nested lists
    ///
    /// # Examples
    /// - `[1, 2, 3]` -> List<U32>
    /// - `[[1, 2], [3, 4]]` -> List<List<U32>>
    pub fn deserialize_list(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (seq, list_type) = match ron {
            RonValue::Seq(seq) => match target_type {
                Type::List(list_type) => (seq, list_type),
                _ => bail!("Expected list type, got {:?}", target_type),
            },
            _ => bail!("Expected list sequence, got {:?}", ron),
        };

        let elem_type = list_type.ty();
        let vals: Result<Vec<_>> = seq
            .into_iter()
            .map(|v| deserialize(v, &elem_type))
            .collect();

        Ok(Val::List(vals?))
    }

    /// Deserialize Tuple<T1, T2, ...> including nested tuples
    ///
    /// # Examples
    /// - `(1, "hello")` -> Tuple<U32, String>
    /// - `("name", [1, 2, 3])` -> Tuple<String, List<U32>>
    pub fn deserialize_tuple(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (seq, tuple_type) = match ron {
            RonValue::Seq(seq) => match target_type {
                Type::Tuple(tuple_type) => (seq, tuple_type),
                _ => bail!("Expected tuple type, got {:?}", target_type),
            },
            _ => bail!("Expected tuple sequence, got {:?}", ron),
        };

        let elem_types: Vec<_> = tuple_type.types().collect();
        if seq.len() != elem_types.len() {
            bail!(
                "Tuple length mismatch: expected {}, got {}",
                elem_types.len(),
                seq.len()
            );
        }

        let vals: Result<Vec<_>> = seq
            .into_iter()
            .zip(elem_types.iter())
            .map(|(v, t)| deserialize(v, t))
            .collect();

        Ok(Val::Tuple(vals?))
    }

    /// Deserialize Record { field: Type, ... }
    ///
    /// # Examples
    /// - `{x: 1, y: 2}` -> Record { x: U32, y: U32 }
    /// - `{data: [1, 2, 3]}` -> Record { data: List<U32> }
    pub fn deserialize_record(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (map, record_type) = match ron {
            RonValue::Map(map) => match target_type {
                Type::Record(record_type) => (map, record_type),
                _ => bail!("Expected record type, got {:?}", target_type),
            },
            _ => bail!("Expected record map, got {:?}", ron),
        };

        let mut field_vals = Vec::new();
        for field in record_type.fields() {
            let field_name = field.name;
            let field_type = field.ty;

            // Find field value in map (RON stores keys as Strings)
            let val = map
                .iter()
                .find(|(k, _)| *k == &RonValue::String(field_name.to_string()))
                .map(|(_, v)| v)
                .ok_or_else(|| anyhow!("Missing field: {}", field_name))?;

            field_vals.push((
                field_name.to_string(),
                deserialize(val.clone(), &field_type)?,
            ));
        }

        Ok(Val::Record(field_vals))
    }

    /// Deserialize Variant with optional payload
    ///
    /// # Examples
    /// - `Some(42)` -> Variant::Some with U32
    /// - `None` -> Variant::None (unit)
    pub fn deserialize_variant(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (mut map, variant_type) = match ron {
            RonValue::Map(map) => match target_type {
                Type::Variant(variant_type) => (map, variant_type),
                _ => bail!("Expected variant type, got {:?}", target_type),
            },
            _ => bail!("Expected variant map, got {:?}", ron),
        };

        // Try each case
        for case in variant_type.cases() {
            let case_name = case.name;
            let key = RonValue::String(case_name.to_string());

            if let Some(val_ron) = map.remove(&key) {
                // Case has optional type
                let case_val = match case.ty {
                    Some(t) => Some(Box::new(deserialize(val_ron, &t)?)),
                    None => None, // Unit variant
                };

                return Ok(Val::Variant(case_name.to_string(), case_val));
            }
        }

        bail!("No matching variant case found in: {:?}", map)
    }

    /// Deserialize Result<T, E>
    ///
    /// # Examples
    /// - `Ok(42)` -> Result::Ok<U32, _>
    /// - `Err("error")` -> Result::Err<_, String>
    pub fn deserialize_result(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (mut map, result_type) = match ron {
            RonValue::Map(map) => match target_type {
                Type::Result(result_type) => (map, result_type),
                _ => bail!("Expected result type, got {:?}", target_type),
            },
            _ => bail!("Expected result map, got {:?}", ron),
        };

        // Try "Ok" first
        let ok_key = RonValue::String("Ok".to_string());
        if let Some(ok_ron) = map.remove(&ok_key) {
            let val = match result_type.ok() {
                Some(t) => Some(Box::new(deserialize(ok_ron, &t)?)),
                None => None, // Unit type
            };
            return Ok(Val::Result(Ok(val)));
        }

        // Try "Err"
        let err_key = RonValue::String("Err".to_string());
        if let Some(err_ron) = map.remove(&err_key) {
            let val = match result_type.err() {
                Some(t) => Some(Box::new(deserialize(err_ron, &t)?)),
                None => None, // Unit type
            };
            return Ok(Val::Result(Err(val)));
        }

        bail!("Invalid Result type: missing 'Ok' or 'Err' field")
    }

    /// Deserialize Option<T>
    ///
    /// # Examples
    /// - `Some(42)` -> Option::Some<U32>
    /// - `None` -> Option::None
    pub fn deserialize_option(
        ron: RonValue,
        target_type: &Type,
        deserialize: Deserializer,
    ) -> Result<Val> {
        let (option_value, option_type) = match ron {
            RonValue::Option(v) => match target_type {
                Type::Option(option_type) => (v, option_type),
                _ => bail!("Expected option type, got {:?}", target_type),
            },
            _ => bail!("Expected option, got {:?}", ron),
        };

        match option_value {
            Some(v) => {
                let inner_type = option_type.ty();
                Ok(Val::Option(Some(Box::new(deserialize(*v, &inner_type)?))))
            }
            None => Ok(Val::Option(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_basic_types() {
        // Bool
        let val = ron_to_val("true", &Type::Bool).unwrap();
        assert!(matches!(val, Val::Bool(true)));

        // U32
        let val = ron_to_val("42", &Type::U32).unwrap();
        assert!(matches!(val, Val::U32(42)));

        // String
        let val = ron_to_val("\"hello\"", &Type::String).unwrap();
        assert!(matches!(val, Val::String(_)));
    }

    // Note: Tests for complex types (list, tuple, option, result) are in
    // integration_test.rs and mod.rs tests, where we can get types from
    // actual WASM component functions instead of manually constructing them.
}
