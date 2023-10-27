use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use sea_orm::ProxyExecResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RequestMsg {
    Query(String),
    Execute(String),

    Debug(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseMsg {
    Query(Vec<BTreeMap<String, ResponseQueryType>>),
    Execute(ProxyExecResult),

    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseQueryType {
    Boolean(bool),
    Integer(i64),
    Float(f32),
    Text(String),
    Decimal(rust_decimal::Decimal),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    DateTime(chrono::NaiveDateTime),
    TimeStamp(chrono::DateTime<chrono::Utc>),
    ByteA(Vec<u8>),
    Uuid(Uuid),
}
