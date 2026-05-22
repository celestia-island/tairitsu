//! CSS value parsing using Pest grammar.

use pest::Parser;
use pest_derive::Parser;

use super::{
    error::{CssValueParseError, ParseResult},
    length::{CssBinOp, CssExpression, CssLength},
};

/// CSS value parser using Pest grammar.
#[derive(Parser)]
#[grammar = "src/values/grammar.pest"]
pub struct CssValueParser;

impl CssLength {
    /// Parse a CSS length from a string.
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_style::CssLength;
    ///
    /// let length = CssLength::from_css_str("100px").unwrap();
    /// assert_eq!(length, CssLength::px(100));
    /// ```
    pub fn from_css_str(s: &str) -> ParseResult<Self> {
        if s.trim().is_empty() {
            return Err(CssValueParseError::EmptyInput);
        }

        let pairs = CssValueParser::parse(Rule::length, s)
            .map_err(|e| CssValueParseError::ParseError(e.to_string()))?;

        for pair in pairs {
            if pair.as_rule() == Rule::length {
                return parse_length(pair);
            }
        }

        Err(CssValueParseError::ParseError(
            "No valid length found".to_string(),
        ))
    }

    /// Parse a CSS length from a string at compile time.
    ///
    /// This function is `const` and can be used in const contexts.
    /// Note: Full parsing support in const contexts is limited in stable Rust.
    /// For now, this always returns an error - use from_css_str for runtime parsing.
    #[inline]
    pub const fn from_css_str_const(_s: &str) -> ParseResult<Self> {
        // Note: const parsing is limited in stable Rust.
        // Use the calc! macro or from_css_str at runtime instead.
        Err(CssValueParseError::ParseErrorWithoutMessage)
    }
}

