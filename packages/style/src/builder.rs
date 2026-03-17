use super::properties::{CssProperty, Property};

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

    pub fn add_px(mut self, property: CssProperty, pixels: u32) -> Self {
        self.0
            .push((Property::Known(property), format!("{}px", pixels)));
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

    pub fn add_px(mut self, property: CssProperty, pixels: u32) -> Self {
        self.properties
            .push((Property::Known(property), format!("{}px", pixels)));
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
            match property {
                Property::Known(_) => {
                    let prop_str = property.as_str();
                    style = style.add(prop_str, &value);
                }
                Property::Custom(name) => {
                    style = style.add_custom(&name, &value);
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
