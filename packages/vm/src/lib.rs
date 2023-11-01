mod runtime;
mod stream;

pub use runtime::{Container, Image};

#[cfg(test)]
mod test {
    use anyhow::Result;
    use bytes::Bytes;
    use std::collections::BTreeMap;

    use gluesql::{
        memory_storage::MemoryStorage,
        prelude::{Glue, Payload},
    };
    use sea_orm::ProxyExecResult;

    use crate::Image;
    use tairitsu_utils::types::proto::backend::{RequestMsg, ResponseMsg};

    #[async_std::test]
    async fn component_test() -> Result<()> {
        let bin = Bytes::from_static(include_bytes!(
            "../../../target/wasm32-wasi/release/tairitsu-proto.wasm"
        ));

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
        std::thread::spawn(move || {
            container.run().unwrap();
        });

        while let Ok(msg) = rx.recv() {
            match msg {
                RequestMsg::Execute(sql) => {
                    println!("SQL execute: {:?}", sql);
                    let ret = db.execute(sql).await?;

                    println!("SQL execute result: {:?}", ret);
                    let ret =
                        ResponseMsg::Execute(match ret.last().expect("Failed to get result") {
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
}
