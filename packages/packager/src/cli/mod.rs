use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::daemon::{self, is_daemon, is_tty};

/// Walk up from `start` directory looking for a workspace root (contains `packages/`).
fn find_workspace_root(start: &Path) -> PathBuf {
    let mut current = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if current.join("packages").is_dir() && current.join("Cargo.toml").is_file() {
            return current;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return start.to_path_buf(),
        }
    }
}

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

    /// Force: kill a foreign daemon occupying the port before starting
    #[arg(long, global = true)]
    force: bool,

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
        /// Port to listen on (default: 3000)
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

        /// Enable debug/inspection API server for agent automation (screenshots, DOM query, click/input)
        #[arg(long)]
        debug: bool,

        /// Port for debug API server (default: same as dev port + 1, or 3001)
        #[arg(long)]
        debug_port: Option<u16>,

        /// Clean output directory before building
        #[arg(long)]
        clean: bool,
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

    /// Pre-check and resolve build dependencies (icons, WIT, etc.)
    Check {
        #[command(subcommand)]
        action: CheckCommands,
    },

    /// Package application for distribution
    Package {
        /// Target platform (windows, macos, linux, all)
        #[arg(short, long, default_value = "all")]
        platform: String,
    },

    /// Preview production build
    Preview {
        /// Port to listen on (default: 3000)
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Create a new Tairitsu project (alias for init)
    New {
        /// Project name
        name: String,
    },

    /// Initialize a new Tairitsu project in current directory
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

    /// Visual regression testing (pixel comparison, HTML report generation)
    VisualDiff {
        /// Directory containing actual screenshots to compare against baseline
        #[arg(short, long, default_value = "target/visual-diff/actual")]
        actual_dir: String,

        /// Baseline image directory
        #[arg(short = 'b', long, default_value = "tests/visual/baseline")]
        baseline_dir: String,

        /// Output directory for diff images and report
        #[arg(short = 'o', long, default_value = "target/visual-diff")]
        output_dir: String,

        /// Pixel difference tolerance ratio (default: 0.01 = 1%)
        #[arg(long, default_value = "0.01")]
        tolerance: f32,

        /// Update baseline images from actual screenshots instead of comparing
        #[arg(long)]
        update_baseline: bool,

        /// Skip HTML report generation
        #[arg(long)]
        no_report: bool,
    },

    /// Run visual regression + event bridge tests via debug API
    Test {
        /// Debug API base URL (default: http://localhost:3001)
        #[arg(short, long, default_value = "http://localhost:3001")]
        url: String,

        /// Baseline image directory
        #[arg(short = 'b', long, default_value = "tests/visual/baseline")]
        baseline_dir: String,

        /// Actual screenshot output directory
        #[arg(short, long, default_value = "target/test-runner/actual")]
        actual_dir: String,

        /// Pixel difference tolerance ratio (default: 0.05 = 5%)
        #[arg(long, default_value = "0.05")]
        tolerance: f32,

        /// Update baseline images instead of comparing
        #[arg(long)]
        update_baselines: bool,

        /// Also run event bridge verification tests
        #[arg(long)]
        events: bool,
    },

    /// MCP server — exposes browser automation tools to AI coding assistants
    Mcp {
        /// Base URL of the tairitsu daemon debug API (auto-detected if omitted)
        #[arg(short, long)]
        url: Option<String>,

        #[command(subcommand)]
        action: Option<McpCommands>,
    },
}

#[derive(Subcommand)]
enum McpCommands {}

#[derive(Clone, Subcommand)]
enum CheckCommands {
    /// Resolve hikari-icons data (download SVGs, generate impl_icons! code)
    Icons {
        /// Use only cached SVGs, skip network downloads
        #[arg(long)]
        offline: bool,
    },

    /// Fetch and verify WIT packages
    Wit {
        /// Use only cached packages, skip network downloads
        #[arg(long)]
        offline: bool,
    },

