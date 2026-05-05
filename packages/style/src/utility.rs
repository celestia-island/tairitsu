//! Utility class system for Tailwind-like CSS classes.
//!
//! This module provides a trait-based system for defining utility classes
//! that can be registered with the ClassesBuilder and used to generate
//! CSS classes with variants (responsive, state, etc.).

use std::sync::Arc;

use crate::properties::{CssCategory, CssProperty, Property};

/// Responsive breakpoint variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    /// Small screens (640px and up)
    Sm,
    /// Medium screens (768px and up)
    Md,
    /// Large screens (1024px and up)
    Lg,
    /// Extra large screens (1280px and up)
    Xl,
    /// 2X Large screens (1536px and up)
    Xxl,
}

impl Breakpoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            Breakpoint::Sm => "sm",
            Breakpoint::Md => "md",
            Breakpoint::Lg => "lg",
            Breakpoint::Xl => "xl",
            Breakpoint::Xxl => "2xl",
        }
    }

    pub fn min_width(&self) -> &'static str {
        match self {
            Breakpoint::Sm => "640px",
            Breakpoint::Md => "768px",
            Breakpoint::Lg => "1024px",
            Breakpoint::Xl => "1280px",
            Breakpoint::Xxl => "1536px",
        }
    }
}

/// Pseudo-class state variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Hover state
    Hover,
    /// Focus state
    Focus,
    /// Active state
    Active,
    /// Focus-within state
    FocusWithin,
    /// Focus-visible state
    FocusVisible,
    /// Disabled state
    Disabled,
    /// Checked state (for checkboxes/radios)
    Checked,
    /// First-child
    First,
    /// Last-child
    Last,
    /// Odd child
    Odd,
    /// Even child
    Even,
}

impl State {
    pub fn as_str(&self) -> &'static str {
        match self {
            State::Hover => ":hover",
            State::Focus => ":focus",
            State::Active => ":active",
            State::FocusWithin => ":focus-within",
            State::FocusVisible => ":focus-visible",
            State::Disabled => ":disabled",
            State::Checked => ":checked",
            State::First => ":first-child",
            State::Last => ":last-child",
            State::Odd => ":nth-child(odd)",
            State::Even => ":nth-child(even)",
        }
    }
}

/// A utility class variant (responsive or state)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Responsive breakpoint variant
    Breakpoint(Breakpoint),
    /// Pseudo-class state variant
    State(State),
    /// Combined breakpoint and state variant (e.g., "hover:md:text-center")
    Combined {
        breakpoint: Breakpoint,
        state: State,
    },
}

impl Variant {
    /// Parse a variant from a string prefix
    ///
    /// Examples: "sm:", "hover:", "md:hover:"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim_end_matches(':').split(':').collect();

        match parts.as_slice() {
            [bp] => {
                // Try breakpoint first, then state
                if let Some(bp) = Breakpoint::from_str(bp) {
                    Some(Variant::Breakpoint(bp))
                } else {
                    State::from_str(bp).map(Variant::State)
                }
            }
            [first, second] => {
                // Try both orders for combined variants
                if let (Some(bp), Some(st)) = (Breakpoint::from_str(first), State::from_str(second))
                {
                    Some(Variant::Combined {
                        breakpoint: bp,
                        state: st,
                    })
                } else if let (Some(st), Some(bp)) =
                    (State::from_str(first), Breakpoint::from_str(second))
                {
                    Some(Variant::Combined {
                        breakpoint: bp,
                        state: st,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert the variant to its CSS media query or pseudo-class selector
    pub fn as_css_selector(&self, class: &str) -> String {
        match self {
            Variant::Breakpoint(bp) => {
                format!(
                    "@media (min-width: {}) {{ .{} }}",
                    bp.min_width(),
                    class.trim_start_matches('.')
                )
            }
            Variant::State(st) => {
                format!("{}{}", class, st.as_str())
            }
            Variant::Combined { breakpoint, state } => {
                format!(
                    "@media (min-width: {}) {{ .{}{} }}",
                    breakpoint.min_width(),
                    class.trim_start_matches('.'),
                    state.as_str()
                )
            }
        }
    }

    /// Get the variant prefix for class naming
    pub fn as_prefix(&self) -> String {
        match self {
            Variant::Breakpoint(bp) => format!("{}:", bp.as_str()),
            Variant::State(st) => {
                // State.as_str() returns ":hover", so we trim the colon and add it back
                let state_name = st.as_str().trim_start_matches(':');
                format!("{}:", state_name)
            }
            Variant::Combined { breakpoint, state } => {
                // State.as_str() returns ":hover", so we trim the colon
                let state_name = state.as_str().trim_start_matches(':');
                format!("{}:{}:", breakpoint.as_str(), state_name)
            }
        }
    }
}

impl Breakpoint {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "sm" => Some(Breakpoint::Sm),
            "md" => Some(Breakpoint::Md),
            "lg" => Some(Breakpoint::Lg),
            "xl" => Some(Breakpoint::Xl),
            "2xl" | "xxl" => Some(Breakpoint::Xxl),
            _ => None,
        }
    }
}

impl State {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "hover" => Some(State::Hover),
            "focus" => Some(State::Focus),
            "active" => Some(State::Active),
            "focus-within" => Some(State::FocusWithin),
            "focus-visible" => Some(State::FocusVisible),
            "disabled" => Some(State::Disabled),
            "checked" => Some(State::Checked),
            "first" => Some(State::First),
            "last" => Some(State::Last),
            "odd" => Some(State::Odd),
            "even" => Some(State::Even),
            _ => None,
        }
    }
}

/// A parsed utility class with optional variant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUtility {
    /// The base class name (without variants)
    pub base: String,
    /// Optional variant
    pub variant: Option<Variant>,
    /// Optional arbitrary value (e.g., "4px" from "p-[4px]")
    pub arbitrary_value: Option<String>,
}

