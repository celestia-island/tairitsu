//! CSS length and expression types.

use std::fmt;

/// A CSS length value with type-safe units.
///
/// # Example
///
/// ```rust
/// use tairitsu_style::CssLength;
///
/// let width = CssLength::px(100);
/// assert_eq!(width.to_css_string(), "100px");
///
/// let height = CssLength::vh(100);
/// assert_eq!(height.to_css_string(), "100vh");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CssLength {
    // === Absolute units ===
    /// Pixels (1px = 1/96th of 1in)
    Px(f64),

    /// Inches (1in = 2.54cm = 96px)
    In(f64),

    /// Centimeters (1cm = 96px/2.54)
    Cm(f64),

    /// Millimeters (1mm = 1/10th of 1cm)
    Mm(f64),

    /// Points (1pt = 1/72th of 1in)
    Pt(f64),

    /// Picas (1pc = 12pt)
    Pc(f64),

    // === Relative units ===
    /// Font size of the element
    Em(f64),

    /// Font size of the root element
    Rem(f64),

    /// X-height of the element's font
    Ex(f64),

    /// Width of the "0" glyph
    Ch(f64),

    // === Viewport units ===
    /// 1% of viewport's width
    Vw(f64),

    /// 1% of viewport's height
    Vh(f64),

    /// 1% of viewport's smaller dimension
    Vmin(f64),

    /// 1% of viewport's larger dimension
    Vmax(f64),

    // === Percentage ===
    /// Percentage value
    Percent(f64),

    // === Functions ===
    /// `calc()` expression
    Calc(Box<CssExpression>),

    /// `min()` function
    Min(Vec<CssLength>),

    /// `max()` function
    Max(Vec<CssLength>),

    /// `clamp()` function
    Clamp {
        /// Minimum value
        min: Box<CssLength>,
        /// Preferred value
        preferred: Box<CssLength>,
        /// Maximum value
        max: Box<CssLength>,
    },

    // === Keywords ===
    /// `auto` keyword
    Auto,

    /// `min-content` keyword
    MinContent,

    /// `max-content` keyword
    MaxContent,

    /// `fit-content()` function
    FitContent(f64),
}

impl CssLength {
    // === Absolute units ===

    /// Create a pixel length.
    #[inline]
    pub fn px(value: impl Into<f64>) -> Self {
        Self::Px(value.into())
    }

    /// Create an inch length.
    #[inline]
    pub fn inches(value: impl Into<f64>) -> Self {
        Self::In(value.into())
    }

    /// Create a centimeter length.
    #[inline]
    pub fn cm(value: impl Into<f64>) -> Self {
        Self::Cm(value.into())
    }

    /// Create a millimeter length.
    #[inline]
    pub fn mm(value: impl Into<f64>) -> Self {
        Self::Mm(value.into())
    }

    /// Create a point length.
    #[inline]
    pub fn pt(value: impl Into<f64>) -> Self {
        Self::Pt(value.into())
    }

    /// Create a pica length.
    #[inline]
    pub fn pc(value: impl Into<f64>) -> Self {
        Self::Pc(value.into())
    }

    // === Relative units ===

    /// Create an em length.
    #[inline]
    pub fn em(value: impl Into<f64>) -> Self {
        Self::Em(value.into())
    }

    /// Create a rem length.
    #[inline]
    pub fn rem(value: impl Into<f64>) -> Self {
        Self::Rem(value.into())
    }

    /// Create an ex length.
    #[inline]
    pub fn ex(value: impl Into<f64>) -> Self {
        Self::Ex(value.into())
    }

    /// Create a ch length.
    #[inline]
    pub fn ch(value: impl Into<f64>) -> Self {
        Self::Ch(value.into())
    }

    // === Viewport units ===

    /// Create a viewport width length.
    #[inline]
    pub fn vw(value: impl Into<f64>) -> Self {
        Self::Vw(value.into())
    }

