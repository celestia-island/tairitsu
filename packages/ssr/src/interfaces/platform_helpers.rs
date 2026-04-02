//! Platform helpers interface implementation for SSR
//!
//! Returns simulated values for viewport dimensions and creates dummy observers.

use crate::host_state::SsrHostState;

/// Add platform-helpers interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // get-bounding-client-rect - returns zero rectangle in SSR
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "get-bounding-client-rect",
        move |_caller: wasmtime::Caller<'_, T>,
              _element: u64|
              -> Result<(f64, f64, f64, f64), wasmtime::Error> {
            // Return zero rectangle
            Ok((0.0, 0.0, 0.0, 0.0))
        },
    )?;

    // inner-width - return configured viewport width
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "inner-width",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<i32, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.viewport_width())
        },
    )?;

    // inner-height - return configured viewport height
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "inner-height",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<i32, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.viewport_height())
        },
    )?;

    // set-timeout - return dummy timer ID (callbacks never fire)
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "set-timeout",
        move |_caller: wasmtime::Caller<'_, T>,
              _callback_id: u64,
              _ms: i32|
              -> Result<i32, wasmtime::Error> {
            // Return a dummy timer ID
            Ok(1)
        },
    )?;

    // clear-timeout - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "clear-timeout",
        move |_caller: wasmtime::Caller<'_, T>, _id: i32| Ok(()),
    )?;

    // request-animation-frame - return dummy ID (callbacks never fire)
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "request-animation-frame",
        move |_caller: wasmtime::Caller<'_, T>,
              _callback_id: u64|
              -> Result<u32, wasmtime::Error> { Ok(1) },
    )?;

    // cancel-animation-frame - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "cancel-animation-frame",
        move |_caller: wasmtime::Caller<'_, T>, _id: u32| Ok(()),
    )?;

    // create-resize-observer - return dummy observer ID
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "create-resize-observer",
        move |_caller: wasmtime::Caller<'_, T>,
              _callback_id: u64|
              -> Result<u64, wasmtime::Error> { Ok(1) },
    )?;

    // observe-resize - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "observe-resize",
        move |_caller: wasmtime::Caller<'_, T>, _observer: u64, _element: u64| Ok(()),
    )?;

    // unobserve-resize - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "unobserve-resize",
        move |_caller: wasmtime::Caller<'_, T>, _observer: u64, _element: u64| Ok(()),
    )?;

    // disconnect-resize - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "disconnect-resize",
        move |_caller: wasmtime::Caller<'_, T>, _observer: u64| Ok(()),
    )?;

    // create-mutation-observer - return dummy observer ID
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "create-mutation-observer",
        move |_caller: wasmtime::Caller<'_, T>,
              _callback_id: u64|
              -> Result<u64, wasmtime::Error> { Ok(1) },
    )?;

    // observe-mutations - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "observe-mutations",
        move |_caller: wasmtime::Caller<'_, T>,
              _observer: u64,
              _element: u64,
              _options: Option<u64>| { Ok(()) },
    )?;

    // disconnect-mutation - no-op
    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "disconnect-mutation",
        move |_caller: wasmtime::Caller<'_, T>, _observer: u64| Ok(()),
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "get-element-by-id",
        move |_caller: wasmtime::Caller<'_, T>,
              _id: String|
              -> Result<Option<u64>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "query-selector",
        move |_caller: wasmtime::Caller<'_, T>,
              _selector: String|
              -> Result<Option<u64>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "query-selector-all",
        move |_caller: wasmtime::Caller<'_, T>,
              _selector: String|
              -> Result<Vec<u64>, wasmtime::Error> { Ok(vec![]) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "element-from-point",
        move |_caller: wasmtime::Caller<'_, T>,
              _x: i32,
              _y: i32|
              -> Result<Option<u64>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "element-closest",
        move |_caller: wasmtime::Caller<'_, T>,
              _element: u64,
              _selector: String|
              -> Result<Option<u64>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "get-scroll-y",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<f64, wasmtime::Error> { Ok(0.0) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "scroll-to",
        move |_caller: wasmtime::Caller<'_, T>,
              _top: f64,
              _behavior: String|
              -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "on-scroll",
        move |_caller: wasmtime::Caller<'_, T>, _callback_id: u64| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "on-resize-callback",
        move |_caller: wasmtime::Caller<'_, T>, _callback_id: u64| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "copy-to-clipboard",
        move |_caller: wasmtime::Caller<'_, T>, _text: String| -> Result<bool, wasmtime::Error> {
            Ok(false)
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "read-clipboard",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "prefers-dark-mode",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<bool, wasmtime::Error> { Ok(false) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "get-element-rect-by-id",
        move |_caller: wasmtime::Caller<'_, T>,
              _id: String|
              -> Result<Option<(f64, f64, f64, f64)>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "get-bounding-rect-by-class",
        move |_caller: wasmtime::Caller<'_, T>,
              _class_name: String,
              _element: u64|
              -> Result<Option<(f64, f64, f64, f64)>, wasmtime::Error> { Ok(None) },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/platform-helpers@0.2.0",
        "request-fullscreen",
        move |_caller: wasmtime::Caller<'_, T>, _element: u64| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    Ok(())
}
