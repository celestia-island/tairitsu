//! Browser management module

pub mod cdp;
mod cache;
mod downloader;
mod platform;

pub use cache::BrowserCache;
pub use cdp::CdpClient;
pub use downloader::{BrowserDownloader, DownloadProgress, CHROME_VERSION};
pub use platform::{detect_platform, Platform};
