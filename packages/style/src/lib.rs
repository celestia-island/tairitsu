//! Type-safe CSS property builders for Tairitsu framework.
//!
//! This crate provides builders for constructing CSS style strings and class lists
//! with compile-time validation and type safety.
//!
//! # CSS Properties
//!
//! The crate includes a comprehensive `CssProperty` enum covering 300+ W3C standard
//! CSS properties, automatically generated from the MDN Web Docs database.
//!
//! # Property Categories
//!
//! Properties are organized into categories:
//! - Layout: Positioning, display, overflow
//! - Box Model: Width, height, margin, padding, border
//! - Flexbox: Flex layout properties
//! - Grid: Grid layout properties
//! - Typography: Font and text properties
//! - Color: Color and background properties
//! - Visual: Filters, masks, effects
//! - Transform: 2D/3D transforms
//! - Transition: Transition properties
//! - Animation: Animation properties
//! - Interaction: User interaction properties
//! - And more...
//!
//! # Example
//!
//! ```rust
//! use tairitsu_style::{StyleStringBuilder, CssProperty, DisplayValue};
//!
//! let style = StyleStringBuilder::new()
//!     .add(CssProperty::Display, DisplayValue::Flex.as_str())
//!     .add(CssProperty::FlexDirection, "column")
//!     .add_px(CssProperty::Height, 100)
//!     .build_clean();
//! ```

mod builder;
mod classes;
mod properties;
mod typed;
mod utility;
mod values;

