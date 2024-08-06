use super::KVStore;

#[derive(Clone)]
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

    async fn create_multipart_upload(
        &self,
        _key: String,
    ) -> Result<Box<dyn BucketMultipartUploader>> {
        unimplemented!()
    }

    async fn resume_multipart_upload(
        &self,
        _key: String,
        _upload_id: String,
    ) -> Result<Box<dyn BucketMultipartUploader>> {
        unimplemented!()
    }
}

pub async fn init_bucket(path: impl ToString) -> Result<ProxyBucket> {
    Ok(ProxyBucket {
        path: path.to_string(),
    })
}
