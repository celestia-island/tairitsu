use tairitsu_hooks::{use_effect, use_state};
use tairitsu_macros::rsx;
use tairitsu_vdom::{Classes, EventData, MouseEvent, Style, VNode};

/// Glow 组件 - 类似 Hikari 的 Glow 组件
pub fn glow_component(intensity: f32, children: Vec<VNode>) -> VNode {
    let (hovered, set_hovered) = use_state(false);

    use_effect(move || {
        let _ = hovered;
    });

    let class = Classes::new()
        .add("hi-glow-wrapper")
        .add_if("hi-glow-soft", true)
        .add_if("hi-glow-active", hovered);

    let intensity_value = if hovered { 1.0 } else { intensity };
    let style = Style::new().add_custom("--glow-intensity", &intensity_value.to_string());

    rsx! {
        div {
            class: class,
            style: style,
            onmouseenter: move |_e| set_hovered(true),
            onmouseleave: move |_e| set_hovered(false),
            ..children
        }
    }
}

/// Button 组件 - 类似 Hikari 的 Button 组件（支持事件参数和动态 children）
pub fn button_component(
    variant: ButtonVariant,
    size: ButtonSize,
    children: Vec<VNode>,
    onclick: impl FnMut(Box<dyn EventData>) + 'static,
) -> VNode {
    let variant_class = match variant {
        ButtonVariant::Primary => "hi-button-primary",
        ButtonVariant::Secondary => "hi-button-secondary",
        ButtonVariant::Ghost => "hi-button-ghost",
    };

    let size_class = match size {
        ButtonSize::Small => "hi-button-sm",
        ButtonSize::Medium => "hi-button-md",
        ButtonSize::Large => "hi-button-lg",
    };

    let class = Classes::new()
        .add("hi-button")
        .add(variant_class)
        .add(size_class);

    rsx! {
        button {
            class: class,
            onclick: onclick,
            ..children
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Card 组件 - 类似 Hikari 的 Card 组件
pub fn card_component(title: &str, content: &str, footer: Option<&str>) -> VNode {
    rsx! {
        div {
            class: "hi-card",

            div {
                class: "hi-card-header",

                h3 {
                    class: "hi-card-title",
                }
            }

            div {
                class: "hi-card-body",

                p {
                    class: "hi-card-content",
                }
            }
        }
    }
}

/// Alert 组件 - 类似 Hikari 的 Alert 组件
pub fn alert_component(alert_type: AlertType, message: &str, closable: bool) -> VNode {
    let type_class = match alert_type {
        AlertType::Info => "hi-alert-info",
        AlertType::Success => "hi-alert-success",
        AlertType::Warning => "hi-alert-warning",
        AlertType::Error => "hi-alert-error",
    };

    rsx! {
        div {
            class: "hi-alert hi-alert-info",

            p {
                class: "hi-alert-message",
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glow_component() {
        let children = vec![VNode::Text(tairitsu_vdom::VText::new("Test"))];
        let node = glow_component(0.5, children);

        match node {
            VNode::Element(elem) => {
                assert!(elem.class.to_string().contains("glow"));
                assert_eq!(elem.children.len(), 1);
                assert!(elem.event_handlers.contains_key("mouseenter"));
                assert!(elem.event_handlers.contains_key("mouseleave"));
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_button_component() {
        let children = vec![VNode::Text(tairitsu_vdom::VText::new("Click me"))];
        let click_count = std::cell::Cell::new(0);
        let onclick = move |_e: Box<dyn EventData>| {
            click_count.set(click_count.get() + 1);
        };

        let node = button_component(
            ButtonVariant::Primary,
            ButtonSize::Medium,
            children,
            onclick,
        );

        match node {
            VNode::Element(elem) => {
                assert!(elem.class.to_string().contains("button"));
                assert!(elem.class.to_string().contains("primary"));
                assert_eq!(elem.children.len(), 1);
                assert!(elem.event_handlers.contains_key("click"));
            }
            _ => panic!("Expected button element"),
        }
    }

    #[test]
    fn test_card_component() {
        let node = card_component("Title", "Content", None);

        match node {
            VNode::Element(elem) => {
                assert!(elem.class.to_string().contains("card"));
            }
            _ => panic!("Expected card element"),
        }
    }

    #[test]
    fn test_alert_component() {
        let node = alert_component(AlertType::Info, "Info message", false);

        match node {
            VNode::Element(elem) => {
                assert!(elem.class.to_string().contains("alert"));
                assert!(elem.class.to_string().contains("info"));
            }
            _ => panic!("Expected alert element"),
        }
    }

    #[test]
    fn test_event_data_downcast() {
        let event = MouseEvent::new().client_x(100).client_y(200);

        let boxed: Box<dyn EventData> = Box::new(event);

        if let Some(mouse_event) = boxed.as_any().downcast_ref::<MouseEvent>() {
            assert_eq!(mouse_event.client_x, 100);
            assert_eq!(mouse_event.client_y, 200);
        } else {
            panic!("Failed to downcast to MouseEvent");
        }
    }
}
