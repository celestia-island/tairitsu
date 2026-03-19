//! CSS value type enums for type-safe style construction.
//!
//! This module provides strongly-typed enums for common CSS values,
//! reducing string typos and improving code maintainability.

/// Common CSS global and keyword values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssValue {
    /// Automatic value (browser calculates)
    Auto,
    /// No value (none/transparent)
    None,
    /// Inherit from parent element
    Inherit,
    /// Use initial value from CSS spec
    Initial,
    /// Unset the property (acts as inherit or initial)
    Unset,
    /// Revert to user agent or user stylesheet
    Revert,
}

impl CssValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            CssValue::Auto => "auto",
            CssValue::None => "none",
            CssValue::Inherit => "inherit",
            CssValue::Initial => "initial",
            CssValue::Unset => "unset",
            CssValue::Revert => "revert",
        }
    }
}

/// CSS display property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayValue {
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    None,
    Contents,
    ListItem,
    Table,
    InlineTable,
    TableCaption,
    TableCell,
    TableColumn,
    TableColumnGroup,
    TableFooterGroup,
    TableHeaderGroup,
    TableRow,
    TableRowGroup,
    FlowRoot,
}

impl DisplayValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayValue::Block => "block",
            DisplayValue::Inline => "inline",
            DisplayValue::InlineBlock => "inline-block",
            DisplayValue::Flex => "flex",
            DisplayValue::InlineFlex => "inline-flex",
            DisplayValue::Grid => "grid",
            DisplayValue::InlineGrid => "inline-grid",
            DisplayValue::None => "none",
            DisplayValue::Contents => "contents",
            DisplayValue::ListItem => "list-item",
            DisplayValue::Table => "table",
            DisplayValue::InlineTable => "inline-table",
            DisplayValue::TableCaption => "table-caption",
            DisplayValue::TableCell => "table-cell",
            DisplayValue::TableColumn => "table-column",
            DisplayValue::TableColumnGroup => "table-column-group",
            DisplayValue::TableFooterGroup => "table-footer-group",
            DisplayValue::TableHeaderGroup => "table-header-group",
            DisplayValue::TableRow => "table-row",
            DisplayValue::TableRowGroup => "table-row-group",
            DisplayValue::FlowRoot => "flow-root",
        }
    }
}

/// CSS position property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionValue {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl PositionValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            PositionValue::Static => "static",
            PositionValue::Relative => "relative",
            PositionValue::Absolute => "absolute",
            PositionValue::Fixed => "fixed",
            PositionValue::Sticky => "sticky",
        }
    }
}

/// CSS flex-direction property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexDirectionValue {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirectionValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlexDirectionValue::Row => "row",
            FlexDirectionValue::RowReverse => "row-reverse",
            FlexDirectionValue::Column => "column",
            FlexDirectionValue::ColumnReverse => "column-reverse",
        }
    }
}

/// CSS flex-wrap property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexWrapValue {
    Nowrap,
    Wrap,
    WrapReverse,
}

impl FlexWrapValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlexWrapValue::Nowrap => "nowrap",
            FlexWrapValue::Wrap => "wrap",
            FlexWrapValue::WrapReverse => "wrap-reverse",
        }
    }
}

/// CSS justify-content property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JustifyContentValue {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Start,
    End,
    Left,
    Right,
    Stretch,
}

impl JustifyContentValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            JustifyContentValue::FlexStart => "flex-start",
            JustifyContentValue::FlexEnd => "flex-end",
            JustifyContentValue::Center => "center",
            JustifyContentValue::SpaceBetween => "space-between",
            JustifyContentValue::SpaceAround => "space-around",
            JustifyContentValue::SpaceEvenly => "space-evenly",
            JustifyContentValue::Start => "start",
            JustifyContentValue::End => "end",
            JustifyContentValue::Left => "left",
            JustifyContentValue::Right => "right",
            JustifyContentValue::Stretch => "stretch",
        }
    }
}

/// CSS align-items property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignItemsValue {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
    SelfStart,
    SelfEnd,
}

impl AlignItemsValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlignItemsValue::FlexStart => "flex-start",
            AlignItemsValue::FlexEnd => "flex-end",
            AlignItemsValue::Center => "center",
            AlignItemsValue::Baseline => "baseline",
            AlignItemsValue::Stretch => "stretch",
            AlignItemsValue::Start => "start",
            AlignItemsValue::End => "end",
            AlignItemsValue::SelfStart => "self-start",
            AlignItemsValue::SelfEnd => "self-end",
        }
    }
}

/// CSS text-align property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAlignValue {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

impl TextAlignValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextAlignValue::Left => "left",
            TextAlignValue::Right => "right",
            TextAlignValue::Center => "center",
            TextAlignValue::Justify => "justify",
            TextAlignValue::Start => "start",
            TextAlignValue::End => "end",
        }
    }
}

/// CSS overflow property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowValue {
    Visible,
    Hidden,
    Clip,
    Scroll,
    Auto,
}

impl OverflowValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            OverflowValue::Visible => "visible",
            OverflowValue::Hidden => "hidden",
            OverflowValue::Clip => "clip",
            OverflowValue::Scroll => "scroll",
            OverflowValue::Auto => "auto",
        }
    }
}

