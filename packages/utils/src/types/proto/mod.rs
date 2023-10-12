pub mod backend;
pub mod frontend;

use serde::{Deserialize, Serialize};

pub mod request {
    pub use crate::types::functions::*;
    pub use crate::types::secure::*;

    pub use crate::types::proto::frontend::filter;
    pub use crate::types::proto::frontend::LimitOffset;
    pub use crate::types::proto::frontend::UuidData;
}

pub mod response {
    pub use crate::types::functions::*;

    pub use crate::types::proto::frontend::Count;
    pub use crate::types::proto::frontend::UuidData;
}

pub mod models {
    pub use crate::types::functions::*;
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestPackage {
    UserInfo(crate::types::functions::UserType),

    Login(crate::types::secure::LoginInfo),
    Verify(crate::types::secure::VerifyInfo),

    Uuid(crate::types::proto::frontend::UuidData),
    LimitOffset(crate::types::proto::frontend::LimitOffset),
    Filter(Vec<crate::types::proto::frontend::filter::FilterPackage>),
    None,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ResponseStruct {
    UserInfo(crate::types::functions::UserType),

    Count(crate::types::proto::frontend::Count),
    Token(crate::types::proto::frontend::UuidData),
    Ok,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePackage {
    Data(Vec<ResponseStruct>),
    ErrorReason(String),
}
