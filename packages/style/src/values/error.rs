//! Error types for CSS value parsing.

use thiserror::Error;

/// Result type for CSS value parsing operations.
pub type ParseResult<T> = Result<T, CssValueParseError>;

/// Error that can occur when parsing CSS values.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CssValueParseError {
    /// The input string is empty or contains only whitespace.
    #[error("Input is empty")]
    EmptyInput,

    /// Unknown or invalid CSS unit.
    #[error("Invalid CSS unit: {0}")]
    InvalidUnit(String),

    /// Invalid number format.
    #[error("Invalid number format: {0}")]
    InvalidNumber(String),

    /// Invalid expression syntax.
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// Unterminated function call.
    #[error("Unterminated function: {0}")]
    UnterminatedFunction(String),

    /// Unexpected token in expression.
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),

    /// Missing required argument.
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Generic parsing error with message.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Parsing error without message (for const contexts).
    #[error("Parse error")]
    ParseErrorWithoutMessage,
}
