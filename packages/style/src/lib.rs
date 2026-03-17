//! Type-safe CSS property builders for Tairitsu framework.
//!
//! This crate provides builders for constructing CSS style strings and class lists
//! with compile-time validation and type safety.

mod builder;
mod classes;
mod properties;

// Re-export public API
pub use builder::{StyleBuilder, StyleStringBuilder};
pub use classes::ClassesBuilder;
pub use properties::{CssProperty, Property};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_property_names() {
        assert_eq!(CssProperty::Display.as_str(), "display");
        assert_eq!(CssProperty::FlexDirection.as_str(), "flex-direction");
        assert_eq!(CssProperty::ZIndex.as_str(), "z-index");
        assert_eq!(CssProperty::BackgroundColor.as_str(), "background-color");
    }

    #[test]
    fn test_style_string_builder_basic() {
        let style = StyleStringBuilder::new()
            .add(CssProperty::Width, "100px")
            .add_px(CssProperty::Height, 50)
            .build_clean();

        assert!(style.contains("width:100px"));
        assert!(style.contains("height:50px"));
    }

    #[test]
    fn test_style_string_builder_custom() {
        let style = StyleStringBuilder::new()
            .add_custom("--glow-x", "100px")
            .add_custom("--glow-y", "200px")
            .build_clean();

        assert!(style.contains("--glow-x:100px"));
        assert!(style.contains("--glow-y:200px"));
    }

    #[test]
    fn test_style_builder_with_closure() {
        let style = StyleBuilder::build_clean(|b| {
            b.add(CssProperty::Width, "200px").add_px(CssProperty::Height, 100)
        });
        assert!(style.contains("width:200px"));
        assert!(style.contains("height:100px"));
    }

    #[test]
    fn test_classes_builder() {
        let classes = ClassesBuilder::new()
            .add("container")
            .add("flex")
            .add_if("active", true)
            .add_if("hidden", false)
            .build();

        assert!(classes.contains("container"));
        assert!(classes.contains("flex"));
        assert!(classes.contains("active"));
        assert!(!classes.contains("hidden"));
    }

    #[test]
    fn test_classes_builder_from_str() {
        let builder = ClassesBuilder::from("foo bar baz");
        let classes = builder.build();
        assert!(classes.contains("foo"));
        assert!(classes.contains("bar"));
        assert!(classes.contains("baz"));
    }

    #[test]
    fn test_classes_builder_add_all() {
        let classes = ClassesBuilder::new()
            .add_all(&["a", "b", "c"])
            .build();
        assert_eq!(classes, "a b c");
    }

    #[test]
    fn test_style_builder_to_vdom() {
        let style = StyleBuilder::new()
            .add(CssProperty::Width, "100px")
            .to_vdom_style();
        assert!(style.to_string().contains("width"));
    }
}
