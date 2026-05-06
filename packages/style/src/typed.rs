pub trait TypedClass {
    fn class_name(&self) -> &'static str;

    fn class_names(&self) -> Vec<&'static str> {
        vec![self.class_name()]
    }
}

#[macro_export]
macro_rules! define_typed_classes {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($variant:ident => $class:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant),*
        }

        impl $crate::TypedClass for $name {
            fn class_name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $class),*
                }
            }
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::ClassesBuilder;

    define_typed_classes! {
        TestDisplay {
            Flex => "hi-flex",
            Hidden => "hi-hidden",
            Grid => "hi-grid",
        }
    }

    define_typed_classes! {
        TestButton {
            Button => "hi-button",
            Primary => "hi-button-primary",
            Loading => "hi-button-loading",
        }
    }

    #[test]
    fn test_typed_class_name() {
        assert_eq!(TestDisplay::Flex.class_name(), "hi-flex");
        assert_eq!(TestButton::Primary.class_name(), "hi-button-primary");
    }

    #[test]
    fn test_classes_builder_add_typed() {
        let classes = ClassesBuilder::new()
            .add_typed(TestButton::Button)
            .add_typed(TestButton::Primary)
            .build();
        assert_eq!(classes, "hi-button hi-button-primary");
    }

    #[test]
    fn test_classes_builder_add_typed_if() {
        let classes = ClassesBuilder::new()
            .add_typed(TestButton::Button)
            .add_typed_if(TestButton::Loading, true)
            .add_typed_if(TestButton::Primary, false)
            .build();
        assert_eq!(classes, "hi-button hi-button-loading");
    }

    #[test]
    fn test_classes_builder_mixed_typed_and_raw() {
        let classes = ClassesBuilder::new()
            .add_typed(TestButton::Button)
            .add("custom-class")
            .add_typed_if(TestButton::Primary, true)
            .build();
        assert_eq!(classes, "hi-button custom-class hi-button-primary");
    }

    #[test]
    fn test_classes_builder_add_typed_all() {
        let classes = ClassesBuilder::new()
            .add_typed_all(&[TestDisplay::Flex, TestDisplay::Hidden])
            .build();
        assert_eq!(classes, "hi-flex hi-hidden");
    }

    #[test]
    fn test_typed_class_default_class_names() {
        assert_eq!(TestDisplay::Flex.class_names(), vec!["hi-flex"]);
    }

    #[test]
    fn test_enum_derives_work() {
        let v1 = TestButton::Button;
        let v2 = v1;
        assert_eq!(v1, v2);
        assert!(!(v1 != v2));
        let _ = format!("{:?}", v1);
    }

    #[test]
    fn test_empty_class_name_skipped() {
        define_typed_classes! {
            TestEmpty {
                Valid => "hi-valid",
                Empty => "",
            }
        }
        let classes = ClassesBuilder::new()
            .add_typed(TestEmpty::Valid)
            .add_typed(TestEmpty::Empty)
            .build();
        assert_eq!(classes, "hi-valid");
    }

    #[test]
    fn test_macro_with_doc_comment() {
        define_typed_classes! {
            /// Display property values
            TestDocDisplay {
                Block => "hi-block",
                Inline => "hi-inline",
            }
        }
        assert_eq!(TestDocDisplay::Block.class_name(), "hi-block");
        assert_eq!(TestDocDisplay::Inline.class_name(), "hi-inline");
    }
}
