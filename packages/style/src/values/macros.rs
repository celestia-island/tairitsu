//! The `calc!` macro for compile-time CSS value construction.

/// Macro for creating CSS length values with compile-time validation.
///
/// # Examples
///
/// ```ignore
/// use tairitsu_style::{CssLength, calc};
///
/// // Parse string literals at runtime (compile-time parsing limited in stable Rust)
/// let width = calc!("100px");
/// let height = calc!("50%");
/// ```
///
/// Note: Full compile-time parsing is limited in stable Rust.
/// For now, this macro provides a convenient syntax but parsing happens at runtime.
#[macro_export]
macro_rules! calc {
    // String literal - parse at runtime
    ($lit:literal) => {{
        $crate::CssLength::from_css_str($lit).expect("Invalid CSS length")
    }};

    // Direct value passthrough
    ($expr:expr) => {
        $expr
    };
}

/// Macro for creating CSS `min()` function calls.
///
/// # Examples
///
/// ```
/// use tairitsu_style::{CssLength, css_min};
///
/// let flexible = css_min!(CssLength::px(100), CssLength::percent(50));
/// ```
#[macro_export]
macro_rules! css_min {
    ($($val:expr),+ $(,)?) => {
        $crate::CssLength::min(vec![$($val),+])
    };
}

/// Macro for creating CSS `max()` function calls.
///
/// # Examples
///
/// ```
/// use tairitsu_style::{CssLength, css_max};
///
/// let flexible = css_max!(CssLength::vw(100), CssLength::px(1200));
/// ```
#[macro_export]
macro_rules! css_max {
    ($($val:expr),+ $(,)?) => {
        $crate::CssLength::max(vec![$($val),+])
    };
}

/// Macro for creating CSS `clamp()` function calls.
///
/// # Examples
///
/// ```
/// use tairitsu_style::{CssLength, css_clamp};
///
/// let flexible = css_clamp!(CssLength::px(300), CssLength::percent(50), CssLength::px(800));
/// ```
#[macro_export]
macro_rules! css_clamp {
    ($min:expr, $preferred:expr, $max:expr) => {
        $crate::CssLength::clamp($min, $preferred, $max)
    };
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::CssLength;

    #[test]
    fn test_calc_macro_string() {
        let width = calc!("100px");
        assert_eq!(width, CssLength::px(100));

        let height = calc!("50%");
        assert_eq!(height, CssLength::percent(50));

        let em = calc!("1.5em");
        assert_eq!(em, CssLength::em(1.5));
    }

    #[test]
    fn test_css_min_macro() {
        let min_val = css_min!(CssLength::px(100), CssLength::percent(50));
        assert!(matches!(min_val, CssLength::Min(_)));

        let min_val = css_min!(CssLength::px(100), CssLength::percent(50),);
        assert!(matches!(min_val, CssLength::Min(_)));
    }

    #[test]
    fn test_css_max_macro() {
        let max_val = css_max!(CssLength::vw(100), CssLength::px(1200));
        assert!(matches!(max_val, CssLength::Max(_)));
    }

    #[test]
    fn test_css_clamp_macro() {
        let clamp_val = css_clamp!(CssLength::px(300), CssLength::percent(50), CssLength::px(800));
        assert!(matches!(clamp_val, CssLength::Clamp { .. }));
    }
}
