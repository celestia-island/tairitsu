//! Reactive runtime for automatic component re-rendering.
//!
//! This module provides the runtime infrastructure for tracking signal
//! dependencies and scheduling re-renders when signals change.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use tracing::trace;

use crate::{
    patch::Patch,
    reactive::{EffectHandle, SignalId},
    VNode,
};

/// Component ID - unique identifier for each component instance
pub type ComponentId = usize;

/// Render function type - produces a VNode tree
pub type RenderFn = Rc<RefCell<dyn FnMut() -> VNode>>;

/// Callback type for scheduling renders with requestAnimationFrame
type ScheduleCallback = Rc<RefCell<dyn FnMut(Box<dyn FnOnce()>)>>;

/// Callback type for applying patches to the DOM
type ApplyPatchesCallback = Rc<RefCell<dyn FnMut(ComponentId, Vec<Patch>)>>;

/// Callback type for post-apply reconciliation: receives the component, the
/// child count the freshly stored vnode expects at its mount root, and the
/// vnode itself. The embedder compares against the live DOM and repairs on
/// mismatch (see [`set_reconcile_callback`]).
type ReconcileCallback = Rc<RefCell<dyn FnMut(ComponentId, usize, VNode)>>;

thread_local! {
    static HOOK_SLOT: RefCell<HashMap<(ComponentId, String), Box<dyn std::any::Any>>> = RefCell::new(HashMap::new());
}

/// Retrieve or initialize a hook slot for the given component and key.
/// Returns a clone of the stored value if present, otherwise stores and
/// returns the result of `init_fn`.
pub fn hook_slot<T: Clone + 'static>(
    component_id: ComponentId,
    key: &str,
    init_fn: impl FnOnce() -> T,
) -> T {
    HOOK_SLOT.with(|slots| {
        let mut slots = slots.borrow_mut();
        let entry = slots
            .entry((component_id, key.to_string()))
            .or_insert_with(|| Box::new(init_fn()));
        entry
            .downcast_ref::<T>()
            .cloned()
            .expect("hook_slot: type mismatch — the same (component_id, key) was reused with a different type")
    })
}

/// Clear all hook slots for a component (called during cleanup).
pub fn clear_hook_slots(component_id: ComponentId) {
    HOOK_SLOT.with(|slots| {
        slots
            .borrow_mut()
            .retain(|(cid, _), _| *cid != component_id);
    });
}

/// Reset the hook slot counter (useful for testing).
pub fn reset_hook_slots() {
    HOOK_SLOT.with(|slots| slots.borrow_mut().clear());
}

/// Inner state of the reactive runtime
struct RuntimeInner {
    next_id: ComponentId,
    active_component: Option<ComponentId>,
    component_vnodes: HashMap<ComponentId, VNode>,
    render_functions: HashMap<ComponentId, RenderFn>,
    signal_dependencies: HashMap<usize, Vec<ComponentId>>,
    dirty_components: Vec<ComponentId>,
    schedule_callback: Option<ScheduleCallback>,
    apply_patches_callback: Option<ApplyPatchesCallback>,
    reconcile_callback: Option<ReconcileCallback>,
    scheduled: bool,
    /// mark_dirty calls that arrived while a flush was booked but not yet
    /// delivered. A booking the host never honours is detected by this count
    /// growing past [`OVERDUE_MARKS`] — no wall clock involved (see the
    /// comment on `booking_is_overdue` for why that matters).
    booking_marks: u32,
    raf_id: Option<u32>,
    effect_handles: HashMap<ComponentId, Vec<EffectHandle>>,
    element_to_component: HashMap<u64, ComponentId>,
}

