fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let config = tairitsu_mcp::McpConfig {
        base_url: String::new(),
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(async {
            if let Err(e) = tairitsu_mcp::run(config).await {
                eprintln!("[tairitsu-mcp] Fatal: {}", e);
                std::process::exit(1);
            }
        });
}
