mod runtime;
mod stream;

use anyhow::Result;
use bytes::Bytes;
use std::collections::BTreeMap;

use gluesql::{
    memory_storage::MemoryStorage,
    prelude::{Glue, Payload},
};
use sea_orm::ProxyExecResult;
use wasmtime::{Config, Engine};
use wit_component::ComponentEncoder;

use runtime::Runtime;
use tairitsu_utils::types::proto::backend::{RequestMsg, ResponseMsg};

#[async_std::main]
async fn main() -> Result<()> {
    // Transfer the wasm binary to wasm component binary
    let adapter = include_bytes!("../res/wasi_snapshot_preview1.command.wasm");
    let component = &ComponentEncoder::default()
        .module(include_bytes!(
            "../../../target/wasm32-wasi/release/tairitsu-proto.wasm"
        ))?
        .validate(true)
        .adapter("wasi_snapshot_preview1", adapter)?
        .encode()?;

    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = &Engine::new(&config)?;

    let cwasm = engine.precompile_component(component)?;
    let cwasm = Bytes::from(cwasm);

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
    )?;

    // Run the prototype demo
    println!("Running prototype demo...");
    let mut runner = Runtime::new(cwasm).init()?;

    let tx = runner.tx.clone();
    let rx = runner.rx.clone();

    std::thread::spawn(move || {
        runner.run().unwrap();
    });

    while let Ok(msg) = rx.recv() {
        match msg {
            RequestMsg::Execute(sql) => {
                println!("SQL execute: {:?}", sql);
                let ret = db.execute(sql)?;

                println!("SQL execute result: {:?}", ret);
                let ret = ResponseMsg::Execute(match ret.last().expect("Failed to get result") {
                    Payload::Insert(_) => {
                        // Get the count of all the rows
                        let count = db.execute("SELECT id FROM posts ORDER BY id DESC LIMIT 1")?;
                        let count = match count.last().expect("Failed to get count") {
                            Payload::Select { rows, .. } => {
                                match rows.first().unwrap().0.first().unwrap() {
                                    gluesql::prelude::Value::I64(val) => *val,
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        };
                        let count = count + 1;

                        // Rewrite the last insert id
                        db.execute(format!("UPDATE posts SET id = {} WHERE id = 0", count))?;

                        ProxyExecResult {
                            last_insert_id: count as u64,
                            rows_affected: 1,
                        }
                    }
                    _ => todo!("Unsupported result"),
                });
                tx.send(ret)?;
            }
            RequestMsg::Query(sql) => {
                println!("SQL query: {:?}", sql);

                let mut ret = vec![];
                for payload in db.execute(sql)?.iter() {
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
                                            _ => unreachable!("Unsupported value: {:?}", column),
                                        },
                                    );
                                }
                                ret.push(map.into());
                            }
                        }
                        _ => unreachable!("Unsupported payload: {:?}", payload),
                    }
                }

                println!("SQL query result: {:?}", ret);
                let ret = ResponseMsg::Query(ret);
                tx.send(ret)?;
            }
            RequestMsg::Debug(msg) => {
                println!("VM Debug: {}", msg);
            }
        }
    }

    Ok(())
}