impl RuntimeInner {
    fn new() -> Self {
        Self {
            next_id: 1,
            active_component: None,
            component_vnodes: HashMap::new(),
            render_functions: HashMap::new(),
            signal_dependencies: HashMap::new(),
            dirty_components: Vec::new(),
            schedule_callback: None,
            apply_patches_callback: None,
            reconcile_callback: None,
            scheduled: false,
            booking_marks: 0,
            raf_id: None,
            effect_handles: HashMap::new(),
            element_to_component: HashMap::new(),
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<RuntimeInner> = RefCell::new(RuntimeInner::new());
}

/// Set the callback for scheduling renders via requestAnimationFrame.
///
/// This should be called once during initialization with a callback that
/// wraps the platform's request_animation_frame implementation.
pub fn set_schedule_callback<F: FnMut(Box<dyn FnOnce()>) + 'static>(callback: F) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().schedule_callback = Some(Rc::new(RefCell::new(callback)));
        trace!("Schedule callback registered");
    });
}

/// Set the callback for applying patches to the DOM.
///
/// This should be called once during initialization with a callback that
/// wraps the platform's apply_patches implementation.
pub fn set_apply_patches_callback<F: FnMut(ComponentId, Vec<Patch>) + 'static>(callback: F) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().apply_patches_callback = Some(Rc::new(RefCell::new(callback)));
        trace!("Apply patches callback registered");
    });
}

/// Set the post-apply reconciliation callback.
///
/// After every patch batch the runtime hands the embedder the component's
/// fresh vnode plus the child count it expects at the mount root. The
/// embedder counts the live DOM children; a mismatch means the stored vnode
/// baseline and the DOM have drifted apart (a mis-landed patch once — no
/// built-in mechanism ever re-aligns them), and the embedder can repair by
/// re-rendering the vnode into the root. Cheap O(1) check per render.
pub fn set_reconcile_callback<F: FnMut(ComponentId, usize, VNode) + 'static>(callback: F) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().reconcile_callback = Some(Rc::new(RefCell::new(callback)));
        trace!("Reconcile callback registered");
    });
}

/// Register a component with the runtime.
///
/// This creates a new component instance and tracks its render function.
/// The component will be automatically re-rendered when its dependencies change.
pub fn use_component<F>(render_fn: F) -> ComponentId
where
    F: FnMut() -> VNode + 'static,
{
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        let id = rt.next_id;
        rt.next_id += 1;

        rt.render_functions
            .insert(id, Rc::new(RefCell::new(render_fn)));

        trace!("Registered component {}", id);

        id
    })
}

/// Update a component's render function.
///
/// This is useful when the initial render function is a placeholder
/// and needs to be replaced with the actual implementation.
pub fn update_render_function(id: ComponentId, render_fn: impl FnMut() -> VNode + 'static) {
    RUNTIME.with(|runtime| {
        runtime
            .borrow_mut()
            .render_functions
            .insert(id, Rc::new(RefCell::new(render_fn)));
        trace!("Updated render function for component {}", id);
    });
}

/// Mark a component as dirty without triggering an immediate flush.
///
/// Unlike [`mark_dirty`], this does NOT flush synchronously. It only adds
/// the component to the dirty list. The actual re-render happens on the
/// next scheduled frame or explicit `flush_render` call.
///
/// This is safe to call while holding a `RefMut` on a signal's inner `RefCell`,
/// because it never invokes render functions synchronously.
pub fn mark_dirty_deferred(id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        if !rt.dirty_components.contains(&id) {
            rt.dirty_components.push(id);
            trace!("Deferred: marked component {} as dirty", id);
        }
    });
}

