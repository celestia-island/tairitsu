use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Init;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub enum InitSQLParams {
    Cloudflare((Arc<worker::Env>, String)),
    Native(String),
    WASI,
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl Init<Box<DatabaseConnection>> for InitSQLParams {
    async fn init(self) -> Result<Box<DatabaseConnection>> {
        match self {
            InitSQLParams::Cloudflare((env, bucket_name)) => {
                #[cfg(feature = "cloudflare")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_cloudflare::sql::init_db(env, bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "cloudflare"))]
                Err(anyhow!("Cloudflare feature not enabled"))
            }

            InitSQLParams::Native(bucket_name) => {
                #[cfg(feature = "native")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_native::sql::init_db(bucket_name).await?,
                    ))
                }

                #[cfg(not(feature = "native"))]
                Err(anyhow!("Native feature not enabled"))
            }

            InitSQLParams::WASI => {
                #[cfg(feature = "wasi")]
                {
                    Ok(Box::new(
                        tairitsu_database_driver_wasi::sql::init_db().await?,
                    ))
                }

                #[cfg(not(feature = "wasi"))]
                Err(anyhow!("WASI feature not enabled"))
            }
        }
    }
}