fn parse_length(pair: pest::iterators::Pair<Rule>) -> ParseResult<CssLength> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::px
        | Rule::inch
        | Rule::cm
        | Rule::mm
        | Rule::pt
        | Rule::pc
        | Rule::em
        | Rule::rem
        | Rule::ex
        | Rule::ch
        | Rule::vw
        | Rule::vh
        | Rule::vmin
        | Rule::vmax
        | Rule::percent => parse_length_from_pair(inner),
        Rule::calc_expr => {
            // The calc_expr pair should contain the expression directly
            // For "calc(" ~ expression ~ ")", the inner pair should be the expression
            let inner_pairs = inner.into_inner();
            let mut found_expr = None;
            for pair in inner_pairs {
                match pair.as_rule() {
                    Rule::expression => {
                        let expr = parse_expression(pair)?;
                        found_expr = Some(expr);
                    }
                    Rule::term => {
                        // Direct term without wrapping expression
                        let expr = parse_term(pair)?;
                        found_expr = Some(expr);
                    }
                    _ => {
                        // Skip other tokens like "calc(", ")", etc.
                    }
                }
            }
            match found_expr {
                Some(expr) => Ok(CssLength::calc(expr)),
                None => Err(CssValueParseError::ParseError(
                    "Empty calc expression".to_string(),
                )),
            }
        }
        Rule::min_expr => {
            let inner_pairs = inner.into_inner();
            let mut values = Vec::new();
            for pair in inner_pairs {
                if let Rule::min_args = pair.as_rule() {
                    for inner_arg in pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            values.push(parse_term_to_length(inner_arg)?);
                        }
                    }
                }
            }
            Ok(CssLength::min(values))
        }
        Rule::max_expr => {
            let inner_pairs = inner.into_inner();
            let mut values = Vec::new();
            for pair in inner_pairs {
                if let Rule::max_args = pair.as_rule() {
                    for inner_arg in pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            values.push(parse_term_to_length(inner_arg)?);
                        }
                    }
                }
            }
            Ok(CssLength::max(values))
        }
        Rule::clamp_expr => {
            let inner_pairs = inner.into_inner();
            let mut args = Vec::new();
            for pair in inner_pairs {
                if let Rule::clamp_args = pair.as_rule() {
                    for inner_arg in pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            args.push(parse_term_to_length(inner_arg)?);
                        }
                    }
                }
            }
            if args.len() == 3 {
                Ok(CssLength::clamp(
                    args.remove(0),
                    args.remove(0),
                    args.remove(0),
                ))
            } else {
                Err(CssValueParseError::ParseError(format!(
                    "clamp requires 3 arguments, got {}",
                    args.len()
                )))
            }
        }
        Rule::keyword => match inner.as_str() {
            "auto" => Ok(CssLength::Auto),
            "min-content" => Ok(CssLength::MinContent),
            "max-content" => Ok(CssLength::MaxContent),
            s if s.starts_with("fit-content(") => {
                // Extract the number
                let num_str = s
                    .strip_prefix("fit-content(")
                    .unwrap()
                    .strip_suffix(")")
                    .unwrap()
                    .trim()
                    .trim_end_matches("px");
                let num: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| {
                    CssValueParseError::InvalidNumber(e.to_string())
                })?;
                Ok(CssLength::FitContent(num))
            }
            s => Err(CssValueParseError::InvalidExpression(s.to_string())),
        },
        _ => Err(CssValueParseError::ParseError(format!(
            "Unknown rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<CssExpression> {
    let mut inner = pair.into_inner();
    let mut left = parse_term(
        inner
            .next()
            .ok_or_else(|| CssValueParseError::ParseError("Empty expression".to_string()))?,
    )?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_rule() {
            Rule::add => CssBinOp::Add,
            Rule::sub => CssBinOp::Sub,
            Rule::mul => CssBinOp::Mul,
            Rule::div => CssBinOp::Div,
            _ => break,
        };

        let right =
            parse_term(inner.next().ok_or_else(|| {
                CssValueParseError::ParseError("Missing right operand".to_string())
            })?)?;

        left = CssExpression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_term(pair: pest::iterators::Pair<Rule>) -> ParseResult<CssExpression> {
    let mut inner = pair.into_inner();

    // Get the first inner pair
    let first = inner
        .next()
        .ok_or_else(|| CssValueParseError::ParseError("Empty term".to_string()))?;

    match first.as_rule() {
        Rule::length | Rule::simple_length => {
            // Parse length directly from the pair
            let length = parse_length_from_pair(first)?;
            Ok(CssExpression::Value(length))
        }
        Rule::unitless_number => {
            // Parse unitless number (for multiplication/division)
            let s = first.as_str();
            let num: f64 = s.parse().map_err(|e: std::num::ParseFloatError| {
                CssValueParseError::InvalidNumber(e.to_string())
            })?;
            Ok(CssExpression::Value(CssLength::px(num)))
        }
        Rule::expression => parse_expression(first),
        Rule::unary => {
            let mut pairs_iter = first.into_inner();
            let op_str = pairs_iter
                .next()
                .ok_or_else(|| {
                    CssValueParseError::ParseError("Missing unary operator".to_string())
                })?
                .as_str();
            let term = parse_term(pairs_iter.next().ok_or_else(|| {
                CssValueParseError::ParseError("Missing unary operand".to_string())
            })?)?;

            match op_str {
                "+" => Ok(term),
                "-" => {
                    // Negate the value
                    if let CssExpression::Value(length) = term {
                        if let Some(n) = length.as_numeric_value() {
                            let negated = match length {
                                CssLength::Px(_) => CssLength::px(-n),
                                CssLength::Percent(_) => CssLength::percent(-n),
                                CssLength::Em(_) => CssLength::em(-n),
                                CssLength::Rem(_) => CssLength::rem(-n),
                                CssLength::Vw(_) => CssLength::vw(-n),
                                CssLength::Vh(_) => CssLength::vh(-n),
                                _ => length,
                            };
                            Ok(CssExpression::Value(negated))
                        } else {
                            Ok(CssExpression::Value(length))
                        }
                    } else {
                        Ok(term)
                    }
                }
                _ => Ok(term),
            }
        }
        Rule::min_expr => {
            let mut args = Vec::new();
            for arg_pair in first.into_inner() {
                if let Rule::min_args = arg_pair.as_rule() {
                    for inner_arg in arg_pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            args.push(parse_term(inner_arg)?);
                        }
                    }
                }
            }
            Ok(CssExpression::Min(args))
        }
        Rule::max_expr => {
            let mut args = Vec::new();
            for arg_pair in first.into_inner() {
                if let Rule::max_args = arg_pair.as_rule() {
                    for inner_arg in arg_pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            args.push(parse_term(inner_arg)?);
                        }
                    }
                }
            }
            Ok(CssExpression::Max(args))
        }
        Rule::clamp_expr => {
            let mut args = Vec::new();
            for arg_pair in first.into_inner() {
                if let Rule::clamp_args = arg_pair.as_rule() {
                    for inner_arg in arg_pair.into_inner() {
                        if matches!(inner_arg.as_rule(), Rule::term | Rule::expression) {
                            args.push(parse_term(inner_arg)?);
                        }
                    }
                }
            }
            if args.len() == 3 {
                Ok(CssExpression::Clamp {
                    min: Box::new(args.remove(0)),
                    preferred: Box::new(args.remove(0)),
                    max: Box::new(args.remove(0)),
                })
            } else {
                Err(CssValueParseError::ParseError(format!(
                    "clamp requires 3 arguments, got {}",
                    args.len()
                )))
            }
        }
        Rule::calc_expr => {
            for inner_pair in first.into_inner() {
                if matches!(inner_pair.as_rule(), Rule::expression) {
                    return parse_expression(inner_pair);
                }
            }
            Err(CssValueParseError::ParseError(
                "Empty calc expression".to_string(),
            ))
        }
        _ => Err(CssValueParseError::ParseError(format!(
            "Unexpected term: {:?}",
            first.as_rule()
        ))),
    }
}

