mod bucket;
mod kv;
mod sql;

pub use bucket::*;
pub use kv::*;
pub use sql::*;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Init<T> {
    async fn init(self) -> Result<T>;
}
