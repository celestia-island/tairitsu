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
use bytes::Bytes;

#[async_trait::async_trait]
pub trait BucketStore {
    async fn get(&self, key: String) -> Result<Option<Bytes>>;
    async fn set(&self, key: String, value: Bytes) -> Result<()>;
    async fn delete(&self, key: String) -> Result<()>;
}
