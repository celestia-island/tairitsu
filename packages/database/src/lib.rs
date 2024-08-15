#![allow(unused_imports)]
#![allow(ambiguous_glob_reexports)]

pub mod init;
pub mod mock;

pub mod prelude {
    #[allow(unused_imports)]
    use anyhow::Result;
    use sea_orm::DatabaseConnection;

    pub use super::init::*;
    pub use tairitsu_database_types::providers::{bucket::*, kv::*};

    pub async fn init_bucket(param: impl Into<InitBucketParams>) -> Result<Box<dyn BucketStore>> {
        let param: InitBucketParams = param.into();
        param.init().await
    }

    pub async fn init_kv(param: impl Into<InitKVParams>) -> Result<Box<dyn KVStore>> {
        let param: InitKVParams = param.into();
        param.init().await
    }

    pub async fn init_db(param: impl Into<InitSQLParams>) -> Result<Box<DatabaseConnection>> {
        let param: InitSQLParams = param.into();
        param.init().await
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "cloudflare")] {
            pub use tairitsu_database_driver_cloudflare::{kv::ProxyKV, bucket::ProxyBucket};
        } else if #[cfg(feature = "native")] {
            pub use tairitsu_database_driver_native::{kv::ProxyKV, bucket::ProxyBucket};
        } else if #[cfg(feature = "wasi")] {
            pub use tairitsu_database_driver_wasi::{kv::ProxyKV, bucket::ProxyBucket};
        } else {
            pub use crate::mock::{kv::ProxyKV, bucket::ProxyBucket};
        }
    }
}
