//! Console interface implementation for SSR
//!
//! Logs to tracing instead of browser console.

use crate::host_state::SsrHostState;

/// Add console interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    _get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // Console functions don't need state, so we use func_wrap directly
    linker.func_wrap(
        "tairitsu-browser:full/console@0.2.0",
        "log",
        move |_caller: wasmtime::Caller<'_, T>, msg: String| {
            tracing::info!("{}", msg);
            Ok(())
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/console@0.2.0",
        "warn",
        move |_caller: wasmtime::Caller<'_, T>, msg: String| {
            tracing::warn!("{}", msg);
            Ok(())
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/console@0.2.0",
        "error",
        move |_caller: wasmtime::Caller<'_, T>, msg: String| {
            tracing::error!("{}", msg);
            Ok(())
        },
    )?;

    Ok(())
}
