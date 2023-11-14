use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: Uuid,
    pub command: String,
    pub data: Value,
}

impl Msg {
    pub fn new(command: impl ToString, data: impl Into<Value>) -> Self {
        Self {
            id: Uuid::new_v4(),
            command: command.to_string(),
            data: data.into(),
        }
    }
}
