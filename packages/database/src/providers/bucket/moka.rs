use super::KVStore;

pub struct ProxyBucket {
    path: String,
}

#[async_trait::async_trait]
impl KVStore for ProxyBucket {
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

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: path.to_string(),
    })
}
