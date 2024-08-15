use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait BucketStore {
    async fn get(&self, key: String) -> Result<Option<Bytes>>;
    async fn set(&self, key: String, value: Bytes) -> Result<()>;
    async fn delete(&self, key: String) -> Result<()>;

    async fn create_multipart_upload(&self) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        let init_data: usize = 0;
        let init_data = postcard::to_allocvec(&init_data)?;
        self.set(format!("__multipart_{}", id), Bytes::from(init_data))
            .await?;

        Ok(id)
    }

    async fn append_multipart_upload(&self, upload_id: String, data: Bytes) -> Result<()> {
        let count = self
            .get(format!("__multipart_{}", upload_id))
            .await?
            .ok_or(anyhow::anyhow!(
                "Multipart upload with id {} not found",
                upload_id
            ))?;
        let count: usize = postcard::from_bytes(&count)?;
        let count = count + 1;

        self.set(format!("__multipart_{}_{}", upload_id, count), data)
            .await?;

        let count = postcard::to_allocvec(&count)?;
        self.set(format!("__multipart_{}", upload_id), Bytes::from(count))
            .await?;

        Ok(())
    }

    async fn complete_multipart_upload(
        &self,
        upload_id: String,
        final_data_key: Option<String>,
    ) -> Result<BucketMultipartUploadResult> {
        let count = format!("__multipart_{}", upload_id);
        let count = self.get(count).await?.ok_or(anyhow::anyhow!(
            "Multipart upload with id {} not found",
            upload_id
        ))?;
        let count: usize = postcard::from_bytes(&count)?;

        let mut parts = Vec::with_capacity(count);
        for i in 0..count {
            let key = format!("__multipart_{}_{}", upload_id, i);
            let data = self.get(key).await?;

            let data = data.unwrap_or_default();
            parts.push(data);
        }

        let final_data = parts.concat();
        let size = final_data.len();
        let key = final_data_key.unwrap_or(upload_id.clone());
        self.set(key.clone(), Bytes::from(final_data)).await?;

        for i in 0..count {
            self.delete(format!("__multipart_{}_{}", upload_id, i))
                .await?;
        }
        self.delete(format!("__multipart_{}", upload_id)).await?;

        Ok(BucketMultipartUploadResult {
            key,
            version: "1".to_string(),
            size,

            etag: "".to_string(),
            http_etag: "".to_string(),
            uploaded: Utc::now(),

            http_metadata: BucketMultipartUploadResultHttpMetadata {
                content_type: None,
                content_language: None,
                content_disposition: None,
                content_encoding: None,
                cache_control: None,
                cache_expiry: None,
            },
            custom_metadata: HashMap::new(),
        })
    }

    async fn abort_multipart_upload(&self, upload_id: String) -> Result<()> {
        let count = format!("__multipart_{}", upload_id);
        let count = self.get(count).await?.ok_or(anyhow::anyhow!(
            "Multipart upload with id {} not found",
            upload_id
        ))?;
        let count: usize = postcard::from_bytes(&count)?;

        for i in 0..count {
            self.delete(format!("__multipart_{}_{}", upload_id, i))
                .await?;
        }
        self.delete(format!("__multipart_{}", upload_id)).await?;

        Ok(())
    }
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
