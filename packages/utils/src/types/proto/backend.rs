use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: u32,
    pub data: String,
}