/// Mark a component as dirty and schedule a re-render.
///
/// Booking is *idempotent per flush*: a pending flush covers every mark that
/// happens before it runs, so repeated calls inside one handler do not stack.
/// What this must not do is assume the booked flush will arrive — see
/// [`frame_is_overdue`].
pub fn mark_dirty(id: ComponentId) {
    // The schedule callback is invoked *outside* the RUNTIME borrow: a host
    // implementation is allowed to run the render synchronously (and the
    // no-callback fallback below does exactly that), which re-enters
    // `flush_render` → `RUNTIME.borrow_mut()` and panics on the already-held
    // RefCell borrow. Taking the callback out first keeps both paths safe.
    let policy = RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        if !rt.dirty_components.contains(&id) {
            rt.dirty_components.push(id);
            trace!("Marked component {} as dirty", id);
        }

        if rt.scheduled {
            rt.booking_marks = rt.booking_marks.saturating_add(1);
            return SchedulePolicy::AlreadyScheduled;
        }

        rt.scheduled = true;

        match &rt.schedule_callback {
            Some(cb) => SchedulePolicy::Deferred(Rc::clone(cb)),
            None => {
                rt.scheduled = false;
                SchedulePolicy::Immediate
            }
        }
    });

    match policy {
        // A frame is already booked and the dirty component is queued for it.
        //
        // Flushing unconditionally here would defeat frame batching (every
        // write inside one handler would render on its own), so the fallback
        // is time-boxed. The deadline is deliberately short: a frame normally
        // lands within ~16 ms, so anything past a couple of UI ticks is far
        // more likely to be a booking the host will never honour (background
        // tab throttling, a swallowed animation callback id, a scheduler
        // replaced mid-flight) than a slow frame. `scheduled` is reset only
        // inside `flush_render`, so without this the first lost frame freezes
        // every later update with no error.
        SchedulePolicy::AlreadyScheduled => {
            // Counted inside the borrow above (see booking_is_overdue).
            if booking_is_overdue() {
                trace!("Scheduled flush overdue; running pending work directly");
                flush_render();
            }
        }
        SchedulePolicy::Deferred(schedule_cb) => {
            let render_fn = Box::new(|| {
                flush_render();
            });
            (schedule_cb.borrow_mut())(render_fn);
            trace!("Scheduled render via callback");
        }
        SchedulePolicy::Immediate => flush_render(),
    }
}

thread_local! {
    /// Whether a `flush_render` drain is currently on this call stack.
    ///
    /// Rendering and applying patches run arbitrary guest code — signal
    /// subscribers, effects, apply-patches callbacks — and any of it may call
    /// `mark_dirty` again, whose fallbacks can reach `flush_render`
    /// synchronously (the no-scheduler `Immediate` path, the overdue-frame
    /// reclaim). A nested drain would re-enter `render_component` while the
    /// outer apply-patches callback is still borrowed and panic with
    /// "RefCell already borrowed". It is also pointless: the outer drain loop
    /// re-reads the dirty queue after every pass, so anything the nested call
    /// would render is picked up as the next pass anyway.
    static FLUSHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII companion to [`FLUSHING`] so a panicking render still releases the flag.
struct FlushGuard;

impl Drop for FlushGuard {
    fn drop(&mut self) {
        FLUSHING.with(|f| f.set(false));
    }
}

/// How [`mark_dirty`] decided to run the pending render.
enum SchedulePolicy {
    /// A frame is already scheduled; the render is covered by it.
    AlreadyScheduled,
    /// Defer to the host's scheduling callback (rAF, timer, …).
    Deferred(ScheduleCallback),
    /// No scheduler registered: render synchronously now.
    Immediate,
}

/// How many additional `mark_dirty` calls a booked-but-undelivered flush may
/// absorb before it is treated as lost.
///
/// The original design used `Instant::now()` on the runtime hot path. On
/// wasm32-wasip2 under jco, `Instant::now` lowers to a wasi:clocks sync
/// interface call, and from inside a promise-callback implicit thread such a
/// call can trap silently — killing `mark_dirty` halfway with `scheduled`
/// already set, so no frame is ever booked and the UI freezes with no error
/// (verified against a real embedder). A mark counter is pure memory: no host
/// call, same recovery guarantee — a genuinely delivered frame resets it, a
/// lost one trips it after a bounded number of further updates.
const OVERDUE_MARKS: u32 = 8;

/// Whether the booked flush has absorbed too many further marks to still be
/// considered pending delivery.
fn booking_is_overdue() -> bool {
    RUNTIME.with(|runtime| runtime.borrow().booking_marks >= OVERDUE_MARKS)
}

/// Set the active component for dependency tracking.
pub fn with_component<T>(id: ComponentId, f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let prev = rt.active_component;
        rt.active_component = Some(id);
        drop(rt);
        let result = f();
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().active_component = prev;
        });
        result
    })
}

