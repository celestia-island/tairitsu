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
        }
    }
}

/// CSS overflow property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowValue {
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

impl OverflowValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            OverflowValue::Visible => "visible",
            OverflowValue::Hidden => "hidden",
            OverflowValue::Scroll => "scroll",
            OverflowValue::Auto => "auto",
            OverflowValue::Clip => "clip",
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

/// CSS align-items property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignItemsValue {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl AlignItemsValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlignItemsValue::FlexStart => "flex-start",
            AlignItemsValue::FlexEnd => "flex-end",
            AlignItemsValue::Center => "center",
            AlignItemsValue::Baseline => "baseline",
            AlignItemsValue::Stretch => "stretch",
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

/// CSS cursor property values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorValue {
    Auto,
    Default,
    Pointer,
    Crosshair,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    Grab,
}

impl CursorValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            CursorValue::Auto => "auto",
            CursorValue::Default => "default",
            CursorValue::Pointer => "pointer",
            CursorValue::Crosshair => "crosshair",
            CursorValue::Move => "move",
            CursorValue::Text => "text",
            CursorValue::Wait => "wait",
            CursorValue::Help => "help",
            CursorValue::Progress => "progress",
            CursorValue::NotAllowed => "not-allowed",
            CursorValue::Grab => "grab",
        }
    }
}
