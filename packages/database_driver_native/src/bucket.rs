use anyhow::{anyhow, ensure, Result};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::RangeInclusive,
    os::windows::fs::MetadataExt,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use moka::future::Cache;

use tairitsu_database_types::providers::bucket::*;

#[derive(Clone)]
pub struct ProxyBucket {
    path: PathBuf,
    cache: Cache<(String, (usize, usize)), Bytes>,
    multipart_cache: Arc<Mutex<HashMap<String, Vec<Bytes>>>>,
}

#[async_trait::async_trait]
impl BucketStore for ProxyBucket {
    async fn set(&self, key: String, value: Bytes) -> Result<()> {
        check_key(&key)?;

        std::fs::write(
            {
                let mut path = self.path.to_path_buf();
                path.push(key.clone());
                path
            },
            value.as_ref(),
        )
        .map_err(|err| anyhow!("Failed to write to file '{}': {}", key, err))?;

        Ok(())
    }

    async fn get(
        &self,
        key: String,
        range: Option<RangeInclusive<usize>>,
    ) -> Result<Option<Bytes>> {
        check_key(&key)?;

        if let Some(range) = range {
            let (start, end) = (*range.start(), *range.end());

            if let Some(data) = self.cache.get(&(key.clone(), (start, end))).await {
                return Ok(Some(data));
            } else {
                let mut file = File::open({
                    let mut path = self.path.to_path_buf();
                    path.push(key.clone());
                    path
                })
                .map_err(|err| anyhow!("Failed to open file '{}': {}", key, err))?;
                file.seek(SeekFrom::Start(start as u64))
                    .map_err(|err| anyhow!("Failed to seek in file '{}': {}", key, err))?;

                let mut data = vec![0; (end - start) as usize];
                file.read_exact(&mut data)
                    .map_err(|err| anyhow!("Failed to read from file '{}': {}", key, err))?;

                let data = Bytes::from(data);
                self.cache
                    .insert((key.clone(), (start, end)), data.clone())
                    .await;
                return Ok(Some(data));
            }
        } else {
            let data = std::fs::read({
                let mut path = self.path.to_path_buf();
                path.push(key.clone());
                path
            })
            .map_err(|err| anyhow!("Failed to read file '{}': {}", key, err))?;

            Ok(Some(Bytes::from(data)))
        }
    }

    async fn get_metadata(&self, key: String) -> Result<BucketItemMetadata> {
        check_key(&key)?;

        let metadata = std::fs::metadata({
            let mut path = self.path.to_path_buf();
            path.push(key.clone());
            path
        })
        .map_err(|err| anyhow!("Failed to get metadata for file '{}': {}", key, err))?;
        ensure!(metadata.is_file(), "Path '{}' is not a file", key);

        Ok(BucketItemMetadata {
            key: key.clone(),
            version: metadata.last_write_time().to_string(),
            size: metadata.len() as usize,

            etag: "".to_string(),
            http_etag: "".to_string(),
            uploaded: DateTime::from_timestamp_nanos(metadata.creation_time() as i64),

            http_metadata: Default::default(),
            custom_metadata: Default::default(),
        })
    }

    async fn delete(&self, key: String) -> Result<()> {
        check_key(&key)?;

        for (item_key_raw, _) in self.cache.iter() {
            let item_key_raw = (*item_key_raw).clone();
            let item_key = item_key_raw.0.clone();

            if item_key == key {
                self.cache.remove(&item_key_raw).await;
            }
        }

        if let Err(err) = std::fs::remove_file({
            let mut path = self.path.to_path_buf();
            path.push(key.clone());
            path
        }) {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(());
            } else {
                return Err(anyhow!("Failed to delete file '{}': {}", key, err));
            }
        }

        Ok(())
    }

    async fn create_multipart_upload(&self) -> Result<String> {
        let upload_id = uuid::Uuid::new_v4().to_string();
        self.multipart_cache
            .try_lock()
            .map_err(|_| anyhow!("Failed to lock multipart cache"))?
            .insert(upload_id.clone(), Vec::new());

        Ok(upload_id)
    }

    async fn append_multipart_upload(&self, upload_id: String, data: Bytes) -> Result<()> {
        if let Some(upload) = self
            .multipart_cache
            .try_lock()
            .map_err(|_| anyhow!("Failed to lock multipart cache"))?
            .get_mut(&upload_id)
        {
            upload.push(data);
        } else {
            return Err(anyhow!("Upload ID '{}' not found", upload_id));
        }

        Ok(())
    }

    async fn complete_multipart_upload(
        &self,
        upload_id: String,
        final_data_key: Option<String>,
    ) -> Result<BucketItemMetadata> {
        let upload = self
            .multipart_cache
            .try_lock()
            .map_err(|_| anyhow!("Failed to lock multipart cache"))?
            .remove(&upload_id)
            .ok_or_else(|| anyhow!("Upload ID '{}' not found or already completed", upload_id))?;
        let data = upload.concat();
        let data = Bytes::from(data);

        let key = final_data_key.unwrap_or_else(|| upload_id.to_string());
        check_key(&key)?;

        self.set(key.clone(), data.clone()).await?;

        Ok(BucketItemMetadata {
            key,
            version: "".to_string(),
            size: data.len(),

            etag: "".to_string(),
            http_etag: "".to_string(),
            uploaded: Utc::now(),

            http_metadata: Default::default(),
            custom_metadata: Default::default(),
        })
    }

    async fn abort_multipart_upload(&self, upload_id: String) -> Result<()> {
        self.multipart_cache
            .try_lock()
            .map_err(|_| anyhow!("Failed to lock multipart cache"))?
            .remove(&upload_id)
            .ok_or_else(|| anyhow!("Upload ID '{}' not found or already completed", upload_id))?;

        Ok(())
    }
}

fn check_key(key: &String) -> Result<()> {
    // Check the key is a valid file name
    ensure!(
        key.chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-')),
        "Invalid key '{}': must only contain alphanumeric characters, '.', '_', or '-'",
        key
    );

    Ok(())
}

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: PathBuf::from(path.to_string()),
        cache: Cache::new(1_000),
        multipart_cache: Arc::new(Mutex::new(HashMap::new())),
    })
}