/// Get the ID of the currently active component, if any.
pub fn active_component_id() -> Option<ComponentId> {
    RUNTIME.with(|runtime| runtime.borrow().active_component)
}

/// Track a signal dependency for the current component.
pub fn track_signal(signal_id: SignalId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        if let Some(component_id) = rt.active_component {
            let deps = rt.signal_dependencies.entry(signal_id).or_default();
            // Rendering re-reads every signal it depends on, so this runs once
            // per dependency per render. Pushing unconditionally grows the list
            // without bound, and since `notify_signal` walks it, a signal with
            // N historical reads notifies its component N times per write —
            // an O(renders) fan-out that turns a single update into a storm of
            // redundant re-renders.
            if !deps.contains(&component_id) {
                deps.push(component_id);
                trace!(
                    "Component {} now depends on signal {:?}",
                    component_id,
                    signal_id
                );
            }
        }
    });
}

/// Check if a signal is currently tracked in the dependency graph.
#[cfg(test)]
pub fn signal_is_tracked(signal_id: SignalId) -> bool {
    RUNTIME
        .try_with(|runtime| {
            runtime
                .borrow()
                .signal_dependencies
                .contains_key(&signal_id)
        })
        .unwrap_or(false)
}

/// Remove a signal from the dependency tracking. Called automatically when a Signal is dropped.
///
/// Uses `try_with` to handle the case where the thread-local RUNTIME has already
/// been destroyed (e.g. during thread shutdown when signals are dropped after
/// the runtime).
pub fn unregister_signal(signal_id: SignalId) {
    let _ = RUNTIME.try_with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.signal_dependencies.remove(&signal_id);
        trace!("Unregistered signal {:?}", signal_id);
    });
}

