use anyhow::Result;
use std::io;

use tairitsu_utils::types::proto::backend::Msg;

fn main() -> Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer)?;

    let msg: Msg = ron::from_str(&buffer)?;

    let ret = Msg {
        id: msg.id + 1,
        data: msg.data + " hahaha",
    };
    let ret = ron::to_string(&ret)?;

    println!("{}", ret);

    Ok(())
}
