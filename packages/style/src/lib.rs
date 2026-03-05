mod builder;
mod classes;
mod properties;

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
}
