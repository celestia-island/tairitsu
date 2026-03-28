use std::{cell::RefCell, rc::Rc};

/// Represents the current interaction state of an interactive element.
///
/// This enum models the five possible states of a button or similar interactive element:
/// - **Idle**: The element is in its default state, with no user interaction
/// - **Hover**: The user's pointer is over the element (`:hover`)
/// - **Active**: The user is actively pressing the element (`:active`)
/// - **Focused**: The element has keyboard focus (`:focus-visible`)
/// - **Disabled**: The element is disabled and cannot be interacted with
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InteractionState {
    Idle,
    Hover,
    Active,
    Focused,
    Disabled,
}

impl InteractionState {
    /// Returns true if the element is in an interactive state (not disabled).
    #[inline]
    pub fn is_interactive(self) -> bool {
        self != Self::Disabled
    }
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Events that can trigger state transitions in the interaction state machine.
///
/// These events correspond to DOM events and programmatic state changes:
/// - **MouseEnter**: Pointer entered the element
/// - **MouseLeave**: Pointer left the element
/// - **MouseDown**: Mouse/touch button pressed down
/// - **MouseUp**: Mouse/touch button released
/// - **Focus**: Element received keyboard focus
/// - **Blur**: Element lost keyboard focus
/// - **Disable**: Element was programmatically disabled
/// - **Enable**: Element was programmatically enabled
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InteractionEvent {
    MouseEnter,
    MouseLeave,
    MouseDown,
    MouseUp,
    Focus,
    Blur,
    Disable,
    Enable,
}

/// A lightweight state machine for managing button interaction states.
///
/// The state machine enforces valid state transitions according to the following table:
///
/// | Current State | Event        | Next State |
/// |---------------|--------------|------------|
/// | Idle          | MouseEnter   | Hover      |
/// | Hover         | MouseLeave   | Idle       |
/// | Hover         | MouseDown    | Active     |
/// | Hover         | Focus        | Focused    |
/// | Active        | MouseUp      | Hover      |
/// | Active        | MouseLeave   | Idle       |
/// | Focused       | MouseEnter   | Hover      |
/// | Focused       | Blur         | Idle       |
/// | Any           | Disable      | Disabled   |
/// | Disabled      | Enable       | Idle       |
///
/// Invalid transitions return `None` and are ignored.
///
/// # Example
/// ```
/// use tairitsu_hooks::{ButtonStateMachine, InteractionEvent, InteractionState};
///
/// let mut sm = ButtonStateMachine::new();
/// assert_eq!(sm.state(), InteractionState::Idle);
///
/// // Enter hover state
/// assert_eq!(sm.transition(InteractionEvent::MouseEnter), Some(InteractionState::Hover));
///
/// // Press down
/// assert_eq!(sm.transition(InteractionEvent::MouseDown), Some(InteractionState::Active));
///
/// // Release
/// assert_eq!(sm.transition(InteractionEvent::MouseUp), Some(InteractionState::Hover));
///
/// // Leave
/// assert_eq!(sm.transition(InteractionEvent::MouseLeave), Some(InteractionState::Idle));
/// ```
pub struct ButtonStateMachine {
    state: InteractionState,
}

impl ButtonStateMachine {
    /// Creates a new state machine in the Idle state.
    #[inline]
    pub fn new() -> Self {
        Self {
            state: InteractionState::Idle,
        }
    }

    /// Transitions the state machine based on the given event.
    ///
    /// Returns `Some(new_state)` if the state changed, or `None` if the transition
    /// was invalid and the state remained unchanged.
    ///
    /// # Example
    /// ```
    /// # use tairitsu_hooks::{ButtonStateMachine, InteractionEvent, InteractionState};
    /// let mut sm = ButtonStateMachine::new();
    ///
    /// // Valid transition
    /// assert!(sm.transition(InteractionEvent::MouseEnter).is_some());
    ///
    /// // Invalid transition - already hovering, can't hover again
    /// assert!(sm.transition(InteractionEvent::MouseEnter).is_none());
    /// ```
    pub fn transition(&mut self, event: InteractionEvent) -> Option<InteractionState> {
        let new_state = match (self.state, event) {
            // From Idle
            (InteractionState::Idle, InteractionEvent::MouseEnter) => Some(InteractionState::Hover),
            (InteractionState::Idle, InteractionEvent::Focus) => Some(InteractionState::Focused),
            (InteractionState::Idle, InteractionEvent::Disable) => Some(InteractionState::Disabled),

            // From Hover
            (InteractionState::Hover, InteractionEvent::MouseLeave) => Some(InteractionState::Idle),
            (InteractionState::Hover, InteractionEvent::MouseDown) => {
                Some(InteractionState::Active)
            }
            (InteractionState::Hover, InteractionEvent::Focus) => Some(InteractionState::Focused),
            (InteractionState::Hover, InteractionEvent::Disable) => {
                Some(InteractionState::Disabled)
            }

            // From Active
            (InteractionState::Active, InteractionEvent::MouseUp) => Some(InteractionState::Hover),
            (InteractionState::Active, InteractionEvent::MouseLeave) => {
                Some(InteractionState::Idle)
            }
            (InteractionState::Active, InteractionEvent::Disable) => {
                Some(InteractionState::Disabled)
            }

            // From Focused
            (InteractionState::Focused, InteractionEvent::Blur) => Some(InteractionState::Idle),
            (InteractionState::Focused, InteractionEvent::MouseEnter) => {
                Some(InteractionState::Hover)
            }
            (InteractionState::Focused, InteractionEvent::Disable) => {
                Some(InteractionState::Disabled)
            }

            // From Disabled
            (InteractionState::Disabled, InteractionEvent::Enable) => Some(InteractionState::Idle),

            // Invalid transitions - return None
            _ => None,
        };

        if let Some(new_state) = new_state
            && new_state != self.state
        {
            self.state = new_state;
            return Some(new_state);
        }

        None
    }

