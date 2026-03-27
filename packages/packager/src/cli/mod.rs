use std::path::PathBuf;

use clap::{Parser, Subcommand};
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

        /// Enable server-side rendering
        #[arg(long)]
        ssr: bool,
    },

    /// Build for production
    Build {
        /// Build target (wasm, native)
        #[arg(short, long, default_value = "component")]
        target: String,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,

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

pub async fn run() -> crate::Result<()> {
    let cli = Cli::parse();
    let t = crate::i18n::translations();

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
        Commands::Dev {
            port,
            open,
            watch,
            ssr,
        } => {
            let config = crate::config::Config::load(&manifest_path)?;
            if ssr {
                info!("Starting SSR development server...");
                let port = port.unwrap_or(3000);
                crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
            } else {
                info!("{}", t.cli.starting_dev_server);
                let port = port.unwrap_or(config.dev.port);
                crate::wasm::dev_server(&config, port, open, watch).await?;
            }
        }
        Commands::Build {
            target,
            release,
            ssr,
            routes,
        } => {
            let config = crate::config::Config::load(&manifest_path)?;
            info!("{} {}...", t.cli.building_for, target);
            match target.as_str() {
                "component" => {
                    crate::wasm::build_component(&config, release, None)?;

                    // Handle SSR pre-rendering if requested
                    if ssr {
                        let routes = if routes.is_empty() {
                            vec!["".to_string()]
                        } else {
                            routes
                        };
                        let output_dir = std::path::PathBuf::from("dist/prerendered");
                        info!("Pre-rendering {} routes...", routes.len());
                        crate::ssr::prerender_routes(&config, &routes, &output_dir)?;
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
        Commands::Package { platform } => {
            info!("{} {}...", t.cli.packaging_for, platform);
            eprintln!("{}", t.cli.packaging_not_implemented);
            std::process::exit(1);
        }
        Commands::Preview { port } => {
            info!("{}", t.cli.preview_starting);
            let port = port.unwrap_or(3001);
            let _port = port;
            eprintln!("{}", t.cli.preview_not_implemented);
            std::process::exit(1);
        }
        Commands::Init { name } => {
            info!("{}", t.cli.init_starting);
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
        Commands::Icons { action } => match action {
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
        Commands::Ssr {
            port,
            open,
            watch,
            build,
            routes,
            output,
        } => {
            let config = crate::config::Config::load(&manifest_path)?;

            if build {
                // Pre-render mode
                let routes = if routes.is_empty() {
                    vec!["".to_string()]
                } else {
                    routes
                };
                let output_dir = output
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("dist/prerendered"));

                info!(
                    "Pre-rendering {} routes to {}...",
                    routes.len(),
                    output_dir.display()
                );
                crate::ssr::prerender_routes(&config, &routes, &output_dir)?;
            } else {
                // Dev server mode
                info!("Starting SSR development server...");
                crate::ssr::ssr_dev_server(&config, port, open, watch).await?;
            }
        }
        Commands::Resources { action } => match action {
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
    }

    Ok(())
}
