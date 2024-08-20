use anyhow::Result;
use bytes::Bytes;
use std::ops::RangeInclusive;

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

    async fn get(
        &self,
        _key: String,
        _range: Option<RangeInclusive<usize>>,
    ) -> Result<Option<Bytes>> {
        todo!()
    }

    async fn get_metadata(&self, _key: String) -> Result<BucketItemMetadata> {
        todo!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        todo!()
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

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: path.to_string(),
    })
}
