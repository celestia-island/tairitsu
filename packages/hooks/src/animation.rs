use std::{cell::RefCell, rc::Rc, time::Duration};

use tairitsu_vdom::Platform;

/// Animation update callback - called on each animation frame with the current progress
pub type AnimationCallback = Rc<dyn Fn(f32)>;

/// Handle for managing an active animation
pub struct AnimationHandle {
    state: Rc<RefCell<AnimationState>>,
    raf_id: Rc<RefCell<Option<u32>>>,
}

impl AnimationHandle {
    /// Cancel the animation and stop the requestAnimationFrame loop
    pub fn cancel(&self) {
        *self.state.borrow_mut() = AnimationState::Idle;
    }

    /// Check if the animation is still running
    pub fn is_running(&self) -> bool {
        *self.state.borrow() == AnimationState::Running
    }
}

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

impl EasingFunction {
    /// Evaluate the easing function at the given progress value.
    /// The input `t` is clamped to the range [0.0, 1.0].
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Self::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            Self::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            Self::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
        }
    }
}

/// Solve a cubic Bezier curve for a given parameter t.
/// Uses Newton-Raphson iteration to find the x coordinate that corresponds to t,
/// then evaluates the curve at that point to get the y coordinate.
///
/// # Arguments
/// * `x1, y1` - First control point
/// * `x2, y2` - Second control point
/// * `t` - Parameter value in [0, 1]
///
/// # Returns
/// The y value of the cubic Bezier curve at parameter t
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // Handle edge cases
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }

    // Newton-Raphson iteration to solve for x
    let mut x = t;
    for _ in 0..8 {
        // Evaluate cubic Bezier and its derivative at x
        let (bx, dbx) = cubic_bezier_x_and_derivative(x1, x2, x);
        if dbx.abs() < f32::EPSILON {
            break;
        }
        let error = bx - t;
        x = x - error / dbx;
        // Clamp to valid range
        x = x.clamp(0.0, 1.0);
    }

    // Evaluate y at the found x value
    cubic_bezier_y(0.0, y1, y2, 1.0, x)
}

/// Evaluate the x coordinate of a cubic Bezier curve and its derivative.
/// The curve is defined by control points (0, 0), (x1, *), (x2, *), (1, 1).
fn cubic_bezier_x_and_derivative(x1: f32, x2: f32, t: f32) -> (f32, f32) {
    // Coefficients for the cubic polynomial
    // B(t) = (1-t)^3 * P0 + 3*(1-t)^2*t * P1 + 3*(1-t)*t^2 * P2 + t^3 * P3
    // With P0 = 0 and P3 = 1 for x
    let inv_t = 1.0 - t;
    let inv_t2 = inv_t * inv_t;
    let t2 = t * t;

    // Calculate the cubic Bezier value at t
    let bx = 3.0 * inv_t2 * t * x1 + 3.0 * inv_t * t2 * x2 + t2 * t;

    // Calculate the derivative
    // B'(t) = 3*(1-t)^2 * (P1 - P0) + 6*(1-t)*t * (P2 - P1) + 3*t^2 * (P3 - P2)
    let dbx = 3.0 * inv_t2 * (x1 - 0.0) + 6.0 * inv_t * t * (x2 - x1) + 3.0 * t2 * (1.0 - x2);

    (bx, dbx)
}

/// Evaluate the y coordinate of a cubic Bezier curve at parameter t.
fn cubic_bezier_y(y0: f32, y1: f32, y2: f32, y3: f32, t: f32) -> f32 {
    let inv_t = 1.0 - t;
    let inv_t2 = inv_t * inv_t;
    let t2 = t * t;

    inv_t2 * inv_t * y0 + 3.0 * inv_t2 * t * y1 + 3.0 * inv_t * t2 * y2 + t2 * t * y3
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
    /// Optional callback invoked on each frame update
    on_update: Rc<RefCell<Option<AnimationCallback>>>,
    /// Current requestAnimationFrame ID (for cancellation)
    raf_id: Rc<RefCell<Option<u32>>>,
    /// Animation start time (for pause/resume)
    start_time: Rc<RefCell<Option<f64>>>,
    /// Paused time accumulator (for pause/resume)
    paused_time: Rc<RefCell<f64>>,
    /// Current iteration count
    current_iteration: Rc<RefCell<u32>>,
}