    /// Run all checks (icons, WIT, etc.)
    All {
        /// Use only cached resources, skip network downloads
        #[arg(long)]
        offline: bool,
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

    /// Check that every WIT interface function has a browser-glue implementation
    /// (catches missing exports like location::reload before they hit production)
    CheckCompleteness {
        /// Verbose: show covered functions too, not just missing ones
        #[arg(short, long)]
        verbose: bool,

        /// Path to browser-glue source directory (auto-detected by default)
        #[arg(long)]
        glue_src: Option<PathBuf>,

        /// Path to browser-glue runtime bundle (auto-detected by default)
        #[arg(long)]
        glue_runtime: Option<PathBuf>,
    },
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

fn run_check(
    action: CheckCommands,
    manifest_path: &Option<PathBuf>,
    verbose: u8,
) -> crate::Result<()> {
    let manifest_path = resolve_manifest_dir(manifest_path);

    match action {
        CheckCommands::Icons { offline } => {
            crate::log_progress!("Resolving hikari-icons...");
            let path = crate::wasm::resolve_hikari_icons(&manifest_path, offline, verbose > 0)?;
            crate::log_ok!("hikari-icons resolved → {}", path.display());
        }
        CheckCommands::Wit { offline: _ } => {
            crate::log_progress!("Checking WIT packages...");
            crate::log_info!("(WIT check not yet implemented)");
        }
        CheckCommands::All { offline } => {
            let mut ok = true;

            crate::log_progress!("[1/2] Resolving hikari-icons...");
            match crate::wasm::resolve_hikari_icons(&manifest_path, offline, verbose > 0) {
                Ok(path) => crate::log_ok!("[1/2] hikari-icons → {}", path.display()),
                Err(e) => {
                    crate::log_fail!("[1/2] hikari-icons: {}", e);
                    ok = false;
                }
            }

            crate::log_progress!("[2/2] Checking WIT packages...");
            crate::log_info!("[2/2] (WIT check not yet implemented)");

            if !ok {
                return Err(crate::TairitsuPackagerError::DoctorError(
                    "Some checks failed".to_string(),
                ));
            }
            crate::log_ok!("All checks passed.");
        }
    }
    Ok(())
}

fn resolve_manifest_dir(path: &Option<PathBuf>) -> PathBuf {
    match path {
        Some(p) => {
            if p.is_dir() {
                p.clone()
            } else {
                p.parent()
                    .map(|d| d.to_path_buf())
                    .unwrap_or_else(|| p.clone())
            }
        }
        None => std::path::PathBuf::from("."),
    }
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
            crate::log_progress!("Stopping daemon...");
            if daemon::kill_daemon().unwrap_or(false) {
                crate::log_ok!("Daemon stopped.");
            } else {
                crate::log_info!("No daemon was running (stale PID file cleaned).");
            }
            Ok(())
        } else {
            crate::log_info!("No daemon is currently running.");
            Ok(())
        });
    }

    if cli.daemon && !daemon::is_daemon() && !cli.dry_run {
        // SAFETY: single-threaded at this point
        unsafe {
            std::env::set_var("TAIRITSU_LOG_TS", "1");
        }
        let was_running = daemon::is_daemon_running();
        if was_running {
            crate::log_progress!("Restarting daemon...");
            let _ = daemon::kill_daemon();
            crate::log_ok!("Old daemon stopped.");
        } else {
            crate::log_progress!("Starting daemon...");
        }
        daemon::cleanup_ready_file();
        let _ = daemon::truncate_log_files();
        let child_pid = match daemon::fork_daemon() {
            Ok(pid) => pid,
            Err(error) => return Some(Err(error.into())),
        };
        match daemon::wait_for_child_signal(120, Some(child_pid)) {
            Ok(Some(port)) => {
                let port_info = if port > 0 {
                    format!(" — http://localhost:{}", port)
                } else {
                    String::new()
                };
                if was_running {
                    crate::log_ok!("Daemon restarted (PID {}){}", child_pid, port_info);
                } else {
                    crate::log_ok!("Daemon started (PID {}){}", child_pid, port_info);
                }
            }
            Ok(None) => {
                crate::log_fail!("Daemon failed to start. Check logs for details.");
                std::process::exit(1);
            }
            Err(e) => {
                crate::log_fail!("{}", e);
                std::process::exit(1);
            }
        }
        return SYNC_OK;
    }

    // --daemon --dry-run also exits synchronously
    if cli.daemon && cli.dry_run {
        let _ = daemon::print_daemon_status();
        return SYNC_OK;
    }

    // `check` is fully synchronous — no tokio needed
    if let Some(Commands::Check { action }) = cli.command {
        return Some(run_check(action, &cli.manifest_path, cli.verbose));
    }

    // Not a sync daemon operation — proceed to async mode
    None
}

