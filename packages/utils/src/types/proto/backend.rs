use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: u32,
    pub data: String,
}
