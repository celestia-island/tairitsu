use anyhow::{anyhow, Result};
use bytes::Bytes;
use chrono::DateTime;
use std::sync::Arc;

use worker::{send::SendFuture, Env};

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    env: Arc<Env>,
    bucket_name: String,
}

#[async_trait::async_trait]
impl BucketStore<ProxyBucketMultipartUploader> for ProxyBucket {
    async fn set(&self, key: String, value: Bytes) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        SendFuture::new(async move {
            let _ = env
                .put(key.to_string().as_str(), worker::Data::Bytes(value.into()))
                .execute()
                .await
                .map_err(|err| anyhow!("Failed to set key-value pair: {:?}", err));
        })
        .await;

        Ok(())
    }

    async fn get(&self, key: String) -> Result<Option<Bytes>> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let ret = SendFuture::new(async move {
            match env.get(key.to_string().as_str()).execute().await {
                Ok(data) => match data {
                    Some(data) => match data.body() {
                        Some(body) => match body.bytes().await {
                            Ok(bytes) => Ok(Some(Bytes::from(bytes))),
                            Err(err) => Err(anyhow!("Failed to get key-value pair: {:?}", err)),
                        },
                        None => Ok(None),
                    },
                    None => Ok(None),
                },
                Err(err) => Err(anyhow!("Failed to get key-value pair: {:?}", err)),
            }
        })
        .await;

        ret
    }

    async fn delete(&self, key: String) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        SendFuture::new(async move {
            let _ = env
                .delete(key.as_str())
                .await
                .map_err(|err| anyhow!("Failed to delete key-value pair: {:?}", err));
        })
        .await;

        Ok(())
    }

    async fn create_multipart_upload(&self, key: String) -> Result<ProxyBucketMultipartUploader> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let ret = SendFuture::new(async move {
            match env.create_multipart_upload(key.clone()).execute().await {
                Ok(data) => Ok(ProxyBucketMultipartUploader::new(data)),
                Err(err) => Err(anyhow!(
                    "Failed to create multipart upload for key '{}': {:?}",
                    key,
                    err
                )),
            }
        })
        .await;

        ret
    }

    async fn resume_multipart_upload(
        &self,
        key: String,
        upload_id: String,
    ) -> Result<ProxyBucketMultipartUploader> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        match env.resume_multipart_upload(key.clone(), upload_id.clone()) {
            Ok(data) => Ok(ProxyBucketMultipartUploader::new(data)),
            Err(err) => Err(anyhow!(
                "Failed to resume multipart upload for key '{}' with upload id '{}': {:?}",
                key,
                upload_id,
                err
            )),
        }
    }
}

pub struct ProxyBucketMultipartUploader {
    inner: worker::MultipartUpload,
}

unsafe impl Send for ProxyBucketMultipartUploader {}
unsafe impl Sync for ProxyBucketMultipartUploader {}

impl ProxyBucketMultipartUploader {
    pub fn new(inner: worker::MultipartUpload) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl BucketMultipartUploader for ProxyBucketMultipartUploader {
    async fn upload_id(self) -> Result<String> {
        let env = self.inner;

        SendFuture::new(async move { Ok(env.upload_id().await) }).await
    }

    async fn upload_part(
        self,
        part_number: u16,
        data: Bytes,
    ) -> Result<BucketMultipartUploadePart> {
        let env = self.inner;

        let ret = SendFuture::new(async move {
            match env
                .upload_part(part_number, worker::Data::Bytes(data.to_vec()))
                .await
            {
                Ok(data) => Ok(BucketMultipartUploadePart {
                    part_number: data.part_number(),
                    etag: data.etag().to_string(),
                }),
                Err(err) => Err(anyhow!("Failed to upload part: {:?}", err)),
            }
        })
        .await;

        ret
    }

    async fn complete(
        self,
        parts: Vec<BucketMultipartUploadePart>,
    ) -> Result<BucketMultipartUploadResult> {
        let env = self.inner;

        let ret = SendFuture::new(async move {
            let parts = parts
                .into_iter()
                .map(|part| worker::UploadedPart::new(part.part_number, part.etag))
                .collect::<Vec<_>>();

            match env.complete(parts).await {
                Ok(data) => Ok(BucketMultipartUploadResult {
                    key: data.key().to_string(),
                    version: data.version().to_string(),
                    size: data.size() as usize,

                    etag: data.etag().to_string(),
                    http_etag: data.http_etag().to_string(),
                    uploaded: DateTime::from_timestamp_millis(data.uploaded().as_millis() as i64)
                        .unwrap_or_default()
                        .to_utc(),

                    http_metadata: {
                        let obj = data.http_metadata();

                        BucketMultipartUploadResultHttpMetadata {
                            content_type: obj.content_type.map(|s| s.to_string()),
                            content_language: obj.content_language.map(|s| s.to_string()),
                            content_disposition: obj.content_disposition.map(|s| s.to_string()),
                            content_encoding: obj.content_encoding.map(|s| s.to_string()),
                            cache_control: obj.cache_control.map(|s| s.to_string()),
                            cache_expiry: obj.cache_expiry.map(|ts| {
                                DateTime::from_timestamp_millis(ts.as_millis() as i64)
                                    .unwrap_or_default()
                                    .to_utc()
                            }),
                        }
                    },
                    custom_metadata: data.custom_metadata().unwrap_or_default(),
                }),
                Err(err) => Err(anyhow!("Failed to complete multipart upload: {:?}", err)),
            }
        })
        .await;

        ret
    }

    async fn abort(self) -> Result<()> {
        let env = self.inner;

        let ret = SendFuture::new(async move {
            match env.abort().await {
                Ok(_) => Ok(()),
                Err(err) => Err(anyhow!("Failed to abort multipart upload: {:?}", err)),
            }
        })
        .await;

        ret
    }
}

pub async fn init_bucket(env: Arc<Env>, bucket_name: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        env,
        bucket_name: bucket_name.to_string(),
    })
}