    /// Create a viewport height length.
    #[inline]
    pub fn vh(value: impl Into<f64>) -> Self {
        Self::Vh(value.into())
    }

    /// Create a viewport minimum length.
    #[inline]
    pub fn vmin(value: impl Into<f64>) -> Self {
        Self::Vmin(value.into())
    }

    /// Create a viewport maximum length.
    #[inline]
    pub fn vmax(value: impl Into<f64>) -> Self {
        Self::Vmax(value.into())
    }

    // === Percentage ===

    /// Create a percentage length.
    #[inline]
    pub fn percent(value: impl Into<f64>) -> Self {
        Self::Percent(value.into())
    }

    // === Functions ===

    /// Create a `calc()` expression.
    #[inline]
    pub fn calc(expr: CssExpression) -> Self {
        Self::Calc(Box::new(expr))
    }

    /// Create a `min()` function.
    #[inline]
    pub fn min(values: Vec<CssLength>) -> Self {
        Self::Min(values)
    }

    /// Create a `max()` function.
    #[inline]
    pub fn max(values: Vec<CssLength>) -> Self {
        Self::Max(values)
    }

    /// Create a `clamp()` function.
    #[inline]
    pub fn clamp(min: CssLength, preferred: CssLength, max: CssLength) -> Self {
        Self::Clamp {
            min: Box::new(min),
            preferred: Box::new(preferred),
            max: Box::new(max),
        }
    }

    /// Check if this length is absolute (can be resolved without context).
    pub fn is_absolute(&self) -> bool {
        matches!(
            self,
            Self::Px(_) | Self::In(_) | Self::Cm(_) | Self::Mm(_) | Self::Pt(_) | Self::Pc(_)
        )
    }

