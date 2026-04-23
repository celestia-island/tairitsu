use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::info;

use crate::daemon::{self, is_daemon, is_tty};

#[derive(Parser)]
#[command(name = "tairitsu")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to Cargo.toml (defaults to current directory)
    #[arg(short, long, global = true)]
    manifest_path: Option<PathBuf>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Run as / manage daemon (background service)
    ///   --daemon             start or restart daemon
    ///   --daemon --dry-run   only check status, don't restart
    #[arg(long, global = true)]
    daemon: bool,

    /// Internal marker for the Windows daemon child process.
    #[arg(long = "daemon-child-process", hide = true, global = true)]
    daemon_child_process: bool,

    /// With --daemon: only check status, don't restart/start
    #[arg(long, global = true, requires = "daemon")]
    dry_run: bool,

    /// Shutdown running daemon
    #[arg(long, global = true)]
    shutdown: bool,

    /// Check daemon status only
    #[arg(long, global = true)]
    status: bool,

    /// Force TTY interactive mode
    #[arg(long, global = true)]
    tty: bool,

    /// Force non-interactive mode
    #[arg(long, global = true)]
    no_tty: bool,
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

        /// Enable server-side rendering
        #[arg(long)]
        ssr: bool,
    },

    /// Build for production
    Build {
        /// Build target (wasm, native)
        #[arg(short, long, default_value = "component")]
        target: String,

        /// Build in debug mode (default is release)
        #[arg(long)]
        debug: bool,

        /// Enable server-side rendering (pre-render static HTML)
        #[arg(long)]
        ssr: bool,

        /// Routes to pre-render (comma-separated)
        #[arg(long, value_delimiter = ',')]
        routes: Vec<String>,
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

    /// Manage icons (fetch, build, list)
    Icons {
        #[command(subcommand)]
        action: IconsCommands,
    },

    /// Manage resources (index, list)
    Resources {
        #[command(subcommand)]
        action: ResourcesCommands,
    },

    /// Server-Side Rendering commands
    Ssr {
        /// Port to listen on for SSR dev server (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Open browser automatically
        #[arg(long)]
        open: bool,

        /// Watch source files and rebuild automatically on changes
        #[arg(short, long)]
        watch: bool,

        /// Build pre-rendered static HTML instead of running dev server
        #[arg(long)]
        build: bool,

        /// Routes to pre-render (comma-separated, used with --build)
        #[arg(long, value_delimiter = ',')]
        routes: Vec<String>,

        /// Output directory for pre-rendered HTML (used with --build)
        #[arg(long)]
        output: Option<String>,
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

#[derive(Subcommand)]
enum IconsCommands {
    /// Fetch icons from the configured source and cache them
    Fetch {
        /// Icon source (mdi, lucide, custom)
        #[arg(short, long, default_value = "mdi")]
        source: String,

        /// Force refresh (ignore cache)
        #[arg(short, long)]
        force: bool,
    },

    /// Build icon module from cached icons
    Build {
        /// Output file path (overrides Cargo.toml setting)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Icon names to include (comma-separated)
        #[arg(short, long)]
        icons: Option<String>,

        /// Tags to include (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// List available icons (optionally filter by source/tag)
    List {
        /// Icon source (mdi, lucide)
        #[arg(short, long, default_value = "mdi")]
        source: String,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Search query
        #[arg(short, long)]
        search: Option<String>,
    },
}

#[derive(Subcommand)]
enum ResourcesCommands {
    /// Scan and index resources (SCSS, SVG) with content hashing
    Index {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// List indexed resources from the cache
    List {
        /// Filter by resource type (scss, svg)
        #[arg(short, long)]
        r#type: Option<String>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

/// Handle synchronous daemon operations (status, shutdown, parent fork).
/// Returns `Some(result)` if the operation was handled synchronously (process should exit).
/// Returns `None` if we should proceed to async/tokio mode (daemon child or foreground).
pub fn handle_sync_daemon() -> Option<crate::Result<()>> {
    let cli = Cli::parse();
    const SYNC_OK: Option<crate::Result<()>> = Some(Ok(()));

    if let Some(ref mp) = cli.manifest_path {
        let canonical = mp.canonicalize().unwrap_or_else(|_| mp.clone());
        let root = if canonical.is_file() {
            canonical.parent().unwrap_or(&canonical).to_path_buf()
        } else {
            canonical
        };
        daemon::set_project_root(root);
    }

    if cli.status {
        let _ = daemon::print_daemon_status();
        return SYNC_OK;
    }

    if cli.shutdown {
        return Some(if daemon::is_daemon_running() {
            println!("Stopping daemon...");
            if daemon::kill_daemon().unwrap_or(false) {
                println!("Daemon stopped successfully.");
            } else {
                println!("No daemon was running (stale PID file cleaned).");
            }
            Ok(())
        } else {
            println!("No daemon is currently running.");
            Ok(())
        });
    }

    if cli.daemon && !daemon::is_daemon() && !cli.dry_run {
        let was_running = daemon::is_daemon_running();
        if was_running {
            println!("Restarting daemon...");
            let _ = daemon::kill_daemon();
            println!("Old daemon stopped.");
        } else {
            println!("Starting daemon...");
        }
        if let Err(error) = daemon::fork_daemon() {
            return Some(Err(error.into()));
        }
        #[cfg(unix)]
        {
            match daemon::wait_for_child_signal(120) {
                Ok(true) => {
                    if was_running { println!("Daemon restarted."); }
                    else { println!("Daemon started in background."); }
                }
                Ok(false) => {
                    eprintln!("Daemon failed to start.");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Daemon startup timed out: {}", e);
                    std::process::exit(1);
                }
            }
        }
        return SYNC_OK;
    }

    // --daemon --dry-run also exits synchronously
    if cli.daemon && cli.dry_run {
        let _ = daemon::print_daemon_status();
        return SYNC_OK;
    }

    // Not a sync daemon operation — proceed to async mode
    None
}

pub fn run_tokio() {
    #[tokio::main]
    async fn inner() -> crate::Result<()> {
        run().await
    }

    if let Err(e) = inner() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub async fn run() -> crate::Result<()> {
    run_with_cli(Cli::parse()).await
}

async fn run_with_cli(cli: Cli) -> crate::Result<()> {
    let t = crate::i18n::translations();

    // Resolve project root from manifest_path as early as possible
    // so daemon paths are consistent regardless of CWD
    if let Some(ref mp) = cli.manifest_path {
        let canonical = mp.canonicalize().unwrap_or_else(|_| mp.clone());
        let root = if canonical.is_file() {
            canonical.parent().unwrap_or(&canonical).to_path_buf()
        } else {
            canonical
        };
        daemon::set_project_root(root);
    }

    // Determine TTY mode
    let is_interactive = if cli.tty {
        true
    } else if cli.no_tty {
        false
    } else {
        is_tty()
    };

    // Handle --status: print daemon status and exit
    if cli.status {
        daemon::print_daemon_status()?;
        return Ok(());
    }

    // Handle --shutdown: stop daemon and exit
    if cli.shutdown {
        if daemon::is_daemon_running() {
            println!("Stopping daemon...");
            if daemon::kill_daemon()? {
                println!("Daemon stopped successfully.");
            } else {
                println!("No daemon was running (stale PID file cleaned).");
            }
        } else {
            println!("No daemon is currently running.");
        }
        return Ok(());
    }

    // Handle --daemon
    if cli.daemon {
        // Check if we're already the daemon child process
        if is_daemon() {
            // Daemon child: set up PID file and status, then fall through to dev server
            daemon::cleanup_ready_file();
            daemon::write_pid_file(std::process::id())?;
            let status = daemon::DaemonStatus {
                pid: std::process::id(),
                start_time: chrono::Utc::now(),
                build_logs: Vec::new(),
            };
            daemon::write_daemon_status(&status)?;
        } else if cli.dry_run {
            // --daemon --dry-run: only check status (handled in main.rs, but keep as fallback)
            daemon::print_daemon_status()?;
            return Ok(());
        }
        // Note: parent process fork+exit is now handled in main() OUTSIDE tokio runtime
        // to avoid Windows hang on exit. If we reach here in parent mode, just proceed.
    }

    // Early check: daemon already running, foreground dev blocked
    if !cli.daemon && daemon::is_daemon_running() {
        let is_dev_command = matches!(cli.command, Some(Commands::Dev { .. }) | Some(Commands::Ssr { .. }));
        if is_dev_command {
            eprintln!("Error: A daemon is already running (PID {}).", daemon::read_pid().unwrap_or(0));
            eprintln!();
            eprintln!("  Use --daemon   to restart the daemon");
            eprintln!("  Use --shutdown to stop it first");
            eprintln!("  Use --status   to check daemon state");
            std::process::exit(1);
        }
    }

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

    let is_dev_command = matches!(cli.command, Some(Commands::Dev { .. }) | Some(Commands::Ssr { .. }));

    if !is_interactive && !is_daemon() && is_dev_command {
        daemon::print_non_tty_hint();
        return Ok(());
    }

    // Load configuration
    let manifest_path = cli.manifest_path.unwrap_or_else(|| PathBuf::from("."));

    match cli.command {
        #[allow(unused_variables)]
        Some(Commands::Dev {
            port,
            open,
            watch,
            ssr,
        }) => {
            let config = crate::config::Config::load(&manifest_path)?;
            if ssr {
                #[cfg(feature = "ssr")]
                {
                    info!("Starting SSR development server...");
                    let port = port.unwrap_or(3000);
                    crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
                }
                #[cfg(not(feature = "ssr"))]
                {
                    eprintln!("SSR feature is not enabled. Please enable the 'ssr' feature.");
                    std::process::exit(1);
                }
            } else {
                info!("{}", t.cli.starting_dev_server);
                let port = port.unwrap_or(config.dev.port);
                #[cfg(feature = "dev-server")]
                {
                    crate::wasm::dev_server(&config, port, open, watch).await?;
                }
                #[cfg(not(feature = "dev-server"))]
                {
                    eprintln!(
                        "Dev server feature is not enabled. Please enable the 'dev-server' feature."
                    );
                    std::process::exit(1);
                }
            }
        }
        #[allow(unused_variables)]
        Some(Commands::Build {
            target,
            debug,
            ssr,
            routes,
        }) => {
            let config = crate::config::Config::load(&manifest_path)?;
            let release = !debug;
            info!("{} {}...", t.cli.building_for, target);
            match target.as_str() {
                "component" => {
                    crate::wasm::build_component(&config, release, None)?;

                    // Handle SSR pre-rendering if requested
                    if ssr {
                        #[cfg(feature = "ssr")]
                        {
                            let effective_routes = if routes.is_empty() {
                                let discovered = config.discovered_routes();
                                if discovered.is_empty() {
                                    vec!["".to_string()]
                                } else {
                                    discovered.iter().map(|r| r.path.clone()).collect()
                                }
                            } else {
                                routes
                            };
                            let output_dir = std::path::PathBuf::from("dist/prerendered");
                            info!("Pre-rendering {} routes...", effective_routes.len());
                            crate::ssr::prerender_routes(&config, &effective_routes, &output_dir)?;
                        }
                        #[cfg(not(feature = "ssr"))]
                        {
                            eprintln!(
                                "SSR feature is not enabled. Please enable the 'ssr' feature."
                            );
                            std::process::exit(1);
                        }
                    }
                }
                "native" => {
                    eprintln!("{}", t.cli.native_not_implemented);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!(
                        "{}: {}. Use 'component' or 'native'",
                        t.cli.unknown_target, target
                    );
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Package { platform }) => {
            info!("{} {}...", t.cli.packaging_for, platform);
            eprintln!("{}", t.cli.packaging_not_implemented);
            std::process::exit(1);
        }
        Some(Commands::Preview { port }) => {
            info!("{}", t.cli.preview_starting);
            let port = port.unwrap_or(3001);
            let _port = port;
            eprintln!("{}", t.cli.preview_not_implemented);
            std::process::exit(1);
        }
        Some(Commands::Init { name }) => {
            info!("{}", t.cli.init_starting);
            let name = name.unwrap_or_else(|| "my-tairitsu-app".to_string());
            crate::utils::init_project(&name)?;
        }
        Some(Commands::Wit { action }) => match action {
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
        Some(Commands::Icons { action }) => match action {
            IconsCommands::Fetch { source, force } => {
                info!("Fetching icons from {}...", source);
                let target_dir = std::path::PathBuf::from("target");
                let icon_source: crate::icons::IconSource = source.parse().unwrap_or_default();
                let cache_dir = target_dir
                    .join(crate::icons::ICON_CACHE_DIR)
                    .join(icon_source.to_string());

                if force {
                    crate::icons::force_fetch_icons(&icon_source, &cache_dir)?;
                } else {
                    crate::icons::fetch_icons(&icon_source, &cache_dir)?;
                }
                info!("Icons cached successfully.");
            }
            IconsCommands::Build {
                output,
                icons,
                tags,
            } => {
                info!("Building icon module...");

                // Load Cargo.toml to get configuration
                let cargo_toml_path = if manifest_path.is_dir() {
                    manifest_path.join("Cargo.toml")
                } else {
                    manifest_path.clone()
                };

                let content = std::fs::read_to_string(&cargo_toml_path)?;
                let manifest: toml::Value = toml::from_str(&content)?;
                let icons_config = crate::icons::parse_icons_config(&manifest)?;

                let output_path = output.unwrap_or_else(|| {
                    icons_config
                        .output
                        .as_ref()
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from("src/generated/icons.rs"))
                });

                let icon_names: Vec<String> = icons
                    .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_else(|| icons_config.icons.clone());

                let tag_names: Vec<String> = tags
                    .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_else(|| icons_config.tags.clone());

                let icon_config = crate::icons::IconConfig {
                    source: icons_config
                        .source
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    names: icon_names,
                    tags: tag_names,
                    styles: vec![crate::icons::IconStyle::Filled],
                    output: output_path,
                };

                let target_dir = std::path::PathBuf::from("target");
                let result = crate::icons::build_icons(&icon_config, &target_dir)?;

                info!(
                    "Generated {} icons to {}",
                    result.icons_count,
                    result.output_path.display()
                );
            }
            IconsCommands::List {
                source,
                tag,
                search,
            } => {
                info!("Listing icons from {}...", source);

                let icon_source: crate::icons::IconSource = source.parse().unwrap_or_default();
                let target_dir = std::path::PathBuf::from("target");
                let cache_dir = target_dir
                    .join(crate::icons::ICON_CACHE_DIR)
                    .join(icon_source.to_string());

                let metadata = crate::icons::fetch_icons(&icon_source, &cache_dir)?;

                let icons = if let Some(query) = search {
                    metadata.search(&query)
                } else if let Some(tag_filter) = tag {
                    metadata.filter_icons(&[], &[tag_filter])
                } else {
                    metadata.filter_icons(&[], &[])
                };

                println!("Found {} icons:", icons.len());
                for icon in icons.iter().take(100) {
                    let tags_str = if icon.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", icon.tags.join(", "))
                    };
                    println!("  {}{}", icon.name, tags_str);
                }

                if icons.len() > 100 {
                    println!("  ... and {} more", icons.len() - 100);
                }
            }
        },
        #[allow(unused_variables)]
        Some(Commands::Ssr {
            port,
            open,
            watch,
            build,
            routes,
            output,
        }) => {
            #[allow(unused_variables)]
            let config = crate::config::Config::load(&manifest_path)?;

            if build {
                // Pre-render mode
                let effective_routes = if routes.is_empty() {
                    let discovered = config.discovered_routes();
                    if discovered.is_empty() {
                        vec!["".to_string()]
                    } else {
                        discovered.iter().map(|r| r.path.clone()).collect()
                    }
                } else {
                    routes
                };
                let output_dir = output
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("dist/prerendered"));

                info!(
                    "Pre-rendering {} routes to {}...",
                    effective_routes.len(),
                    output_dir.display()
                );
                #[cfg(feature = "ssr")]
                {
                    crate::ssr::prerender_routes(&config, &effective_routes, &output_dir)?;
                }
                #[cfg(not(feature = "ssr"))]
                {
                    eprintln!("SSR feature is not enabled. Please enable the 'ssr' feature.");
                    std::process::exit(1);
                }
            } else {
                // Dev server mode
                info!("Starting SSR development server...");
                #[cfg(feature = "ssr")]
                {
                    crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
                }
                #[cfg(not(feature = "ssr"))]
                {
                    eprintln!("SSR feature is not enabled. Please enable the 'ssr' feature.");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Resources { action }) => match action {
            ResourcesCommands::Index { format } => {
                info!("Indexing resources...");

                let target_dir = std::path::PathBuf::from("target");
                let indexer = crate::resources::ResourceIndexer::new(&manifest_path);
                let (index, output_path) = indexer.index_to_target(&target_dir)?;

                if format == "json" {
                    let json = serde_json::to_string_pretty(&index)?;
                    println!("{}", json);
                } else {
                    println!("Resource index saved to: {}", output_path.display());
                    println!();
                    println!("SCSS files ({}):", index.scss.len());
                    for resource in &index.scss {
                        println!(
                            "  {} -> {} (hash: {})",
                            resource.source, resource.output, resource.hash
                        );
                    }
                    println!();
                    println!("SVG files ({}):", index.svg.len());
                    for resource in &index.svg {
                        println!(
                            "  {} -> {} (hash: {})",
                            resource.source, resource.id, resource.hash
                        );
                    }
                    println!();
                    println!("Total: {} resources indexed", index.count());
                }
            }
            ResourcesCommands::List { r#type, format } => {
                let target_dir = std::path::PathBuf::from("target");
                let index_path = target_dir
                    .join(crate::resources::RESOURCE_DIR)
                    .join(crate::resources::INDEX_FILE);

                if !index_path.exists() {
                    eprintln!("No resource index found. Run 'tairitsu resources index' first.");
                    std::process::exit(1);
                }

                let index = crate::resources::ResourceIndex::load(&index_path)?;

                if format == "json" {
                    let filtered_index = match r#type.as_deref() {
                        Some("scss") => {
                            let mut filtered = crate::resources::ResourceIndex::new();
                            filtered.scss = index.scss.clone();
                            filtered
                        }
                        Some("svg") => {
                            let mut filtered = crate::resources::ResourceIndex::new();
                            filtered.svg = index.svg.clone();
                            filtered
                        }
                        _ => index.clone(),
                    };
                    let json = serde_json::to_string_pretty(&filtered_index)?;
                    println!("{}", json);
                } else {
                    match r#type.as_deref() {
                        Some("scss") => {
                            println!("SCSS files ({}):", index.scss.len());
                            for resource in &index.scss {
                                println!(
                                    "  {} -> {} (hash: {})",
                                    resource.source, resource.output, resource.hash
                                );
                            }
                        }
                        Some("svg") => {
                            println!("SVG files ({}):", index.svg.len());
                            for resource in &index.svg {
                                println!(
                                    "  {} -> {} (hash: {})",
                                    resource.source, resource.id, resource.hash
                                );
                            }
                        }
                        _ => {
                            println!("SCSS files ({}):", index.scss.len());
                            for resource in &index.scss {
                                println!(
                                    "  {} -> {} (hash: {})",
                                    resource.source, resource.output, resource.hash
                                );
                            }
                            println!();
                            println!("SVG files ({}):", index.svg.len());
                            for resource in &index.svg {
                                println!(
                                    "  {} -> {} (hash: {})",
                                    resource.source, resource.id, resource.hash
                                );
                            }
                        }
                    }
                    println!();
                    println!("Total: {} resources", index.count());
                }
            }
        },
        None => {
            if cli.status || cli.shutdown || cli.daemon {
                return Ok(());
            }
            eprintln!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}
