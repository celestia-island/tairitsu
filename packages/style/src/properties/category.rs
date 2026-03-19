//! CSS property categories.
//!
//! This module defines the `CssCategory` enum, which categorizes CSS properties
//! into logical groups for better organization and API discoverability.

/// Category of a CSS property.
///
/// Categories group related properties together, making it easier to find
/// properties that serve similar purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssCategory {
    /// Properties that control the layout and positioning of elements.
    ///
    /// Includes: `display`, `position`, `top`, `right`, `bottom`, `left`, `z-index`, `float`, `clear`, `overflow`
    Layout,

    /// Properties related to the box model: width, height, margin, padding, border.
    ///
    /// Includes: `width`, `height`, `margin`, `padding`, `border`, `border-radius`, etc.
    BoxModel,

    /// CSS Flexible Box Layout properties.
    ///
    /// Includes: `flex`, `flex-direction`, `align-items`, `justify-content`, etc.
    Flexbox,

    /// CSS Grid Layout properties.
    ///
    /// Includes: `grid`, `grid-template`, `grid-area`, `gap`, etc.
    Grid,

    /// Font and text properties.
    ///
    /// Includes: `font`, `font-size`, `text-align`, `line-height`, etc.
    Typography,

    /// Color and background properties.
    ///
    /// Includes: `color`, `background`, `background-color`, `opacity`, etc.
    Color,

    /// Visual effects and filters.
    ///
    /// Includes: `filter`, `clip-path`, `mask`, `box-shadow`, etc.
    Visual,

    /// 2D and 3D transforms.
    ///
    /// Includes: `transform`, `transform-origin`, `perspective`, etc.
    Transform,

    /// CSS transition properties.
    ///
    /// Includes: `transition`, `transition-duration`, `transition-timing-function`, etc.
    Transition,

    /// CSS animation properties.
    ///
    /// Includes: `animation`, `animation-name`, `animation-duration`, etc.
    Animation,

    /// User interaction properties.
    ///
    /// Includes: `cursor`, `pointer-events`, `user-select`, `outline`, etc.
    Interaction,

    /// List and counter properties.
    ///
    /// Includes: `list-style`, `list-style-type`, `counter-increment`, etc.
    List,

    /// Table-specific properties.
    ///
    /// Includes: `table-layout`, `border-collapse`, `caption-side`, etc.
    Table,

    /// Paged media properties.
    ///
    /// Includes: `page-break-before`, `orphans`, `widows`, etc.
    Paged,

    /// Generated content properties.
    ///
    /// Includes: `content`, `quotes`, etc.
    GeneratedContent,

    /// Gap properties for flex and grid layouts.
    ///
    /// Includes: `gap`, `row-gap`, `column-gap`
    FlexGridGap,

    /// Miscellaneous CSS properties that don't fit into other categories.
    ///
    /// Includes: `object-fit`, `vertical-align`, `columns`, etc.
    Miscellaneous,
}

impl CssCategory {
    /// Get the display name of this category.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            CssCategory::Layout => "Layout",
            CssCategory::BoxModel => "Box Model",
            CssCategory::Flexbox => "Flexbox",
            CssCategory::Grid => "Grid",
            CssCategory::Typography => "Typography",
            CssCategory::Color => "Color",
            CssCategory::Visual => "Visual Effects",
            CssCategory::Transform => "Transform",
            CssCategory::Transition => "Transition",
            CssCategory::Animation => "Animation",
            CssCategory::Interaction => "Interaction",
            CssCategory::List => "List",
            CssCategory::Table => "Table",
            CssCategory::Paged => "Paged Media",
            CssCategory::GeneratedContent => "Generated Content",
            CssCategory::FlexGridGap => "Flex & Grid Gap",
            CssCategory::Miscellaneous => "Miscellaneous",
        }
    }

    /// Get a description of what kinds of properties are in this category.
    #[inline]
    pub const fn description(self) -> &'static str {
        match self {
            CssCategory::Layout => "Properties that control the layout and positioning of elements",
            CssCategory::BoxModel => "Properties related to the box model: width, height, margin, padding, border",
            CssCategory::Flexbox => "CSS Flexible Box Layout properties",
            CssCategory::Grid => "CSS Grid Layout properties",
            CssCategory::Typography => "Font and text properties",
            CssCategory::Color => "Color and background properties",
            CssCategory::Visual => "Visual effects and filters",
            CssCategory::Transform => "2D and 3D transforms",
            CssCategory::Transition => "CSS transition properties",
            CssCategory::Animation => "CSS animation properties",
            CssCategory::Interaction => "User interaction properties",
            CssCategory::List => "List and counter properties",
            CssCategory::Table => "Table-specific properties",
            CssCategory::Paged => "Paged media properties",
            CssCategory::GeneratedContent => "Generated content properties",
            CssCategory::FlexGridGap => "Gap properties for flex and grid layouts",
            CssCategory::Miscellaneous => "Miscellaneous CSS properties",
        }
    }

    /// Get all category variants.
    #[inline]
    pub const fn all() -> [CssCategory; 17] {
        [
            CssCategory::Layout,
            CssCategory::BoxModel,
            CssCategory::Flexbox,
            CssCategory::Grid,
            CssCategory::Typography,
            CssCategory::Color,
            CssCategory::Visual,
            CssCategory::Transform,
            CssCategory::Transition,
            CssCategory::Animation,
            CssCategory::Interaction,
            CssCategory::List,
            CssCategory::Table,
            CssCategory::Paged,
            CssCategory::GeneratedContent,
            CssCategory::FlexGridGap,
            CssCategory::Miscellaneous,
        ]
    }
}

impl std::fmt::Display for CssCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_names() {
        assert_eq!(CssCategory::Layout.name(), "Layout");
        assert_eq!(CssCategory::Flexbox.name(), "Flexbox");
        assert_eq!(CssCategory::Typography.name(), "Typography");
    }

    #[test]
    fn test_category_descriptions() {
        assert!(!CssCategory::Layout.description().is_empty());
        assert!(!CssCategory::BoxModel.description().is_empty());
    }

    #[test]
    fn test_all_categories() {
        let categories = CssCategory::all();
        assert_eq!(categories.len(), 17);
        assert!(categories.contains(&CssCategory::Layout));
        assert!(categories.contains(&CssCategory::Miscellaneous));
    }

    #[test]
    fn test_display() {
        assert_eq!(CssCategory::Grid.to_string(), "Grid");
        assert_eq!(CssCategory::Animation.to_string(), "Animation");
    }
}
