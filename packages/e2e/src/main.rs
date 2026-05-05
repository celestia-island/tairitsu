use clap::Parser;
use tairitsu_e2e::run_all_tests;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tracing::info;

#[derive(Parser)]
#[command(name = "tairitsu-e2e")]
#[command(about = "E2E testing framework for Tairitsu")]
struct Args {
    #[arg(short, long, default_value = "http://localhost:4444/wd/hub")]
    selenium_url: String,

    #[arg(short, long, default_value = "http://localhost:8080")]
    website_url: String,

    #[arg(short, long, default_value = "./screenshots")]
    screenshots_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().with_env_filter("info").init();

    unsafe {
        std::env::set_var("WEBSITE_BASE_URL", &args.website_url);
        std::env::set_var("E2E_SCREENSHOTS_DIR", &args.screenshots_dir);
    }

    info!("Starting E2E tests...");
    info!("Selenium URL: {}", args.selenium_url);
    info!("Website URL: {}", args.website_url);
    info!("Screenshots dir: {}", args.screenshots_dir);

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(&args.selenium_url, caps).await?;

    let results = run_all_tests(&driver).await?;

    let passed = results
        .iter()
        .filter(|r| matches!(r.status, tairitsu_e2e::tests::TestStatus::Success))
        .count();
    let total = results.len();

    info!("\nTest Summary: {}/{} passed", passed, total);

    driver.quit().await?;

    if passed < total {
        std::process::exit(1);
    }

    Ok(())
}
