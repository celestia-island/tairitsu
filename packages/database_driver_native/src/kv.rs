use anyhow::{anyhow, Result};

use sled::Db;

use tairitsu_database_types::providers::kv::*;

#[derive(Clone)]
pub struct ProxyKV {
    db: Db,
}

#[async_trait::async_trait]
impl KVStore for ProxyKV {
    async fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(key.into_bytes(), value.into_bytes())?;

        Ok(())
    }

    async fn get(&self, key: String) -> Result<Option<String>> {
        let value = self.db.get(key.into_bytes())?;

        if let Some(value) = value {
            Ok(Some(String::from_utf8(value.to_vec())?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: String) -> Result<()> {
        self.db.remove(key.into_bytes())?;

        Ok(())
    }

    async fn list_by_prefix(
        &self,
        prefix: String,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> Result<Vec<String>> {
        let ret = self
            .db
            .scan_prefix(prefix.as_bytes())
            .keys()
            .skip(cursor.map_or(0, |cursor| cursor.parse().unwrap_or(0)))
            .take(limit.unwrap_or(usize::MAX))
            .map(|key| {
                key.map(|key| String::from_utf8(key.to_vec()).unwrap_or_default())
                    .map_err(|err| anyhow!("{}", err))
            })
            .collect::<Vec<_>>();
        ret.into_iter().collect::<Result<Vec<_>>>()
    }
}

pub async fn init_kv(path: impl ToString) -> Result<ProxyKV> {
    Ok(ProxyKV {
        db: sled::Config::default()
            .cache_capacity(10 * 1024 * 1024) // 10 MiB
            .path(path.to_string())
            .open()?,
    })
}