    /// Returns the current state.
    #[inline]
    pub fn state(&self) -> InteractionState {
        self.state
    }

    /// Sets the state directly (primarily for testing purposes).
    #[inline]
    pub fn set_state(&mut self, state: InteractionState) {
        self.state = state;
    }

    /// Returns true if the element is interactive (not disabled).
    #[inline]
    pub fn is_interactive(&self) -> bool {
        self.state.is_interactive()
    }

    /// Resets the state machine to Idle.
    pub fn reset(&mut self) {
        self.state = InteractionState::Idle;
    }
}

impl Default for ButtonStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// A callback type for handling interaction events.
///
/// This is a type alias for a reference-counted callback function
/// that takes an `InteractionEvent` and returns nothing.
pub type InteractionCallback = Rc<dyn Fn(InteractionEvent)>;

/// A hook for managing interaction state in components.
///
/// Returns a tuple of:
/// - The current interaction state
/// - A callback that can be used to send events to the state machine
///
/// # Example
/// ```ignore
/// let (state, on_event) = use_interaction_state();
///
/// // In your event handlers:
/// // on_mouse_enter: move |_| on_event(InteractionEvent::MouseEnter)
/// // on_mouse_leave: move |_| on_event(InteractionEvent::MouseLeave)
/// ```
pub fn use_interaction_state() -> (Rc<RefCell<InteractionState>>, InteractionCallback) {
    let state = Rc::new(RefCell::new(InteractionState::Idle));
    let state_clone = Rc::clone(&state);

    let callback: InteractionCallback = Rc::new(move |event: InteractionEvent| {
        let mut sm = ButtonStateMachine::new();
        sm.state = *state_clone.borrow();
        if let Some(new_state) = sm.transition(event) {
            *state_clone.borrow_mut() = new_state;
        }
    });

    (state, callback)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the state machine basic transitions
    #[test]
    fn test_button_state_machine_basic_flow() {
        let mut sm = ButtonStateMachine::new();
        assert_eq!(sm.state(), InteractionState::Idle);
        assert!(sm.is_interactive());

        // Idle -> Hover
        assert_eq!(
            sm.transition(InteractionEvent::MouseEnter),
            Some(InteractionState::Hover)
        );
        assert_eq!(sm.state(), InteractionState::Hover);
        assert!(sm.is_interactive());

        // Hover -> Active
        assert_eq!(
            sm.transition(InteractionEvent::MouseDown),
            Some(InteractionState::Active)
        );
        assert_eq!(sm.state(), InteractionState::Active);
        assert!(sm.is_interactive());

        // Active -> Hover
        assert_eq!(
            sm.transition(InteractionEvent::MouseUp),
            Some(InteractionState::Hover)
        );
        assert_eq!(sm.state(), InteractionState::Hover);

        // Hover -> Idle
        assert_eq!(
            sm.transition(InteractionEvent::MouseLeave),
            Some(InteractionState::Idle)
        );
        assert_eq!(sm.state(), InteractionState::Idle);
    }

    #[test]
    fn test_focus_transitions() {
        let mut sm = ButtonStateMachine::new();

        // Idle -> Focused
        assert_eq!(
            sm.transition(InteractionEvent::Focus),
            Some(InteractionState::Focused)
        );
        assert_eq!(sm.state(), InteractionState::Focused);

        // Focused -> Hover (mouse enters while focused)
        assert_eq!(
            sm.transition(InteractionEvent::MouseEnter),
            Some(InteractionState::Hover)
        );
        assert_eq!(sm.state(), InteractionState::Hover);

        // Hover -> Focused (user tabs while hovering)
        sm.transition(InteractionEvent::MouseLeave); // Back to Idle
        sm.transition(InteractionEvent::Focus); // To Focused
        assert_eq!(sm.state(), InteractionState::Focused);

        // Focused -> Idle
        assert_eq!(
            sm.transition(InteractionEvent::Blur),
            Some(InteractionState::Idle)
        );
        assert_eq!(sm.state(), InteractionState::Idle);
    }

    #[test]
    fn test_active_to_idle_on_leave() {
        let mut sm = ButtonStateMachine::new();

        // Idle -> Hover -> Active
        sm.transition(InteractionEvent::MouseEnter);
        sm.transition(InteractionEvent::MouseDown);
        assert_eq!(sm.state(), InteractionState::Active);

        // Active -> Idle (mouse leaves while pressed)
        assert_eq!(
            sm.transition(InteractionEvent::MouseLeave),
            Some(InteractionState::Idle)
        );
        assert_eq!(sm.state(), InteractionState::Idle);
    }

    #[test]
    fn test_disable_enable() {
        let mut sm = ButtonStateMachine::new();

        // From Idle to Disabled
        assert_eq!(
            sm.transition(InteractionEvent::Disable),
            Some(InteractionState::Disabled)
        );
        assert_eq!(sm.state(), InteractionState::Disabled);
        assert!(!sm.is_interactive());

        // While disabled, other events should be ignored
        assert!(sm.transition(InteractionEvent::MouseEnter).is_none());
        assert!(sm.transition(InteractionEvent::Focus).is_none());
        assert_eq!(sm.state(), InteractionState::Disabled);

        // Disabled -> Idle
        assert_eq!(
            sm.transition(InteractionEvent::Enable),
            Some(InteractionState::Idle)
        );
        assert_eq!(sm.state(), InteractionState::Idle);
        assert!(sm.is_interactive());
    }

    #[test]
    fn test_disable_from_any_state() {
        for initial_state in &[
            InteractionState::Idle,
            InteractionState::Hover,
            InteractionState::Active,
            InteractionState::Focused,
        ] {
            let mut sm = ButtonStateMachine::new();
            sm.state = *initial_state;

            assert_eq!(
                sm.transition(InteractionEvent::Disable),
                Some(InteractionState::Disabled)
            );
            assert_eq!(sm.state(), InteractionState::Disabled);
        }
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = ButtonStateMachine::new();

        // Can't MouseDown from Idle (must be Hover first)
        assert!(sm.transition(InteractionEvent::MouseDown).is_none());
        assert_eq!(sm.state(), InteractionState::Idle);

        // Can't MouseUp from Idle
        assert!(sm.transition(InteractionEvent::MouseUp).is_none());
        assert_eq!(sm.state(), InteractionState::Idle);

        // Go to Hover, then try invalid transitions
        sm.transition(InteractionEvent::MouseEnter);
        assert!(sm.transition(InteractionEvent::MouseEnter).is_none()); // Already hovering
        assert!(sm.transition(InteractionEvent::Blur).is_none()); // Can't blur from Hover
        assert_eq!(sm.state(), InteractionState::Hover);

        // Can't Enable from Idle (already enabled)
        sm.reset();
        assert!(sm.transition(InteractionEvent::Enable).is_none());
    }

    #[test]
    fn test_hover_focus_interaction() {
        let mut sm = ButtonStateMachine::new();

        // Start with focus
        sm.transition(InteractionEvent::Focus);
        assert_eq!(sm.state(), InteractionState::Focused);

        // Mouse enters while focused
        sm.transition(InteractionEvent::MouseEnter);
        assert_eq!(sm.state(), InteractionState::Hover);

        // Can press from hover
        sm.transition(InteractionEvent::MouseDown);
        assert_eq!(sm.state(), InteractionState::Active);

        // Release back to hover
        sm.transition(InteractionEvent::MouseUp);
        assert_eq!(sm.state(), InteractionState::Hover);

        // Mouse leaves, back to focused (not idle)
        sm.transition(InteractionEvent::MouseLeave);
        // This should go to Idle because we lost focus when mouse left
        // Actually, let me check the state table again...
        // From Hover, MouseLeave -> Idle
        // So we go to Idle, which means we lost the focus
        assert_eq!(sm.state(), InteractionState::Idle);
    }

    #[test]
    fn test_interaction_state_is_interactive() {
        assert!(InteractionState::Idle.is_interactive());
        assert!(InteractionState::Hover.is_interactive());
        assert!(InteractionState::Active.is_interactive());
        assert!(InteractionState::Focused.is_interactive());
        assert!(!InteractionState::Disabled.is_interactive());
    }

    #[test]
    fn test_state_machine_is_interactive() {
        let mut sm = ButtonStateMachine::new();

        for state in &[
            InteractionState::Idle,
            InteractionState::Hover,
            InteractionState::Active,
            InteractionState::Focused,
        ] {
            sm.state = *state;
            assert!(sm.is_interactive());
        }

        sm.state = InteractionState::Disabled;
        assert!(!sm.is_interactive());
    }

    #[test]
    fn test_reset() {
        let mut sm = ButtonStateMachine::new();
        sm.transition(InteractionEvent::MouseEnter);
        sm.transition(InteractionEvent::MouseDown);
        assert_eq!(sm.state(), InteractionState::Active);

        sm.reset();
        assert_eq!(sm.state(), InteractionState::Idle);
        assert!(sm.is_interactive());
    }

    #[test]
    fn test_default() {
        let sm = ButtonStateMachine::default();
        assert_eq!(sm.state(), InteractionState::Idle);
    }

    #[test]
    fn test_interaction_state_default() {
        let state = InteractionState::default();
        assert_eq!(state, InteractionState::Idle);
    }

    #[test]
    fn test_acceptance_criteria() {
        let mut sm = ButtonStateMachine::new();
        assert_eq!(
            sm.transition(InteractionEvent::MouseEnter),
            Some(InteractionState::Hover)
        );
        assert_eq!(
            sm.transition(InteractionEvent::MouseDown),
            Some(InteractionState::Active)
        );
        assert_eq!(
            sm.transition(InteractionEvent::MouseUp),
            Some(InteractionState::Hover)
        );
        assert_eq!(
            sm.transition(InteractionEvent::MouseLeave),
            Some(InteractionState::Idle)
        );
    }

    #[test]
    fn test_use_interaction_state_hook() {
        let (state, callback) = use_interaction_state();

        // Initial state
        assert_eq!(*state.borrow(), InteractionState::Idle);

        // Send MouseEnter event
        callback(InteractionEvent::MouseEnter);
        assert_eq!(*state.borrow(), InteractionState::Hover);

        // Send MouseDown event
        callback(InteractionEvent::MouseDown);
        assert_eq!(*state.borrow(), InteractionState::Active);

        // Send MouseUp event
        callback(InteractionEvent::MouseUp);
        assert_eq!(*state.borrow(), InteractionState::Hover);

        // Send MouseLeave event
        callback(InteractionEvent::MouseLeave);
        assert_eq!(*state.borrow(), InteractionState::Idle);
    }

    #[test]
    fn test_comprehensive_state_coverage() {
        // Test all possible valid transitions from the table
        let test_cases = vec![
            // (initial_state, event, expected_state)
            (
                InteractionState::Idle,
                InteractionEvent::MouseEnter,
                InteractionState::Hover,
            ),
            (
                InteractionState::Idle,
                InteractionEvent::Focus,
                InteractionState::Focused,
            ),
            (
                InteractionState::Idle,
                InteractionEvent::Disable,
                InteractionState::Disabled,
            ),
            (
                InteractionState::Hover,
                InteractionEvent::MouseLeave,
                InteractionState::Idle,
            ),
            (
                InteractionState::Hover,
                InteractionEvent::MouseDown,
                InteractionState::Active,
            ),
            (
                InteractionState::Hover,
                InteractionEvent::Focus,
                InteractionState::Focused,
            ),
            (
                InteractionState::Hover,
                InteractionEvent::Disable,
                InteractionState::Disabled,
            ),
            (
                InteractionState::Active,
                InteractionEvent::MouseUp,
                InteractionState::Hover,
            ),
            (
                InteractionState::Active,
                InteractionEvent::MouseLeave,
                InteractionState::Idle,
            ),
            (
                InteractionState::Active,
                InteractionEvent::Disable,
                InteractionState::Disabled,
            ),
            (
                InteractionState::Focused,
                InteractionEvent::Blur,
                InteractionState::Idle,
            ),
            (
                InteractionState::Focused,
                InteractionEvent::MouseEnter,
                InteractionState::Hover,
            ),
            (
                InteractionState::Focused,
                InteractionEvent::Disable,
                InteractionState::Disabled,
            ),
            (
                InteractionState::Disabled,
                InteractionEvent::Enable,
                InteractionState::Idle,
            ),
        ];

        for (initial, event, expected) in test_cases {
            let mut sm = ButtonStateMachine::new();
            sm.state = initial;
            let result = sm.transition(event);
            assert_eq!(
                result,
                Some(expected),
                "Failed: {:?} + {:?} should be {:?}, got {:?}",
                initial,
                event,
                expected,
                result
            );
            assert_eq!(
                sm.state(),
                expected,
                "State mismatch after transition: {:?} + {:?}",
                initial,
                event
            );
        }
    }
}
