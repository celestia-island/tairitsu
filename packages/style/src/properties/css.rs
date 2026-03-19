//! CSS property definitions.
//!
//! This module contains the auto-generated `CssProperty` enum and its implementations.
//! The enum is generated from the CSS properties data in `css_data/css_properties.json`
//! by the build script.

// Include the auto-generated code
include!("generated.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_names() {
        // Test basic properties
        assert_eq!(CssProperty::Display.as_str(), "display");
        assert_eq!(CssProperty::Position.as_str(), "position");
        assert_eq!(CssProperty::Width.as_str(), "width");
        assert_eq!(CssProperty::Height.as_str(), "height");
    }

    #[test]
    fn test_hyphenated_properties() {
        // Test properties with hyphens
        assert_eq!(CssProperty::ZIndex.as_str(), "z-index");
        assert_eq!(CssProperty::FlexDirection.as_str(), "flex-direction");
        assert_eq!(CssProperty::BackgroundColor.as_str(), "background-color");
    }

    #[test]
    fn test_property_categories() {
        // Test that properties have correct categories
        assert_eq!(CssProperty::Display.category(), CssCategory::Layout);
        assert_eq!(CssProperty::Width.category(), CssCategory::BoxModel);
        assert_eq!(CssProperty::FlexDirection.category(), CssCategory::Flexbox);
        assert_eq!(CssProperty::GridTemplateColumns.category(), CssCategory::Grid);
        assert_eq!(CssProperty::FontSize.category(), CssCategory::Typography);
        assert_eq!(CssProperty::Color.category(), CssCategory::Color);
    }

    #[test]
    fn test_shorthand_detection() {
        // Test that shorthands are correctly identified
        assert!(CssProperty::Margin.is_shorthand());
        assert!(CssProperty::Padding.is_shorthand());
        assert!(CssProperty::Border.is_shorthand());
        assert!(CssProperty::Flex.is_shorthand());
        assert!(CssProperty::Background.is_shorthand());
        assert!(CssProperty::Transition.is_shorthand());
        assert!(CssProperty::Animation.is_shorthand());
    }

    #[test]
    fn test_non_shorthand_properties() {
        // Test that non-shorthands are correctly identified
        assert!(!CssProperty::Display.is_shorthand());
        assert!(!CssProperty::Width.is_shorthand());
        assert!(!CssProperty::FlexDirection.is_shorthand());
    }

    #[test]
    fn test_experimental_detection() {
        // Most current properties should be non-experimental
        assert!(!CssProperty::Display.is_experimental());
        assert!(!CssProperty::FlexDirection.is_experimental());
    }

    #[test]
    fn test_mdn_url() {
        // Test that MDN URLs are present and well-formed
        let url = CssProperty::Display.mdn_url();
        assert!(url.contains("developer.mozilla.org"));
        assert!(url.contains("display"));

        let url = CssProperty::FlexDirection.mdn_url();
        assert!(url.contains("developer.mozilla.org"));
        assert!(url.contains("flex-direction"));
    }
}