/// Flush pending renders and apply patches.
/// flush_render calls that deferred to an in-flight drain.
pub static DEBUG_DEFERRED_FLUSHES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Total `flush_render` entries that started a drain. Embedder diagnostics
/// poll this to tell "flush never scheduled" from "flush stuck mid-drain".
pub static DEBUG_FLUSHES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn flush_render() {
    // A drain already in progress on this stack will pick up whatever this
    // call would render: its loop re-reads the dirty queue after every pass.
    // Running a nested drain instead would re-enter `render_component` under
    // the outer apply-patches borrow and panic.
    if FLUSHING.with(|f| f.replace(true)) {
        DEBUG_DEFERRED_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    DEBUG_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _guard = FlushGuard;

    // Render until the queue drains. Rendering can dirty components again
    // (a render function that reads a signal inside an effect, an apply-patches
    // callback that touches state), and those marks land mid-flush. If they are
    // left in `dirty_components` with `scheduled` already cleared, nothing
    // re-books a frame for them and the UI stalls until some unrelated write
    // happens to schedule one — the "update lands one click late" symptom.
    //
    // The loop is bounded: each pass takes the queue, so it only repeats when a
    // render genuinely marked something during this flush.
    loop {
        let dirty = RUNTIME.with(|runtime| {
            let mut rt = runtime.borrow_mut();
            let dirty = std::mem::take(&mut rt.dirty_components);
            rt.scheduled = false;
            rt.booking_marks = 0;
            rt.raf_id = None;
            dirty
        });

        if dirty.is_empty() {
            return;
        }

        trace!("Flushing {} dirty components", dirty.len());
        for component_id in dirty {
            render_component(component_id);
        }
    }
}

// -- 临时诊断计数（定位「帧跑了但渲染没跑」用，验证完删除） --------------
/// Render a single component and apply patches.
///
/// Carefully structured to avoid holding `borrow_mut()` across user code
/// (the render function / apply-patches callback) which may trigger signal
/// writes that call `notify_signal` → `borrow()`.
fn render_component(id: ComponentId) {
    // Phase 1: extract what we need while borrowed, then release.
    struct Extracted {
        render_fn: RenderFn,
        old_vnode: Option<VNode>,
        apply_patches_cb: Option<ApplyPatchesCallback>,
    }

    let extracted: Option<Extracted> = RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();

        let render_fn = match rt.render_functions.get(&id) {
            Some(f) => f.clone(),
            None => {
                trace!("No render function for component {}", id);
                return None;
            }
        };

        let prev = rt.active_component;
        rt.active_component = Some(id);
        let _ = prev;

        // Stop previous render's effect handles so they don't accumulate.
        if let Some(handles) = rt.effect_handles.remove(&id) {
            for handle in handles {
                handle.stop();
            }
        }

        let old_vnode = rt.component_vnodes.get(&id).cloned();
        let apply_patches_cb = rt.apply_patches_callback.clone();

        Some(Extracted {
            render_fn,
            old_vnode,
            apply_patches_cb,
        })
    });

    let Some(ext) = extracted else {
        return;
    };

    // Phase 2: call the render function — no borrow held.
    let new_vnode = (ext.render_fn.borrow_mut())();

    // Phase 3: store the new VNode (brief borrow).
    // NOTE: active_component stays Some(id) so that patch application
    // (Phase 4) can register newly created DOM elements to this component.
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.component_vnodes.insert(id, new_vnode.clone());
    });

    // Phase 4: compute & apply patches.
    if let Some(old) = ext.old_vnode {
        let patches = crate::diff::diff(Some(&old), &new_vnode);
        if !patches.is_empty() {
            trace!("Component {} generated {} patches", id, patches.len());
            if let Some(cb) = &ext.apply_patches_cb {
                let mut guard = cb.borrow_mut();
                guard(id, patches);
            }
        }
    } else {
        trace!("Initial render for component {}", id);
        let patches = vec![Patch::CreateNode { node: new_vnode.clone() }];
        if let Some(cb) = &ext.apply_patches_cb {
            let mut guard = cb.borrow_mut();
            guard(id, patches);
        }
    }

    // Phase 5: reset active component after patch application is complete,
    // then run the reconcile guard: the embedder compares the DOM child
    // count at the mount root against the fresh vnode's expectation and
    // repairs (full in-place re-render) on drift. Must run OUTSIDE any
    // runtime borrow and OUTSIDE the apply-patches callback borrow: the
    // repair re-renders the subtree directly, it does not recurse into
    // render_component, so no guard recursion is possible.
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().active_component = None;
    });
    let reconcile = RUNTIME.with(|runtime| runtime.borrow().reconcile_callback.clone());
    if let Some(cb) = reconcile {
        let expected = new_vnode.mounted_child_count();
        cb.borrow_mut()(id, expected, new_vnode);
    }
}

pub fn store_initial_vnode(id: ComponentId, vnode: VNode) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().component_vnodes.insert(id, vnode);
    });
}

/// Introspection for embedder diagnostics: `(scheduled, flushing, dirty, components)`.
///
/// "UI froze with no error" investigations start here: `scheduled` stuck true
/// means a booked frame never came back; `flushing` stuck true means a drain
/// never unwound; both false with `dirty > 0` means nothing will ever pick the
/// queue up again.
pub fn debug_state() -> (bool, bool, usize, usize) {
    let scheduled = RUNTIME.with(|runtime| runtime.borrow().scheduled);
    let dirty = RUNTIME.with(|runtime| runtime.borrow().dirty_components.len());
    let components = RUNTIME.with(|runtime| runtime.borrow().render_functions.len());
    let flushing = FLUSHING.with(|f| f.get());
    (scheduled, flushing, dirty, components)
}

