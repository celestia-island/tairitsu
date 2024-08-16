use anyhow::Result;
use bytes::Bytes;

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    #[allow(dead_code)]
    path: String,
}

#[async_trait::async_trait]
impl BucketStore for ProxyBucket {
    async fn set(&self, _key: String, _value: Bytes) -> Result<()> {
        todo!()
    }

    async fn get(&self, _key: String) -> Result<Option<Bytes>> {
        todo!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        todo!()
    }
}

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: path.to_string(),
    })
}