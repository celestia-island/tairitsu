pub struct ClassesBuilder {
    classes: Vec<String>,
}

impl ClassesBuilder {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

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

    pub fn build(self) -> String {
        self.classes.join(" ")
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
