//! Type-safe CSS value types for Tairitsu framework.
//!
//! This crate provides strongly-typed representations of CSS values,
//! enabling compile-time validation and better IDE support for CSS properties.
//!
//! # Features
//!
//! - **Type-safe length units**: Pixels, percentages, em/rem, vw/vh, etc.
//! - **Calc expressions**: Support for `calc()`, `min()`, `max()`, `clamp()`
//! - **String conversion**: Bidirectional conversion with CSS strings
//! - **Compile-time parsing**: Parse string literals at compile time with `calc!` macro
//!
//! # Example
//!
//! ```rust
//! use tairitsu_css_values::CssLength;
//!
//! // Create length values
//! let width = CssLength::px(100);
//! let height = CssLength::vh(100);
//! let margin = CssLength::percent(50);
//!
//! // Convert to CSS string
//! assert_eq!(width.to_css_string(), "100px");
//! assert_eq!(height.to_css_string(), "100vh");
//! assert_eq!(margin.to_css_string(), "50%");
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod length;
mod macros;

pub use error::{CssValueParseError, ParseResult};
pub use length::{CssBinOp, CssExpression, CssLength, LengthUnit};
// Macros are exported at root via #[macro_export]

#[cfg(feature = "parse")]
pub mod parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_creation() {
        let px = CssLength::px(100);
        assert_eq!(px.to_css_string(), "100px");
    }

    #[test]
    fn test_percent_creation() {
        let pct = CssLength::percent(50);
        assert_eq!(pct.to_css_string(), "50%");
    }

    #[test]
    fn test_em_creation() {
        let em = CssLength::em(1.5);
        assert_eq!(em.to_css_string(), "1.5em");
    }

    #[test]
    fn test_rem_creation() {
        let rem = CssLength::rem(16);
        assert_eq!(rem.to_css_string(), "16rem");
    }

    #[test]
    fn test_vw_creation() {
        let vw = CssLength::vw(100);
        assert_eq!(vw.to_css_string(), "100vw");
    }

    #[test]
    fn test_vh_creation() {
        let vh = CssLength::vh(50);
        assert_eq!(vh.to_css_string(), "50vh");
    }

    #[test]
    fn test_vmin_creation() {
        let vmin = CssLength::vmin(75);
        assert_eq!(vmin.to_css_string(), "75vmin");
    }

    #[test]
    fn test_vmax_creation() {
        let vmax = CssLength::vmax(25);
        assert_eq!(vmax.to_css_string(), "25vmax");
    }

    #[test]
    fn test_auto_keyword() {
        let auto = CssLength::Auto;
        assert_eq!(auto.to_css_string(), "auto");
    }

    #[test]
    fn test_absolute_units() {
        assert_eq!(CssLength::inches(1).to_css_string(), "1in");
        assert_eq!(CssLength::cm(2.5).to_css_string(), "2.5cm");
        assert_eq!(CssLength::mm(100).to_css_string(), "100mm");
        assert_eq!(CssLength::pt(12).to_css_string(), "12pt");
        assert_eq!(CssLength::pc(1).to_css_string(), "1pc");
    }

    #[test]
    fn test_relative_units() {
        assert_eq!(CssLength::ex(1).to_css_string(), "1ex");
        assert_eq!(CssLength::ch(0.5).to_css_string(), "0.5ch");
    }

    #[test]
    fn test_content_keywords() {
        assert_eq!(CssLength::MinContent.to_css_string(), "min-content");
        assert_eq!(CssLength::MaxContent.to_css_string(), "max-content");
        assert_eq!(
            CssLength::FitContent(100.0).to_css_string(),
            "fit-content(100px)"
        );
    }

    #[test]
    fn test_expression_display() {
        let expr = CssExpression::Value(CssLength::px(100));
        assert_eq!(expr.to_string(), "100px");

        let expr = CssExpression::Min(vec![
            CssExpression::Value(CssLength::px(100)),
            CssExpression::Value(CssLength::percent(50)),
        ]);
        assert_eq!(expr.to_string(), "min(100px, 50%)");
    }

    #[test]
    fn test_calc_expression() {
        let calc = CssLength::calc(CssExpression::Binary {
            left: Box::new(CssExpression::Value(CssLength::percent(100))),
            op: CssBinOp::Sub,
            right: Box::new(CssExpression::Value(CssLength::px(40))),
        });
        assert_eq!(calc.to_css_string(), "calc(100% - 40px)");
    }

    #[test]
    fn test_min_function() {
        let min_val = CssLength::min(vec![CssLength::px(100), CssLength::percent(50)]);
        assert_eq!(min_val.to_css_string(), "min(100px, 50%)");
    }

    #[test]
    fn test_max_function() {
        let max_val = CssLength::max(vec![CssLength::vw(100), CssLength::px(1200)]);
        assert_eq!(max_val.to_css_string(), "max(100vw, 1200px)");
    }

    #[test]
    fn test_clamp_function() {
        let clamp_val = CssLength::clamp(
            CssLength::px(300),
            CssLength::percent(50),
            CssLength::px(800),
        );
        assert_eq!(clamp_val.to_css_string(), "clamp(300px, 50%, 800px)");
    }

    #[test]
    fn test_complex_expression() {
        // (100% - 40px) / 2
        let expr = CssExpression::Binary {
            left: Box::new(CssExpression::Binary {
                left: Box::new(CssExpression::Value(CssLength::percent(100))),
                op: CssBinOp::Sub,
                right: Box::new(CssExpression::Value(CssLength::px(40))),
            }),
            op: CssBinOp::Div,
            right: Box::new(CssExpression::Value(CssLength::px(2))),
        };
        // With parentheses around nested binary ops
        assert_eq!(expr.to_string(), "(100% - 40px) / (2px)");
    }
}
