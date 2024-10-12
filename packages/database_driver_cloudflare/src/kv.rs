use anyhow::{anyhow, Result};
use std::sync::Arc;

use worker::{send::SendFuture, Env};

use tairitsu_database_types::providers::kv::*;

#[derive(Clone)]
pub struct ProxyKV {
    env: Arc<Env>,
    kv_name: String,
}

#[async_trait::async_trait]
impl KVStore for ProxyKV {
    async fn set(&self, key: String, value: String) -> Result<()> {
        let env = self.env.kv(self.kv_name.as_str())?;

        SendFuture::new(async move {
            let _ = env
                .put(key.to_string().as_str(), value.to_string())
                .map_err(|err| anyhow!("Failed to set key-value pair: {:?}", err))
                .unwrap()
                .execute()
                .await
                .map_err(|err| anyhow!("Failed to set key-value pair: {:?}", err));
        })
        .await;

        Ok(())
    }

    async fn get(&self, key: String) -> Result<Option<String>> {
        let env = self.env.kv(self.kv_name.as_str())?;

        SendFuture::new(async move {
            env.get(key.to_string().as_str())
                .text()
                .await
                .map_err(|err| anyhow!("Failed to get key-value pair: {:?}", err))
        })
        .await
    }

    async fn delete(&self, key: String) -> Result<()> {
        let env = self.env.kv(self.kv_name.as_str())?;

        SendFuture::new(async move {
            let _ = env
                .delete(key.as_str())
                .await
                .map_err(|err| anyhow!("Failed to delete key-value pair: {:?}", err));
        })
        .await;

        Ok(())
    }

    async fn list_by_prefix(
        &self,
        prefix: String,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> Result<Vec<String>> {
        let env = self.env.kv(self.kv_name.as_str())?;

        SendFuture::new(async move {
            let ret = env.list().prefix(prefix);

            let ret = if let Some(limit) = limit {
                ret.limit(limit as u64)
            } else {
                ret
            };

            let ret = if let Some(cursor) = cursor {
                ret.cursor(cursor)
            } else {
                ret
            };

            ret.execute()
                .await
                .map_err(|err| anyhow!("Failed to list key-value pair: {:?}", err))
                .map(|ret| {
                    ret.keys
                        .iter()
                        .map(|key| key.name.to_owned())
                        .collect::<Vec<_>>()
                })
        })
        .await
    }
}

pub async fn init_kv(env: Arc<Env>, kv_name: impl ToString) -> Result<ProxyKV> {
    Ok(ProxyKV {
        env,
        kv_name: kv_name.to_string(),
    })
}
