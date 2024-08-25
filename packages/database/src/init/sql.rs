use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Init;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub enum InitSQLParams {
    Cloudflare { env: Arc<worker::Env>, name: String },
    Native { url: String },
    WASI,
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl Init<Box<DatabaseConnection>> for InitSQLParams {
    async fn init(self) -> Result<Box<DatabaseConnection>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cloudflare")] {
                match self {
                    InitSQLParams::Cloudflare { env, name } => {
                        Ok(Box::new(
                            tairitsu_database_driver_cloudflare::sql::init_sql(env, name)
                                .await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "native")] {
                match self {
                    InitSQLParams::Native { url } => {
                        Ok(Box::new(
                            tairitsu_database_driver_native::sql::init_sql(url).await?,
                        ))
                    }

                    _ => Err(anyhow!("Only allow one platform at a time")),
                }
            } else if #[cfg(feature = "wasi")] {
                match self {
                    InitSQLParams::WASI => {
                        Ok(Box::new(
                            tairitsu_database_driver_wasi::sql::init_sql().await?,
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
