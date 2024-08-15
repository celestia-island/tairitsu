use anyhow::Result;

#[async_trait::async_trait]
pub trait KVStore {
    async fn get(&self, key: String) -> Result<Option<String>>;
    async fn set(&self, key: String, value: String) -> Result<()>;
    async fn delete(&self, key: String) -> Result<()>;
    async fn list_by_prefix(
        &self,
        prefix: String,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> Result<Vec<String>>;
}
