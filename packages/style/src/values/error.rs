//! Error types for CSS value parsing.

use std::fmt;

/// Result type for CSS value parsing operations.
pub type ParseResult<T> = Result<T, CssValueParseError>;

/// Error that can occur when parsing CSS values.
#[derive(Debug, Clone, PartialEq)]
pub enum CssValueParseError {
    /// The input string is empty or contains only whitespace.
    EmptyInput,

    /// Unknown or invalid CSS unit.
    InvalidUnit(String),

    /// Invalid number format.
    InvalidNumber(String),

    /// Invalid expression syntax.
    InvalidExpression(String),

    /// Unterminated function call.
    UnterminatedFunction(String),

    /// Unexpected token in expression.
    UnexpectedToken(String),

    /// Missing required argument.
    MissingArgument(String),

    /// Generic parsing error with message.
    ParseError(String),

    /// Parsing error without message (for const contexts).
    ParseErrorWithoutMessage,
}

impl fmt::Display for CssValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Input is empty"),
            Self::InvalidUnit(unit) => write!(f, "Invalid CSS unit: {}", unit),
            Self::InvalidNumber(num) => write!(f, "Invalid number format: {}", num),
            Self::InvalidExpression(expr) => write!(f, "Invalid expression: {}", expr),
            Self::UnterminatedFunction(func) => write!(f, "Unterminated function: {}", func),
            Self::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            Self::MissingArgument(arg) => write!(f, "Missing required argument: {}", arg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::ParseErrorWithoutMessage => write!(f, "Parse error"),
        }
    }
}

impl std::error::Error for CssValueParseError {}
