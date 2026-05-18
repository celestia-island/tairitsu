use super::properties::{CssProperty, Property};
#[cfg(feature = "parse")]
use super::values::CssLength;
use super::values::CssValue;

pub struct StyleStringBuilder(Vec<(Property, String)>);

impl StyleStringBuilder {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(mut self, property: CssProperty, value: &str) -> Self {
        self.0.push((Property::Known(property), value.to_string()));
        self
    }

    pub fn add_custom(mut self, property: &str, value: &str) -> Self {
        self.0
            .push((Property::Custom(property.to_string()), value.to_string()));
        self
    }

    /// Add a CSS custom property (CSS variable).
    /// This is the preferred method for setting --custom-properties.
    pub fn add_var(mut self, name: &str, value: &str) -> Self {
        self.0.push((
            Property::Custom(format!("--{}", name.trim_start_matches("--"))),
            value.to_string(),
        ));
        self
    }

    pub fn add_px(mut self, property: CssProperty, pixels: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}px", pixels)));
        self
    }

    pub fn add_auto(mut self, property: CssProperty) -> Self {
        self.0.push((
            Property::Known(property),
            CssValue::Auto.as_str().to_string(),
        ));
        self
    }

    pub fn add_none(mut self, property: CssProperty) -> Self {
        self.0.push((
            Property::Known(property),
            CssValue::None.as_str().to_string(),
        ));
        self
    }

    pub fn add_inherit(mut self, property: CssProperty) -> Self {
        self.0.push((
            Property::Known(property),
            CssValue::Inherit.as_str().to_string(),
        ));
        self
    }

    pub fn add_percent(mut self, property: CssProperty, value: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}%", value)));
        self
    }

    pub fn add_em(mut self, property: CssProperty, value: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}em", value)));
        self
    }

    pub fn add_rem(mut self, property: CssProperty, value: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}rem", value)));
        self
    }

    pub fn add_vw(mut self, property: CssProperty, value: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}vw", value)));
        self
    }

    pub fn add_vh(mut self, property: CssProperty, value: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}vh", value)));
        self
    }

    pub fn add_px_f64(mut self, property: CssProperty, pixels: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}px", pixels)));
        self
    }

    pub fn add_percent_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}%", value)));
        self
    }

    pub fn add_em_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}em", value)));
        self
    }

    pub fn add_rem_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}rem", value)));
        self
    }

    pub fn add_vw_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}vw", value)));
        self
    }

    pub fn add_vh_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.0
            .push((Property::Known(property), format!("{}vh", value)));
        self
    }

    #[cfg(feature = "parse")]
    /// Add a CSS property with a type-safe `CssLength` value.
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_style::{CssProperty, StyleStringBuilder};
    /// use tairitsu_style::CssLength;
    ///
    /// let style = StyleStringBuilder::new()
    ///     .add_length(CssProperty::Width, CssLength::px(100))
    ///     .add_length(CssProperty::Height, CssLength::vh(100))
    ///     .build_clean();
    /// ```
    pub fn add_length(mut self, property: CssProperty, length: CssLength) -> Self {
        self.0
            .push((Property::Known(property), length.to_css_string()));
        self
    }

    #[cfg(feature = "parse")]
    /// Add a CSS custom property (CSS variable) with a type-safe `CssLength` value.
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_style::StyleStringBuilder;
    /// use tairitsu_style::CssLength;
    ///
    /// let style = StyleStringBuilder::new()
    ///     .add_var_with_length("glow-x", CssLength::percent(50))
    ///     .add_var_with_length("glow-y", CssLength::percent(50))
    ///     .build_clean();
    /// ```
    pub fn add_var_with_length(mut self, name: &str, length: CssLength) -> Self {
        self.0.push((
            Property::Custom(format!("--{}", name.trim_start_matches("--"))),
            length.to_css_string(),
        ));
        self
    }

    pub fn build(self) -> String {
        let mut result = String::new();
        for (property, value) in self.0 {
            if !result.is_empty() {
                result.push(';');
            }
            result.push_str(&format!("{}:{}", property.as_str(), value));
        }
        if !result.is_empty() {
            result.push(';');
        }
        result
    }

    pub fn build_clean(self) -> String {
        let parts: Vec<String> = self
            .0
            .into_iter()
            .map(|(property, value)| format!("{}:{}", property.as_str(), value))
            .collect();
        parts.join(";")
    }
}

impl Default for StyleStringBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StyleBuilder {
    properties: Vec<(Property, String)>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    pub fn add(mut self, property: CssProperty, value: &str) -> Self {
        self.properties
            .push((Property::Known(property), value.to_string()));
        self
    }

