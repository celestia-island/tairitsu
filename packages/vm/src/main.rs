mod runtime;
mod stream;

use anyhow::Result;
use bytes::Bytes;
use std::collections::BTreeMap;

use gluesql::{
    memory_storage::MemoryStorage,
    prelude::{Glue, Payload},
};
use sea_orm::{ProxyExecResult, ProxyExecResultIdType};
use wasmtime::{Config, Engine};
use wit_component::ComponentEncoder;

use runtime::Runtime;
use tairitsu_utils::types::proto::backend::{RequestMsg, ResponseMsg, ResponseQueryType};

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
                id UUID PRIMARY KEY DEFAULT GENERATE_UUID(),
                create_at TIMESTAMP DEFAULT NOW(),

                title TEXT NOT NULL,
                text TEXT NOT NULL,
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
                let ret = ResponseMsg::Execute(if let Some(ret) = ret.first() {
                    match ret {
                        Payload::Insert(num) => ProxyExecResult::Inserted({
                            db.execute(format!(
                                "SELECT id FROM posts ORDER BY create_at DESC LIMIT {num}"
                            ))?
                            .iter()
                            .map(|ret| match ret {
                                Payload::Select { rows, .. } => {
                                    if let Some(column) = rows.first() {
                                        match column
                                            .to_owned()
                                            .take_first_value()
                                            .expect("Empty rows")
                                        {
                                            gluesql::prelude::Value::Uuid(val) => {
                                                let ret = ProxyExecResultIdType::Uuid(
                                                    uuid::Uuid::from_u128(val),
                                                );
                                                ret
                                            }
                                            _ => {
                                                unreachable!("Unsupported value: {:?}", column)
                                            }
                                        }
                                    } else {
                                        unreachable!("Empty rows");
                                    }
                                }
                                _ => unreachable!("Unsupported payload: {:?}", ret),
                            })
                            .collect::<Vec<ProxyExecResultIdType>>()
                        }),
                        _ => ProxyExecResult::Conflicted,
                    }
                } else {
                    ProxyExecResult::Empty
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
                                                ResponseQueryType::Integer((*val).into())
                                            }
                                            gluesql::prelude::Value::Str(val) => {
                                                ResponseQueryType::Text(val.to_owned())
                                            }
                                            gluesql::prelude::Value::Uuid(val) => {
                                                ResponseQueryType::Uuid(uuid::Uuid::from_u128(*val))
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