// Re-export public API
pub use builder::{StyleBuilder, StyleStringBuilder};
pub use classes::ClassesBuilder;
pub use properties::{CssCategory, CssProperty, Property};
pub use typed::TypedClass;
// Note: define_typed_classes! macro is automatically exported at crate root via #[macro_export]
pub use utility::create_default_registry;
pub use utility::{
    AlignItemsUtility, BgColorUtility, Breakpoint, DisplayUtility, FlexDirectionUtility,
    FlexWrapUtility, FontSizeUtility, FontWeightUtility, JustifyContentUtility, MarginUtility,
    PaddingUtility, ParsedUtility, PositionUtility, State, TextAlignUtility, TextColorUtility,
    UtilityClass, UtilityRegistry, Variant,
};
pub use values::{
    AlignItemsValue,
    CssBinOp,
    CssExpression,
    // CSS values exports
    CssLength,
    CssValue,
    CssValueParseError,
    CursorValue,
    DisplayValue,
    FlexDirectionValue,
    FlexWrapValue,
    JustifyContentValue,
    LengthUnit,
    OverflowValue,
    ParseResult,
    PositionValue,
    TextAlignValue,
};

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
            b.add(CssProperty::Width, "200px")
                .add_px(CssProperty::Height, 100)
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
        let classes = ClassesBuilder::new().add_all(&["a", "b", "c"]).build();
        assert_eq!(classes, "a b c");
    }

    #[test]
    fn test_style_builder_to_vdom() {
        let style = StyleBuilder::new()
            .add(CssProperty::Width, "100px")
            .to_vdom_style();
        assert!(style.to_string().contains("width"));
    }

    // New tests for value types and enhanced builders

    #[test]
    fn test_css_value_enums() {
        assert_eq!(CssValue::Auto.as_str(), "auto");
        assert_eq!(CssValue::None.as_str(), "none");
        assert_eq!(CssValue::Inherit.as_str(), "inherit");
        assert_eq!(CssValue::Initial.as_str(), "initial");
        assert_eq!(CssValue::Unset.as_str(), "unset");
        assert_eq!(CssValue::Revert.as_str(), "revert");
    }

    #[test]
    fn test_display_value_enums() {
        assert_eq!(DisplayValue::Flex.as_str(), "flex");
        assert_eq!(DisplayValue::Grid.as_str(), "grid");
        assert_eq!(DisplayValue::Block.as_str(), "block");
        assert_eq!(DisplayValue::InlineFlex.as_str(), "inline-flex");
        assert_eq!(DisplayValue::InlineGrid.as_str(), "inline-grid");
        assert_eq!(DisplayValue::None.as_str(), "none");
    }

    #[test]
    fn test_position_value_enums() {
        assert_eq!(PositionValue::Static.as_str(), "static");
        assert_eq!(PositionValue::Relative.as_str(), "relative");
        assert_eq!(PositionValue::Absolute.as_str(), "absolute");
        assert_eq!(PositionValue::Fixed.as_str(), "fixed");
        assert_eq!(PositionValue::Sticky.as_str(), "sticky");
    }

    #[test]
    fn test_length_unit_enums() {
        assert_eq!(LengthUnit::Px.as_str(), "px");
        assert_eq!(LengthUnit::Percent.as_str(), "%");
        assert_eq!(LengthUnit::Em.as_str(), "em");
        assert_eq!(LengthUnit::Rem.as_str(), "rem");
        assert_eq!(LengthUnit::Vw.as_str(), "vw");
        assert_eq!(LengthUnit::Vh.as_str(), "vh");
        assert_eq!(LengthUnit::Vmin.as_str(), "vmin");
        assert_eq!(LengthUnit::Vmax.as_str(), "vmax");
    }

    #[test]
    fn test_style_string_builder_add_auto() {
        let style = StyleStringBuilder::new()
            .add_auto(CssProperty::Width)
            .add_auto(CssProperty::Height)
            .build_clean();

        assert!(style.contains("width:auto"));
        assert!(style.contains("height:auto"));
    }

    #[test]
    fn test_style_string_builder_add_none() {
        let style = StyleStringBuilder::new()
            .add_none(CssProperty::Border)
            .add_none(CssProperty::Display)
            .build_clean();

        assert!(style.contains("border:none"));
        assert!(style.contains("display:none"));
    }

    #[test]
    fn test_style_string_builder_add_inherit() {
        let style = StyleStringBuilder::new()
            .add_inherit(CssProperty::Color)
            .add_inherit(CssProperty::FontSize)
            .build_clean();

        assert!(style.contains("color:inherit"));
        assert!(style.contains("font-size:inherit"));
    }

    #[test]
    fn test_style_string_builder_add_percent() {
        let style = StyleStringBuilder::new()
            .add_percent(CssProperty::Width, 100)
            .add_percent(CssProperty::MaxWidth, 50)
            .build_clean();

        assert!(style.contains("width:100%"));
        assert!(style.contains("max-width:50%"));
    }

    #[test]
    fn test_style_string_builder_add_em() {
        let style = StyleStringBuilder::new()
            .add_em(CssProperty::FontSize, 16)
            .add_em(CssProperty::MarginTop, 2)
            .build_clean();

        assert!(style.contains("font-size:16em"));
        assert!(style.contains("margin-top:2em"));
    }

    #[test]
    fn test_style_string_builder_add_rem() {
        let style = StyleStringBuilder::new()
            .add_rem(CssProperty::FontSize, 14)
            .add_rem(CssProperty::Padding, 10)
            .build_clean();

        assert!(style.contains("font-size:14rem"));
        assert!(style.contains("padding:10rem"));
    }

    #[test]
    fn test_style_string_builder_add_vw() {
        let style = StyleStringBuilder::new()
            .add_vw(CssProperty::Width, 100)
            .add_vw(CssProperty::MaxWidth, 50)
            .build_clean();

        assert!(style.contains("width:100vw"));
        assert!(style.contains("max-width:50vw"));
    }

    #[test]
    fn test_style_string_builder_add_vh() {
        let style = StyleStringBuilder::new()
            .add_vh(CssProperty::Height, 100)
            .add_vh(CssProperty::MinHeight, 50)
            .build_clean();

        assert!(style.contains("height:100vh"));
        assert!(style.contains("min-height:50vh"));
    }

    #[test]
    fn test_style_string_builder_add_percent_f64() {
        let style = StyleStringBuilder::new()
            .add_percent_f64(CssProperty::Width, 100.5)
            .add_percent_f64(CssProperty::MaxWidth, 33.33)
            .build_clean();

        assert!(style.contains("width:100.5%"));
        assert!(style.contains("max-width:33.33%"));
    }

    #[test]
    fn test_style_string_builder_add_em_f64() {
        let style = StyleStringBuilder::new()
            .add_em_f64(CssProperty::FontSize, 1.5)
            .add_em_f64(CssProperty::LineHeight, 1.2)
            .build_clean();

        assert!(style.contains("font-size:1.5em"));
        assert!(style.contains("line-height:1.2em"));
    }

    #[test]
    fn test_style_string_builder_add_rem_f64() {
        let style = StyleStringBuilder::new()
            .add_rem_f64(CssProperty::FontSize, 1.25)
            .add_rem_f64(CssProperty::Margin, 1.5)
            .build_clean();

        assert!(style.contains("font-size:1.25rem"));
        assert!(style.contains("margin:1.5rem"));
    }

    #[test]
    fn test_style_string_builder_add_vw_f64() {
        let style = StyleStringBuilder::new()
            .add_vw_f64(CssProperty::Width, 50.5)
            .add_vw_f64(CssProperty::MaxWidth, 33.33)
            .build_clean();

        assert!(style.contains("width:50.5vw"));
        assert!(style.contains("max-width:33.33vw"));
    }

    #[test]
    fn test_style_string_builder_add_vh_f64() {
        let style = StyleStringBuilder::new()
            .add_vh_f64(CssProperty::Height, 100.0)
            .add_vh_f64(CssProperty::MinHeight, 50.75)
            .build_clean();

        assert!(style.contains("height:100vh"));
        assert!(style.contains("min-height:50.75vh"));
    }

    #[test]
    fn test_style_string_builder_combined() {
        let style = StyleStringBuilder::new()
            .add(CssProperty::Display, DisplayValue::Flex.as_str())
            .add_auto(CssProperty::Width)
            .add_percent(CssProperty::MaxWidth, 100)
            .add_px(CssProperty::Height, 300)
            .add_rem(CssProperty::Padding, 16)
            .build_clean();

        assert!(style.contains("display:flex"));
        assert!(style.contains("width:auto"));
        assert!(style.contains("max-width:100%"));
        assert!(style.contains("height:300px"));
        assert!(style.contains("padding:16rem"));
    }

    #[test]
    fn test_style_builder_add_auto() {
        let style = StyleBuilder::new()
            .add_auto(CssProperty::Width)
            .add_auto(CssProperty::Height)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("width:auto"));
        assert!(style_str.contains("height:auto"));
    }

    #[test]
    fn test_style_builder_add_none() {
        let style = StyleBuilder::new()
            .add_none(CssProperty::Border)
            .add_none(CssProperty::Display)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("border:none"));
        assert!(style_str.contains("display:none"));
    }

    #[test]
    fn test_style_builder_add_inherit() {
        let style = StyleBuilder::new()
            .add_inherit(CssProperty::Color)
            .add_inherit(CssProperty::FontSize)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("color:inherit"));
        assert!(style_str.contains("font-size:inherit"));
    }

    #[test]
    fn test_style_builder_add_percent() {
        let style = StyleBuilder::new()
            .add_percent(CssProperty::Width, 100)
            .add_percent(CssProperty::MaxWidth, 50)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("width:100%"));
        assert!(style_str.contains("max-width:50%"));
    }

    #[test]
    fn test_style_builder_add_em() {
        let style = StyleBuilder::new()
            .add_em(CssProperty::FontSize, 16)
            .add_em(CssProperty::MarginTop, 2)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("font-size:16em"));
        assert!(style_str.contains("margin-top:2em"));
    }

    #[test]
    fn test_style_builder_add_rem() {
        let style = StyleBuilder::new()
            .add_rem(CssProperty::FontSize, 14)
            .add_rem(CssProperty::Padding, 10)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("font-size:14rem"));
        assert!(style_str.contains("padding:10rem"));
    }

    #[test]
    fn test_style_builder_add_vw() {
        let style = StyleBuilder::new()
            .add_vw(CssProperty::Width, 100)
            .add_vw(CssProperty::MaxWidth, 50)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("width:100vw"));
        assert!(style_str.contains("max-width:50vw"));
    }

    #[test]
    fn test_style_builder_add_vh() {
        let style = StyleBuilder::new()
            .add_vh(CssProperty::Height, 100)
            .add_vh(CssProperty::MinHeight, 50)
            .to_vdom_style();
        let style_str = style.to_string();
        assert!(style_str.contains("height:100vh"));
        assert!(style_str.contains("min-height:50vh"));
    }

    #[test]
    fn test_flex_direction_value_enums() {
        assert_eq!(FlexDirectionValue::Row.as_str(), "row");
        assert_eq!(FlexDirectionValue::Column.as_str(), "column");
        assert_eq!(FlexDirectionValue::RowReverse.as_str(), "row-reverse");
        assert_eq!(FlexDirectionValue::ColumnReverse.as_str(), "column-reverse");
    }

    #[test]
    fn test_flex_wrap_value_enums() {
        assert_eq!(FlexWrapValue::Nowrap.as_str(), "nowrap");
        assert_eq!(FlexWrapValue::Wrap.as_str(), "wrap");
        assert_eq!(FlexWrapValue::WrapReverse.as_str(), "wrap-reverse");
    }

    #[test]
    fn test_justify_content_value_enums() {
        assert_eq!(JustifyContentValue::Center.as_str(), "center");
        assert_eq!(JustifyContentValue::SpaceBetween.as_str(), "space-between");
        assert_eq!(JustifyContentValue::SpaceEvenly.as_str(), "space-evenly");
        assert_eq!(JustifyContentValue::FlexStart.as_str(), "flex-start");
        assert_eq!(JustifyContentValue::FlexEnd.as_str(), "flex-end");
    }

    #[test]
    fn test_align_items_value_enums() {
        assert_eq!(AlignItemsValue::Center.as_str(), "center");
        assert_eq!(AlignItemsValue::Stretch.as_str(), "stretch");
        assert_eq!(AlignItemsValue::Baseline.as_str(), "baseline");
        assert_eq!(AlignItemsValue::FlexStart.as_str(), "flex-start");
        assert_eq!(AlignItemsValue::FlexEnd.as_str(), "flex-end");
    }

    #[test]
    fn test_text_align_value_enums() {
        assert_eq!(TextAlignValue::Left.as_str(), "left");
        assert_eq!(TextAlignValue::Right.as_str(), "right");
        assert_eq!(TextAlignValue::Center.as_str(), "center");
        assert_eq!(TextAlignValue::Justify.as_str(), "justify");
        assert_eq!(TextAlignValue::Start.as_str(), "start");
        assert_eq!(TextAlignValue::End.as_str(), "end");
    }

    #[test]
    fn test_overflow_value_enums() {
        assert_eq!(OverflowValue::Visible.as_str(), "visible");
        assert_eq!(OverflowValue::Hidden.as_str(), "hidden");
        assert_eq!(OverflowValue::Scroll.as_str(), "scroll");
        assert_eq!(OverflowValue::Auto.as_str(), "auto");
        assert_eq!(OverflowValue::Clip.as_str(), "clip");
    }

    #[test]
    fn test_cursor_value_enums() {
        assert_eq!(CursorValue::Pointer.as_str(), "pointer");
        assert_eq!(CursorValue::Grab.as_str(), "grab");
        assert_eq!(CursorValue::NotAllowed.as_str(), "not-allowed");
        assert_eq!(CursorValue::Help.as_str(), "help");
        assert_eq!(CursorValue::Text.as_str(), "text");
        assert_eq!(CursorValue::Move.as_str(), "move");
    }

    #[test]
    fn test_complete_flex_layout() {
        let style = StyleStringBuilder::new()
            .add(CssProperty::Display, DisplayValue::Flex.as_str())
            .add(CssProperty::FlexDirection, FlexDirectionValue::Row.as_str())
            .add(
                CssProperty::JustifyContent,
                JustifyContentValue::SpaceBetween.as_str(),
            )
            .add(CssProperty::AlignItems, AlignItemsValue::Center.as_str())
            .add_px(CssProperty::Gap, 16)
            .add_auto(CssProperty::Width)
            .add_vh(CssProperty::MinHeight, 100)
            .build_clean();

        assert!(style.contains("display:flex"));
        assert!(style.contains("flex-direction:row"));
        assert!(style.contains("justify-content:space-between"));
        assert!(style.contains("align-items:center"));
        assert!(style.contains("gap:16px"));
        assert!(style.contains("width:auto"));
        assert!(style.contains("min-height:100vh"));
    }

    #[test]
    fn test_complete_grid_layout() {
        let style = StyleStringBuilder::new()
            .add(CssProperty::Display, DisplayValue::Grid.as_str())
            .add(CssProperty::GridTemplateColumns, "repeat(3, 1fr)")
            .add(CssProperty::GridTemplateRows, "auto 1fr auto")
            .add_px(CssProperty::Gap, 20)
            .add_percent(CssProperty::Width, 100)
            .build_clean();

        assert!(style.contains("display:grid"));
        assert!(style.contains("grid-template-columns:repeat(3, 1fr)"));
        assert!(style.contains("grid-template-rows:auto 1fr auto"));
        assert!(style.contains("gap:20px"));
        assert!(style.contains("width:100%"));
    }

    #[test]
    fn test_responsive_design() {
        let style = StyleStringBuilder::new()
            .add_percent(CssProperty::Width, 100)
            .add_percent(CssProperty::MaxWidth, 1200)
            .add_auto(CssProperty::MarginLeft)
            .add_auto(CssProperty::MarginRight)
            .add_px(CssProperty::PaddingLeft, 16)
            .add_px(CssProperty::PaddingRight, 16)
            .build_clean();

        assert!(style.contains("width:100%"));
        assert!(style.contains("max-width:1200%"));
        assert!(style.contains("margin-left:auto"));
        assert!(style.contains("margin-right:auto"));
        assert!(style.contains("padding-left:16px"));
        assert!(style.contains("padding-right:16px"));
    }

    #[test]
    fn test_positioning() {
        let style = StyleStringBuilder::new()
            .add(CssProperty::Position, PositionValue::Relative.as_str())
            .add_inherit(CssProperty::Top)
            .add_auto(CssProperty::Left)
            .add(CssProperty::ZIndex, "10")
            .build_clean();

        assert!(style.contains("position:relative"));
        assert!(style.contains("top:inherit"));
        assert!(style.contains("left:auto"));
        assert!(style.contains("z-index:10"));
    }

    #[test]
    fn test_spacing_with_multiple_units() {
        let style = StyleStringBuilder::new()
            .add_px(CssProperty::Margin, 16)
            .add_rem(CssProperty::Padding, 14)
            .add_em(CssProperty::FontSize, 16)
            .add_percent(CssProperty::Width, 100)
            .add_vw(CssProperty::Height, 50)
            .add_vh(CssProperty::MinHeight, 100)
            .build_clean();

        assert!(style.contains("margin:16px"));
        assert!(style.contains("padding:14rem"));
        assert!(style.contains("font-size:16em"));
        assert!(style.contains("width:100%"));
        assert!(style.contains("height:50vw"));
        assert!(style.contains("min-height:100vh"));
    }

    // UtilityClass integration tests

    #[test]
    fn test_classes_builder_with_utilities() {
        let classes = ClassesBuilder::new()
            .add_utility("p-4")
            .add_utility("flex")
            .add_utility("text-center")
            .build();

        assert!(classes.contains("p-4"));
        assert!(classes.contains("flex"));
        assert!(classes.contains("text-center"));
    }

    #[test]
    fn test_classes_builder_utility_if() {
        let classes = ClassesBuilder::new()
            .add_utility_if("p-4", true)
            .add_utility_if("m-4", false)
            .build();

        assert!(classes.contains("p-4"));
        assert!(!classes.contains("m-4"));
    }

    #[test]
    fn test_classes_builder_add_utilities() {
        let classes = ClassesBuilder::new()
            .add_utilities("p-4 flex text-center")
            .build();

        assert!(classes.contains("p-4"));
        assert!(classes.contains("flex"));
        assert!(classes.contains("text-center"));
    }

    #[test]
    fn test_classes_builder_from_utilities() {
        let classes = ClassesBuilder::from_utilities("flex p-4 text-center").build();
        assert!(classes.contains("flex"));
        assert!(classes.contains("p-4"));
        assert!(classes.contains("text-center"));
    }

    #[test]
    fn test_classes_builder_add_mixed() {
        let classes = ClassesBuilder::new()
            .add_mixed("custom-class p-4 flex")
            .build();

        assert!(classes.contains("custom-class"));
        assert!(classes.contains("p-4"));
        assert!(classes.contains("flex"));
    }

    #[test]
    fn test_classes_builder_generate_css() {
        let builder = ClassesBuilder::new().add_utility("p-4").add_utility("flex");

        let css = builder.generate_css();
        assert!(css.contains("padding"));
        assert!(css.contains("display"));
    }

    #[test]
    fn test_classes_builder_generate_stylesheet() {
        let builder = ClassesBuilder::new().add_utility("p-4").add_utility("flex");

        let stylesheet = builder.generate_stylesheet();
        assert!(stylesheet.contains("/* Utility Classes */"));
        assert!(stylesheet.contains("padding"));
        assert!(stylesheet.contains("display"));
    }

    #[test]
    fn test_classes_builder_with_variants() {
        let classes = ClassesBuilder::new()
            .add_utility("p-4")
            .add_utility("hover:text-center")
            .add_utility("md:flex")
            .build();

        assert!(classes.contains("p-4"));
        assert!(classes.contains("hover:text-center"));
        assert!(classes.contains("md:flex"));
    }

    #[test]
    fn test_classes_builder_with_arbitrary_values() {
        let classes = ClassesBuilder::new()
            .add_utility("p-[10px]")
            .add_utility("m-[1.5rem]")
            .build();

        assert!(classes.contains("p-[10px]"));
        assert!(classes.contains("m-[1.5rem]"));
    }

    #[test]
    fn test_utility_with_css_generation() {
        let (builder, css) = ClassesBuilder::new().add_utility_with_css("p-4");

        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("padding"));
        assert!(css.contains("1rem"));

        let classes = builder.build();
        assert!(classes.contains("p-4"));
    }

    #[test]
    fn test_combined_responsive_and_state_variants() {
        let classes = ClassesBuilder::new().add_utility("md:hover:flex").build();

        assert!(classes.contains("md:hover:flex"));
    }

    #[test]
    fn test_spacing_utilities_comprehensive() {
        let test_cases = [
            ("p-0", "padding:0"),
            ("p-4", "padding:1rem"),
            ("px-4", "padding-left"),
            ("py-4", "padding-top"),
            ("pt-4", "padding-top"),
            ("pr-4", "padding-right"),
            ("pb-4", "padding-bottom"),
            ("pl-4", "padding-left"),
        ];

        for (class, expected) in test_cases {
            let (_, css) = ClassesBuilder::new().add_utility_with_css(class);
            assert!(
                css.is_some(),
                "Expected CSS to be generated for class: {}",
                class
            );
            let css = css.unwrap();
            assert!(
                css.contains(expected),
                "Expected CSS for {} to contain {}",
                class,
                expected
            );
        }
    }

    #[test]
    fn test_margin_utilities_with_auto() {
        let (_, css) = ClassesBuilder::new().add_utility_with_css("mx-auto");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("margin-left"));
        assert!(css.contains("auto"));
    }

    #[test]
    fn test_layout_utilities() {
        let test_cases = [
            ("flex", "display:flex"),
            ("grid", "display:grid"),
            ("block", "display:block"),
            ("inline-block", "display:inline-block"),
            ("hidden", "display:none"),
        ];

        for (class, expected) in test_cases {
            let (_, css) = ClassesBuilder::new().add_utility_with_css(class);
            assert!(
                css.is_some(),
                "Expected CSS to be generated for class: {}",
                class
            );
            let css = css.unwrap();
            assert!(
                css.contains(expected),
                "Expected CSS for {} to contain {}",
                class,
                expected
            );
        }
    }

    #[test]
    fn test_flexbox_utilities() {
        let test_cases = [
            ("flex-row", "flex-direction:row"),
            ("flex-col", "flex-direction:column"),
            ("flex-wrap", "flex-wrap:wrap"),
            ("flex-nowrap", "flex-wrap:nowrap"),
            ("justify-center", "justify-content:center"),
            ("justify-between", "justify-content:space-between"),
            ("items-center", "align-items:center"),
            ("items-start", "align-items:flex-start"),
        ];

        for (class, expected) in test_cases {
            let (_, css) = ClassesBuilder::new().add_utility_with_css(class);
            assert!(
                css.is_some(),
                "Expected CSS to be generated for class: {}",
                class
            );
            let css = css.unwrap();
            assert!(
                css.contains(expected),
                "Expected CSS for {} to contain {}",
                class,
                expected
            );
        }
    }

    #[test]
    fn test_typography_utilities() {
        let test_cases = [
            ("text-left", "text-align:left"),
            ("text-center", "text-align:center"),
            ("text-right", "text-align:right"),
            ("text-lg", "font-size:1.125rem"),
            ("text-xl", "font-size:1.25rem"),
            ("font-bold", "font-weight:700"),
            ("font-normal", "font-weight:400"),
        ];

        for (class, expected) in test_cases {
            let (_, css) = ClassesBuilder::new().add_utility_with_css(class);
            assert!(
                css.is_some(),
                "Expected CSS to be generated for class: {}",
                class
            );
            let css = css.unwrap();
            assert!(
                css.contains(expected),
                "Expected CSS for {} to contain {}",
                class,
                expected
            );
        }
    }

    #[test]
    fn test_position_utilities() {
        let test_cases = [
            ("static", "position:static"),
            ("fixed", "position:fixed"),
            ("absolute", "position:absolute"),
            ("relative", "position:relative"),
            ("sticky", "position:sticky"),
        ];

        for (class, expected) in test_cases {
            let (_, css) = ClassesBuilder::new().add_utility_with_css(class);
            assert!(
                css.is_some(),
                "Expected CSS to be generated for class: {}",
                class
            );
            let css = css.unwrap();
            assert!(
                css.contains(expected),
                "Expected CSS for {} to contain {}",
                class,
                expected
            );
        }
    }

    #[test]
    fn test_color_utilities() {
        let (_, css) = ClassesBuilder::new().add_utility_with_css("text-white");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("color:#ffffff"));

        let (_, css) = ClassesBuilder::new().add_utility_with_css("bg-black");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("background-color:#000000"));
    }

    #[test]
    fn test_invalid_utility_class() {
        let (_, css) = ClassesBuilder::new().add_utility_with_css("invalid-class");
        assert!(css.is_none());
    }

    #[test]
    fn test_empty_utility_class() {
        let (_, css) = ClassesBuilder::new().add_utility_with_css("");
        assert!(css.is_none());
    }

    #[test]
    fn test_variant_css_generation() {
        // Test responsive variant
        let (_, css) = ClassesBuilder::new().add_utility_with_css("md:p-4");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("@media"));
        assert!(css.contains("min-width: 768px"));
        assert!(css.contains("padding"));

        // Test state variant
        let (_, css) = ClassesBuilder::new().add_utility_with_css("hover:text-center");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains(":hover"));

        // Test combined variant
        let (_, css) = ClassesBuilder::new().add_utility_with_css("md:hover:flex");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("@media"));
        assert!(css.contains(":hover"));
    }

    #[test]
    fn test_arbitrary_value_css_generation() {
        let (_, css) = ClassesBuilder::new().add_utility_with_css("p-[10px]");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("padding"));
        assert!(css.contains("10px"));

        let (_, css) = ClassesBuilder::new().add_utility_with_css("m-[1.5rem]");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("margin"));
        assert!(css.contains("1.5rem"));
    }

    #[test]
    fn test_utility_categories() {
        let registry = create_default_registry();

        // Test that utilities have correct categories
        let p_utility = registry.find("p-4").unwrap();
        assert_eq!(p_utility.category(), CssCategory::BoxModel);

        let flex_utility = registry.find("flex").unwrap();
        assert_eq!(flex_utility.category(), CssCategory::Layout);

        let text_utility = registry.find("text-center").unwrap();
        assert_eq!(text_utility.category(), CssCategory::Typography);
    }

    #[test]
    fn test_custom_registry() {
        let custom_registry = UtilityRegistry::new();
        let builder = ClassesBuilder::with_registry(custom_registry);

        // Custom registry starts empty
        let classes = builder.add_utility("p-4").build();
        assert!(!classes.contains("p-4"));
    }

    #[test]
    fn test_register_custom_utility() {
        use std::sync::Arc;

        // Create a simple custom utility
        struct CustomUtility;
        impl UtilityClass for CustomUtility {
            fn pattern(&self) -> &'static str {
                "custom"
            }

            fn generate_css(&self, class_name: &str, _parsed: &ParsedUtility) -> Option<String> {
                Some(format!(".{} {{ custom: prop; }}", class_name))
            }

            fn matches(&self, class_name: &str) -> bool {
                class_name == "custom"
            }

            fn property(&self) -> Property {
                Property::Custom("custom".to_string())
            }

            fn category(&self) -> CssCategory {
                CssCategory::Miscellaneous
            }
        }

        let builder = ClassesBuilder::new()
            .register_utility(Arc::new(CustomUtility))
            .add_utility("custom");

        let classes = builder.build();
        assert!(classes.contains("custom"));

        let (_, css) = ClassesBuilder::new()
            .register_utility(Arc::new(CustomUtility))
            .add_utility_with_css("custom");
        assert!(css.is_some());
    }

    #[test]
    fn test_comprehensive_class_combination() {
        let classes = ClassesBuilder::new()
            .add("container")
            .add_utility("flex")
            .add_if("active", true)
            .add_utility_if("p-4", true)
            .add_utility_if("m-4", false)
            .add_utilities("text-center justify-center")
            .add_mixed("custom-class items-center")
            .build();

        assert!(classes.contains("container"));
        assert!(classes.contains("flex"));
        assert!(classes.contains("active"));
        assert!(classes.contains("p-4"));
        assert!(!classes.contains("m-4"));
        assert!(classes.contains("text-center"));
        assert!(classes.contains("justify-center"));
        assert!(classes.contains("custom-class"));
        assert!(classes.contains("items-center"));
    }

    #[test]
    fn test_utility_property_access() {
        let registry = create_default_registry();

        let padding_util = registry.find("p-4").unwrap();
        if let Property::Known(prop) = padding_util.property() {
            assert_eq!(prop, CssProperty::Padding);
        } else {
            panic!("Expected Known property");
        }

        let display_util = registry.find("flex").unwrap();
        if let Property::Known(prop) = display_util.property() {
            assert_eq!(prop, CssProperty::Display);
        } else {
            panic!("Expected Known property");
        }
    }
}

