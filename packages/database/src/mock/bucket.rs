use std::ops::RangeInclusive;

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

    async fn get(
        &self,
        _key: String,
        _range: Option<RangeInclusive<usize>>,
    ) -> Result<Option<Bytes>> {
        unimplemented!()
    }

    async fn get_metadata(&self, _key: String) -> Result<BucketItemMetadata> {
        unimplemented!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        unimplemented!()
    }

    async fn create_multipart_upload(&self) -> Result<String> {
        todo!()
    }

    async fn append_multipart_upload(&self, _upload_id: String, _data: Bytes) -> Result<()> {
        todo!()
    }

    async fn complete_multipart_upload(
        &self,
        _upload_id: String,
        _final_data_key: Option<String>,
    ) -> Result<BucketItemMetadata> {
        todo!()
    }

    async fn abort_multipart_upload(&self, _upload_id: String) -> Result<()> {
        todo!()
    }
}
