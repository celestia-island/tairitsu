use anyhow::Result;
use bytes::Bytes;

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    #[allow(dead_code)]
    path: String,
}

#[async_trait::async_trait]
impl BucketStore<ProxyBucketMultipartUploader> for ProxyBucket {
    async fn set(&self, _key: String, _value: Bytes) -> Result<()> {
        todo!()
    }

    async fn get(&self, _key: String) -> Result<Option<Bytes>> {
        todo!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        todo!()
    }

    async fn create_multipart_upload(&self, _key: String) -> Result<ProxyBucketMultipartUploader> {
        todo!()
    }

    async fn resume_multipart_upload(
        &self,
        _key: String,
        _upload_id: String,
    ) -> Result<ProxyBucketMultipartUploader> {
        todo!()
    }
}

pub struct ProxyBucketMultipartUploader {}

impl ProxyBucketMultipartUploader {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl BucketMultipartUploader for ProxyBucketMultipartUploader {
    async fn upload_id(self) -> Result<String> {
        todo!()
    }

    async fn upload_part(
        self,
        _part_number: u16,
        _data: Bytes,
    ) -> Result<BucketMultipartUploadePart> {
        todo!()
    }

    async fn complete(
        self,
        _parts: Vec<BucketMultipartUploadePart>,
    ) -> Result<BucketMultipartUploadResult> {
        todo!()
    }

    async fn abort(self) -> Result<()> {
        todo!()
    }
}

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: path.to_string(),
    })
}
