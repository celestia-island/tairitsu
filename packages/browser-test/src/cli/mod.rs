//! CLI module

use anyhow::Result;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    browser::{detect_platform, BrowserCache, BrowserDownloader},
    runner::{TestHarness, TestHarnessConfig},
};

/// Tairitsu Browser Test CLI
#[derive(Parser, Debug)]
#[command(name = "tairitsu-browser-test")]
#[command(about = "Browser testing infrastructure for Tairitsu", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Browser management commands
    #[command(subcommand)]
    Browser(BrowserCommands),

    /// Test execution commands
    #[command(subcommand)]
    Test(TestCommands),
}

#[derive(Subcommand, Debug)]
pub enum BrowserCommands {
    /// Download and cache Chromium browser
    Install {
        /// Chrome version to download
        #[arg(long, default_value = crate::browser::CHROME_VERSION)]
        version: String,

        /// Mirror URL for downloads
        #[arg(long, env = "TAIRITSU_BROWSER_MIRROR")]
        mirror: Option<String>,

        /// Custom cache directory
        #[arg(long, env = "TAIRITSU_BROWSER_CACHE_DIR")]
        cache_dir: Option<PathBuf>,

        /// Force re-download even if cached
        #[arg(long)]
        force: bool,
    },

    /// List cached browser versions
    List {
        /// Custom cache directory
        #[arg(long, env = "TAIRITSU_BROWSER_CACHE_DIR")]
        cache_dir: Option<PathBuf>,
    },

    /// Clear browser cache
    Clear {
        /// Custom cache directory
        #[arg(long, env = "TAIRITSU_BROWSER_CACHE_DIR")]
        cache_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TestCommands {
    /// Run browser-glue tests
    Run {
        /// Path to Chromium executable (auto-detected if not specified)
        #[arg(long, env = "CHROMIUM_BIN")]
        chromium: Option<PathBuf>,

        /// Run in headless mode (default)
        #[arg(long, default_value = "true")]
        headless: bool,

        /// Filter tests by name pattern (supports * wildcard)
        #[arg(long)]
        filter: Option<String>,

        /// Custom cache directory for browser
        #[arg(long, env = "TAIRITSU_BROWSER_CACHE_DIR")]
        cache_dir: Option<PathBuf>,

        /// Port for local test server
        #[arg(long, default_value = "3847")]
        port: u16,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Browser(browser_cmd) => self.run_browser_command(browser_cmd).await,
            Commands::Test(test_cmd) => self.run_test_command(test_cmd).await,
        }
    }

    async fn run_browser_command(&self, cmd: &BrowserCommands) -> Result<()> {
        match cmd {
            BrowserCommands::Install {
                version,
                mirror,
                cache_dir,
                force,
            } => {
                let cache = BrowserCache::new(cache_dir.clone());

                if *force {
                    cache.clear()?;
                }

                let platform = detect_platform();
                println!("Installing Chromium {} for {}", version, platform);

                let downloader = BrowserDownloader::new(cache, mirror.clone());
                let path = downloader.download(version, platform).await?;

                println!("✅ Chromium installed at: {}", path.display());
            }
            BrowserCommands::List { cache_dir } => {
                let cache = BrowserCache::new(cache_dir.clone());
                let cached = cache.list_cached()?;

                if cached.is_empty() {
                    println!("No browsers cached.");
                } else {
                    println!("Cached browsers:");
                    for (version, platform) in cached {
                        let path = cache.executable_path(&version, platform);
                        println!("  {} ({}) at {}", version, platform, path.display());
                    }
                }
            }
            BrowserCommands::Clear { cache_dir } => {
                let cache = BrowserCache::new(cache_dir.clone());
                cache.clear()?;
                println!("✅ Browser cache cleared.");
            }
        }
        Ok(())
    }

    async fn run_test_command(&self, cmd: &TestCommands) -> Result<()> {
        match cmd {
            TestCommands::Run {
                chromium,
                headless,
                filter,
                cache_dir,
                port,
            } => {
                let browser_cache = BrowserCache::new(cache_dir.clone());

                // Determine Chromium path
                let chromium_path = match chromium {
                    Some(path) => path.clone(),
                    None => {
                        // Try to find in cache first
                        let platform = detect_platform();
                        let version = crate::browser::CHROME_VERSION;
                        let cached_path = browser_cache.executable_path(version, platform);

                        if cached_path.exists() {
                            println!("Using cached Chromium: {}", cached_path.display());
                            cached_path
                        } else {
                            // Try system PATH
                            which::which("chromium")
                                .or_else(|_| which::which("google-chrome"))
                                .or_else(|_| which::which("chrome"))
                                .map_err(|_| anyhow::anyhow!(
                                    "Chromium not found. Run 'tairitsu-browser-test browser install' first, \
                                     or set CHROMIUM_BIN environment variable."
                                ))?
                        }
                    }
                };

                let config = TestHarnessConfig {
                    chromium_path,
                    headless: *headless,
                    filter: filter.clone(),
                    server_port: *port,
                    ..Default::default()
                };

                let mut harness = TestHarness::new(config);
                harness.start().await?;

                let report = harness.run_tests().await?;

                println!("{}", report);

                harness.shutdown().await?;

                if !report.is_success() {
                    std::process::exit(1);
                }
            }
        }
        Ok(())
    }
}
