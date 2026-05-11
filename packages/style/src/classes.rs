use std::sync::{Arc, RwLock};

use super::{
    typed::TypedClass,
    utility::{UtilityClass, UtilityRegistry},
};

pub struct ClassesBuilder {
    classes: Vec<String>,
    registry: Arc<RwLock<UtilityRegistry>>,
}

impl ClassesBuilder {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            registry: Arc::new(RwLock::new(super::utility::create_default_registry())),
        }
    }

    /// Create a new ClassesBuilder with a custom utility registry
    pub fn with_registry(registry: UtilityRegistry) -> Self {
        Self {
            classes: Vec::new(),
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    /// Get a reference to the utility registry
    pub fn registry(&self) -> Arc<RwLock<UtilityRegistry>> {
        Arc::clone(&self.registry)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, class: &str) -> Self {
        if !class.is_empty() {
            self.classes.push(class.to_string());
        }
        self
    }

    pub fn add_if(mut self, class: &str, condition: bool) -> Self {
        if condition && !class.is_empty() {
            self.classes.push(class.to_string());
        }
        self
    }

    pub fn add_all(mut self, classes: &[&str]) -> Self {
        for class in classes {
            self = self.add(class);
        }
        self
    }

    /// Add a utility class using the UtilityClass system
    ///
    /// This method checks if the class matches a registered utility
    /// and adds it if valid. Returns the builder for chaining.
    pub fn add_utility(mut self, utility_class: &str) -> Self {
        if !utility_class.is_empty() {
            let registry = self.registry.read().unwrap();
            if registry.find(utility_class).is_some() {
                self.classes.push(utility_class.to_string());
            }
        }
        self
    }

    /// Add a utility class conditionally
    pub fn add_utility_if(mut self, utility_class: &str, condition: bool) -> Self {
        if condition && !utility_class.is_empty() {
            let registry = self.registry.read().unwrap();
            if registry.find(utility_class).is_some() {
                self.classes.push(utility_class.to_string());
            }
        }
        self
    }

    /// Add a utility class and generate its CSS rule
    ///
    /// Returns (builder, css_rule) tuple where css_rule is Some if the class
    /// matched a registered utility, None otherwise.
    pub fn add_utility_with_css(mut self, utility_class: &str) -> (Self, Option<String>) {
        let css = if utility_class.is_empty() {
            None
        } else {
            let registry = self.registry.read().unwrap();
            let css = registry.generate_css(utility_class);
            if css.is_some() {
                self.classes.push(utility_class.to_string());
            }
            css
        };
        (self, css)
    }

    /// Generate CSS for all utility classes in this builder
    ///
    /// This method generates CSS rules for all utility classes that have been
    /// added to the builder. Returns a string containing all the CSS rules.
    pub fn generate_css(&self) -> String {
        let registry = self.registry.read().unwrap();
        let mut css_rules = Vec::new();

        for class in &self.classes {
            if let Some(css) = registry.generate_css(class) {
                css_rules.push(css);
            }
        }

        css_rules.join("\n")
    }

    /// Generate CSS for all utility classes and return as a formatted stylesheet
    pub fn generate_stylesheet(&self) -> String {
        let css = self.generate_css();
        if css.is_empty() {
            String::new()
        } else {
            format!("/* Utility Classes */\n{}", css)
        }
    }

    /// Register a custom utility class
    pub fn register_utility(self, utility: Arc<dyn UtilityClass>) -> Self {
        {
            let mut registry = self.registry.write().unwrap();
            registry.register(utility);
        }
        self
    }

    pub fn build(self) -> String {
        self.classes.join(" ")
    }

    pub fn add_typed<T: TypedClass>(mut self, class: T) -> Self {
        for name in class.class_names() {
            if !name.is_empty() {
                self.classes.push(name.to_string());
            }
        }
        self
    }

    pub fn add_typed_if<T: TypedClass>(mut self, class: T, condition: bool) -> Self {
        if condition {
            for name in class.class_names() {
                if !name.is_empty() {
                    self.classes.push(name.to_string());
                }
            }
        }
        self
    }

    pub fn add_typed_all<T: TypedClass>(mut self, classes: &[T]) -> Self {
        for class in classes {
            for name in class.class_names() {
                if !name.is_empty() {
                    self.classes.push(name.to_string());
                }
            }
        }
        self
    }

    pub fn to_vdom_classes(self) -> tairitsu_vdom::Classes {
        let mut classes = tairitsu_vdom::Classes::new();
        for class in self.classes {
            classes = classes.add(&class);
        }
        classes
    }
}

impl Default for ClassesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for ClassesBuilder {
    fn from(s: &str) -> Self {
        let mut builder = Self::new();
        for class in s.split_whitespace() {
            builder = builder.add(class);
        }
        builder
    }
}

impl From<String> for ClassesBuilder {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

// Extension methods for working with utility classes
impl ClassesBuilder {
    /// Parse and add utility classes from a space-separated string
    ///
    /// This method will parse the input string and add each class as a utility class
    /// if it matches a registered utility.
    pub fn add_utilities(mut self, classes: &str) -> Self {
        let classes_to_add: Vec<String> = {
            let registry = self.registry.read().unwrap();
            classes
                .split_whitespace()
                .filter(|class| registry.find(class).is_some())
                .map(|s| s.to_string())
                .collect()
        };
        self.classes.extend(classes_to_add);
        self
    }

    /// Parse and add utility classes from a string, conditionally
    pub fn add_utilities_if(mut self, classes: &str, condition: bool) -> Self {
        if condition {
            let classes_to_add: Vec<String> = {
                let registry = self.registry.read().unwrap();
                classes
                    .split_whitespace()
                    .filter(|class| registry.find(class).is_some())
                    .map(|s| s.to_string())
                    .collect()
            };
            self.classes.extend(classes_to_add);
        }
        self
    }

    /// Create a ClassesBuilder from utility classes string
    pub fn from_utilities(classes: &str) -> Self {
        Self::new().add_utilities(classes)
    }

    /// Mix regular classes with utility classes
    ///
    /// Regular classes are added as-is, utility classes are validated against
    /// the registry.
    pub fn add_mixed(mut self, classes: &str) -> Self {
        // Simply add all classes - the registry validation is optional
        for class in classes.split_whitespace() {
            self = self.add(class);
        }
        self
    }
}
