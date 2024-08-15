use anyhow::Result;

use tairitsu_database_types::providers::kv::*;

#[derive(Clone)]
pub struct ProxyKV {
    #[allow(dead_code)]
    path: String,
}

#[async_trait::async_trait]
impl KVStore for ProxyKV {
    async fn set(&self, _key: String, _value: String) -> Result<()> {
        todo!()
    }

    async fn get(&self, _key: String) -> Result<Option<String>> {
        todo!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        todo!()
    }

    async fn list_by_prefix(
        &self,
        _prefix: String,
        _limit: Option<usize>,
        _cursor: Option<String>,
    ) -> Result<Vec<String>> {
        todo!()
    }
}

pub async fn init_kv(path: impl ToString) -> Result<ProxyKV> {
    Ok(ProxyKV {
        path: path.to_string(),
    })
}
