pub mod entity;

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyRow, Statement,
};

use entity::post::{ActiveModel, Entity};
use tairitsu_utils::types::proto::backend::Msg;

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let sql = statement.sql.clone();
        println!(
            "{}",
            serde_json::to_string(&Msg::new("query", sql)).unwrap()
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let ret: Msg = serde_json::from_str(&input).unwrap();
        let ret: Vec<BTreeMap<String, Value>> = if ret.command == "query" {
            match ret.data {
                Value::Array(v) => v
                    .into_iter()
                    .map(|v| match v {
                        Value::Object(v) => v.into_iter().collect::<BTreeMap<String, Value>>(),
                        _ => unreachable!("Not a query result"),
                    })
                    .collect::<Vec<BTreeMap<String, Value>>>(),
                _ => unreachable!("Not a query result"),
            }
        } else {
            unreachable!("Not a query result")
        };

        let mut rows: Vec<ProxyRow> = vec![];
        for row in ret {
            let mut map: BTreeMap<String, sea_orm::Value> = BTreeMap::new();
            for (k, v) in row.iter() {
                map.insert(k.to_owned(), {
                    if v.is_string() {
                        sea_orm::Value::String(Some(Box::new(v.as_str().unwrap().to_string())))
                    } else if v.is_number() {
                        sea_orm::Value::BigInt(Some(v.as_i64().unwrap()))
                    } else if v.is_boolean() {
                        sea_orm::Value::Bool(Some(v.as_bool().unwrap()))
                    } else {
                        unreachable!("Unknown json type")
                    }
                });
            }
            rows.push(ProxyRow { values: map });
        }

        Ok(rows)
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let sql = {
            if let Some(values) = statement.values {
                // Replace all the '?' with the statement values
                use sqlparser::ast::{Expr, Value};
                use sqlparser::dialect::GenericDialect;
                use sqlparser::parser::Parser;

                let dialect = GenericDialect {};
                let mut ast = Parser::parse_sql(&dialect, statement.sql.as_str()).unwrap();
                match &mut ast[0] {
                    sqlparser::ast::Statement::Insert {
                        columns, source, ..
                    } => {
                        for item in columns.iter_mut() {
                            item.quote_style = Some('"');
                        }

                        if let Some(obj) = source {
                            match &mut *obj.body {
                                sqlparser::ast::SetExpr::Values(obj) => {
                                    for (mut item, val) in
                                        obj.rows[0].iter_mut().zip(values.0.iter())
                                    {
                                        match &mut item {
                                            Expr::Value(item) => {
                                                *item = match val {
                                                    sea_orm::Value::String(val) => {
                                                        Value::SingleQuotedString(match val {
                                                            Some(val) => val.to_string(),
                                                            None => "".to_string(),
                                                        })
                                                    }
                                                    sea_orm::Value::BigInt(val) => Value::Number(
                                                        val.unwrap_or(0).to_string(),
                                                        false,
                                                    ),
                                                    _ => todo!(),
                                                };
                                            }
                                            _ => todo!(),
                                        }
                                    }
                                }
                                _ => todo!(),
                            }
                        }
                    }
                    _ => todo!(),
                }

                let statement = &ast[0];
                statement.to_string()
            } else {
                statement.sql
            }
        };

        // Send the query to stdout
        let msg = Msg::new("execute", sql);
        let msg = serde_json::to_string(&msg).unwrap();
        println!("{}", msg);

        // Get the result from stdin
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let ret: Msg = serde_json::from_str(&input).unwrap();
        let ret = if ret.command == "execute" {
            match ret.data {
                Value::Object(v) => ProxyExecResult {
                    last_insert_id: v["last_insert_id"].as_u64().unwrap(),
                    rows_affected: v["rows_affected"].as_u64().unwrap(),
                },
                _ => unreachable!("Not an execute result"),
            }
        } else {
            unreachable!("Not an execute result")
        };

        Ok(ret)
    }
}

pub async fn init() {
    let db = Database::connect_proxy(
        DbBackend::Sqlite,
        Arc::new(Mutex::new(Box::new(ProxyDb {}))),
    )
    .await
    .unwrap();

    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("いいよ、来いよ".to_owned()),
        ..Default::default()
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("そうだよ".to_owned()),
        ..Default::default()
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("悔い改めて".to_owned()),
        ..Default::default()
    };
    Entity::insert(data).exec(&db).await.unwrap();

    let list = Entity::find().all(&db).await.unwrap().to_vec();
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", format!("{:?}", list))).unwrap()
    );
}
