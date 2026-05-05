use std::{cell::RefCell, rc::Rc};

use super::{Portal, PortalMaskMode, PortalPosition};

#[derive(Clone, Default)]
pub struct PortalManager {
    inner: Rc<RefCell<PortalManagerInner>>,
}

#[derive(Default)]
struct PortalManagerInner {
    portals: Vec<Portal>,
}

impl PortalManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, portal: Portal) {
        self.inner.borrow_mut().portals.push(portal);
    }

    pub fn remove(&self, id: &str) -> Option<Portal> {
        let mut inner = self.inner.borrow_mut();
        let pos = inner.portals.iter().position(|p| p.id == id)?;
        Some(inner.portals.remove(pos))
    }

    pub fn get(&self, id: &str) -> Option<Portal> {
        self.inner
            .borrow()
            .portals
            .iter()
            .find(|p| p.id == id)
            .cloned()
    }

    pub fn get_all(&self) -> Vec<Portal> {
        self.inner.borrow().portals.clone()
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().portals.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().portals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().portals.is_empty()
    }

    pub fn update_position(&self, id: &str, position: PortalPosition) -> bool {
        let mut inner = self.inner.borrow_mut();
        if let Some(portal) = inner.portals.iter_mut().find(|p| p.id == id) {
            portal.position = position;
            true
        } else {
            false
        }
    }

    pub fn update_mask(&self, id: &str, mask: PortalMaskMode) -> bool {
        let mut inner = self.inner.borrow_mut();
        if let Some(portal) = inner.portals.iter_mut().find(|p| p.id == id) {
            portal.mask = mask;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FixedPosition, VNode, VText};

    #[test]
    fn test_portal_manager_basic() {
        let manager = PortalManager::new();
        assert!(manager.is_empty());

        let portal = Portal::new("test-1", "body", VNode::Text(VText::new("Hello")));
        manager.add(portal);

        assert_eq!(manager.len(), 1);
        assert!(manager.get("test-1").is_some());

        let removed = manager.remove("test-1");
        assert!(removed.is_some());
        assert!(manager.is_empty());
    }

    #[test]
    fn test_portal_manager_update() {
        let manager = PortalManager::new();
        let portal = Portal::new("test-1", "body", VNode::Text(VText::new("Hello")));
        manager.add(portal);

        assert!(manager.update_position("test-1", PortalPosition::Fixed(FixedPosition::Top)));
        assert!(manager.update_mask("test-1", PortalMaskMode::Full));

        let updated = manager.get("test-1").unwrap();
        assert_eq!(updated.position, PortalPosition::Fixed(FixedPosition::Top));
        assert_eq!(updated.mask, PortalMaskMode::Full);
    }

    #[test]
    fn test_portal_manager_clear() {
        let manager = PortalManager::new();
        manager.add(Portal::new("test-1", "body", VNode::Text(VText::new("A"))));
        manager.add(Portal::new("test-2", "body", VNode::Text(VText::new("B"))));

        assert_eq!(manager.len(), 2);
        manager.clear();
        assert!(manager.is_empty());
    }
}