    /// Get the numeric value if this is a simple unit length.
    pub fn as_numeric_value(&self) -> Option<f64> {
        match self {
            Self::Px(v)
            | Self::In(v)
            | Self::Cm(v)
            | Self::Mm(v)
            | Self::Pt(v)
            | Self::Pc(v)
            | Self::Em(v)
            | Self::Rem(v)
            | Self::Ex(v)
            | Self::Ch(v)
            | Self::Vw(v)
            | Self::Vh(v)
            | Self::Vmin(v)
            | Self::Vmax(v)
            | Self::Percent(v)
            | Self::FitContent(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to CSS string representation.
    pub fn to_css_string(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for CssLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Absolute units
            Self::Px(n) => write!(f, "{}px", n),
            Self::In(n) => write!(f, "{}in", n),
            Self::Cm(n) => write!(f, "{}cm", n),
            Self::Mm(n) => write!(f, "{}mm", n),
            Self::Pt(n) => write!(f, "{}pt", n),
            Self::Pc(n) => write!(f, "{}pc", n),

            // Relative units
            Self::Em(n) => write!(f, "{}em", n),
            Self::Rem(n) => write!(f, "{}rem", n),
            Self::Ex(n) => write!(f, "{}ex", n),
            Self::Ch(n) => write!(f, "{}ch", n),

            // Viewport units
            Self::Vw(n) => write!(f, "{}vw", n),
            Self::Vh(n) => write!(f, "{}vh", n),
            Self::Vmin(n) => write!(f, "{}vmin", n),
            Self::Vmax(n) => write!(f, "{}vmax", n),

            // Percentage
            Self::Percent(n) => write!(f, "{}%", n),

            // Functions
            Self::Calc(expr) => write!(f, "calc({})", expr),
            Self::Min(values) => {
                write!(f, "min(")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Self::Max(values) => {
                write!(f, "max(")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Self::Clamp {
                min,
                preferred,
                max,
            } => {
                write!(f, "clamp({}, {}, {})", min, preferred, max)
            }

            // Keywords
            Self::Auto => write!(f, "auto"),
            Self::MinContent => write!(f, "min-content"),
            Self::MaxContent => write!(f, "max-content"),
            Self::FitContent(n) => write!(f, "fit-content({}px)", n),
        }
    }
}

impl From<f64> for CssLength {
    fn from(value: f64) -> Self {
        Self::px(value)
    }
}

impl From<i32> for CssLength {
    fn from(value: i32) -> Self {
        Self::px(value as f64)
    }
}

impl From<LengthUnit> for CssLength {
    fn from(unit: LengthUnit) -> Self {
        match unit {
            LengthUnit::Px => Self::px(1),
            LengthUnit::Percent => Self::percent(1),
            LengthUnit::Em => Self::em(1),
            LengthUnit::Rem => Self::rem(1),
            LengthUnit::Vw => Self::vw(1),
            LengthUnit::Vh => Self::vh(1),
            LengthUnit::Vmin => Self::vmin(1),
            LengthUnit::Vmax => Self::vmax(1),
        }
    }
}

/// Binary operator for CSS calc expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CssBinOp {
    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Division (`/`)
    Div,
}

impl fmt::Display for CssBinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

/// A CSS expression used in calc(), min(), max(), clamp().
///
/// # Example
///
/// ```rust
/// use tairitsu_style::{CssExpression, CssLength, CssBinOp};
///
/// let expr = CssExpression::Binary {
///     left: Box::new(CssExpression::Value(CssLength::percent(100))),
///     op: CssBinOp::Sub,
///     right: Box::new(CssExpression::Value(CssLength::px(40))),
/// };
/// assert_eq!(expr.to_string(), "100% - 40px");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CssExpression {
    /// A simple length value.
    Value(CssLength),

    /// Binary operation: `left op right`.
    Binary {
        /// Left operand.
        left: Box<CssExpression>,
        /// Operator.
        op: CssBinOp,
        /// Right operand.
        right: Box<CssExpression>,
    },

    /// `min()` function with multiple arguments.
    Min(Vec<CssExpression>),

    /// `max()` function with multiple arguments.
    Max(Vec<CssExpression>),

    /// `clamp(min, preferred, max)` function.
    Clamp {
        /// Minimum value.
        min: Box<CssExpression>,
        /// Preferred value.
        preferred: Box<CssExpression>,
        /// Maximum value.
        max: Box<CssExpression>,
    },
}

impl fmt::Display for CssExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(length) => write!(f, "{}", length),
            Self::Binary { left, op, right } => {
                // For better readability, add parentheses around nested binary ops
                let needs_parens = matches!(left.as_ref(), Self::Binary { .. })
                    || matches!(right.as_ref(), Self::Binary { .. });
                if needs_parens {
                    write!(f, "({}) {} ({})", left, op, right)
                } else {
                    write!(f, "{} {} {}", left, op, right)
                }
            }
            Self::Min(args) => {
                write!(f, "min(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::Max(args) => {
                write!(f, "max(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::Clamp {
                min,
                preferred,
                max,
            } => {
                write!(f, "clamp({}, {}, {})", min, preferred, max)
            }
        }
    }
}

impl From<CssLength> for CssExpression {
    fn from(length: CssLength) -> Self {
        Self::Value(length)
    }
}

/// CSS length unit identifier (without value).
///
/// This is useful for APIs that need to specify units without values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LengthUnit {
    /// Pixels
    Px,
    /// Percentage
    Percent,
    /// Em
    Em,
    /// Rem
    Rem,
    /// Viewport width
    Vw,
    /// Viewport height
    Vh,
    /// Viewport minimum
    Vmin,
    /// Viewport maximum
    Vmax,
}

