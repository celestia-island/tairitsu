use anyhow::Result;
use bytes::Bytes;

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {}

#[async_trait::async_trait]
impl BucketStore for ProxyBucket {
    async fn set(&self, _key: String, _value: Bytes) -> Result<()> {
        unimplemented!()
    }

    async fn get(&self, _key: String) -> Result<Option<Bytes>> {
        unimplemented!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        unimplemented!()
    }
}