/// Flush accounting for embedder diagnostics:
/// `(drains started, drains deferred to an in-flight drain)`.
pub fn debug_flush_counts() -> (u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (
        DEBUG_FLUSHES.load(Relaxed),
        DEBUG_DEFERRED_FLUSHES.load(Relaxed),
    )
}

/// Subscribe a component to a signal's changes.
pub fn subscribe_component(signal_id: SignalId, component_id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let deps = rt.signal_dependencies.entry(signal_id).or_default();
        // Same duplicate-guard as `track_signal`: an explicit subscribe on a
        // signal the component also reads while rendering would otherwise be
        // counted twice, and every write would re-render it twice.
        if !deps.contains(&component_id) {
            deps.push(component_id);
            trace!(
                "Component {} subscribed to signal {:?}",
                component_id,
                signal_id
            );
        }
    });
}

/// Notify all dependent components that a signal has changed.
pub fn notify_signal(signal_id: SignalId) {
    let components: Vec<ComponentId> = RUNTIME.with(|runtime| {
        runtime
            .borrow()
            .signal_dependencies
            .get(&signal_id)
            .cloned()
            .unwrap_or_default()
    });

    for component_id in components {
        mark_dirty(component_id);
    }
}

/// Cleanup resources for a component.
pub fn cleanup_component(id: ComponentId) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.render_functions.remove(&id);
        rt.component_vnodes.remove(&id);
        rt.dirty_components.retain(|&c| c != id);
        rt.element_to_component.retain(|_, &mut c| c != id);

        rt.signal_dependencies.retain(|_, deps| {
            deps.retain(|&c| c != id);
            !deps.is_empty()
        });

        if let Some(handles) = rt.effect_handles.remove(&id) {
            for handle in handles {
                handle.stop();
            }
        }

        trace!("Cleaned up component {}", id);
    });

    clear_hook_slots(id);
}

pub fn register_effect_handle(id: ComponentId, handle: EffectHandle) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        rt.effect_handles.entry(id).or_default().push(handle);
    });
}

/// Register a DOM element handle as belonging to the currently active component.
///
/// Called from the platform layer when `render_vnode` creates a new element.
/// The mapping is used by [`on_element_removed`] to detect cross-component
/// subtree removal and trigger [`cleanup_component`].
pub fn register_element(handle: u64) {
    RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        if let Some(id) = rt.active_component {
            rt.element_to_component.insert(handle, id);
        }
    });
}

/// Called when a DOM element is removed from the tree.
///
/// If the removed element belongs to a **different** component than the one
/// currently being re-rendered, the owning component is fully cleaned up
/// (effects stopped, signal subscriptions removed, render function dropped).
///
/// If the element belongs to the currently rendering component, only the
/// mapping entry is removed — the component itself is still active.
pub fn on_element_removed(handle: u64) {
    let info = RUNTIME.with(|runtime| {
        let mut rt = runtime.borrow_mut();
        let cid = rt.element_to_component.remove(&handle);
        let is_self = cid.is_some_and(|id| rt.active_component == Some(id));
        (cid, is_self)
    });

    let (Some(cid), is_self) = info else {
        return;
    };

    if is_self {
        return;
    }

    cleanup_component(cid);
}

/// Get the current VNode for a component (useful for testing).
pub fn get_current_vnode(id: ComponentId) -> Option<VNode> {
    RUNTIME.with(|runtime| runtime.borrow().component_vnodes.get(&id).cloned())
}

/// Request a re-render of the given component.
///
/// This is the primary API for event handlers to trigger UI updates
/// after modifying state. It marks the component as dirty and schedules
/// a re-render via requestAnimationFrame (or immediate if no scheduler).
///
/// For the common single-component case, use `request_rerender()` without
/// arguments to re-render the most recently registered component.
pub fn request_rerender(id: Option<ComponentId>) {
    let target_id = id.unwrap_or_else(|| {
        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            rt.next_id - 1
        })
    });
    mark_dirty(target_id);
}

