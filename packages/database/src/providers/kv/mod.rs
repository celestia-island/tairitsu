#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "native")]
pub mod sled;
#[cfg(feature = "wasi")]
pub mod wasmtime_wasi;

#[cfg(feature = "cloudflare")]
pub use cloudflare::*;
#[cfg(feature = "native")]
pub use sled::*;
#[cfg(feature = "wasi")]
pub use wasmtime_wasi::*;

pub trait KVStore {
    async fn get(&self, key: impl ToString) -> Option<String>;
    async fn set(&self, key: impl ToString, value: impl ToString);
    async fn delete(&self, key: impl ToString);
}