impl ParsedUtility {
    /// Parse a utility class string into its components
    ///
    /// Examples:
    /// - "p-4" -> ParsedUtility { base: "p-4", variant: None, arbitrary_value: None }
    /// - "hover:text-center" -> ParsedUtility { base: "text-center", variant: Some(State::Hover), ... }
    /// - "md:p-[10px]" -> ParsedUtility { base: "p-[10px]", variant: Some(Breakpoint::Md), arbitrary_value: Some("10px") }
    pub fn parse(s: &str) -> Self {
        // Extract arbitrary value if present
        let arbitrary_value = if let Some(start) = s.find('[') {
            s.rfind(']').map(|end| s[start + 1..end].to_string())
        } else {
            None
        };

        // Find variant prefix (ends with ':')
        let variant = s
            .split(':')
            .take(s.matches(':').count())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(":");

        let base = if variant.is_empty() {
            s.to_string()
        } else {
            s[variant.len() + 1..].to_string()
        };

        let variant = if variant.is_empty() {
            None
        } else {
            Variant::parse(&format!("{}:", variant))
        };

        ParsedUtility {
            base,
            variant,
            arbitrary_value,
        }
    }
}

/// Trait for types that can generate utility classes
pub trait UtilityClass: Send + Sync {
    /// Get the class name pattern this utility handles
    ///
    /// Examples: "p-{n}", "m-{n}", "flex"
    fn pattern(&self) -> &'static str;

    /// Generate the CSS rules for this utility class
    ///
    /// # Arguments
    /// * `class_name` - The full class name (including variants)
    /// * `parsed` - The parsed utility components
    ///
    /// # Returns
    /// A CSS rule string (e.g., ".p-4 { padding: 1rem; }")
    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String>;

    /// Validate if a class name matches this utility
    fn matches(&self, class_name: &str) -> bool;

    /// Get the CSS property this utility modifies
    fn property(&self) -> Property;

    /// Get the category of this utility
    fn category(&self) -> CssCategory;
}

/// Registry for utility classes
#[derive(Clone)]
pub struct UtilityRegistry {
    utilities: Vec<Arc<dyn UtilityClass>>,
}

impl UtilityRegistry {
    pub fn new() -> Self {
        Self {
            utilities: Vec::new(),
        }
    }

    /// Register a new utility class
    pub fn register(&mut self, utility: Arc<dyn UtilityClass>) {
        self.utilities.push(utility);
    }

    /// Find a utility class that matches the given class name
    pub fn find(&self, class_name: &str) -> Option<Arc<dyn UtilityClass>> {
        let parsed = ParsedUtility::parse(class_name);
        self.utilities
            .iter()
            .find(|u| u.matches(&parsed.base))
            .cloned()
    }

    /// Generate CSS for a class name
    pub fn generate_css(&self, class_name: &str) -> Option<String> {
        let utility = self.find(class_name)?;
        let parsed = ParsedUtility::parse(class_name);
        utility.generate_css(class_name, &parsed)
    }

    /// Get all registered utilities
    pub fn all(&self) -> &[Arc<dyn UtilityClass>] {
        &self.utilities
    }
}

impl Default for UtilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Spacing Utilities
// ============================================================================

/// Padding utility class (p-{n}, px-{n}, py-{n}, pt-{n}, pr-{n}, pb-{n}, pl-{n})
#[derive(Debug, Clone, Copy)]
pub struct PaddingUtility {
    base: &'static str,
    property: CssProperty,
}

impl PaddingUtility {
    pub const fn new(base: &'static str, property: CssProperty) -> Self {
        Self { base, property }
    }