/// Parse a length directly from a Pest pair (without going through parse_length which expects Rule::length wrapper)
fn parse_length_from_pair(pair: pest::iterators::Pair<Rule>) -> ParseResult<CssLength> {
    match pair.as_rule() {
        Rule::px => parse_unit_value(pair, "px", CssLength::px),
        Rule::inch => parse_unit_value(pair, "in", CssLength::inches),
        Rule::cm => parse_unit_value(pair, "cm", CssLength::cm),
        Rule::mm => parse_unit_value(pair, "mm", CssLength::mm),
        Rule::pt => parse_unit_value(pair, "pt", CssLength::pt),
        Rule::pc => parse_unit_value(pair, "pc", CssLength::pc),
        Rule::em => parse_unit_value(pair, "em", CssLength::em),
        Rule::rem => parse_unit_value(pair, "rem", CssLength::rem),
        Rule::ex => parse_unit_value(pair, "ex", CssLength::ex),
        Rule::ch => parse_unit_value(pair, "ch", CssLength::ch),
        Rule::vw => parse_unit_value(pair, "vw", CssLength::vw),
        Rule::vh => parse_unit_value(pair, "vh", CssLength::vh),
        Rule::vmin => parse_unit_value(pair, "vmin", CssLength::vmin),
        Rule::vmax => parse_unit_value(pair, "vmax", CssLength::vmax),
        Rule::percent => parse_unit_value(pair, "%", CssLength::percent),
        Rule::simple_length => {
            // For simple_length, we need to check the inner rule
            let inner_pair = pair.into_inner().next().unwrap();
            parse_length_from_pair(inner_pair)
        }
        _ => Err(CssValueParseError::ParseError(format!(
            "Unknown length rule: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Helper function to parse a unit value from a Pest pair
fn parse_unit_value<F>(
    pair: pest::iterators::Pair<Rule>,
    unit: &str,
    constructor: F,
) -> ParseResult<CssLength>
where
    F: FnOnce(f64) -> CssLength,
{
    let s = pair.as_str();
    let num_str = s
        .strip_suffix(unit)
        .ok_or_else(|| CssValueParseError::InvalidUnit(s.to_string()))?;
    let num: f64 = num_str
        .parse()
        .map_err(|e: std::num::ParseFloatError| CssValueParseError::InvalidNumber(e.to_string()))?;
    Ok(constructor(num))
}

/// Parse a term to a CssLength (simplifies conversion from expression to length)
fn parse_term_to_length(pair: pest::iterators::Pair<Rule>) -> ParseResult<CssLength> {
    let expr = parse_term(pair)?;
    match expr {
        CssExpression::Value(length) => Ok(length),
        _ => Ok(CssLength::calc(expr)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pixels() {
        assert_eq!(
            CssLength::from_css_str("100px").unwrap(),
            CssLength::px(100)
        );
        assert_eq!(CssLength::from_css_str("0px").unwrap(), CssLength::px(0));
        assert_eq!(
            CssLength::from_css_str("12.5px").unwrap(),
            CssLength::px(12.5)
        );
        assert_eq!(
            CssLength::from_css_str("-10px").unwrap(),
            CssLength::px(-10)
        );
    }

    #[test]
    fn test_parse_percent() {
        assert_eq!(
            CssLength::from_css_str("100%").unwrap(),
            CssLength::percent(100)
        );
        assert_eq!(
            CssLength::from_css_str("50.5%").unwrap(),
            CssLength::percent(50.5)
        );
    }

    #[test]
    fn test_parse_relative_units() {
        assert_eq!(CssLength::from_css_str("1em").unwrap(), CssLength::em(1));
        assert_eq!(
            CssLength::from_css_str("16rem").unwrap(),
            CssLength::rem(16)
        );
        assert_eq!(CssLength::from_css_str("1ex").unwrap(), CssLength::ex(1));
        assert_eq!(
            CssLength::from_css_str("0.5ch").unwrap(),
            CssLength::ch(0.5)
        );
    }

    #[test]
    fn test_parse_viewport_units() {
        assert_eq!(
            CssLength::from_css_str("100vw").unwrap(),
            CssLength::vw(100)
        );
        assert_eq!(CssLength::from_css_str("50vh").unwrap(), CssLength::vh(50));
        assert_eq!(
            CssLength::from_css_str("75vmin").unwrap(),
            CssLength::vmin(75)
        );
        assert_eq!(
            CssLength::from_css_str("25vmax").unwrap(),
            CssLength::vmax(25)
        );
    }

    #[test]
    fn test_parse_absolute_units() {
        assert_eq!(
            CssLength::from_css_str("1in").unwrap(),
            CssLength::inches(1)
        );
        assert_eq!(
            CssLength::from_css_str("2.5cm").unwrap(),
            CssLength::cm(2.5)
        );
        assert_eq!(
            CssLength::from_css_str("100mm").unwrap(),
            CssLength::mm(100)
        );
        assert_eq!(CssLength::from_css_str("12pt").unwrap(), CssLength::pt(12));
        assert_eq!(CssLength::from_css_str("1pc").unwrap(), CssLength::pc(1));
    }

    #[test]
    fn test_parse_keywords() {
        assert_eq!(CssLength::from_css_str("auto").unwrap(), CssLength::Auto);
        assert_eq!(
            CssLength::from_css_str("min-content").unwrap(),
            CssLength::MinContent
        );
        assert_eq!(
            CssLength::from_css_str("max-content").unwrap(),
            CssLength::MaxContent
        );
    }

    #[test]
    fn test_parse_calc() {
        // Test simple calc expression
        let calc = CssLength::from_css_str("calc(100% - 40px)").unwrap();
        assert!(matches!(calc, CssLength::Calc(_)));
        assert_eq!(calc.to_css_string(), "calc(100% - 40px)");

        // Test calc with addition
        let calc_add = CssLength::from_css_str("calc(100px + 50px)").unwrap();
        assert_eq!(calc_add.to_css_string(), "calc(100px + 50px)");

        // Test calc with multiplication
        let calc_mul = CssLength::from_css_str("calc(2 * 50px)").unwrap();
        assert_eq!(calc_mul.to_css_string(), "calc(2px * 50px)");

        // Test calc with division
        let calc_div = CssLength::from_css_str("calc(100px / 2)").unwrap();
        assert_eq!(calc_div.to_css_string(), "calc(100px / 2px)");

        // Test nested calc with min
        let calc_nested = CssLength::from_css_str("calc(100% - min(20px, 5%))").unwrap();
        assert!(matches!(calc_nested, CssLength::Calc(_)));
    }

    #[test]
    fn test_parse_min() {
        // Test min with two values
        let min_val = CssLength::from_css_str("min(100px, 50%)").unwrap();
        assert!(matches!(min_val, CssLength::Min(_)));
        assert_eq!(min_val.to_css_string(), "min(100px, 50%)");

        // Test min with multiple values
        let min_multi = CssLength::from_css_str("min(100px, 50%, 20vw)").unwrap();
        assert!(matches!(min_multi, CssLength::Min(_)));
        assert_eq!(min_multi.to_css_string(), "min(100px, 50%, 20vw)");
    }

    #[test]
    fn test_parse_max() {
        // Test max with two values
        let max_val = CssLength::from_css_str("max(100vw, 1200px)").unwrap();
        assert!(matches!(max_val, CssLength::Max(_)));
        assert_eq!(max_val.to_css_string(), "max(100vw, 1200px)");

        // Test max with multiple values
        let max_multi = CssLength::from_css_str("max(100px, 50%, 20vh)").unwrap();
        assert!(matches!(max_multi, CssLength::Max(_)));
        assert_eq!(max_multi.to_css_string(), "max(100px, 50%, 20vh)");
    }

    #[test]
    fn test_parse_clamp() {
        // Test clamp with three values
        let clamp_val = CssLength::from_css_str("clamp(300px, 50%, 800px)").unwrap();
        assert!(matches!(clamp_val, CssLength::Clamp { .. }));
        assert_eq!(clamp_val.to_css_string(), "clamp(300px, 50%, 800px)");

        // Test clamp with different units
        let clamp_mix = CssLength::from_css_str("clamp(10rem, 50vw, 100vh)").unwrap();
        assert_eq!(clamp_mix.to_css_string(), "clamp(10rem, 50vw, 100vh)");
    }

    #[test]
    fn test_parse_nested_expressions() {
        // Test calc containing min
        let calc_min = CssLength::from_css_str("calc(100% - min(20px, 5%))").unwrap();
        assert!(matches!(calc_min, CssLength::Calc(_)));
        let css_str = calc_min.to_css_string();
        assert!(css_str.contains("calc(") && css_str.contains("min("));

        // Test calc containing max
        let calc_max = CssLength::from_css_str("calc(100vw - max(200px, 20%))").unwrap();
        assert!(matches!(calc_max, CssLength::Calc(_)));

        // Test min containing calc
        let min_calc = CssLength::from_css_str("min(calc(100% - 20px), 300px)").unwrap();
        assert!(matches!(min_calc, CssLength::Min(_)));

        // Test clamp containing expressions
        let clamp_expr =
            CssLength::from_css_str("clamp(min(100px, 10%), 50%, max(800px, 80%))").unwrap();
        assert!(matches!(clamp_expr, CssLength::Clamp { .. }));
    }

    #[test]
    fn test_parse_unary_operators() {
        // Test negative values
        let neg = CssLength::from_css_str("calc(-10px)").unwrap();
        assert!(matches!(neg, CssLength::Calc(_)));

        // Test positive values
        let pos = CssLength::from_css_str("calc(+10px)").unwrap();
        assert!(matches!(pos, CssLength::Calc(_)));

        // Test negative in expression
        let neg_expr = CssLength::from_css_str("calc(100% + -10px)").unwrap();
        assert!(matches!(neg_expr, CssLength::Calc(_)));
    }

    #[test]
    fn test_parse_parenthesized_expressions() {
        // Test parentheses in calc
        let parens = CssLength::from_css_str("calc((100% - 20px) * 2)").unwrap();
        assert!(matches!(parens, CssLength::Calc(_)));

        // Test nested parentheses
        let nested_parens = CssLength::from_css_str("calc(((100% - 20px) * 2) + 10px)").unwrap();
        assert!(matches!(nested_parens, CssLength::Calc(_)));
    }

    #[test]
    fn test_parse_whitespace_tolerance() {
        // Test that leading/trailing whitespace is handled
        // (Pest's WHITESPACE rule should handle this)
        let result = CssLength::from_css_str("100px");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CssLength::px(100));
    }

    #[test]
    fn test_parse_errors() {
        assert!(matches!(
            CssLength::from_css_str(""),
            Err(CssValueParseError::EmptyInput)
        ));
        assert!(CssLength::from_css_str("invalid").is_err());
        assert!(CssLength::from_css_str("100").is_err()); // Missing unit
    }
}
