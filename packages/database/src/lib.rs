pub mod providers;

#[cfg(any(
    all(feature = "cloudflare", feature = "native"),
    all(feature = "cloudflare", feature = "wasi"),
    all(feature = "native", feature = "wasi")
))]
compile_error!(
    "Only one of the `cloudflare`, `native`, or `wasi` features can be enabled at a time."
);

#[cfg(not(any(feature = "cloudflare", feature = "native", feature = "wasi")))]
compile_error!("At least one of the `cloudflare`, `native`, or `wasi` features must be enabled.");
