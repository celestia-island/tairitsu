use super::KVStore;

pub struct ProxyKV {
    path: String,
}

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
}

pub async fn init_kv(path: impl ToString) -> Result<ProxyKV> {
    Ok(ProxyKV {
        path: path.to_string(),
    })
}