    /// Convert spacing scale value to rem
    fn scale_to_rem(value: &str) -> Option<String> {
        // Tailwind's default spacing scale
        let scale = match value {
            "0" => "0",
            "px" => "1px",
            "0.5" => "0.125rem",
            "1" => "0.25rem",
            "1.5" => "0.375rem",
            "2" => "0.5rem",
            "2.5" => "0.625rem",
            "3" => "0.75rem",
            "3.5" => "0.875rem",
            "4" => "1rem",
            "5" => "1.25rem",
            "6" => "1.5rem",
            "7" => "1.75rem",
            "8" => "2rem",
            "9" => "2.25rem",
            "10" => "2.5rem",
            "11" => "2.75rem",
            "12" => "3rem",
            "14" => "3.5rem",
            "16" => "4rem",
            "20" => "5rem",
            "24" => "6rem",
            "28" => "7rem",
            "32" => "8rem",
            "36" => "9rem",
            "40" => "10rem",
            "44" => "11rem",
            "48" => "12rem",
            "52" => "13rem",
            "56" => "14rem",
            "60" => "15rem",
            "64" => "16rem",
            "72" => "18rem",
            "80" => "20rem",
            "96" => "24rem",
            _ => return None,
        };
        Some(scale.to_string())
    }
}

impl UtilityClass for PaddingUtility {
    fn pattern(&self) -> &'static str {
        self.base
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let value_str = if let Some(ref arbitrary) = parsed.arbitrary_value {
            arbitrary.clone()
        } else {
            // Extract value from class name (e.g., "p-4" -> "4")
            parsed
                .base
                .strip_prefix(self.base)?
                .strip_prefix('-')?
                .to_string()
        };

        let value = Self::scale_to_rem(&value_str).unwrap_or(value_str);

        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!(
            "{} {{ {}:{}; }}",
            selector,
            self.property.as_str(),
            value
        ))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name.starts_with(self.base)
            && (class_name.len() > self.base.len())
            && class_name.chars().nth(self.base.len()) == Some('-')
    }

    fn property(&self) -> Property {
        Property::Known(self.property)
    }

    fn category(&self) -> CssCategory {
        CssCategory::BoxModel
    }
}

/// Margin utility class (m-{n}, mx-{n}, my-{n}, mt-{n}, mr-{n}, mb-{n}, ml-{n})
#[derive(Debug, Clone, Copy)]
pub struct MarginUtility {
    base: &'static str,
    property: CssProperty,
}

impl MarginUtility {
    pub const fn new(base: &'static str, property: CssProperty) -> Self {
        Self { base, property }
    }

    fn scale_to_rem(value: &str) -> Option<String> {
        PaddingUtility::scale_to_rem(value)
    }
}

impl UtilityClass for MarginUtility {
    fn pattern(&self) -> &'static str {
        self.base
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let value_str = if let Some(ref arbitrary) = parsed.arbitrary_value {
            arbitrary.clone()
        } else {
            parsed
                .base
                .strip_prefix(self.base)?
                .strip_prefix('-')?
                .to_string()
        };

        let value = if value_str == "auto" {
            "auto".to_string()
        } else {
            Self::scale_to_rem(&value_str).unwrap_or(value_str)
        };

        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!(
            "{} {{ {}:{}; }}",
            selector,
            self.property.as_str(),
            value
        ))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name.starts_with(self.base)
            && (class_name.len() > self.base.len())
            && class_name.chars().nth(self.base.len()) == Some('-')
    }

    fn property(&self) -> Property {
        Property::Known(self.property)
    }

    fn category(&self) -> CssCategory {
        CssCategory::BoxModel
    }
}

// ============================================================================
// Layout Utilities
// ============================================================================

/// Display utility class (flex, grid, block, inline-block, etc.)
#[derive(Debug, Clone, Copy)]
pub struct DisplayUtility {
    value: &'static str,
}

impl DisplayUtility {
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

impl UtilityClass for DisplayUtility {
    fn pattern(&self) -> &'static str {
        self.value
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ display:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.value
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::Display)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Layout
    }
}

/// Hidden utility class (special case: display: none)
#[derive(Debug, Clone, Copy)]
pub struct HiddenUtility;

impl UtilityClass for HiddenUtility {
    fn pattern(&self) -> &'static str {
        "hidden"
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ display:none; }}", selector))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == "hidden"
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::Display)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Layout
    }
}

/// Flex direction utility
#[derive(Debug, Clone, Copy)]
pub struct FlexDirectionUtility {
    value: &'static str,
    class: &'static str,
}

impl FlexDirectionUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for FlexDirectionUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ flex-direction:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::FlexDirection)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Flexbox
    }
}

/// Flex wrap utility
#[derive(Debug, Clone, Copy)]
pub struct FlexWrapUtility {
    value: &'static str,
    class: &'static str,
}

impl FlexWrapUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for FlexWrapUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ flex-wrap:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::FlexWrap)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Flexbox
    }
}

/// Justify content utility
#[derive(Debug, Clone, Copy)]
pub struct JustifyContentUtility {
    value: &'static str,
    class: &'static str,
}