#[cfg(target_family = "wasm")]
mod wasm_export {
    wit_bindgen::generate!({
        path: "wit",
        world: "style",
    });

    pub struct StyleExports;

    impl exports::tairitsu::style::version::Guest for StyleExports {
        fn get_version() -> String {
            env!("CARGO_PKG_VERSION").to_string()
        }
    }

    impl exports::tairitsu::style::builder::Guest for StyleExports {
        fn build_style_string(declarations: Vec<(String, String)>) -> String {
            let mut builder = crate::StyleStringBuilder::new();
            for (prop, val) in declarations {
                builder = builder.add_custom(&prop, &val);
            }
            builder.build()
        }

        fn css_property_to_string(name: String) -> String {
            match crate::Property::from(name.as_str()) {
                crate::Property::Known(p) => p.as_str().to_string(),
                crate::Property::Custom(s) => s,
            }
        }

        fn css_length_to_string(value: f64, unit: String) -> String {
            use crate::LengthUnit;
            let u = match unit.as_str() {
                "%" => "%",
                s => LengthUnit::from_str(s).map_or(s, |u| u.as_str()),
            };
            format!("{}{}", value, u)
        }
    }

    impl exports::tairitsu::style::utility::Guest for StyleExports {
        fn generate_utility_css(class_name: String) -> Option<String> {
            crate::create_default_registry().generate_css(&class_name)
        }

        fn generate_stylesheet(class_names: Vec<String>) -> String {
            let mut builder = crate::ClassesBuilder::new();
            for name in &class_names {
                builder = builder.add_utility(name);
            }
            builder.generate_stylesheet()
        }

        fn parse_utility_base(class_name: String) -> String {
            crate::ParsedUtility::parse(&class_name).base
        }
    }

    impl exports::tairitsu::style::classes::Guest for StyleExports {
        fn build_classes(classes: Vec<String>) -> String {
            let mut builder = crate::ClassesBuilder::new();
            for c in &classes {
                builder = builder.add(c);
            }
            builder.build()
        }

        fn build_classes_conditional(classes: Vec<(String, bool)>) -> String {
            let mut builder = crate::ClassesBuilder::new();
            for (c, cond) in &classes {
                builder = builder.add_if(c, *cond);
            }
            builder.build()
        }
    }

    export!(StyleExports);
}
