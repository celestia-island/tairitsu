use std::collections::BTreeMap;

use sea_orm::ProxyExecResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RequestMsg {
    Query(String),
    Execute(String),

    Debug(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseMsg {
    Query(Vec<BTreeMap<String, serde_json::Value>>),
    Execute(ProxyExecResult),

    None,
}
