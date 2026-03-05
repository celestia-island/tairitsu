use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub duration: Duration,
    pub delay: Duration,
    pub iterations: u32,
    pub direction: AnimationDirection,
    pub easing: EasingFunction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(300),
            delay: Duration::from_millis(0),
            iterations: 1,
            direction: AnimationDirection::Normal,
            easing: EasingFunction::Ease,
        }
    }
}

pub struct UseAnimation {
    state: Rc<RefCell<AnimationState>>,
    progress: Rc<RefCell<f32>>,
    config: AnimationConfig,
}

impl UseAnimation {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            state: Rc::new(RefCell::new(AnimationState::Idle)),
            progress: Rc::new(RefCell::new(0.0)),
            config,
        }
    }

    pub fn start(&self) {
        *self.state.borrow_mut() = AnimationState::Running;
    }

    pub fn pause(&self) {
        if *self.state.borrow() == AnimationState::Running {
            *self.state.borrow_mut() = AnimationState::Paused;
        }
    }

    pub fn resume(&self) {
        if *self.state.borrow() == AnimationState::Paused {
            *self.state.borrow_mut() = AnimationState::Running;
        }
    }

    pub fn stop(&self) {
        *self.state.borrow_mut() = AnimationState::Idle;
        *self.progress.borrow_mut() = 0.0;
    }

    pub fn finish(&self) {
        *self.state.borrow_mut() = AnimationState::Finished;
        *self.progress.borrow_mut() = 1.0;
    }

    pub fn state(&self) -> AnimationState {
        *self.state.borrow()
    }

    pub fn progress(&self) -> f32 {
        *self.progress.borrow()
    }

    pub fn set_progress(&self, value: f32) {
        *self.progress.borrow_mut() = value.clamp(0.0, 1.0);
    }

    pub fn config(&self) -> &AnimationConfig {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        *self.state.borrow() == AnimationState::Running
    }

    pub fn is_finished(&self) -> bool {
        *self.state.borrow() == AnimationState::Finished
    }
}

pub fn use_animation(config: Option<AnimationConfig>) -> UseAnimation {
    UseAnimation::new(config.unwrap_or_default())
}

pub fn use_simple_animation(duration_ms: u64) -> UseAnimation {
    use_animation(Some(AnimationConfig {
        duration: Duration::from_millis(duration_ms),
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_states() {
        let anim = use_simple_animation(300);

        assert_eq!(anim.state(), AnimationState::Idle);
        assert_eq!(anim.progress(), 0.0);
        assert!(!anim.is_running());

        anim.start();
        assert_eq!(anim.state(), AnimationState::Running);
        assert!(anim.is_running());

        anim.pause();
        assert_eq!(anim.state(), AnimationState::Paused);
        assert!(!anim.is_running());

        anim.resume();
        assert_eq!(anim.state(), AnimationState::Running);

        anim.stop();
        assert_eq!(anim.state(), AnimationState::Idle);
        assert_eq!(anim.progress(), 0.0);
    }

    #[test]
    fn test_animation_progress() {
        let anim = use_simple_animation(300);

        anim.set_progress(0.5);
        assert_eq!(anim.progress(), 0.5);

        anim.set_progress(1.5);
        assert_eq!(anim.progress(), 1.0);

        anim.set_progress(-0.5);
        assert_eq!(anim.progress(), 0.0);
    }

    #[test]
    fn test_animation_config() {
        let config = AnimationConfig {
            duration: Duration::from_millis(500),
            delay: Duration::from_millis(100),
            iterations: 3,
            direction: AnimationDirection::Alternate,
            easing: EasingFunction::EaseInOut,
        };

        let anim = use_animation(Some(config.clone()));

        assert_eq!(anim.config().duration, Duration::from_millis(500));
        assert_eq!(anim.config().delay, Duration::from_millis(100));
        assert_eq!(anim.config().iterations, 3);
        assert_eq!(anim.config().direction, AnimationDirection::Alternate);
    }

    #[test]
    fn test_easing_functions() {
        let config = AnimationConfig {
            easing: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
            ..Default::default()
        };

        let anim = use_animation(Some(config));

        if let EasingFunction::CubicBezier(x1, y1, x2, y2) = anim.config().easing {
            assert!((x1 - 0.42).abs() < f32::EPSILON);
            assert!((y1 - 0.0).abs() < f32::EPSILON);
            assert!((x2 - 0.58).abs() < f32::EPSILON);
            assert!((y2 - 1.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected CubicBezier easing");
        }
    }
}