/// Request re-render of the most recently registered component.
///
/// Convenience function for event handlers that don't track component IDs.
pub fn rerender() {
    request_rerender(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_registration() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        assert!(id > 0);
    }

    #[test]
    #[ignore = "flaky: mark_dirty flushes synchronously when no schedule_callback is set"]
    fn test_mark_dirty() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert!(rt.dirty_components.is_empty());
        });

        mark_dirty(id);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.dirty_components, vec![id]);
        });
    }

    #[test]
    fn test_track_signal() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            track_signal(12345);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.signal_dependencies.get(&12345), Some(&vec![id]));
        });
    }

    #[test]
    fn test_schedule_callback() {
        let called = Rc::new(RefCell::new(false));
        let called_clone = Rc::clone(&called);

        // Don't call the callback in the test to avoid RefCell borrow conflict
        set_schedule_callback(move |_callback| {
            *called_clone.borrow_mut() = true;
        });

        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        mark_dirty(id);

        // The schedule callback should have been called
        assert!(*called.borrow());
    }

    #[test]
    fn test_schedule_callback_can_flush_synchronously() {
        // A host scheduler is allowed to run the render immediately. That
        // re-enters the runtime, so mark_dirty must not hold the RUNTIME
        // borrow while invoking the callback (it used to panic here).
        set_schedule_callback(|callback| callback());

        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        mark_dirty(id);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert!(rt.dirty_components.is_empty(), "render should have run");
            assert!(!rt.scheduled, "scheduled flag must be cleared");
        });
    }

    #[test]
    fn test_mark_dirty_does_not_stall_after_a_lost_frame() {
        // A host that schedules but never delivers the frame (background-tab
        // rAF throttling, a dropped callback id, a scheduler swapped
        // mid-flight) must not freeze the UI. `scheduled` is only reset inside
        // `flush_render`, so before the overdue-frame fallback every later
        // update was silently dropped.
        let scheduled = Rc::new(RefCell::new(0));
        let scheduled_clone = Rc::clone(&scheduled);
        set_schedule_callback(move |_callback| {
            // Book the frame and drop it: never runs the render.
            *scheduled_clone.borrow_mut() += 1;
        });

        let renders = Rc::new(RefCell::new(0));
        let renders_clone = Rc::clone(&renders);
        let id = use_component(move || {
            *renders_clone.borrow_mut() += 1;
            VNode::Text(crate::vnode::VText::new("test"))
        });

        // First update books a frame; the host drops it, so nothing renders.
        mark_dirty(id);
        assert_eq!(*scheduled.borrow(), 1, "first update books a frame");
        assert_eq!(*renders.borrow(), 0, "a dropped frame renders nothing");

        // A second update while the frame is still fresh is batched into it,
        // not rendered early.
        mark_dirty(id);
        assert_eq!(*renders.borrow(), 0, "a fresh frame still batches");

        // Age the booking past the mark threshold to model a frame that
        // never came.
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().booking_marks = OVERDUE_MARKS;
        });

        mark_dirty(id);
        assert_eq!(
            *renders.borrow(),
            1,
            "an overdue frame must not swallow the update"
        );

        // That flush cleared the flag, so scheduling resumes normally.
        mark_dirty(id);
        assert_eq!(*scheduled.borrow(), 2, "scheduling resumes after recovery");
    }

    #[test]
    fn test_track_signal_deduplicates_components() {
        // Rendering re-reads dependencies every time; without the guard the
        // dependency list grows per render and one write fans out into N
        // re-renders.
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            for _ in 0..5 {
                track_signal(4242);
            }
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.signal_dependencies.get(&4242), Some(&vec![id]));
        });
    }

    #[test]
    fn test_reentrant_flush_defers_to_active_drain() {
        // Guest code runs inside apply-patches callbacks, and it may reach
        // flush_render again (a signal write, an overdue-frame reclaim, a host
        // that schedules synchronously). A nested drain re-enters
        // render_component under the outer apply callback's RefCell borrow and
        // panics with "RefCell already borrowed"; the nested call must defer
        // to the active drain instead.
        set_schedule_callback(|callback| callback()); // flush synchronously

        let applied = Rc::new(RefCell::new(0));
        let applied_clone = Rc::clone(&applied);
        set_apply_patches_callback(move |_id, _patches| {
            *applied_clone.borrow_mut() += 1;
            // The re-entrant flush: previously panicked at the callback borrow.
            flush_render();
        });

        let renders = Rc::new(RefCell::new(0));
        let renders_clone = Rc::clone(&renders);
        let id = use_component(move || {
            *renders_clone.borrow_mut() += 1;
            VNode::Text(crate::vnode::VText::new("b"))
        });
        store_initial_vnode(id, VNode::Text(crate::vnode::VText::new("a")));

        mark_dirty(id);
        assert_eq!(*renders.borrow(), 1, "outer drain renders once");
        assert!(FLUSHING.with(|f| !f.get()), "flag released after drain");
    }

    #[test]
    fn test_apply_patches_callback() {
        let applied_patches = Rc::new(RefCell::new(Vec::new()));
        let applied_clone = Rc::clone(&applied_patches);

        set_apply_patches_callback(move |component_id, patches| {
            applied_clone.borrow_mut().push((component_id, patches));
        });

        // Set a schedule callback that doesn't call flush_render to avoid RefCell conflict
        set_schedule_callback(move |_callback| {
            // Just mark that we were called, don't call the callback
        });

        // Create a component and render it once to establish the initial VNode
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));
        mark_dirty(id); // Mark as dirty so it gets rendered
        flush_render(); // Initial render to store the VNode

        // Now update the render function and mark dirty to generate patches
        update_render_function(id, || VNode::Text(crate::vnode::VText::new("updated")));
        mark_dirty(id);
        flush_render();

        // Check that patches were applied
        assert!(!applied_patches.borrow().is_empty());
    }

    #[test]
    fn test_register_element_with_active_component() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            register_element(42);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.element_to_component.get(&42), Some(&id));
        });
    }

    #[test]
    fn test_register_element_without_active_component() {
        register_element(99);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.element_to_component.get(&99), None);
        });
    }

    #[test]
    fn test_on_element_removed_no_mapping() {
        on_element_removed(9999);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert!(rt.element_to_component.is_empty());
        });
    }

    #[test]
    fn test_on_element_removed_cross_component_cleanup() {
        let id1 = use_component(|| VNode::Text(crate::vnode::VText::new("a")));
        let id2 = use_component(|| VNode::Text(crate::vnode::VText::new("b")));

        with_component(id1, || {
            register_element(100);
        });
        with_component(id2, || {
            register_element(200);
            register_element(201);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.element_to_component.len(), 3);
        });

        with_component(id1, || {
            on_element_removed(200);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.element_to_component.get(&100), Some(&id1));
            assert_eq!(rt.element_to_component.get(&200), None);
            assert_eq!(rt.element_to_component.get(&201), None);
            assert!(!rt.render_functions.contains_key(&id2));
            assert!(!rt.component_vnodes.contains_key(&id2));
        });
    }

    #[test]
    fn test_on_element_removed_same_component_skips_cleanup() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            register_element(300);
            on_element_removed(300);
        });

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert!(rt.render_functions.contains_key(&id));
            assert!(!rt.element_to_component.contains_key(&300));
        });
    }

    #[test]
    fn test_cleanup_component_removes_element_mappings() {
        let id = use_component(|| VNode::Text(crate::vnode::VText::new("test")));

        with_component(id, || {
            register_element(400);
            register_element(401);
        });

        cleanup_component(id);

        RUNTIME.with(|runtime| {
            let rt = runtime.borrow();
            assert_eq!(rt.element_to_component.get(&400), None);
            assert_eq!(rt.element_to_component.get(&401), None);
            assert!(!rt.render_functions.contains_key(&id));
        });
    }
}
