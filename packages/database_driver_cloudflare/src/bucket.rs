use anyhow::{anyhow, Result};
use bytes::Bytes;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, sync::Arc};
use uuid::Uuid;

use worker::{send::SendFuture, Env};

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    env: Arc<Env>,
    bucket_name: String,
    multipart_kv_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BucketMultipartUploadInfo {
    key: String,
    upload_id: String,
    etags: Vec<String>,
}

#[async_trait::async_trait]
impl BucketStore for ProxyBucket {
    async fn set(&self, key: String, value: Bytes) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let _ = SendFuture::new(async move {
            env.put(key.to_string().as_str(), worker::Data::Bytes(value.into()))
                .execute()
                .await
                .map_err(|err| anyhow!("Failed to set key-value pair: {:?}", err))
        })
        .await?;

        Ok(())
    }

    async fn get(
        &self,
        key: String,
        range: Option<RangeInclusive<usize>>,
    ) -> Result<Option<Bytes>> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let ret = SendFuture::new(async move {
            let handle = env.get(key.to_string().as_str());
            let handle = if let Some(range) = range {
                handle.range(worker::Range::OffsetWithLength {
                    offset: *range.start() as u64,
                    length: (*range.end() - *range.start()) as u64,
                })
            } else {
                handle
            };

            match handle.execute().await {
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
        .await?;

        Ok(ret)
    }

    async fn get_metadata(&self, key: String) -> Result<BucketItemMetadata> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let ret = SendFuture::new(async move {
            match env.head(key.to_string().as_str()).await {
                Ok(data) => match data {
                    Some(data) => Ok(into_metadata(data)),
                    None => Err(anyhow!("Failed to get key-value pair: key not found.")),
                },
                Err(err) => Err(anyhow!("Failed to get key-value pair: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn delete(&self, key: String) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let ret = SendFuture::new(async move {
            env.delete(key.as_str())
                .await
                .map_err(|err| anyhow!("Failed to delete key-value pair: {:?}", err))
        })
        .await?;

        Ok(ret)
    }

    async fn create_multipart_upload(&self) -> Result<String> {
        let env = self.env.bucket(self.bucket_name.as_str())?;
        let multipart_kv_env = self.env.kv(self.multipart_kv_name.as_str())?;

        let ret = SendFuture::new(async move {
            let key = Uuid::new_v4().to_string();
            match env.create_multipart_upload(key.clone()).execute().await {
                Ok(info) => {
                    let upload_id = info.upload_id().await;
                    let parts_metadata = BucketMultipartUploadInfo {
                        key: key.clone(),
                        upload_id: upload_id.clone(),
                        etags: Vec::new(),
                    };
                    let parts_metadata = serde_json::to_string(&parts_metadata)?;

                    multipart_kv_env
                        .put(&format!("__multi_{}", key), parts_metadata)
                        .map_err(|err| {
                            anyhow!("Failed to write multipart upload metadata: {:?}", err)
                        })?
                        .execute()
                        .await
                        .map_err(|err| {
                            anyhow!("Failed to write multipart upload metadata: {:?}", err)
                        })?;
                    Ok(key)
                }
                Err(err) => Err(anyhow!("Failed to create multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn append_multipart_upload(&self, key: String, data: Bytes) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;
        let multipart_kv_env = self.env.kv(self.multipart_kv_name.as_str())?;

        let ret = SendFuture::new(async move {
            let parts_metadata = multipart_kv_env
                .get(&format!("__multi_{}", key))
                .text()
                .await
                .map_err(|err| anyhow!("Failed to read multipart upload metadata: {:?}", err))?
                .ok_or(anyhow!("Failed to read multipart upload metadata."))?;
            let parts_metadata: BucketMultipartUploadInfo = serde_json::from_str(&parts_metadata)?;

            match env.resume_multipart_upload(key.clone(), parts_metadata.upload_id.clone()) {
                Ok(uploader) => match uploader
                    .upload_part(
                        (parts_metadata.etags.len() + 1) as u16,
                        worker::Data::Bytes(data.to_vec()),
                    )
                    .await
                {
                    Ok(info) => {
                        let mut parts_metadata = parts_metadata.clone();
                        parts_metadata.etags.push(info.etag());
                        let parts_metadata = serde_json::to_string(&parts_metadata)?;

                        match multipart_kv_env
                            .put(&format!("__multi_{}", key), parts_metadata)
                            .map_err(|err| {
                                anyhow!("Failed to write multipart upload metadata: {:?}", err)
                            })?
                            .execute()
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(err) => Err(anyhow!(
                                "Failed to set part number for multipart upload: {:?}",
                                err
                            )),
                        }
                    }
                    Err(err) => Err(anyhow!("Failed to append multipart upload: {:?}", err)),
                },
                Err(err) => Err(anyhow!("Failed to resume multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn complete_multipart_upload(
        &self,
        key: String,
        final_data_key: Option<String>,
    ) -> Result<BucketItemMetadata> {
        if final_data_key.is_some() {
            unimplemented!("final_data_key is not supported yet");
        }

        let env = self.env.bucket(self.bucket_name.as_str())?;
        let multipart_kv_env = self.env.kv(self.multipart_kv_name.as_str())?;

        let ret = SendFuture::new(async move {
            let parts_metadata = multipart_kv_env
                .get(&format!("__multi_{}", key))
                .text()
                .await
                .map_err(|err| anyhow!("Failed to read multipart upload metadata: {:?}", err))?
                .ok_or(anyhow!("Failed to read multipart upload metadata."))?;
            let parts_metadata: BucketMultipartUploadInfo = serde_json::from_str(&parts_metadata)?;

            match env.resume_multipart_upload(key.clone(), parts_metadata.upload_id.clone()) {
                Ok(uploader) => match uploader
                    .complete(
                        parts_metadata
                            .etags
                            .iter()
                            .enumerate()
                            .map(|(index, item)| ((index + 1) as u16, item))
                            .map(|(index, item)| worker::UploadedPart::new(index, item.clone()))
                            .collect::<Vec<_>>(),
                    )
                    .await
                {
                    Ok(data) => {
                        multipart_kv_env
                            .delete(&format!("__multi_{}", key))
                            .await
                            .map_err(|err| {
                                anyhow!("Failed to delete multipart upload metadata: {:?}", err)
                            })?;

                        Ok(into_metadata(data))
                    }
                    Err(err) => Err(anyhow!("Failed to append multipart upload: {:?}", err)),
                },
                Err(err) => Err(anyhow!("Failed to resume multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn abort_multipart_upload(&self, key: String) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;
        let multipart_kv_env = self.env.kv(self.multipart_kv_name.as_str())?;

        let ret = SendFuture::new(async move {
            let parts_metadata = multipart_kv_env
                .get(&format!("__multi_{}", key))
                .text()
                .await
                .map_err(|err| anyhow!("Failed to read multipart upload metadata: {:?}", err))?
                .ok_or(anyhow!("Failed to read multipart upload metadata."))?;
            let parts_metadata: BucketMultipartUploadInfo = serde_json::from_str(&parts_metadata)?;

            match env.resume_multipart_upload(key.clone(), parts_metadata.upload_id.clone()) {
                Ok(uploader) => match uploader.abort().await {
                    Ok(_) => {
                        multipart_kv_env
                            .delete(&format!("__multi_{}", key))
                            .await
                            .map_err(|err| {
                                anyhow!("Failed to delete multipart upload metadata: {:?}", err)
                            })?;
                        Ok(())
                    }
                    Err(err) => Err(anyhow!("Failed to abort multipart upload: {:?}", err)),
                },
                Err(err) => Err(anyhow!("Failed to resume multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }
}

pub fn into_metadata(data: worker::Object) -> BucketItemMetadata {
    BucketItemMetadata {
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

            BucketItemHTTPMetadata {
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
    }
}

pub async fn init_bucket(
    env: Arc<Env>,
    bucket_name: impl ToString,
    multipart_kv_name: impl ToString,
) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        env,
        bucket_name: bucket_name.to_string(),
        multipart_kv_name: multipart_kv_name.to_string(),
    })
}
