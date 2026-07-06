//! Browser management module

mod cache;
pub mod cdp;
mod downloader;
mod platform;

pub use cache::BrowserCache;
pub use cdp::CdpClient;
pub use downloader::{BrowserDownloader, DownloadProgress, CHROME_VERSION};
pub use platform::{detect_platform, Platform};