pub fn run_tokio() {
    if daemon::is_daemon() {
        if let Err(e) = daemon::daemonize_self() {
            let _ = daemon::signal_failed(&format!("daemonize failed: {}", e));
            std::process::exit(1);
        }
    }

    #[tokio::main]
    async fn inner() -> crate::Result<()> {
        run().await
    }

    if let Err(e) = inner() {
        if daemon::is_daemon() {
            let _ = daemon::signal_failed(&e.to_string());
        }
        crate::log_fail!("{}", e);
        std::process::exit(1);
    }
}

pub async fn run() -> crate::Result<()> {
    crate::logfmt::init();
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
            crate::log_progress!("Stopping daemon...");
            if daemon::kill_daemon()? {
                crate::log_ok!("Daemon stopped.");
            } else {
                crate::log_info!("No daemon was running (stale PID file cleaned).");
            }
        } else {
            crate::log_info!("No daemon is currently running.");
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
        let is_dev_command = matches!(
            cli.command,
            Some(Commands::Dev { .. }) | Some(Commands::Ssr { .. })
        );
        if is_dev_command {
            crate::log_fail!(
                "A daemon is already running (PID {}).",
                daemon::read_pid().unwrap_or(0)
            );
            crate::log_info!("Use --daemon   to restart the daemon");
            crate::log_info!("Use --shutdown to stop it first");
            crate::log_info!("Use --status   to check daemon state");
            std::process::exit(1);
        }
    }

    // Setup logging
    let log_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let is_mcp = matches!(cli.command, Some(Commands::Mcp { .. }));
    if !is_mcp {
        crate::logfmt::init_tracing(log_level);
    }

    let is_dev_command = matches!(
        cli.command,
        Some(Commands::Dev { .. }) | Some(Commands::Ssr { .. })
    );

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
            debug,
            debug_port,
            clean,
        }) => {
            let config = crate::config::Config::load(&manifest_path)?;
            if ssr {
                #[cfg(feature = "ssr")]
                {
                    crate::log_info!("Starting SSR development server...");
                    let port = port.unwrap_or(3000);
                    crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
                }
                #[cfg(not(feature = "ssr"))]
                {
                    crate::log_fail!(
                        "SSR feature is not enabled. Please enable the 'ssr' feature."
                    );
                    std::process::exit(1);
                }
            } else {
                crate::log_info!("{}", t.cli.starting_dev_server);
                let port = port.unwrap_or(config.dev.port);
                #[cfg(feature = "dev-server")]
                {
                    crate::wasm::dev_server(
                        &config,
                        port,
                        open,
                        watch,
                        cli.force,
                        cli.verbose > 0,
                        debug,
                        debug_port,
                        clean,
                    )
                    .await?;
                }
                #[cfg(not(feature = "dev-server"))]
                {
                    crate::log_fail!(
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
            crate::log_info!("{} {}...", t.cli.building_for, target);
            match target.as_str() {
                "component" => {
                    crate::wasm::build_component(&config, release, None, cli.verbose > 0)?;

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
                            crate::log_info!("Pre-rendering {} routes...", effective_routes.len());
                            crate::ssr::prerender_routes(&config, &effective_routes, &output_dir)?;
                        }
                        #[cfg(not(feature = "ssr"))]
                        {
                            crate::log_fail!(
                                "SSR feature is not enabled. Please enable the 'ssr' feature."
                            );
                            std::process::exit(1);
                        }
                    }
                }
                "native" => {
                    crate::log_fail!("{}", t.cli.native_not_implemented);
                    std::process::exit(1);
                }
                _ => {
                    crate::log_fail!(
                        "{}: {}. Use 'component' or 'native'",
                        t.cli.unknown_target,
                        target
                    );
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Package { platform }) => {
            crate::log_info!("{} {}...", t.cli.packaging_for, platform);
            crate::log_fail!("{}", t.cli.packaging_not_implemented);
            std::process::exit(1);
        }
        Some(Commands::Preview { port }) => {
            crate::log_info!("{}", t.cli.preview_starting);
            let port = port.unwrap_or(3000);
            let _port = port;
            crate::log_fail!("{}", t.cli.preview_not_implemented);
            std::process::exit(1);
        }
        Some(Commands::New { name }) => {
            crate::log_info!("{}", t.cli.init_starting);
            crate::utils::init_project(&name)?;
        }
        Some(Commands::Init { name }) => {
            crate::log_info!("{}", t.cli.init_starting);
            let name = name.unwrap_or_else(|| "my-tairitsu-app".to_string());
            crate::utils::init_project(&name)?;
        }
        Some(Commands::Wit { action }) => match action {
            WitCommands::Fetch { specs, offline } => {
                crate::log_info!("Fetching WIT packages...");
                crate::wit_cmd::cmd_fetch(&manifest_path, &specs, offline)?;
            }
            WitCommands::Verify { specs } => {
                crate::log_info!("Verifying WIT package cache...");
                crate::wit_cmd::cmd_verify(&manifest_path, &specs)?;
            }
            WitCommands::List => {
                crate::wit_cmd::cmd_list(&manifest_path)?;
            }
            WitCommands::CheckCompleteness {
                verbose,
                glue_src,
                glue_runtime,
            } => {
                let start_path = if manifest_path.is_file() {
                    manifest_path
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or(manifest_path)
                } else {
                    manifest_path.clone()
                };

                let ws_root = find_workspace_root(&start_path);

                crate::log_info!("Checking WIT-to-glue completeness...");
                let report = crate::wit_check::check_completeness(
                    &ws_root,
                    glue_runtime.as_deref(),
                    glue_src.as_deref(),
                )?;

                crate::wit_check::print_report(&report, verbose);

                if !report.is_fully_covered() {
                    std::process::exit(1);
                }
            }
        },
        Some(Commands::Icons { action }) => match action {
            IconsCommands::Fetch { source, force } => {
                crate::log_info!("Fetching icons from {}...", source);
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
                crate::log_info!("Icons cached successfully.");
            }
            IconsCommands::Build {
                output,
                icons,
                tags,
            } => {
                crate::log_info!("Building icon module...");

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

                crate::log_info!(
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
                crate::log_info!("Listing icons from {}...", source);

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

                crate::log_ok!("Found {} icons:", icons.len());
                for icon in icons.iter().take(100) {
                    let tags_str = if icon.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", icon.tags.join(", "))
                    };
                    crate::log_info!("  {}{}", icon.name, tags_str);
                }

                if icons.len() > 100 {
                    crate::log_info!("  ... and {} more", icons.len() - 100);
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

                crate::log_info!(
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
                    crate::log_fail!(
                        "SSR feature is not enabled. Please enable the 'ssr' feature."
                    );
                    std::process::exit(1);
                }
            } else {
                // Dev server mode
                crate::log_info!("Starting SSR development server...");
                #[cfg(feature = "ssr")]
                {
                    crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
                }
                #[cfg(not(feature = "ssr"))]
                {
                    crate::log_fail!(
                        "SSR feature is not enabled. Please enable the 'ssr' feature."
                    );
                    std::process::exit(1);
                }
            }
        }
        #[allow(unused_variables)]
        Some(Commands::Mcp { url, action }) => {
            let config = crate::mcp::McpConfig {
                base_url: url.unwrap_or_default(),
            };
            crate::mcp::run(config).await?;
        }
        Some(Commands::Resources { action }) => match action {
            ResourcesCommands::Index { format } => {
                crate::log_info!("Indexing resources...");

                let target_dir = std::path::PathBuf::from("target");
                let indexer = crate::resources::ResourceIndexer::new(&manifest_path);
                let (index, output_path) = indexer.index_to_target(&target_dir)?;

                if format == "json" {
                    let json = serde_json::to_string_pretty(&index)?;
                    println!("{}", json);
                } else {
                    crate::log_ok!("Resource index saved to: {}", output_path.display());
                    crate::log_info!("SCSS files ({}):", index.scss.len());
                    for resource in &index.scss {
                        crate::log_info!(
                            "  {} -> {} (hash: {})",
                            resource.source,
                            resource.output,
                            resource.hash
                        );
                    }
                    crate::log_info!("SVG files ({}):", index.svg.len());
                    for resource in &index.svg {
                        crate::log_info!(
                            "  {} -> {} (hash: {})",
                            resource.source,
                            resource.id,
                            resource.hash
                        );
                    }
                    crate::log_ok!("Total: {} resources indexed", index.count());
                }
            }
            ResourcesCommands::List { r#type, format } => {
                let target_dir = std::path::PathBuf::from("target");
                let index_path = target_dir
                    .join(crate::resources::RESOURCE_DIR)
                    .join(crate::resources::INDEX_FILE);

                if !index_path.exists() {
                    crate::log_fail!(
                        "No resource index found. Run 'tairitsu resources index' first."
                    );
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
                            crate::log_info!("SCSS files ({}):", index.scss.len());
                            for resource in &index.scss {
                                crate::log_info!(
                                    "  {} -> {} (hash: {})",
                                    resource.source,
                                    resource.output,
                                    resource.hash
                                );
                            }
                        }
                        Some("svg") => {
                            crate::log_info!("SVG files ({}):", index.svg.len());
                            for resource in &index.svg {
                                crate::log_info!(
                                    "  {} -> {} (hash: {})",
                                    resource.source,
                                    resource.id,
                                    resource.hash
                                );
                            }
                        }
                        _ => {
                            crate::log_info!("SCSS files ({}):", index.scss.len());
                            for resource in &index.scss {
                                crate::log_info!(
                                    "  {} -> {} (hash: {})",
                                    resource.source,
                                    resource.output,
                                    resource.hash
                                );
                            }
                            crate::log_info!("SVG files ({}):", index.svg.len());
                            for resource in &index.svg {
                                crate::log_info!(
                                    "  {} -> {} (hash: {})",
                                    resource.source,
                                    resource.id,
                                    resource.hash
                                );
                            }
                        }
                    }
                    crate::log_ok!("Total: {} resources", index.count());
                }
            }
        },
        #[allow(unused_variables)]
        Some(Commands::VisualDiff {
            actual_dir,
            baseline_dir,
            output_dir,
            tolerance,
            update_baseline,
            no_report,
        }) => {
            #[cfg(feature = "visual-diff")]
            {
                let actual_path = std::path::PathBuf::from(&actual_dir);
                let baseline_path = std::path::PathBuf::from(&baseline_dir);
                let output_path = std::path::PathBuf::from(&output_dir);

                if update_baseline {
                    let images: Vec<std::path::PathBuf> = actual_path
                        .read_dir()?
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().map(|e| e == "png").unwrap_or(false))
                        .collect();
                    if images.is_empty() {
                        crate::log_fail!("No PNG files found in {}", actual_dir);
                        std::process::exit(1);
                    }
                    let count = crate::visual_diff::update_baseline(&images, &baseline_path)?;
                    crate::log_ok!("Updated {} baseline image(s) in {}", count, baseline_dir);
                } else {
                    let images: Vec<std::path::PathBuf> = actual_path
                        .read_dir()?
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().map(|e| e == "png").unwrap_or(false))
                        .collect();
                    if images.is_empty() {
                        crate::log_fail!("No PNG files found in {}", actual_dir);
                        std::process::exit(1);
                    }
                    let config = crate::visual_diff::DiffConfig {
                        tolerance,
                        output_dir: output_path.clone(),
                        baseline_dir: baseline_path.clone(),
                        generate_html: !no_report,
                        fail_on_diff: true,
                    };
                    crate::log_info!(
                        "Running visual diff (tolerance: {:.1}%)...",
                        tolerance * 100.0
                    );
                    let report = crate::visual_diff::run_visual_diff(&images, &config)?;

                    crate::log_info!(
                        "Results: {}/{} passed, {}/{} failed",
                        report.passed,
                        report.total,
                        report.failed,
                        report.total
                    );
                    if !no_report {
                        crate::log_ok!("Report: {}/index.html", output_dir);
                    }
                    if report.failed > 0 {
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(not(feature = "visual-diff"))]
            {
                crate::log_fail!(
                    "Visual diff feature is not enabled. Please enable the 'visual-diff' feature."
                );
                std::process::exit(1);
            }
        }
        Some(Commands::Test {
            url,
            baseline_dir,
            actual_dir,
            tolerance,
            update_baselines,
            events,
        }) => {
            #[cfg(feature = "test-runner")]
            {
                use crate::test_runner::{PageSpec, TestConfig};

                let pages = vec![
                    PageSpec {
                        url: "/",
                        name: "home",
                        interactions: &[],
                    },
                    PageSpec {
                        url: "/event-test",
                        name: "event_test",
                        interactions: &[("click", "#event-test-btn")],
                    },
                ];

                let config = TestConfig {
                    base_url: url.clone(),
                    baseline_dir: PathBuf::from(&baseline_dir),
                    actual_dir: PathBuf::from(&actual_dir),
                    tolerance,
                    update_baselines,
                    pages,
                };

                crate::log_info!("Running visual regression tests via {}", url);

                let report = match crate::test_runner::run_tests(&config) {
                    Ok(r) => r,
                    Err(e) => {
                        crate::log_fail!("Test runner error: {}", e);
                        std::process::exit(1);
                    }
                };

                for r in &report.results {
                    if r.passed {
                        crate::log_ok!("{}: {}", r.name, r.detail);
                    } else {
                        crate::log_fail!("{}: {}", r.name, r.detail);
                    }
                }
                crate::log_info!(
                    "Results: {}/{} passed, {}/{} failed",
                    report.passed,
                    report.total,
                    report.failed,
                    report.total
                );

                if events {
                    crate::log_info!("Running event bridge tests...");
                    let client = reqwest::blocking::Client::new();
                    let event_pages = vec![
                        PageSpec {
                            url: "/",
                            name: "home",
                            interactions: &[],
                        },
                        PageSpec {
                            url: "/event-test",
                            name: "event_test",
                            interactions: &[],
                        },
                    ];
                    let results = crate::test_runner::run_events(&client, &url, &event_pages);
                    for r in &results {
                        if r.passed {
                            crate::log_ok!("{}: {}", r.name, r.detail);
                        } else {
                            crate::log_fail!("{}: {}", r.name, r.detail);
                        }
                    }
                }

                if report.failed > 0 {
                    std::process::exit(1);
                }
            }
            #[cfg(not(feature = "test-runner"))]
            {
                crate::log_fail!(
                    "Test runner feature is not enabled. Please enable the 'test-runner' feature."
                );
                std::process::exit(1);
            }
        }
        Some(Commands::Check { .. }) => {
            crate::log_info!("Check completed in synchronous phase.");
        }
        None => {
            if cli.status || cli.shutdown || cli.daemon {
                return Ok(());
            }
            crate::log_fail!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}
