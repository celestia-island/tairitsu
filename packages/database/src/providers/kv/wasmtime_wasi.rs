use super::KVStore;

#[derive(Clone)]
pub struct ProxyKV {}

#[async_trait::async_trait]
impl KVStore for ProxyKV {
    async fn set(&self, key: impl ToString, value: impl ToString) {
        todo!()
    }

    async fn get(&self, key: impl ToString) -> Option<String> {
        todo!()
    }

    async fn delete(&self, key: impl ToString) {
        todo!()
    }

    async fn list_by_prefix(
        &self,
        prefix: impl ToString,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> Vec<String> {
        todo!()
    }
}

pub async fn init_kv() -> Result<ProxyKV> {
    Ok(ProxyKV {})
}
