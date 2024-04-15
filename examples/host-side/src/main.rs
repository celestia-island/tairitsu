use anyhow::Result;
use bytes::Bytes;
use serde_json::{json, Value};
use std::collections::BTreeMap;

use gluesql::{
    memory_storage::MemoryStorage,
    prelude::{Glue, Payload},
};

use tairitsu_utils::types::proto::backend::Msg;
use tairitsu_vm::Image;

#[async_std::main]
async fn main() -> Result<()> {
    let bin = Bytes::from(std::fs::read(format!(
        "{}/../../target/wasm32-wasi/release/tairitsu-example-guest-side.wasm",
        env!("CARGO_MANIFEST_DIR")
    ))?);

    // Create the database connection
    println!("Creating database connection...");
    let mem = MemoryStorage::default();
    let mut db = Glue::new(mem);
    db.execute(
        r#"
            CREATE TABLE IF NOT EXISTS posts (
                id INTEGER NOT NULL UNIQUE DEFAULT 0,
                title TEXT NOT NULL,
                text TEXT NOT NULL,

                _timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
            )
        "#,
    )
    .await?;

    // Run the prototype demo
    println!("Running prototype demo...");
    let image = Image::new(bin);
    let mut container = image.init()?;

    let tx = container.tx.clone();
    let rx = container.rx.clone();
    async_std::task::spawn(async move {
        container.run().unwrap();
    });

    while let Ok(msg) = rx.recv() {
        if msg.command == "execute" {
            let sql = match msg.data {
                Value::String(v) => v,
                _ => unreachable!("Unknown data type"),
            };

            println!("SQL execute: {:?}", sql);
            let ret = db.execute(sql).await?;

            println!("SQL execute result: {:?}", ret);
            let ret = match ret.last().expect("Failed to get result") {
                Payload::Insert(_) => {
                    // Get the count of all the rows
                    let count = db
                        .execute("SELECT id FROM posts ORDER BY id DESC LIMIT 1")
                        .await?;
                    let count = match count.last().expect("Failed to get count") {
                        Payload::Select { rows, .. } => {
                            match rows.first().unwrap().first().unwrap() {
                                gluesql::prelude::Value::I64(val) => *val,
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    };
                    let count = count + 1;

                    // Rewrite the last insert id
                    db.execute(format!("UPDATE posts SET id = {} WHERE id = 0", count))
                        .await?;

                    json!({
                        "last_insert_id": count as u64,
                        "rows_affected": 1,
                    })
                }
                _ => todo!("Unsupported result"),
            };
            let ret = Msg::new("execute", ret);
            tx.send(ret)?;
        } else if msg.command == "query" {
            let sql = match msg.data {
                Value::String(v) => v,
                _ => unreachable!("Unknown data type"),
            };

            println!("SQL query: {:?}", sql);

            let mut ret: Vec<BTreeMap<String, Value>> = vec![];
            for payload in db.execute(sql).await?.iter() {
                match payload {
                    gluesql::prelude::Payload::Select { labels, rows } => {
                        for row in rows.iter() {
                            let mut map = BTreeMap::new();
                            for (label, column) in labels.iter().zip(row.iter()) {
                                map.insert(
                                    label.to_owned(),
                                    match column {
                                        gluesql::prelude::Value::I64(val) => {
                                            serde_json::Value::Number((*val).into())
                                        }
                                        gluesql::prelude::Value::Str(val) => {
                                            serde_json::Value::String(val.to_owned())
                                        }
                                        _ => {
                                            unreachable!("Unsupported value: {:?}", column)
                                        }
                                    },
                                );
                            }
                            ret.push(map);
                        }
                    }
                    _ => unreachable!("Unsupported payload: {:?}", payload),
                }
            }

            println!("SQL query result: {:?}", ret);
            let ret = Value::Array(
                ret.iter()
                    .map(|v| Value::from(serde_json::Map::from_iter(v.clone().into_iter())))
                    .collect::<Vec<Value>>(),
            );
            let ret = Msg::new("query", ret);
            tx.send(ret)?;
        } else if msg.command == "debug" {
            let msg = match msg.data {
                Value::String(v) => v,
                _ => unreachable!("Unknown data type"),
            };

            println!("VM Debug: {}", msg);
        } else {
            unreachable!("Unknown command: {:?}", msg.command);
        }
    }

    println!("Will exit...");
    Ok(())
}
