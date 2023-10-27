mod entity;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyRow, Statement,
};

use entity::post::{ActiveModel, Entity};
use tairitsu_utils::types::proto::backend::{RequestMsg, ResponseMsg, ResponseQueryType};

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let sql = statement.sql.clone();
        println!("{}", ron::to_string(&RequestMsg::Query(sql)).unwrap());

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let ret: ResponseMsg = ron::from_str(&input).unwrap();
        let ret = match ret {
            ResponseMsg::Query(v) => v,
            _ => unreachable!("Not a query result"),
        };

        let mut rows: Vec<ProxyRow> = vec![];
        for row in ret {
            let mut map: BTreeMap<String, sea_orm::Value> = BTreeMap::new();
            for (k, v) in row.iter() {
                map.insert(
                    k.to_owned(),
                    match v {
                        ResponseQueryType::Boolean(val) => sea_orm::Value::Bool(Some(*val)),
                        ResponseQueryType::Integer(val) => sea_orm::Value::BigInt(Some(*val)),
                        ResponseQueryType::Float(val) => sea_orm::Value::Float(Some(*val)),
                        ResponseQueryType::Text(val) => {
                            sea_orm::Value::String(Some(Box::new(val.to_owned())))
                        }
                        ResponseQueryType::Decimal(val) => {
                            sea_orm::Value::Decimal(Some(Box::new(val.to_owned().into())))
                        }
                        ResponseQueryType::Date(val) => {
                            sea_orm::Value::ChronoDate(Some(Box::new(val.to_owned().into())))
                        }
                        ResponseQueryType::Time(val) => {
                            sea_orm::Value::ChronoTime(Some(Box::new(val.to_owned().into())))
                        }
                        ResponseQueryType::DateTime(val) => {
                            sea_orm::Value::ChronoDateTime(Some(Box::new(val.to_owned().into())))
                        }
                        ResponseQueryType::TimeStamp(val) => {
                            sea_orm::Value::ChronoDateTimeUtc(Some(Box::new(val.to_owned().into())))
                        }
                        ResponseQueryType::ByteA(val) => {
                            sea_orm::Value::Bytes(Some(Box::new(val.to_owned())))
                        }
                        ResponseQueryType::Uuid(val) => {
                            sea_orm::Value::Uuid(Some(Box::new(val.to_owned())))
                        }
                    },
                );
            }
            rows.push(ProxyRow { values: map });
        }

        Ok(rows)
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let sql = {
            if let Some(values) = statement.values {
                // Replace all the '?' with the statement values
                let mut new_sql = statement.sql.clone();
                let mark_count = new_sql.matches('?').count();
                for (i, v) in values.0.iter().enumerate() {
                    if i >= mark_count {
                        break;
                    }
                    new_sql = new_sql.replacen('?', &v.to_string(), 1);
                }

                new_sql
            } else {
                statement.sql
            }
        };

        // Send the query to stdout
        let msg = RequestMsg::Execute(sql);
        let msg = ron::to_string(&msg).unwrap();
        println!("{}", msg);

        // Get the result from stdin
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let ret: ResponseMsg = ron::from_str(&input).unwrap();
        let ret = match ret {
            ResponseMsg::Execute(v) => v,
            _ => unreachable!(),
        };

        Ok(ret)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
        ron::to_string(&RequestMsg::Debug(format!("{:?}", list))).unwrap()
    );
}
