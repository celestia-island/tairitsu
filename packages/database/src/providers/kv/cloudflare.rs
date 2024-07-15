use anyhow::{anyhow, Result};
use std::sync::Arc;

use worker::{send::SendFuture, Env};

use super::KVStore;

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

        let ret = SendFuture::new(async move {
            env.get(key.to_string().as_str())
                .text()
                .await
                .map_err(|err| anyhow!("Failed to get key-value pair: {:?}", err))
        })
        .await;

        ret
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
}

pub async fn init_kv(env: Arc<Env>, kv_name: impl ToString) -> Result<ProxyKV> {
    Ok(ProxyKV {
        env,
        kv_name: kv_name.to_string(),
    })
}
