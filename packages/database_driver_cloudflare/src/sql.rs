use anyhow::{anyhow, Context, Result};
use std::{collections::BTreeMap, fmt::Write, sync::Arc};
use wasm_bindgen::JsValue;

use sea_orm::{
    Database, DatabaseConnection, DbBackend, DbErr, ProxyDatabaseTrait, ProxyExecResult, ProxyRow,
    RuntimeErr, Statement, Value, Values,
};
use worker::Env;

#[derive(Clone)]
struct ProxyDb {
    env: Arc<Env>,
    db_name: String,
}

impl std::fmt::Debug for ProxyDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(format!("[ProxyDb] {}", self.db_name).as_str())
            .finish()
    }
}

impl ProxyDb {
    async fn do_query(
        env: Arc<Env>,
        db_name: String,
        statement: Statement,
    ) -> Result<Vec<ProxyRow>> {
        let sql = statement.sql.clone();
        let values = match statement.values {
            Some(Values(values)) => values
                .iter()
                .map(|val| match &val {
                    Value::BigInt(Some(val)) => JsValue::from(val.to_string()),
                    Value::BigUnsigned(Some(val)) => JsValue::from(val.to_string()),
                    Value::Int(Some(val)) => JsValue::from(*val),
                    Value::Unsigned(Some(val)) => JsValue::from(*val),
                    Value::SmallInt(Some(val)) => JsValue::from(*val),
                    Value::SmallUnsigned(Some(val)) => JsValue::from(*val),
                    Value::TinyInt(Some(val)) => JsValue::from(*val),
                    Value::TinyUnsigned(Some(val)) => JsValue::from(*val),

                    Value::Float(Some(val)) => JsValue::from_f64(*val as f64),
                    Value::Double(Some(val)) => JsValue::from_f64(*val),

                    Value::Bool(Some(val)) => JsValue::from(*val),
                    Value::Bytes(Some(val)) => JsValue::from(format!(
                        "X'{}'",
                        val.iter().fold("".to_string(), |mut acc, byte| {
                            let _ = write!(&mut acc, "{:02x}", byte);
                            acc
                        })
                    )),
                    Value::Char(Some(val)) => JsValue::from(val.to_string()),
                    Value::Json(Some(val)) => JsValue::from(val.to_string()),
                    Value::String(Some(val)) => JsValue::from(val.to_string()),

                    Value::ChronoDate(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTime(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeLocal(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeUtc(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeWithTimeZone(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoTime(Some(val)) => JsValue::from(val.to_string()),
                    Value::TimeDate(Some(val)) => JsValue::from(val.to_string()),
                    Value::TimeDateTime(Some(val)) => JsValue::from(val.to_string()),
                    Value::TimeDateTimeWithTimeZone(Some(val)) => JsValue::from(val.to_string()),
                    Value::TimeTime(Some(val)) => JsValue::from(val.to_string()),

                    Value::BigDecimal(Some(val)) => JsValue::from(val.to_string()),
                    Value::Decimal(Some(val)) => JsValue::from(val.to_string()),
                    Value::Uuid(Some(val)) => JsValue::from(val.to_string()),

                    _ => JsValue::NULL,
                })
                .collect(),
            None => Vec::new(),
        };

        let ret = env
            .d1(db_name.as_str())?
            .prepare(sql)
            .bind(&values)?
            .all()
            .await?;
        if let Some(message) = ret.error() {
            return Err(anyhow!(message.to_string()));
        }

        let ret = ret.results::<serde_json::Value>()?;
        let ret = ret
            .iter()
            .map(|row| {
                let mut values = BTreeMap::new();
                for (key, value) in row.as_object().unwrap() {
                    values.insert(
                        key.clone(),
                        match &value {
                            serde_json::Value::Bool(val) => Value::Bool(Some(*val)),
                            serde_json::Value::Number(val) => {
                                if val.is_i64() {
                                    Value::BigInt(Some(val.as_i64().unwrap()))
                                } else if val.is_u64() {
                                    Value::BigUnsigned(Some(val.as_u64().unwrap()))
                                } else {
                                    Value::Double(Some(val.as_f64().unwrap()))
                                }
                            }
                            serde_json::Value::String(val) => {
                                Value::String(Some(Box::new(val.clone())))
                            }
                            _ => Value::Json(Some(Box::new(value.clone()))),
                        },
                    );
                }
                ProxyRow { values }
            })
            .collect();

        Ok(ret)
    }

    async fn do_execute(
        env: Arc<Env>,
        db_name: String,
        statement: Statement,
    ) -> Result<ProxyExecResult> {
        let sql = statement.sql.clone();
        let values = match statement.values {
            Some(Values(values)) => values
                .iter()
                .map(|val| match &val {
                    Value::BigInt(Some(val)) => JsValue::from(val.to_string()),
                    Value::BigUnsigned(Some(val)) => JsValue::from(val.to_string()),
                    Value::Int(Some(val)) => JsValue::from(*val),
                    Value::Unsigned(Some(val)) => JsValue::from(*val),
                    Value::SmallInt(Some(val)) => JsValue::from(*val),
                    Value::SmallUnsigned(Some(val)) => JsValue::from(*val),
                    Value::TinyInt(Some(val)) => JsValue::from(*val),
                    Value::TinyUnsigned(Some(val)) => JsValue::from(*val),

                    Value::Float(Some(val)) => JsValue::from_f64(*val as f64),
                    Value::Double(Some(val)) => JsValue::from_f64(*val),

                    Value::Bool(Some(val)) => JsValue::from(*val),
                    Value::Bytes(Some(val)) => JsValue::from(format!(
                        "X'{}'",
                        val.iter().fold("".to_string(), |mut acc, byte| {
                            let _ = write!(&mut acc, "{:02x}", byte);
                            acc
                        })
                    )),
                    Value::Char(Some(val)) => JsValue::from(val.to_string()),
                    Value::Json(Some(val)) => JsValue::from(val.to_string()),
                    Value::String(Some(val)) => JsValue::from(val.to_string()),

                    Value::ChronoDate(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTime(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeLocal(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeUtc(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeWithTimeZone(Some(val)) => JsValue::from(val.to_string()),

                    _ => JsValue::NULL,
                })
                .collect(),
            None => Vec::new(),
        };

        let ret = env
            .d1(db_name.as_str())?
            .prepare(sql)
            .bind(&values)?
            .run()
            .await?
            .meta()?;

        let last_insert_id = ret
            .as_ref()
            .map(|meta| meta.last_row_id.unwrap_or(0))
            .unwrap_or(0) as u64;
        let rows_affected = ret
            .as_ref()
            .map(|meta| meta.rows_written.unwrap_or(0))
            .unwrap_or(0) as u64;

        Ok(ProxyExecResult {
            last_insert_id,
            rows_affected,
        })
    }
}

#[async_trait::async_trait]
impl ProxyDatabaseTrait for ProxyDb {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let env = self.env.clone();
        let db_name = self.db_name.clone();
        let (tx, rx) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let ret = Self::do_query(env, db_name, statement).await;
            tx.send(ret).unwrap();
        });

        let ret = rx.await.unwrap();
        ret.map_err(|err| DbErr::Conn(RuntimeErr::Internal(err.to_string())))
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let env = self.env.clone();
        let db_name = self.db_name.clone();
        let (tx, rx) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let ret = Self::do_execute(env, db_name, statement).await;
            tx.send(ret).unwrap();
        });

        let ret = rx.await.unwrap();
        ret.map_err(|err| DbErr::Conn(RuntimeErr::Internal(err.to_string())))
    }
}

pub async fn init_sql(env: Arc<Env>, db_name: impl ToString) -> Result<DatabaseConnection> {
    let db = Database::connect_proxy(
        DbBackend::Sqlite,
        Arc::new(Box::new(ProxyDb {
            env,
            db_name: db_name.to_string(),
        })),
    )
    .await
    .context("Failed to connect to database")?;

    Ok(db)
}
