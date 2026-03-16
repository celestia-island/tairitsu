use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "tairitsu")]
#[command(about = "Build and packaging tool for Tairitsu applications", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to Cargo.toml (defaults to current directory)
    #[arg(short, long, global = true)]
    manifest_path: Option<PathBuf>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Start development server with hot reload
    Dev {
        /// Port to listen on (default: 3001)
        #[arg(short, long)]
        port: Option<u16>,

        /// Open browser automatically
        #[arg(long)]
        open: bool,

        /// Watch source files and rebuild automatically on changes
        #[arg(short, long)]
        watch: bool,
    },

    /// Build for production
    Build {
        /// Build target (wasm, native)
        #[arg(short, long, default_value = "wasm")]
        target: String,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Package application for distribution
    Package {
        /// Target platform (windows, macos, linux, all)
        #[arg(short, long, default_value = "all")]
        platform: String,
    },

    /// Preview production build
    Preview {
        /// Port to listen on (default: 3001)
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Initialize a new Tairitsu project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Manage WIT browser world packages (fetch, verify, list)
    Wit {
        #[command(subcommand)]
        action: WitCommands,
    },
}

#[derive(Subcommand)]
enum WitCommands {
    /// Fetch WIT packages from the registry and store in target/tairitsu-wit
    Fetch {
        /// Package specs to fetch, e.g. `tairitsu-browser:dom@0.1.0`.
        /// Omit to fetch all default browser-world packages.
        specs: Vec<String>,

        /// Force cache-only mode (fail if package is not already cached)
        #[arg(long)]
        offline: bool,
    },

    /// Verify integrity of cached WIT packages
    Verify {
        /// Package specs to verify. Omit to verify all cached packages.
        specs: Vec<String>,
    },

    /// List all WIT packages in the local cache
    List,
}

pub async fn run() -> crate::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Load configuration
    let manifest_path = cli.manifest_path.unwrap_or_else(|| PathBuf::from("."));

    match cli.command {
        Commands::Dev { port, open, watch } => {
            let config = crate::config::Config::load(&manifest_path)?;
            info!("Starting development server...");
            let port = port.unwrap_or(config.dev.port);
            crate::wasm::dev_server(&config, port, open, watch).await?;
        }
        Commands::Build { target, release } => {
            let config = crate::config::Config::load(&manifest_path)?;
            info!("Building for {}...", target);
            match target.as_str() {
                "wasm" => crate::wasm::build(&config, release, None)?,
                "component" => crate::wasm::build_component(&config, release, None)?,
                "native" => {
                    eprintln!("Native builds not yet implemented");
                    std::process::exit(1);
                }
                _ => {
                    eprintln!(
                        "Unknown target: {}. Use 'wasm', 'component', or 'native'",
                        target
                    );
                    std::process::exit(1);
                }
            }
        }
        Commands::Package { platform } => {
            info!("Packaging for {}...", platform);
            eprintln!("Packaging not yet implemented");
            std::process::exit(1);
        }
        Commands::Preview { port } => {
            info!("Starting preview server...");
            let port = port.unwrap_or(3001);
            let _port = port;
            eprintln!("Preview not yet implemented");
            std::process::exit(1);
        }
        Commands::Init { name } => {
            info!("Initializing new project...");
            let name = name.unwrap_or_else(|| "my-tairitsu-app".to_string());
            crate::utils::init_project(&name)?;
        }
        Commands::Wit { action } => match action {
            WitCommands::Fetch { specs, offline } => {
                info!("Fetching WIT packages...");
                crate::wit_cmd::cmd_fetch(&manifest_path, &specs, offline)?;
            }
            WitCommands::Verify { specs } => {
                info!("Verifying WIT package cache...");
                crate::wit_cmd::cmd_verify(&manifest_path, &specs)?;
            }
            WitCommands::List => {
                crate::wit_cmd::cmd_list(&manifest_path)?;
            }
        },
    }

    Ok(())
}
