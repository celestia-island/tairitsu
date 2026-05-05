//! CSS value types and utilities
//!
//! This module provides type-safe representations of CSS values,
//! enabling compile-time validation and better IDE support for CSS properties.

mod error;
mod length;
mod macros;
#[cfg(feature = "parse")]
mod parser;
mod types;

pub use error::{CssValueParseError, ParseResult};
pub use length::{CssBinOp, CssExpression, CssLength, LengthUnit};
pub use types::{
    AlignItemsValue, CssValue, CursorValue, DisplayValue, FlexDirectionValue, FlexWrapValue,
    JustifyContentValue, OverflowValue, PositionValue, TextAlignValue,
};

// Macros are exported at root via #[macro_export]
