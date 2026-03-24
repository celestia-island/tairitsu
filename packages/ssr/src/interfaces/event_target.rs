//! Event target interface implementation for SSR
//!
//! In SSR, event listeners are registered but never called (no-op).

use wasmtime_wasi::ResourceTable;

use crate::host_state::SsrHostState;

/// Add event-target interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: wasmtime_wasi::WasiView + Send,
{
    // Next listener ID - stored in resource table to be per-store
    // We'll use a simple counter approach

    linker.func_wrap(
        "tairitsu-browser:full/event-target@0.2.0",
        "add-event-listener",
        move |mut caller: wasmtime::Caller<'_, T>,
              _target: u64,
              _event_type: String,
              _use_capture: bool| -> Result<(Result<u64, String>,), wasmtime::Error> {
            // In SSR, just return a dummy listener ID
            // Events will never be dispatched
            let state = get_state(caller.data_mut());

            // Use a simple counter - allocate from resource table
            let listener_id = state.ctx().table_mut().push_any(Box::new(1u64)) as u64;

            Ok((Ok(listener_id),))
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/event-target@0.2.0",
        "remove-event-listener",
        move |_caller: wasmtime::Caller<'_, T>, _target: u64, _listener_id: u64| -> Result<(Result<(), String>,), wasmtime::Error> {
            // No-op in SSR
            Ok((Ok(()),))
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/event-target@0.2.0",
        "prevent-default",
        move |_caller: wasmtime::Caller<'_, T>, _event: u64| {
            // No-op in SSR
            Ok(())
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/event-target@0.2.0",
        "stop-propagation",
        move |_caller: wasmtime::Caller<'_, T>, _event: u64| {
            // No-op in SSR
            Ok(())
        },
    )?;

    Ok(())
}