impl LengthUnit {
    /// Get the CSS string representation of this unit.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Px => "px",
            Self::Percent => "%",
            Self::Em => "em",
            Self::Rem => "rem",
            Self::Vw => "vw",
            Self::Vh => "vh",
            Self::Vmin => "vmin",
            Self::Vmax => "vmax",
        }
    }

    /// Parse a unit from its string representation.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "px" => Some(Self::Px),
            "%" => Some(Self::Percent),
            "em" => Some(Self::Em),
            "rem" => Some(Self::Rem),
            "vw" => Some(Self::Vw),
            "vh" => Some(Self::Vh),
            "vmin" => Some(Self::Vmin),
            "vmax" => Some(Self::Vmax),
            _ => None,
        }
    }
}

impl fmt::Display for LengthUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Implement common operator overloads for CssLength

impl std::ops::Add for CssLength {
    type Output = CssLength;

    fn add(self, rhs: Self) -> Self::Output {
        CssLength::calc(CssExpression::Binary {
            left: Box::new(CssExpression::Value(self)),
            op: CssBinOp::Add,
            right: Box::new(CssExpression::Value(rhs)),
        })
    }
}

impl std::ops::Sub for CssLength {
    type Output = CssLength;

    fn sub(self, rhs: Self) -> Self::Output {
        CssLength::calc(CssExpression::Binary {
            left: Box::new(CssExpression::Value(self)),
            op: CssBinOp::Sub,
            right: Box::new(CssExpression::Value(rhs)),
        })
    }
}

impl std::ops::Mul<f64> for CssLength {
    type Output = CssLength;

    fn mul(self, rhs: f64) -> Self::Output {
        CssLength::calc(CssExpression::Binary {
            left: Box::new(CssExpression::Value(self)),
            op: CssBinOp::Mul,
            right: Box::new(CssExpression::Value(CssLength::px(rhs))),
        })
    }
}

impl std::ops::Div<f64> for CssLength {
    type Output = CssLength;

    fn div(self, rhs: f64) -> Self::Output {
        CssLength::calc(CssExpression::Binary {
            left: Box::new(CssExpression::Value(self)),
            op: CssBinOp::Div,
            right: Box::new(CssExpression::Value(CssLength::px(rhs))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_overloads() {
        let a = CssLength::px(100);
        let b = CssLength::px(50);

        let sum = a.clone() + b.clone();
        assert!(matches!(sum, CssLength::Calc(_)));
        assert_eq!(sum.to_css_string(), "calc(100px + 50px)");

        let diff = a - b;
        assert_eq!(diff.to_css_string(), "calc(100px - 50px)");

        let product = CssLength::px(100) * 2.0;
        assert_eq!(product.to_css_string(), "calc(100px * 2px)");

        let quotient = CssLength::px(100) / 2.0;
        assert_eq!(quotient.to_css_string(), "calc(100px / 2px)");
    }

    #[test]
    fn test_length_unit_from_str() {
        assert_eq!(LengthUnit::from_str("px"), Some(LengthUnit::Px));
        assert_eq!(LengthUnit::from_str("rem"), Some(LengthUnit::Rem));
        assert_eq!(LengthUnit::from_str("invalid"), None);
    }

    #[test]
    fn test_is_absolute() {
        assert!(CssLength::px(100).is_absolute());
        assert!(CssLength::cm(10).is_absolute());
        assert!(!CssLength::em(1).is_absolute());
        assert!(!CssLength::percent(50).is_absolute());
        assert!(!CssLength::Auto.is_absolute());
    }

    #[test]
    fn test_as_numeric_value() {
        assert_eq!(CssLength::px(100).as_numeric_value(), Some(100.0));
        assert_eq!(CssLength::em(1.5).as_numeric_value(), Some(1.5));
        assert_eq!(CssLength::Auto.as_numeric_value(), None);
        assert_eq!(
            CssLength::calc(CssExpression::Value(CssLength::px(100))).as_numeric_value(),
            None
        );
    }

    #[test]
    fn test_from_conversions() {
        assert_eq!(CssLength::from(100.0), CssLength::px(100));
        assert_eq!(CssLength::from(42), CssLength::px(42));
        assert_eq!(CssLength::from(LengthUnit::Rem), CssLength::rem(1));
    }
}
