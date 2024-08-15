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
impl Init<Box<dyn BucketStore>> for InitBucketParams {
    async fn init(self) -> Result<Box<dyn BucketStore>> {
        match self {
            InitBucketParams::Cloudflare((env, bucket_name)) => {
                #[cfg(feature = "cloudflare")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_cloudflare::bucket::init_bucket(env, bucket_name)
                            .await?,
                    ))
                }

                #[cfg(not(feature = "cloudflare"))]
                Err(anyhow!("Cloudflare feature not enabled"))
            }

            InitBucketParams::Native(bucket_name) => {
                #[cfg(feature = "native")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_native::bucket::init_bucket(bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "native"))]
                Err(anyhow!("Native feature not enabled"))
            }

            InitBucketParams::WASI(bucket_name) => {
                #[cfg(feature = "wasi")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_wasi::bucket::init_bucket(bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "wasi"))]
                Err(anyhow!("WASI feature not enabled"))
            }
        }
    }
}
