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
impl BucketStore for ProxyBucket {
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

    async fn create_multipart_upload(&self) -> Result<String> {
        todo!()
    }

    async fn append_multipart_upload(&self, upload_id: String, data: Bytes) -> Result<()> {
        todo!()
    }

    async fn complete_multipart_upload(
        &self,
        upload_id: String,
        final_data_key: Option<String>,
    ) -> Result<BucketMultipartUploadResult> {
        todo!()
    }

    async fn abort_multipart_upload(&self, upload_id: String) -> Result<()> {
        todo!()
    }
}

pub async fn init_bucket(env: Arc<Env>, bucket_name: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        env,
        bucket_name: bucket_name.to_string(),
    })
}