    pub fn add_custom(mut self, property: &str, value: &str) -> Self {
        self.properties
            .push((Property::Custom(property.to_string()), value.to_string()));
        self
    }

    /// Add a CSS custom property (CSS variable).
    /// This is the preferred method for setting --custom-properties.
    pub fn add_var(mut self, name: &str, value: &str) -> Self {
        self.properties.push((
            Property::Custom(format!("--{}", name.trim_start_matches("--"))),
            value.to_string(),
        ));
        self
    }

    pub fn add_px(mut self, property: CssProperty, pixels: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}px", pixels)));
        self
    }

    pub fn add_auto(mut self, property: CssProperty) -> Self {
        self.properties.push((
            Property::Known(property),
            CssValue::Auto.as_str().to_string(),
        ));
        self
    }

    pub fn add_none(mut self, property: CssProperty) -> Self {
        self.properties.push((
            Property::Known(property),
            CssValue::None.as_str().to_string(),
        ));
        self
    }

    pub fn add_inherit(mut self, property: CssProperty) -> Self {
        self.properties.push((
            Property::Known(property),
            CssValue::Inherit.as_str().to_string(),
        ));
        self
    }

    pub fn add_percent(mut self, property: CssProperty, value: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}%", value)));
        self
    }

    pub fn add_em(mut self, property: CssProperty, value: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}em", value)));
        self
    }

    pub fn add_rem(mut self, property: CssProperty, value: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}rem", value)));
        self
    }

    pub fn add_vw(mut self, property: CssProperty, value: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}vw", value)));
        self
    }

    pub fn add_vh(mut self, property: CssProperty, value: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}vh", value)));
        self
    }

    pub fn add_px_f64(mut self, property: CssProperty, pixels: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}px", pixels)));
        self
    }

    pub fn add_percent_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}%", value)));
        self
    }

    pub fn add_em_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}em", value)));
        self
    }

    pub fn add_rem_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}rem", value)));
        self
    }

    pub fn add_vw_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}vw", value)));
        self
    }

    pub fn add_vh_f64(mut self, property: CssProperty, value: f64) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}vh", value)));
        self
    }

    #[cfg(feature = "parse")]
    /// Add a CSS property with a type-safe `CssLength` value.
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_style::{CssProperty, StyleBuilder};
    /// use tairitsu_style::CssLength;
    ///
    /// let style = StyleBuilder::new()
    ///     .add_length(CssProperty::Width, CssLength::px(100))
    ///     .add_length(CssProperty::Height, CssLength::vh(100))
    ///     .to_vdom_style();
    /// ```
    pub fn add_length(mut self, property: CssProperty, length: CssLength) -> Self {
        self.properties
            .push((Property::Known(property), length.to_css_string()));
        self
    }

    #[cfg(feature = "parse")]
    /// Add a CSS custom property (CSS variable) with a type-safe `CssLength` value.
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_style::StyleBuilder;
    /// use tairitsu_style::CssLength;
    ///
    /// let style = StyleBuilder::new()
    ///     .add_var_with_length("glow-x", CssLength::percent(50))
    ///     .add_var_with_length("glow-y", CssLength::percent(50))
    ///     .to_vdom_style();
    /// ```
    pub fn add_var_with_length(mut self, name: &str, length: CssLength) -> Self {
        self.properties.push((
            Property::Custom(format!("--{}", name.trim_start_matches("--"))),
            length.to_css_string(),
        ));
        self
    }

    pub fn add_all(mut self, properties: &[(CssProperty, &str)]) -> Self {
        for &(property, value) in properties {
            self.properties
                .push((Property::Known(property), value.to_string()));
        }
        self
    }

    pub fn build_string<F>(f: F) -> String
    where
        F: FnOnce(StyleStringBuilder) -> StyleStringBuilder,
    {
        f(StyleStringBuilder::new()).build()
    }

    pub fn build_clean<F>(f: F) -> String
    where
        F: FnOnce(StyleStringBuilder) -> StyleStringBuilder,
    {
        f(StyleStringBuilder::new()).build_clean()
    }

    pub fn to_vdom_style(self) -> tairitsu_vdom::Style {
        let mut style = tairitsu_vdom::Style::new();
        for (property, value) in self.properties {
            let prop_str = property.as_str();
            match property {
                Property::Known(_) => {
                    style = style.add(prop_str, &value);
                }
                Property::Custom(_) => {
                    style = style.add_custom(prop_str, &value);
                }
            }
        }
        style
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}
