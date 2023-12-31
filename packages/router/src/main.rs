mod routes;

use anyhow::Result;
use log::info;
use tokio::net::TcpListener;

use axum::serve;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use yew::platform::Runtime;

use crate::routes::route;

#[derive(Clone, Default)]
struct Executor {
    inner: Runtime,
}

impl<F> hyper::rt::Executor<F> for Executor
where
    F: std::future::Future + Send + 'static,
{
    fn execute(&self, fut: F) {
        self.inner.spawn_pinned(move || async move {
            fut.await;
        });
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(23333);

    tairitsu_database::init(tairitsu_database::DatabaseNetworkConfig {
        host: std::env::var("DB_HOST").unwrap_or("localhost".into()),
        port: std::env::var("DB_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3306),
        username: std::env::var("DB_USERNAME").unwrap_or("root".into()),
        password: std::env::var("DB_PASSWORD").unwrap_or("root".into()),
        database: std::env::var("DB_DATABASE").unwrap_or("hikari".into()),
    })
    .await?;

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .into_inner();

    let router = route().await?.layer(middleware_stack);

    info!("Site will run on port {port}");
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");
    serve(listener, router).await?;

    Ok(())
}
