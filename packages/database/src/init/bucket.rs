use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Init;
use tairitsu_database_types::providers::bucket::BucketStore;

#[derive(Clone)]
pub enum InitBucketParams {
    Cloudflare((Arc<worker::Env>, String)),
    Native(String),
    WASI(String),
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl Init<Box<crate::prelude::ProxyBucket>> for InitBucketParams {
    async fn init(self) -> Result<Box<crate::prelude::ProxyBucket>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cloudflare")] {
                match self {
                    InitBucketParams::Cloudflare((env, bucket_name)) => {
                        Ok(Box::new(
                            tairitsu_database_driver_cloudflare::bucket::init_bucket(env, bucket_name)
                                .await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "native")] {
                match self {
                    InitBucketParams::Native(bucket_name) => {
                        Ok(Box::new(
                            tairitsu_database_driver_native::bucket::init_bucket(bucket_name).await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "wasi")] {
                match self {
                    InitBucketParams::WASI(bucket_name) => {
                        Ok(Box::new(
                            tairitsu_database_driver_wasi::bucket::init_bucket(bucket_name).await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else {
                Err(anyhow!("No platform feature enabled"))
            }
        }
    }
}
