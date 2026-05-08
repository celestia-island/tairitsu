use std::path::PathBuf;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = parse_cli();

    if let Some(action) = &cli.action {
        handle_action(action);
        return;
    }

    let disabled_plugins = load_disabled_plugins();

    let config = tairitsu_mcp::McpConfig {
        base_url: String::new(),
        disabled_plugins,
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

struct Cli {
    action: Option<CliAction>,
}

enum CliAction {
    Disable(String),
    Enable(String),
    Reset,
}

fn parse_cli() -> Cli {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--disable" => {
                if i + 1 < args.len() {
                    return Cli {
                        action: Some(CliAction::Disable(args[i + 1].clone())),
                    };
                } else {
                    eprintln!("Usage: tairitsu-mcp --disable <plugin-name>");
                    std::process::exit(1);
                }
            }
            "--enable" => {
                if i + 1 < args.len() {
                    return Cli {
                        action: Some(CliAction::Enable(args[i + 1].clone())),
                    };
                } else {
                    eprintln!("Usage: tairitsu-mcp --enable <plugin-name>");
                    std::process::exit(1);
                }
            }
            "--reset" => {
                return Cli {
                    action: Some(CliAction::Reset),
                };
            }
            _ => {}
        }
        i += 1;
    }
    Cli { action: None }
}

fn handle_action(action: &CliAction) {
    let config_path = config_file_path();
    match action {
        CliAction::Disable(name) => {
            let mut disabled = load_disabled_plugins();
            if !disabled.contains(&name.to_string()) {
                disabled.push(name.clone());
            }
            save_disabled_plugins(&disabled);
            eprintln!("[tairitsu-mcp] Plugin '{}' disabled. Re-run without flags to start.", name);
        }
        CliAction::Enable(name) => {
            let mut disabled = load_disabled_plugins();
            disabled.retain(|n| n != name);
            save_disabled_plugins(&disabled);
            eprintln!("[tairitsu-mcp] Plugin '{}' enabled.", name);
        }
        CliAction::Reset => {
            save_disabled_plugins(&Vec::new());
            eprintln!("[tairitsu-mcp] All plugin preferences reset.");
        }
    }
}

fn config_file_path() -> PathBuf {
    if let Ok(dir) = std::env::var("TAIRITSU_CONFIG_DIR") {
        let p = PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&p);
        return p.join("mcp-disabled-plugins.json");
    }
    if let Some(base) = dirs_data_local() {
        let dir = base.join("tairitsu");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("mcp-disabled-plugins.json");
    }
    PathBuf::from("mcp-disabled-plugins.json")
}

fn dirs_data_local() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join(".local/share"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join("Library").join("Application Support"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return Some(PathBuf::from(appdata));
        }
    }
    None
}

fn load_disabled_plugins() -> Vec<String> {
    let path = config_file_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(list) = serde_json::from_str::<Vec<String>>(&data) {
            return list;
        }
    }
    Vec::new()
}

fn save_disabled_plugins(list: &[String]) {
    let path = config_file_path();
    if let Ok(data) = serde_json::to_string_pretty(list) {
        let _ = std::fs::write(&path, data);
    }
}
