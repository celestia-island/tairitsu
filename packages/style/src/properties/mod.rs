//! CSS property definitions and categorization.
//!
//! This module provides a comprehensive, type-safe enum of CSS properties
//! with metadata including categories, shorthand status, and documentation links.
//!
//! # Generated Code
//!
//! The `CssProperty` enum and its implementations are automatically generated
//! from the CSS properties data in `css_data/css_properties.json` during the
//! build process. To add or modify properties, edit the JSON file and rebuild.
//!
//! # Example
//!
//! ```rust
//! use tairitsu_style::{CssProperty, CssCategory};
//!
//! let prop = CssProperty::FlexDirection;
//! assert_eq!(prop.as_str(), "flex-direction");
//! assert_eq!(prop.category(), CssCategory::Flexbox);
//! assert!(!prop.is_shorthand());
//! ```
//!
//! # Property Categories
//!
//! Properties are organized into the following categories:
//!
//! - [`Layout`](CssCategory::Layout) - Positioning and display properties
//! - [`BoxModel`](CssCategory::BoxModel) - Width, height, margin, padding, border
//! - [`Flexbox`](CssCategory::Flexbox) - Flex layout properties
//! - [`Grid`](CssCategory::Grid) - Grid layout properties
//! - [`Typography`](CssCategory::Typography) - Font and text properties
//! - [`Color`](CssCategory::Color) - Color and background properties
//! - [`Visual`](CssCategory::Visual) - Visual effects and filters
//! - [`Transform`](CssCategory::Transform) - 2D/3D transforms
//! - [`Transition`](CssCategory::Transition) - Transition properties
//! - [`Animation`](CssCategory::Animation) - Animation properties
//! - [`Interaction`](CssCategory::Interaction) - User interaction properties
//! - [`List`](CssCategory::List) - List and counter properties
//! - [`Table`](CssCategory::Table) - Table-specific properties
//! - [`Paged`](CssCategory::Paged) - Paged media properties
//! - [`GeneratedContent`](CssCategory::GeneratedContent) - Generated content properties
//! - [`FlexGridGap`](CssCategory::FlexGridGap) - Gap properties for flex and grid
//! - [`Miscellaneous`](CssCategory::Miscellaneous) - Other properties

mod category;
mod css;

// Re-export the main types
pub use category::CssCategory;
pub use css::{CssProperty, Property};
