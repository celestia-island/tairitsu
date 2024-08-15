use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Init;
use tairitsu_database_types::providers::kv::KVStore;

#[derive(Clone)]
pub enum InitKVParams {
    Cloudflare((Arc<worker::Env>, String)),
    Native(String),
    WASI(String),
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl Init<Box<dyn KVStore>> for InitKVParams {
    async fn init(self) -> Result<Box<dyn KVStore>> {
        match self {
            InitKVParams::Cloudflare((env, bucket_name)) => {
                #[cfg(feature = "cloudflare")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_cloudflare::kv::init_kv(env, bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "cloudflare"))]
                Err(anyhow!("Cloudflare feature not enabled"))
            }

            InitKVParams::Native(bucket_name) => {
                #[cfg(feature = "native")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_native::kv::init_kv(bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "native"))]
                Err(anyhow!("Native feature not enabled"))
            }

            InitKVParams::WASI(bucket_name) => {
                #[cfg(feature = "wasi")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_wasi::kv::init_kv(bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "wasi"))]
                Err(anyhow!("WASI feature not enabled"))
            }
        }
    }
}
