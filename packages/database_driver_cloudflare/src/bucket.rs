use anyhow::{anyhow, Result};
use bytes::Bytes;
use chrono::DateTime;
use std::sync::Arc;
use uuid::Uuid;

use worker::{send::SendFuture, Env};

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    env: Arc<Env>,
    bucket_name: String,
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

        let ret = SendFuture::new(async move {
            let key = Uuid::new_v4().to_string();
            match env.create_multipart_upload(key.clone()).execute().await {
                Ok(info) => {
                    let upload_id = info.upload_id().await;
                    let upload_id = format!("{}_{}", key, upload_id);

                    let parts_metadata: Vec<String> = vec![];
                    let parts_metadata = postcard::to_allocvec(&parts_metadata)?;

                    self.set(
                        format!("__multi_{}", upload_id),
                        Bytes::from(parts_metadata),
                    )
                    .await?;
                    Ok(upload_id)
                }
                Err(err) => Err(anyhow!("Failed to create multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn append_multipart_upload(&self, upload_id: String, data: Bytes) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let (key, upload_id) = upload_id
            .split_once('_')
            .map(|(key, upload_id)| (key.to_string(), upload_id.to_string()))
            .ok_or(anyhow!(
                "Failed to split into key and upload_id: {:?}",
                upload_id
            ))?;

        let ret = SendFuture::new(async move {
            let parts_metadata =
                self.get(format!("__multi_{}", upload_id))
                    .await?
                    .ok_or(anyhow!(
                        "Failed to get part number for multipart upload: {:?}",
                        upload_id
                    ))?;
            let parts_metadata: Vec<String> = postcard::from_bytes(&parts_metadata)?;

            match env.resume_multipart_upload(key, upload_id.clone()) {
                Ok(uploader) => match uploader
                    .upload_part(
                        parts_metadata.len() as u16,
                        worker::Data::Bytes(data.to_vec()),
                    )
                    .await
                {
                    Ok(info) => {
                        let mut parts_metadata = parts_metadata.clone();
                        parts_metadata.push(info.etag());
                        let parts_metadata = postcard::to_allocvec(&parts_metadata)?;

                        match self
                            .set(
                                format!("__multi_{}", upload_id),
                                Bytes::from(parts_metadata),
                            )
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
        upload_id: String,
        final_data_key: Option<String>,
    ) -> Result<BucketMultipartUploadResult> {
        if final_data_key.is_some() {
            unimplemented!("final_data_key is not supported yet");
        }

        let env = self.env.bucket(self.bucket_name.as_str())?;

        let (key, upload_id) = upload_id
            .split_once('_')
            .map(|(key, upload_id)| (key.to_string(), upload_id.to_string()))
            .ok_or(anyhow!(
                "Failed to split into key and upload_id: {:?}",
                upload_id
            ))?;

        let ret = SendFuture::new(async move {
            let parts_metadata =
                self.get(format!("__multi_{}", upload_id))
                    .await?
                    .ok_or(anyhow!(
                        "Failed to get part number for multipart upload: {:?}",
                        upload_id
                    ))?;
            let parts_metadata: Vec<String> = postcard::from_bytes(&parts_metadata)?;

            match env.resume_multipart_upload(key, upload_id.clone()) {
                Ok(uploader) => match uploader
                    .complete(
                        parts_metadata
                            .iter()
                            .enumerate()
                            .map(|(index, item)| {
                                worker::UploadedPart::new(index as u16, item.clone())
                            })
                            .collect::<Vec<_>>(),
                    )
                    .await
                {
                    Ok(data) => Ok(BucketMultipartUploadResult {
                        key: data.key().to_string(),
                        version: data.version().to_string(),
                        size: data.size() as usize,

                        etag: data.etag().to_string(),
                        http_etag: data.http_etag().to_string(),
                        uploaded: DateTime::from_timestamp_millis(
                            data.uploaded().as_millis() as i64
                        )
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
                    Err(err) => Err(anyhow!("Failed to append multipart upload: {:?}", err)),
                },
                Err(err) => Err(anyhow!("Failed to resume multipart upload: {:?}", err)),
            }
        })
        .await?;

        Ok(ret)
    }

    async fn abort_multipart_upload(&self, upload_id: String) -> Result<()> {
        let env = self.env.bucket(self.bucket_name.as_str())?;

        let (key, upload_id) = upload_id
            .split_once('_')
            .map(|(key, upload_id)| (key.to_string(), upload_id.to_string()))
            .ok_or(anyhow!(
                "Failed to split into key and upload_id: {:?}",
                upload_id
            ))?;

        let ret = SendFuture::new(async move {
            match env.resume_multipart_upload(key, upload_id.clone()) {
                Ok(uploader) => match uploader.abort().await {
                    Ok(_) => {
                        self.delete(format!("__multi_{}", upload_id)).await?;
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

pub async fn init_bucket(env: Arc<Env>, bucket_name: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        env,
        bucket_name: bucket_name.to_string(),
    })
}
