#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "native")]
pub mod moka;
#[cfg(feature = "wasi")]
pub mod wasmtime_wasi;

use std::collections::HashMap;

#[cfg(feature = "cloudflare")]
pub use cloudflare::*;
#[cfg(feature = "native")]
pub use moka::*;
#[cfg(feature = "wasi")]
pub use wasmtime_wasi::*;

use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait BucketStore {
    async fn get(&self, key: String) -> Result<Option<Bytes>>;
    async fn set(&self, key: String, value: Bytes) -> Result<()>;
    async fn delete(&self, key: String) -> Result<()>;

    async fn create_multipart_upload(
        &self,
        key: String,
    ) -> Result<Box<dyn BucketMultipartUploader>>;
    async fn resume_multipart_upload(
        &self,
        key: String,
        upload_id: String,
    ) -> Result<Box<dyn BucketMultipartUploader>>;
}

#[async_trait::async_trait]
pub trait BucketMultipartUploader {
    async fn upload_part(
        &self,
        part_number: u16,
        data: Bytes,
    ) -> Result<BucketMultipartUploadePart>;
    async fn complete(
        self,
        parts: Vec<BucketMultipartUploadePart>,
    ) -> Result<BucketMultipartUploadResult>;
    async fn abort(&self) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketMultipartUploadePart {
    pub part_number: u16,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketMultipartUploadResult {
    pub key: String,
    pub version: String,
    pub size: usize,

    pub etag: String,
    pub http_etag: String,
    pub uploaded: DateTime<Utc>,

    pub http_metadata: BucketMultipartUploadResultHttpMetadata,
    pub custom_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketMultipartUploadResultHttpMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub cache_expiry: Option<DateTime<Utc>>,
}
