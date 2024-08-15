use anyhow::{Context, Result};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn init_db(path: impl ToString) -> Result<DatabaseConnection> {
    let options = ConnectOptions::new(format!("sqlite:{}", path.to_string()));

    let db = Database::connect(options)
        .await
        .context("Failed to connect to database")?;

    Ok(db)
}
