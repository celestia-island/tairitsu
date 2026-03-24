//! Window interface implementation for SSR
//!
//! Returns simulated viewport dimensions.

use crate::host_state::SsrHostState;

/// Add window interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // match-media - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "match-media",
        move |_caller: wasmtime::Caller<'_, T>, _query: String| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy MediaQueryList handle
        },
    )?;

    // get-screen - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-screen",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy Screen handle
        },
    )?;

    // get-visual-viewport - returns None
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-visual-viewport",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // move-to - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "move-to",
        move |_caller: wasmtime::Caller<'_, T>, _x: i32, _y: i32| {
            Ok(())
        },
    )?;

    // move-by - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "move-by",
        move |_caller: wasmtime::Caller<'_, T>, _x: i32, _y: i32| {
            Ok(())
        },
    )?;

    // resize-to - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "resize-to",
        move |_caller: wasmtime::Caller<'_, T>, _width: i32, _height: i32| {
            Ok(())
        },
    )?;

    // resize-by - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "resize-by",
        move |_caller: wasmtime::Caller<'_, T>, _x: i32, _y: i32| {
            Ok(())
        },
    )?;

    // get-inner-width - return configured viewport width
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-inner-width",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<i32, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.config.viewport_width)
        },
    )?;

    // get-inner-height - return configured viewport height
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-inner-height",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<i32, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.config.viewport_height)
        },
    )?;

    // get-scroll-x - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-scroll-x",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // get-scroll-y - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-scroll-y",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // get-page-x-offset - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-page-x-offset",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // get-page-y-offset - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-page-y-offset",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // scroll - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "scroll",
        move |_caller: wasmtime::Caller<'_, T>, _options: Option<u64>| {
            Ok(())
        },
    )?;

    // scroll-to - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "scroll-to",
        move |_caller: wasmtime::Caller<'_, T>, _options: Option<u64>| {
            Ok(())
        },
    )?;

    // scroll-by - no-op
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "scroll-by",
        move |_caller: wasmtime::Caller<'_, T>, _options: Option<u64>| {
            Ok(())
        },
    )?;

    // get-location - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-location",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy Location handle
        },
    )?;

    // get-history - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-history",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy History handle
        },
    )?;

    // get-navigation - returns None
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-navigation",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-session-storage - returns None
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-session-storage",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-local-storage - returns None
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-local-storage",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-navigator - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-navigator",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy Navigator handle
        },
    )?;

    // get-document - returns None (document is accessed via platform-helpers)
    linker.func_wrap(
        "tairitsu-browser:full/window@0.2.0",
        "get-document",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    Ok(())
}
