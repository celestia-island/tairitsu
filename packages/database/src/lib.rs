#[cfg(any(
    all(feature = "cloudflare", feature = "native"),
    all(feature = "cloudflare", feature = "wasi"),
    all(feature = "native", feature = "wasi")
))]
compile_error!(
    "Only one of the `cloudflare`, `native`, or `wasi` features can be enabled at a time."
);

pub mod prelude {
    #[allow(unused_imports)]
    pub use tairitsu_database_types::providers::{bucket::*, kv::*};
}

#[cfg(feature = "cloudflare")]
pub use tairitsu_database_driver_cloudflare::*;

#[cfg(feature = "native")]
pub use tairitsu_database_driver_native::*;

#[cfg(feature = "wasi")]
pub use tairitsu_database_driver_wasi::*;