impl UseAnimation {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            state: Rc::new(RefCell::new(AnimationState::Idle)),
            progress: Rc::new(RefCell::new(0.0)),
            config,
            on_update: Rc::new(RefCell::new(None)),
            raf_id: Rc::new(RefCell::new(None)),
            start_time: Rc::new(RefCell::new(None)),
            paused_time: Rc::new(RefCell::new(0.0)),
            current_iteration: Rc::new(RefCell::new(0)),
        }
    }

    pub fn start(&self) {
        *self.state.borrow_mut() = AnimationState::Running;
    }

    /// Set a callback to be invoked on each animation frame update
    pub fn on_update<F: Fn(f32) + 'static>(&self, callback: F) {
        *self.on_update.borrow_mut() = Some(Rc::new(callback));
    }

    /// Clear the update callback
    pub fn clear_on_update(&self) {
        *self.on_update.borrow_mut() = None;
    }

    /// Start the animation with a platform reference
    /// This initiates the requestAnimationFrame loop
    pub fn start_with_platform<P>(&self, platform: &P) -> AnimationHandle
    where
        P: Platform + ?Sized,
    {
        *self.state.borrow_mut() = AnimationState::Running;
        *self.start_time.borrow_mut() = None;
        *self.paused_time.borrow_mut() = 0.0;
        *self.current_iteration.borrow_mut() = 0;

        self.start_raf_loop(platform);

        AnimationHandle {
            state: Rc::clone(&self.state),
            raf_id: Rc::clone(&self.raf_id),
        }
    }

    /// Internal method to start the requestAnimationFrame loop
    /// Note: This is a simplified implementation that only processes one frame.
    /// The callback should handle scheduling the next frame if needed.
    fn start_raf_loop<P>(&self, platform: &P)
    where
        P: Platform + ?Sized,
    {
        // Capture all necessary state
        let state = Rc::clone(&self.state);
        let progress = Rc::clone(&self.progress);
        let config = self.config.clone();
        let on_update = Rc::clone(&self.on_update);
        let raf_id = Rc::clone(&self.raf_id);
        let start_time = Rc::clone(&self.start_time);
        let paused_time = Rc::clone(&self.paused_time);
        let current_iteration = Rc::clone(&self.current_iteration);

        // Create the first frame callback
        let first_callback = Box::new(move |timestamp: f64| {
            // Initialize start time on first frame
            if start_time.borrow().is_none() {
                *start_time.borrow_mut() = Some(timestamp);
            }

            let start = start_time.borrow().unwrap();
            let elapsed = timestamp - start - *paused_time.borrow();
            let delay_ms = config.delay.as_millis() as f64;

            // Check if we're still in the delay period
            if elapsed < delay_ms {
                // Still in delay, don't update progress yet
                return;
            }

            // Calculate actual animation progress (after delay)
            let animation_elapsed = elapsed - delay_ms;
            let duration_ms = config.duration.as_millis() as f64;
            let raw_progress = (animation_elapsed / duration_ms) as f32;

            // Handle iterations
            let iteration_duration = 1.0;
            let total_iterations = config.iterations as f32;

            if raw_progress >= total_iterations * iteration_duration {
                // Animation complete
                *progress.borrow_mut() = 1.0;
                *state.borrow_mut() = AnimationState::Finished;
                *raf_id.borrow_mut() = None;

                // Call update callback with final progress
                if let Some(cb) = on_update.borrow().as_ref() {
                    cb(1.0);
                }
                return;
            }

            // Calculate current iteration
            let iter = (raw_progress / iteration_duration).floor() as u32;
            *current_iteration.borrow_mut() = iter;

            // Calculate progress within current iteration
            let iter_progress = raw_progress % iteration_duration;

            // Apply direction
            let adjusted_progress = match config.direction {
                AnimationDirection::Normal => iter_progress,
                AnimationDirection::Reverse => 1.0 - iter_progress,
                AnimationDirection::Alternate => {
                    if iter % 2 == 0 {
                        iter_progress
                    } else {
                        1.0 - iter_progress
                    }
                }
                AnimationDirection::AlternateReverse => {
                    if iter % 2 == 0 {
                        1.0 - iter_progress
                    } else {
                        iter_progress
                    }
                }
            };

            // Apply easing function
            let eased_progress = config.easing.evaluate(adjusted_progress);

            // Update progress
            *progress.borrow_mut() = eased_progress;

            // Call update callback
            if let Some(cb) = on_update.borrow().as_ref() {
                cb(eased_progress);
            }
        });

        // Schedule the first frame
        let id = platform.request_animation_frame(first_callback);
        *self.raf_id.borrow_mut() = Some(id);
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

    /// Resume animation with platform reference
    pub fn resume_with_platform<P>(&self, platform: &P)
    where
        P: Platform + ?Sized,
    {
        if *self.state.borrow() == AnimationState::Paused {
            *self.state.borrow_mut() = AnimationState::Running;
            self.start_raf_loop(platform);
        }
    }

    pub fn stop(&self) {
        *self.state.borrow_mut() = AnimationState::Idle;
        *self.progress.borrow_mut() = 0.0;
        *self.start_time.borrow_mut() = None;
        *self.paused_time.borrow_mut() = 0.0;
        *self.current_iteration.borrow_mut() = 0;
    }

    /// Stop animation with platform reference (allows cancelling rAF)
    pub fn stop_with_platform<P>(&self, platform: &P)
    where
        P: Platform + ?Sized,
    {
        *self.state.borrow_mut() = AnimationState::Idle;
        *self.progress.borrow_mut() = 0.0;
        *self.start_time.borrow_mut() = None;
        *self.paused_time.borrow_mut() = 0.0;
        *self.current_iteration.borrow_mut() = 0;

        if let Some(id) = *self.raf_id.borrow() {
            platform.cancel_animation_frame(id);
            *self.raf_id.borrow_mut() = None;
        }
    }

    pub fn finish(&self) {
        *self.state.borrow_mut() = AnimationState::Finished;
        *self.progress.borrow_mut() = 1.0;
        // Call update callback with final progress
        if let Some(cb) = self.on_update.borrow().as_ref() {
            cb(1.0);
        }
        // Clear raf_id
        *self.raf_id.borrow_mut() = None;
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

    #[test]
    fn test_easing_evaluate_linear() {
        let easing = EasingFunction::Linear;

        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(0.5), 0.5);
        assert_eq!(easing.evaluate(1.0), 1.0);
        // Test clamping
        assert_eq!(easing.evaluate(-0.5), 0.0);
        assert_eq!(easing.evaluate(1.5), 1.0);
    }

    #[test]
    fn test_easing_evaluate_ease() {
        let easing = EasingFunction::Ease;

        // At t=0 and t=1, should be exactly 0 and 1
        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(1.0), 1.0);

        // At t=0.5, should be approximately 0.5 (with some deviation due to easing)
        let y = easing.evaluate(0.5);
        assert!(y > 0.0 && y < 1.0);

        // Test monotonicity - should be increasing
        let y1 = easing.evaluate(0.25);
        let y2 = easing.evaluate(0.5);
        let y3 = easing.evaluate(0.75);
        assert!(y2 > y1);
        assert!(y3 > y2);
    }

    #[test]
    fn test_easing_evaluate_ease_in() {
        let easing = EasingFunction::EaseIn;

        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(1.0), 1.0);

        // EaseIn starts slow, so at t=0.5, y should be < 0.5
        let y = easing.evaluate(0.5);
        assert!(y < 0.5);
    }

    #[test]
    fn test_easing_evaluate_ease_out() {
        let easing = EasingFunction::EaseOut;

        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(1.0), 1.0);

        // EaseOut starts fast, so at t=0.5, y should be > 0.5
        let y = easing.evaluate(0.5);
        assert!(y > 0.5);
    }

    #[test]
    fn test_easing_evaluate_ease_in_out() {
        let easing = EasingFunction::EaseInOut;

        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(1.0), 1.0);

        // At t=0.5, should be approximately 0.5 (symmetric easing)
        let y = easing.evaluate(0.5);
        assert!((y - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_easing_evaluate_cubic_bezier() {
        // Test a custom cubic bezier
        let easing = EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0); // Linear-like

        assert_eq!(easing.evaluate(0.0), 0.0);
        assert_eq!(easing.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_cubic_bezier_symmetry() {
        // EaseInOut should be symmetric around t=0.5
        let easing = EasingFunction::EaseInOut;

        let y1 = easing.evaluate(0.25);
        let y2 = easing.evaluate(0.75);
        // y2 should be approximately 1 - y1
        assert!((y2 - (1.0 - y1)).abs() < 0.01);
    }
}