/// CSS cursor property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorValue {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ColResize,
    RowResize,
    NResize,
    EResize,
    SResize,
    WResize,
    NeResize,
    NwResize,
    SeResize,
    SwResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ZoomIn,
    ZoomOut,
}

impl CursorValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            CursorValue::Auto => "auto",
            CursorValue::Default => "default",
            CursorValue::None => "none",
            CursorValue::ContextMenu => "context-menu",
            CursorValue::Help => "help",
            CursorValue::Pointer => "pointer",
            CursorValue::Progress => "progress",
            CursorValue::Wait => "wait",
            CursorValue::Cell => "cell",
            CursorValue::Crosshair => "crosshair",
            CursorValue::Text => "text",
            CursorValue::VerticalText => "vertical-text",
            CursorValue::Alias => "alias",
            CursorValue::Copy => "copy",
            CursorValue::Move => "move",
            CursorValue::NoDrop => "no-drop",
            CursorValue::NotAllowed => "not-allowed",
            CursorValue::Grab => "grab",
            CursorValue::Grabbing => "grabbing",
            CursorValue::AllScroll => "all-scroll",
            CursorValue::ColResize => "col-resize",
            CursorValue::RowResize => "row-resize",
            CursorValue::NResize => "n-resize",
            CursorValue::EResize => "e-resize",
            CursorValue::SResize => "s-resize",
            CursorValue::WResize => "w-resize",
            CursorValue::NeResize => "ne-resize",
            CursorValue::NwResize => "nw-resize",
            CursorValue::SeResize => "se-resize",
            CursorValue::SwResize => "sw-resize",
            CursorValue::EwResize => "ew-resize",
            CursorValue::NsResize => "ns-resize",
            CursorValue::NeswResize => "nesw-resize",
            CursorValue::NwseResize => "nwse-resize",
            CursorValue::ZoomIn => "zoom-in",
            CursorValue::ZoomOut => "zoom-out",
        }
    }
}

/// Length unit types for numeric CSS values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnit {
    /// Pixels
    Px,
    /// Percentage
    Percent,
    /// Font size of the element
    Em,
    /// Font size of the root element
    Rem,
    /// Viewport width
    Vw,
    /// Viewport height
    Vh,
    /// Viewport smaller dimension
    Vmin,
    /// Viewport larger dimension
    Vmax,
    /// Character width (X-height)
    Ch,
    /// Zero-width character advance
    Ex,
}

impl LengthUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            LengthUnit::Px => "px",
            LengthUnit::Percent => "%",
            LengthUnit::Em => "em",
            LengthUnit::Rem => "rem",
            LengthUnit::Vw => "vw",
            LengthUnit::Vh => "vh",
            LengthUnit::Vmin => "vmin",
            LengthUnit::Vmax => "vmax",
            LengthUnit::Ch => "ch",
            LengthUnit::Ex => "ex",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_value_display() {
        assert_eq!(CssValue::Auto.as_str(), "auto");
        assert_eq!(CssValue::None.as_str(), "none");
        assert_eq!(CssValue::Inherit.as_str(), "inherit");
        assert_eq!(CssValue::Initial.as_str(), "initial");
        assert_eq!(CssValue::Unset.as_str(), "unset");
        assert_eq!(CssValue::Revert.as_str(), "revert");
    }

    #[test]
    fn test_display_value() {
        assert_eq!(DisplayValue::Flex.as_str(), "flex");
        assert_eq!(DisplayValue::Grid.as_str(), "grid");
        assert_eq!(DisplayValue::Block.as_str(), "block");
        assert_eq!(DisplayValue::InlineFlex.as_str(), "inline-flex");
    }

    #[test]
    fn test_position_value() {
        assert_eq!(PositionValue::Static.as_str(), "static");
        assert_eq!(PositionValue::Relative.as_str(), "relative");
        assert_eq!(PositionValue::Absolute.as_str(), "absolute");
        assert_eq!(PositionValue::Fixed.as_str(), "fixed");
        assert_eq!(PositionValue::Sticky.as_str(), "sticky");
    }

    #[test]
    fn test_flex_direction_value() {
        assert_eq!(FlexDirectionValue::Row.as_str(), "row");
        assert_eq!(FlexDirectionValue::Column.as_str(), "column");
        assert_eq!(FlexDirectionValue::RowReverse.as_str(), "row-reverse");
    }

    #[test]
    fn test_length_unit() {
        assert_eq!(LengthUnit::Px.as_str(), "px");
        assert_eq!(LengthUnit::Percent.as_str(), "%");
        assert_eq!(LengthUnit::Em.as_str(), "em");
        assert_eq!(LengthUnit::Rem.as_str(), "rem");
        assert_eq!(LengthUnit::Vw.as_str(), "vw");
        assert_eq!(LengthUnit::Vh.as_str(), "vh");
    }

    #[test]
    fn test_justify_content_value() {
        assert_eq!(JustifyContentValue::Center.as_str(), "center");
        assert_eq!(JustifyContentValue::SpaceBetween.as_str(), "space-between");
        assert_eq!(JustifyContentValue::SpaceEvenly.as_str(), "space-evenly");
    }

    #[test]
    fn test_cursor_value() {
        assert_eq!(CursorValue::Pointer.as_str(), "pointer");
        assert_eq!(CursorValue::Grab.as_str(), "grab");
        assert_eq!(CursorValue::NotAllowed.as_str(), "not-allowed");
    }
}
