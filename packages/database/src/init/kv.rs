use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Init;
use tairitsu_database_types::providers::kv::KVStore;

#[derive(Clone)]
pub enum InitKVParams {
    Cloudflare { env: Arc<worker::Env>, name: String },
    Native { path: String },
    WASI { name: String },
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl Init<Box<crate::prelude::ProxyKV>> for InitKVParams {
    async fn init(self) -> Result<Box<crate::prelude::ProxyKV>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cloudflare")] {
                match self {
                    InitKVParams::Cloudflare { env, name } => {
                        Ok(Box::new(
                            tairitsu_database_driver_cloudflare::kv::init_kv(env, name)
                                .await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "native")] {
                match self {
                    InitKVParams::Native { path } => {
                        Ok(Box::new(
                            tairitsu_database_driver_native::kv::init_kv(path).await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "wasi")] {
                match self {
                    InitKVParams::WASI { name } => {
                        Ok(Box::new(
                            tairitsu_database_driver_wasi::kv::init_kv(name).await?,
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