impl JustifyContentUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for JustifyContentUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!(
            "{} {{ justify-content:{}; }}",
            selector, self.value
        ))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::JustifyContent)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Flexbox
    }
}

/// Align items utility
#[derive(Debug, Clone, Copy)]
pub struct AlignItemsUtility {
    value: &'static str,
    class: &'static str,
}

impl AlignItemsUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for AlignItemsUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ align-items:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::AlignItems)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Flexbox
    }
}

// ============================================================================
// Typography Utilities
// ============================================================================

/// Text align utility
#[derive(Debug, Clone, Copy)]
pub struct TextAlignUtility {
    value: &'static str,
    class: &'static str,
}

impl TextAlignUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for TextAlignUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ text-align:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::TextAlign)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Typography
    }
}

/// Font size utility
#[derive(Debug, Clone, Copy)]
pub struct FontSizeUtility {
    class: &'static str,
    value: &'static str,
}

impl FontSizeUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for FontSizeUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let value = if let Some(ref arbitrary) = parsed.arbitrary_value {
            arbitrary.clone()
        } else {
            self.value.to_string()
        };

        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ font-size:{}; }}", selector, value))
    }

    fn matches(&self, class_name: &str) -> bool {
        // Only match exact class names or classes with arbitrary values
        class_name == self.class || (class_name.starts_with("text-[") && class_name.ends_with(']'))
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::FontSize)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Typography
    }
}

/// Font weight utility
#[derive(Debug, Clone, Copy)]
pub struct FontWeightUtility {
    value: &'static str,
    class: &'static str,
}

impl FontWeightUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for FontWeightUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ font-weight:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        // Only match exact class names
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::FontWeight)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Typography
    }
}

// ============================================================================
// Position Utilities
// ============================================================================

/// Position utility
#[derive(Debug, Clone, Copy)]
pub struct PositionUtility {
    value: &'static str,
    class: &'static str,
}

impl PositionUtility {
    pub const fn new(class: &'static str, value: &'static str) -> Self {
        Self { class, value }
    }
}

impl UtilityClass for PositionUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ position:{}; }}", selector, self.value))
    }

    fn matches(&self, class_name: &str) -> bool {
        class_name == self.class
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::Position)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Layout
    }
}

// ============================================================================
// Color Utilities (basic)
// ============================================================================

/// Text color utility
#[derive(Debug, Clone, Copy)]
pub struct TextColorUtility {
    color: &'static str,
    class: &'static str,
}

impl TextColorUtility {
    pub const fn new(class: &'static str, color: &'static str) -> Self {
        Self { class, color }
    }
}

impl UtilityClass for TextColorUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let color = if let Some(ref arbitrary) = parsed.arbitrary_value {
            arbitrary.clone()
        } else {
            self.color.to_string()
        };

        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ color:{}; }}", selector, color))
    }

    fn matches(&self, class_name: &str) -> bool {
        // Only match exact class names or classes with arbitrary values
        class_name == self.class || (class_name.starts_with("text-[") && class_name.ends_with(']'))
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::Color)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Color
    }
}

/// Background color utility
#[derive(Debug, Clone, Copy)]
pub struct BgColorUtility {
    color: &'static str,
    class: &'static str,
}

impl BgColorUtility {
    pub const fn new(class: &'static str, color: &'static str) -> Self {
        Self { class, color }
    }
}

impl UtilityClass for BgColorUtility {
    fn pattern(&self) -> &'static str {
        self.class
    }

    fn generate_css(&self, class_name: &str, parsed: &ParsedUtility) -> Option<String> {
        let color = if let Some(ref arbitrary) = parsed.arbitrary_value {
            arbitrary.clone()
        } else {
            self.color.to_string()
        };

        let selector = if let Some(ref variant) = parsed.variant {
            variant.as_css_selector(&format!(".{}", class_name))
        } else {
            format!(".{}", class_name)
        };

        Some(format!("{} {{ background-color:{}; }}", selector, color))
    }

    fn matches(&self, class_name: &str) -> bool {
        // Only match exact class names or classes with arbitrary values
        class_name == self.class || (class_name.starts_with("bg-[") && class_name.ends_with(']'))
    }

    fn property(&self) -> Property {
        Property::Known(CssProperty::BackgroundColor)
    }

    fn category(&self) -> CssCategory {
        CssCategory::Color
    }
}

// ============================================================================
// Default Registry
// ============================================================================

