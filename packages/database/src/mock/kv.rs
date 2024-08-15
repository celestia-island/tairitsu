use anyhow::Result;

use tairitsu_database_types::providers::kv::*;

#[derive(Clone)]
pub struct ProxyKV {}

#[async_trait::async_trait]
impl KVStore for ProxyKV {
    async fn set(&self, _key: String, _value: String) -> Result<()> {
        unimplemented!()
    }

    async fn get(&self, _key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    async fn delete(&self, _key: String) -> Result<()> {
        unimplemented!()
    }

    async fn list_by_prefix(
        &self,
        _prefix: String,
        _limit: Option<usize>,
        _cursor: Option<String>,
    ) -> Result<Vec<String>> {
        unimplemented!()
    }
}
