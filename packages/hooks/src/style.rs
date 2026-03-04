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
