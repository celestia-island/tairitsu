use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::RangeInclusive};

#[async_trait::async_trait]
pub trait BucketStore {
    async fn set(&self, key: String, value: Bytes) -> Result<()>;
    async fn get(&self, key: String, range: Option<RangeInclusive<usize>>)
        -> Result<Option<Bytes>>;
    async fn get_metadata(&self, key: String) -> Result<BucketItemMetadata>;
    async fn delete(&self, key: String) -> Result<()>;

    async fn create_multipart_upload(&self) -> Result<String>;
    async fn append_multipart_upload(&self, upload_id: String, data: Bytes) -> Result<()>;
    async fn complete_multipart_upload(
        &self,
        upload_id: String,
        final_data_key: Option<String>,
    ) -> Result<BucketItemMetadata>;
    async fn abort_multipart_upload(&self, upload_id: String) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketItemMetadata {
    pub key: String,
    pub version: String,
    pub size: usize,

    pub etag: String,
    pub http_etag: String,
    pub uploaded: DateTime<Utc>,

    pub http_metadata: BucketItemHTTPMetadata,
    pub custom_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BucketItemHTTPMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub cache_expiry: Option<DateTime<Utc>>,
}
