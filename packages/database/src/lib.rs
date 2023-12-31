pub mod functions;
pub mod models;

use anyhow::Result;
use async_std::sync::Mutex;
use lazy_static::lazy_static;
use log::info;
use std::{cell::Cell, time::Duration};

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Schema, Statement};

pub struct DatabaseNetworkConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub async fn init(config: DatabaseNetworkConfig) -> Result<()> {
    // Connect to the database
    let mut opt = ConnectOptions::new(format!(
        "mysql://{}:{}@{}:{}/mysql",
        config.username, config.password, config.host, config.port
    ));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Trace);
    let db = Database::connect(opt).await?;
    let builder = db.get_database_backend();

    // Initialize the database
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!(
            "CREATE DATABASE IF NOT EXISTS {} DEFAULT CHARACTER SET utf8mb4;",
            config.database
        ),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("USE {}", config.database),
    ))
    .await?;

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::channel::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::user::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::tag::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::thread::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::post::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    info!("Database is ready");
    DB_CONN.lock().await.replace(db);

    Ok(())
}

lazy_static! {
    static ref DB_CONN: Mutex<Cell<DatabaseConnection>> = Default::default();
}
