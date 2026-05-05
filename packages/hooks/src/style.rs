/// Creates a style string using a builder function.
///
/// This hook provides a simple way to build inline style strings for HTML elements.
/// It takes a function that receives a mutable string and returns a completed style string.
///
/// # Arguments
///
/// * `f` - A function that builds the style string
///
/// # Returns
///
/// A `String` containing the completed style declarations
///
/// # Example
///
/// ```ignore
/// let style = use_style(|mut s| {
///     s.push_str("color: red;");
///     s.push_str("font-size: 16px;");
///     s
/// });
/// ```
pub fn use_style<F>(f: F) -> String
where
    F: FnOnce(String) -> String,
{
    let mut style = String::new();
    style = f(style);
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_style() {
        let style = use_style(|mut s: String| {
            s.push_str("color: red;");
            s
        });

        assert_eq!(style, "color: red;");
    }
}