/// Create a default utility registry with common Tailwind-like utilities
pub fn create_default_registry() -> UtilityRegistry {
    let mut registry = UtilityRegistry::new();

    // Padding utilities
    registry.register(Arc::new(PaddingUtility::new("p", CssProperty::Padding)));
    registry.register(Arc::new(PaddingUtility::new(
        "px",
        CssProperty::PaddingLeft,
    ))); // Will need shorthand handling
    registry.register(Arc::new(PaddingUtility::new("py", CssProperty::PaddingTop)));
    registry.register(Arc::new(PaddingUtility::new("pt", CssProperty::PaddingTop)));
    registry.register(Arc::new(PaddingUtility::new(
        "pr",
        CssProperty::PaddingRight,
    )));
    registry.register(Arc::new(PaddingUtility::new(
        "pb",
        CssProperty::PaddingBottom,
    )));
    registry.register(Arc::new(PaddingUtility::new(
        "pl",
        CssProperty::PaddingLeft,
    )));

    // Margin utilities
    registry.register(Arc::new(MarginUtility::new("m", CssProperty::Margin)));
    registry.register(Arc::new(MarginUtility::new("mx", CssProperty::MarginLeft)));
    registry.register(Arc::new(MarginUtility::new("my", CssProperty::MarginTop)));
    registry.register(Arc::new(MarginUtility::new("mt", CssProperty::MarginTop)));
    registry.register(Arc::new(MarginUtility::new("mr", CssProperty::MarginRight)));
    registry.register(Arc::new(MarginUtility::new(
        "mb",
        CssProperty::MarginBottom,
    )));
    registry.register(Arc::new(MarginUtility::new("ml", CssProperty::MarginLeft)));

    // Display utilities
    registry.register(Arc::new(DisplayUtility::new("flex")));
    registry.register(Arc::new(DisplayUtility::new("grid")));
    registry.register(Arc::new(DisplayUtility::new("block")));
    registry.register(Arc::new(DisplayUtility::new("inline-block")));
    registry.register(Arc::new(DisplayUtility::new("inline")));
    registry.register(Arc::new(HiddenUtility));

    // Flex direction utilities
    registry.register(Arc::new(FlexDirectionUtility::new("flex-row", "row")));
    registry.register(Arc::new(FlexDirectionUtility::new(
        "flex-row-reverse",
        "row-reverse",
    )));
    registry.register(Arc::new(FlexDirectionUtility::new("flex-col", "column")));
    registry.register(Arc::new(FlexDirectionUtility::new(
        "flex-col-reverse",
        "column-reverse",
    )));

    // Flex wrap utilities
    registry.register(Arc::new(FlexWrapUtility::new("flex-wrap", "wrap")));
    registry.register(Arc::new(FlexWrapUtility::new("flex-nowrap", "nowrap")));
    registry.register(Arc::new(FlexWrapUtility::new(
        "flex-wrap-reverse",
        "wrap-reverse",
    )));

    // Justify content utilities
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-start",
        "flex-start",
    )));
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-end",
        "flex-end",
    )));
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-center",
        "center",
    )));
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-between",
        "space-between",
    )));
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-around",
        "space-around",
    )));
    registry.register(Arc::new(JustifyContentUtility::new(
        "justify-evenly",
        "space-evenly",
    )));

    // Align items utilities
    registry.register(Arc::new(AlignItemsUtility::new(
        "items-start",
        "flex-start",
    )));
    registry.register(Arc::new(AlignItemsUtility::new("items-end", "flex-end")));
    registry.register(Arc::new(AlignItemsUtility::new("items-center", "center")));
    registry.register(Arc::new(AlignItemsUtility::new(
        "items-baseline",
        "baseline",
    )));
    registry.register(Arc::new(AlignItemsUtility::new("items-stretch", "stretch")));

    // Text align utilities
    registry.register(Arc::new(TextAlignUtility::new("text-left", "left")));
    registry.register(Arc::new(TextAlignUtility::new("text-center", "center")));
    registry.register(Arc::new(TextAlignUtility::new("text-right", "right")));
    registry.register(Arc::new(TextAlignUtility::new("text-justify", "justify")));

    // Font size utilities (Tailwind scale)
    registry.register(Arc::new(FontSizeUtility::new("text-xs", "0.75rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-sm", "0.875rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-base", "1rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-lg", "1.125rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-xl", "1.25rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-2xl", "1.5rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-3xl", "1.875rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-4xl", "2.25rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-5xl", "3rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-6xl", "3.75rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-7xl", "4.5rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-8xl", "6rem")));
    registry.register(Arc::new(FontSizeUtility::new("text-9xl", "8rem")));

    // Font weight utilities
    registry.register(Arc::new(FontWeightUtility::new("font-thin", "100")));
    registry.register(Arc::new(FontWeightUtility::new("font-extralight", "200")));
    registry.register(Arc::new(FontWeightUtility::new("font-light", "300")));
    registry.register(Arc::new(FontWeightUtility::new("font-normal", "400")));
    registry.register(Arc::new(FontWeightUtility::new("font-medium", "500")));
    registry.register(Arc::new(FontWeightUtility::new("font-semibold", "600")));
    registry.register(Arc::new(FontWeightUtility::new("font-bold", "700")));
    registry.register(Arc::new(FontWeightUtility::new("font-extrabold", "800")));
    registry.register(Arc::new(FontWeightUtility::new("font-black", "900")));

    // Position utilities
    registry.register(Arc::new(PositionUtility::new("static", "static")));
    registry.register(Arc::new(PositionUtility::new("fixed", "fixed")));
    registry.register(Arc::new(PositionUtility::new("absolute", "absolute")));
    registry.register(Arc::new(PositionUtility::new("relative", "relative")));
    registry.register(Arc::new(PositionUtility::new("sticky", "sticky")));

    // Basic color utilities
    registry.register(Arc::new(TextColorUtility::new("text-white", "#ffffff")));
    registry.register(Arc::new(TextColorUtility::new("text-black", "#000000")));
    registry.register(Arc::new(TextColorUtility::new(
        "text-transparent",
        "transparent",
    )));

    registry.register(Arc::new(BgColorUtility::new("bg-white", "#ffffff")));
    registry.register(Arc::new(BgColorUtility::new("bg-black", "#000000")));
    registry.register(Arc::new(BgColorUtility::new(
        "bg-transparent",
        "transparent",
    )));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_from_str() {
        assert_eq!(Breakpoint::from_str("sm"), Some(Breakpoint::Sm));
        assert_eq!(Breakpoint::from_str("md"), Some(Breakpoint::Md));
        assert_eq!(Breakpoint::from_str("lg"), Some(Breakpoint::Lg));
        assert_eq!(Breakpoint::from_str("xl"), Some(Breakpoint::Xl));
        assert_eq!(Breakpoint::from_str("2xl"), Some(Breakpoint::Xxl));
        assert_eq!(Breakpoint::from_str("invalid"), None);
    }

    #[test]
    fn test_breakpoint_min_width() {
        assert_eq!(Breakpoint::Sm.min_width(), "640px");
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert_eq!(Breakpoint::Lg.min_width(), "1024px");
        assert_eq!(Breakpoint::Xl.min_width(), "1280px");
        assert_eq!(Breakpoint::Xxl.min_width(), "1536px");
    }

    #[test]
    fn test_state_from_str() {
        assert_eq!(State::from_str("hover"), Some(State::Hover));
        assert_eq!(State::from_str("focus"), Some(State::Focus));
        assert_eq!(State::from_str("active"), Some(State::Active));
        assert_eq!(State::from_str("disabled"), Some(State::Disabled));
        assert_eq!(State::from_str("invalid"), None);
    }

    #[test]
    fn test_variant_parse() {
        assert_eq!(
            Variant::parse("sm:"),
            Some(Variant::Breakpoint(Breakpoint::Sm))
        );
        assert_eq!(Variant::parse("hover:"), Some(Variant::State(State::Hover)));
        assert_eq!(
            Variant::parse("md:hover:"),
            Some(Variant::Combined {
                breakpoint: Breakpoint::Md,
                state: State::Hover
            })
        );
        assert_eq!(Variant::parse("invalid:"), None);
    }

    #[test]
    fn test_variant_as_css_selector() {
        let bp_variant = Variant::Breakpoint(Breakpoint::Md);
        assert_eq!(
            bp_variant.as_css_selector(".test"),
            "@media (min-width: 768px) { .test }"
        );

        let state_variant = Variant::State(State::Hover);
        assert_eq!(state_variant.as_css_selector(".test"), ".test:hover");

        let combined = Variant::Combined {
            breakpoint: Breakpoint::Md,
            state: State::Hover,
        };
        assert_eq!(
            combined.as_css_selector(".test"),
            "@media (min-width: 768px) { .test:hover }"
        );
    }

    #[test]
    fn test_parsed_utility_basic() {
        let parsed = ParsedUtility::parse("p-4");
        assert_eq!(parsed.base, "p-4");
        assert!(parsed.variant.is_none());
        assert!(parsed.arbitrary_value.is_none());
    }

    #[test]
    fn test_parsed_utility_with_variant() {
        let parsed = ParsedUtility::parse("hover:p-4");
        assert_eq!(parsed.base, "p-4");
        assert_eq!(parsed.variant, Some(Variant::State(State::Hover)));
        assert!(parsed.arbitrary_value.is_none());
    }

    #[test]
    fn test_parsed_utility_with_arbitrary_value() {
        let parsed = ParsedUtility::parse("p-[10px]");
        assert_eq!(parsed.base, "p-[10px]");
        assert!(parsed.variant.is_none());
        assert_eq!(parsed.arbitrary_value, Some("10px".to_string()));
    }

    #[test]
    fn test_parsed_utility_with_variant_and_arbitrary() {
        let parsed = ParsedUtility::parse("md:p-[10px]");
        assert_eq!(parsed.base, "p-[10px]");
        assert_eq!(parsed.variant, Some(Variant::Breakpoint(Breakpoint::Md)));
        assert_eq!(parsed.arbitrary_value, Some("10px".to_string()));
    }

    #[test]
    fn test_padding_utility_matches() {
        let utility = PaddingUtility::new("p", CssProperty::Padding);
        assert!(utility.matches("p-4"));
        assert!(utility.matches("p-0"));
        assert!(!utility.matches("m-4"));
        assert!(!utility.matches("p4"));
    }

    #[test]
    fn test_padding_utility_generate_css() {
        let utility = PaddingUtility::new("p", CssProperty::Padding);
        let parsed = ParsedUtility::parse("p-4");

        let css = utility.generate_css("p-4", &parsed).unwrap();
        assert!(css.contains("padding:"));
        assert!(css.contains("1rem"));
    }

    #[test]
    fn test_padding_utility_with_variant() {
        let utility = PaddingUtility::new("p", CssProperty::Padding);
        let parsed = ParsedUtility::parse("hover:p-4");

        let css = utility.generate_css("hover:p-4", &parsed).unwrap();
        assert!(css.contains(":hover"));
        assert!(css.contains("padding:"));
    }

    #[test]
    fn test_padding_utility_with_arbitrary_value() {
        let utility = PaddingUtility::new("p", CssProperty::Padding);
        let parsed = ParsedUtility::parse("p-[10px]");

        let css = utility.generate_css("p-[10px]", &parsed).unwrap();
        assert!(css.contains("padding:"));
        assert!(css.contains("10px"));
    }

    #[test]
    fn test_margin_utility_auto() {
        let utility = MarginUtility::new("mx", CssProperty::MarginLeft);
        let parsed = ParsedUtility::parse("mx-auto");

        let css = utility.generate_css("mx-auto", &parsed).unwrap();
        assert!(css.contains("margin-left:"));
        assert!(css.contains("auto"));
    }

    #[test]
    fn test_display_utility() {
        let utility = DisplayUtility::new("flex");
        let parsed = ParsedUtility::parse("flex");

        let css = utility.generate_css("flex", &parsed).unwrap();
        assert!(css.contains("display:"));
        assert!(css.contains("flex"));
    }

    #[test]
    fn test_flex_direction_utility() {
        let utility = FlexDirectionUtility::new("flex-col", "column");
        let parsed = ParsedUtility::parse("flex-col");

        let css = utility.generate_css("flex-col", &parsed).unwrap();
        assert!(css.contains("flex-direction:"));
        assert!(css.contains("column"));
    }

    #[test]
    fn test_registry_find() {
        let registry = create_default_registry();
        assert!(registry.find("p-4").is_some());
        assert!(registry.find("flex").is_some());
        assert!(registry.find("text-center").is_some());
        assert!(registry.find("invalid-class").is_none());
    }

    #[test]
    fn test_registry_generate_css() {
        let registry = create_default_registry();

        let css = registry.generate_css("p-4").unwrap();
        assert!(css.contains("padding:"));

        let css = registry.generate_css("flex").unwrap();
        assert!(css.contains("display:"));
    }

    #[test]
    fn test_scale_to_rem() {
        assert_eq!(PaddingUtility::scale_to_rem("0"), Some("0".to_string()));
        assert_eq!(
            PaddingUtility::scale_to_rem("1"),
            Some("0.25rem".to_string())
        );
        assert_eq!(PaddingUtility::scale_to_rem("4"), Some("1rem".to_string()));
        assert_eq!(PaddingUtility::scale_to_rem("8"), Some("2rem".to_string()));
        assert_eq!(PaddingUtility::scale_to_rem("12"), Some("3rem".to_string()));
        assert_eq!(PaddingUtility::scale_to_rem("999"), None);
    }

    #[test]
    fn test_variant_prefix() {
        let bp = Variant::Breakpoint(Breakpoint::Md);
        assert_eq!(bp.as_prefix(), "md:");

        let st = Variant::State(State::Hover);
        assert_eq!(st.as_prefix(), "hover:");

        let combined = Variant::Combined {
            breakpoint: Breakpoint::Md,
            state: State::Hover,
        };
        assert_eq!(combined.as_prefix(), "md:hover:");
    }

    #[test]
    fn test_combined_variant_order() {
        // Both orders should work
        let variant1 = Variant::parse("hover:md:");
        assert!(variant1.is_some());

        let variant2 = Variant::parse("md:hover:");
        assert!(variant2.is_some());

        // Both should produce the same result
        if let Some(Variant::Combined { breakpoint, state }) = variant1 {
            assert_eq!(breakpoint, Breakpoint::Md);
            assert_eq!(state, State::Hover);
        }
    }

    #[test]
    fn test_font_size_utility() {
        let utility = FontSizeUtility::new("text-lg", "1.125rem");
        let parsed = ParsedUtility::parse("text-lg");

        let css = utility.generate_css("text-lg", &parsed).unwrap();
        assert!(css.contains("font-size:"));
        assert!(css.contains("1.125rem"));
    }

    #[test]
    fn test_font_weight_utility() {
        let utility = FontWeightUtility::new("font-bold", "700");
        let parsed = ParsedUtility::parse("font-bold");

        let css = utility.generate_css("font-bold", &parsed).unwrap();
        assert!(css.contains("font-weight:"));
        assert!(css.contains("700"));
    }

    #[test]
    fn test_text_align_utility() {
        let utility = TextAlignUtility::new("text-center", "center");
        let parsed = ParsedUtility::parse("text-center");

        let css = utility.generate_css("text-center", &parsed).unwrap();
        assert!(css.contains("text-align:"));
        assert!(css.contains("center"));
    }

    #[test]
    fn test_position_utility() {
        let utility = PositionUtility::new("absolute", "absolute");
        let parsed = ParsedUtility::parse("absolute");

        let css = utility.generate_css("absolute", &parsed).unwrap();
        assert!(css.contains("position:"));
        assert!(css.contains("absolute"));
    }

    #[test]
    fn test_text_color_utility() {
        let utility = TextColorUtility::new("text-white", "#ffffff");
        let parsed = ParsedUtility::parse("text-white");

        let css = utility.generate_css("text-white", &parsed).unwrap();
        assert!(css.contains("color:"));
        assert!(css.contains("#ffffff"));
    }

    #[test]
    fn test_bg_color_utility() {
        let utility = BgColorUtility::new("bg-black", "#000000");
        let parsed = ParsedUtility::parse("bg-black");

        let css = utility.generate_css("bg-black", &parsed).unwrap();
        assert!(css.contains("background-color:"));
        assert!(css.contains("#000000"));
    }

    #[test]
    fn test_utility_category() {
        let padding = PaddingUtility::new("p", CssProperty::Padding);
        assert_eq!(padding.category(), CssCategory::BoxModel);

        let display = DisplayUtility::new("flex");
        assert_eq!(display.category(), CssCategory::Layout);

        let flex_dir = FlexDirectionUtility::new("flex-col", "column");
        assert_eq!(flex_dir.category(), CssCategory::Flexbox);

        let text_align = TextAlignUtility::new("text-center", "center");
        assert_eq!(text_align.category(), CssCategory::Typography);

        let color = TextColorUtility::new("text-white", "#ffffff");
        assert_eq!(color.category(), CssCategory::Color);
    }

    #[test]
    fn test_complex_variant_parsing() {
        let parsed = ParsedUtility::parse("hover:md:text-center");
        assert_eq!(parsed.base, "text-center");
        assert!(parsed.variant.is_some());

        if let Some(Variant::Combined { breakpoint, state }) = parsed.variant {
            assert_eq!(breakpoint, Breakpoint::Md);
            assert_eq!(state, State::Hover);
        } else {
            panic!("Expected combined variant");
        }
    }

    #[test]
    fn test_arbitrary_value_with_unit() {
        let utility = PaddingUtility::new("p", CssProperty::Padding);
        let parsed = ParsedUtility::parse("p-[1.5rem]");

        let css = utility.generate_css("p-[1.5rem]", &parsed).unwrap();
        assert!(css.contains("padding:"));
        assert!(css.contains("1.5rem"));
    }

    #[test]
    fn test_px_special_case() {
        let parsed = ParsedUtility::parse("p-px");
        assert_eq!(parsed.base, "p-px");

        let utility = PaddingUtility::new("p", CssProperty::Padding);
        let css = utility.generate_css("p-px", &parsed).unwrap();
        assert!(css.contains("padding:"));
        assert!(css.contains("1px"));
    }

    #[test]
    fn test_default_registry_comprehensive() {
        let registry = create_default_registry();

        // Test that various utility categories are registered
        let test_classes = [
            "p-4",
            "m-2",
            "flex",
            "grid",
            "flex-col",
            "justify-center",
            "items-center",
            "text-center",
            "text-lg",
            "font-bold",
            "absolute",
            "text-white",
            "bg-black",
            "hidden",
        ];

        for class in test_classes {
            assert!(
                registry.find(class).is_some(),
                "Expected to find utility for class: {}",
                class
            );
        }
    }

    #[test]
    fn test_state_variants_comprehensive() {
        let states = [
            ("hover", State::Hover),
            ("focus", State::Focus),
            ("active", State::Active),
            ("focus-within", State::FocusWithin),
            ("focus-visible", State::FocusVisible),
            ("disabled", State::Disabled),
            ("checked", State::Checked),
            ("first", State::First),
            ("last", State::Last),
            ("odd", State::Odd),
            ("even", State::Even),
        ];

        for (str, expected) in states {
            assert_eq!(
                State::from_str(str),
                Some(expected),
                "Failed for state: {}",
                str
            );
        }
    }

    #[test]
    fn test_empty_and_invalid_inputs() {
        assert!(Variant::parse("").is_none());
        assert!(Variant::parse("::").is_none());
        assert!(ParsedUtility::parse("").base.is_empty());
    }
}
