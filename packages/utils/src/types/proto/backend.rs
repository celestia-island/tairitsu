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

pub mod wasi {
    use super::Msg;
    use anyhow::Result;

    pub fn read() -> Result<Msg> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let msg: Msg = serde_json::from_str(&input).unwrap();

        Ok(msg)
    }

    pub fn write(channel: impl ToString, content: impl ToString) -> Result<()> {
        println!(
            "{}",
            serde_json::to_string(&Msg::new(channel.to_string(), content.to_string()))?
        );

        Ok(())
    }
}
