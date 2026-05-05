use crate::vnode::VNode;

#[derive(Clone, Debug)]
pub struct Portal {
    pub id: String,
    pub target: String,
    pub content: VNode,
    pub position: PortalPosition,
    pub mask: PortalMaskMode,
}

impl Portal {
    pub fn new(id: impl Into<String>, target: impl Into<String>, content: VNode) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            content,
            position: PortalPosition::default(),
            mask: PortalMaskMode::default(),
        }
    }

    pub fn with_position(mut self, position: PortalPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_mask(mut self, mask: PortalMaskMode) -> Self {
        self.mask = mask;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalPosition {
    FollowTrigger,
    Fixed(FixedPosition),
    Custom(i32, i32),
}

impl Default for PortalPosition {
    fn default() -> Self {
        Self::Fixed(FixedPosition::Center)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedPosition {
    Center,
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PortalMaskMode {
    None,
    Transparent,
    #[default]
    SemiTransparent,
    Full,
}
