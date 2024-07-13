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

use anyhow::Result;

#[async_trait::async_trait]
pub trait KVStore {
    async fn get(&self, key: String) -> Result<Option<String>>;
    async fn set(&self, key: String, value: String) -> Result<()>;
    async fn delete(&self, key: String) -> Result<()>;
}
