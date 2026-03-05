#[cfg(feature = "web")]
use anyhow::Result;
#[cfg(feature = "web")]
use tairitsu_vdom::{FixedPosition, Portal, PortalManager, PortalPosition, VNode};

#[cfg(feature = "web")]
use crate::WebPlatform;
#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::rc::Rc;

#[cfg(feature = "web")]
pub struct PortalRenderer {
    platform: WebPlatform,
    manager: PortalManager,
    portal_containers: Rc<RefCell<HashMap<String, web_sys::Element>>>,
    mask_elements: Rc<RefCell<HashMap<String, web_sys::Element>>>,
}

#[cfg(feature = "web")]
use std::collections::HashMap;

#[cfg(feature = "web")]
impl PortalRenderer {
    pub fn new(platform: WebPlatform, manager: PortalManager) -> Self {
        Self {
            platform,
            manager,
            portal_containers: Rc::new(RefCell::new(HashMap::new())),
            mask_elements: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn render_portal(&self, portal: &Portal) -> Result<()> {
        let document = web_sys::window()
            .ok_or_else(|| anyhow::anyhow!("No window"))?
            .document()
            .ok_or_else(|| anyhow::anyhow!("No document"))?;

        let target = document
            .query_selector(&portal.target)?
            .ok_or_else(|| anyhow::anyhow!("Target not found: {}", portal.target))?;

        if portal.mask != tairitsu_vdom::PortalMaskMode::None {
            self.render_mask(&portal.id, portal.mask, &target)?;
        }

        let container = document.create_element("div")?;
        container.set_class_name("tairitsu-portal");
        container.set_attribute("data-portal-id", &portal.id)?;

        self.apply_position(&container, &portal.position)?;

        self.render_vnode(&portal.content, &container)?;

        target.append_child(&container)?;

        self.portal_containers
            .borrow_mut()
            .insert(portal.id.clone(), container);

        Ok(())
    }

    pub fn remove_portal(&self, id: &str) -> Result<()> {
        if let Some(container) = self.portal_containers.borrow_mut().remove(id) {
            if let Some(parent) = container.parent_element() {
                parent.remove_child(&container)?;
            }
        }

        if let Some(mask) = self.mask_elements.borrow_mut().remove(id) {
            if let Some(parent) = mask.parent_element() {
                parent.remove_child(&mask)?;
            }
        }

        Ok(())
    }

    fn render_mask(
        &self,
        portal_id: &str,
        mask_mode: tairitsu_vdom::PortalMaskMode,
        target: &web_sys::Element,
    ) -> Result<()> {
        let document = target.owner_document().unwrap();

        let mask = document.create_element("div")?;
        mask.set_class_name("tairitsu-portal-mask");
        mask.set_attribute("data-portal-mask-for", portal_id)?;

        let opacity = match mask_mode {
            tairitsu_vdom::PortalMaskMode::Transparent => "0",
            tairitsu_vdom::PortalMaskMode::SemiTransparent => "0.5",
            tairitsu_vdom::PortalMaskMode::Full => "1",
            _ => "0",
        };

        mask.set_attribute("style", &format!(
            "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(0,0,0,{});z-index:9998;",
            opacity
        ))?;

        target.append_child(&mask)?;

        self.mask_elements
            .borrow_mut()
            .insert(portal_id.to_string(), mask);

        Ok(())
    }

    fn apply_position(
        &self,
        container: &web_sys::Element,
        position: &PortalPosition,
    ) -> Result<()> {
        let style = container
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or_else(|| anyhow::anyhow!("Container is not HtmlElement"))?
            .style();

        style.set_property("position", "fixed")?;
        style.set_property("z-index", "9999")?;

        match position {
            PortalPosition::Fixed(fixed) => match fixed {
                FixedPosition::Center => {
                    style.set_property("top", "50%")?;
                    style.set_property("left", "50%")?;
                    style.set_property("transform", "translate(-50%, -50%)")?;
                }
                FixedPosition::Top => {
                    style.set_property("top", "0")?;
                    style.set_property("left", "50%")?;
                    style.set_property("transform", "translateX(-50%)")?;
                }
                FixedPosition::TopLeft => {
                    style.set_property("top", "0")?;
                    style.set_property("left", "0")?;
                }
                FixedPosition::TopRight => {
                    style.set_property("top", "0")?;
                    style.set_property("right", "0")?;
                }
                FixedPosition::Bottom => {
                    style.set_property("bottom", "0")?;
                    style.set_property("left", "50%")?;
                    style.set_property("transform", "translateX(-50%)")?;
                }
                FixedPosition::BottomLeft => {
                    style.set_property("bottom", "0")?;
                    style.set_property("left", "0")?;
                }
                FixedPosition::BottomRight => {
                    style.set_property("bottom", "0")?;
                    style.set_property("right", "0")?;
                }
                FixedPosition::Left => {
                    style.set_property("top", "50%")?;
                    style.set_property("left", "0")?;
                    style.set_property("transform", "translateY(-50%)")?;
                }
                FixedPosition::Right => {
                    style.set_property("top", "50%")?;
                    style.set_property("right", "0")?;
                    style.set_property("transform", "translateY(-50%)")?;
                }
            },
            PortalPosition::Custom(x, y) => {
                style.set_property("top", &format!("{}px", y))?;
                style.set_property("left", &format!("{}px", x))?;
            }
            PortalPosition::FollowTrigger => {
                style.set_property("position", "absolute")?;
            }
        }

        Ok(())
    }

    fn render_vnode(&self, vnode: &VNode, parent: &web_sys::Element) -> Result<()> {
        match vnode {
            VNode::Element(velement) => {
                let element = self.platform.create_element(&velement.tag);

                for (name, value) in &velement.attributes {
                    self.platform.set_attribute(&element, name, value);
                }

                if !velement.class.static_classes.is_empty() {
                    self.platform
                        .set_class(&element, &velement.class.static_classes);
                }

                if !velement.style.static_styles.is_empty() {
                    let style_str = &velement.style.static_styles;
                    for part in style_str.split(';') {
                        let part = part.trim();
                        if !part.is_empty() {
                            if let Some((name, value)) = part.split_once(':') {
                                self.platform.set_style(&element, name.trim(), value.trim());
                            }
                        }
                    }
                }

                for (name, value) in &velement.style.css_variables {
                    self.platform.set_style(&element, name, value);
                }

                for child in &velement.children {
                    self.render_vnode(child, &element.0)?;
                }

                parent.append_child(&element.0)?;
            }
            VNode::Text(vtext) => {
                let text_node = self.platform.create_text_node(&vtext.text);
                parent.append_child(&text_node.0)?;
            }
            VNode::Fragment(children) => {
                for child in children {
                    self.render_vnode(child, parent)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(feature = "web"))]
pub struct PortalRenderer;

#[cfg(not(feature = "web"))]
impl PortalRenderer {
    pub fn new(_platform: crate::WebPlatform, _manager: tairitsu_vdom::PortalManager) -> Self {
        Self
    }
}
