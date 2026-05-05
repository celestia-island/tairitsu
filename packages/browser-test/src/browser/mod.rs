//! Browser management module

mod cache;
mod downloader;
mod platform;

pub use cache::BrowserCache;
pub use downloader::{BrowserDownloader, CHROME_VERSION, DownloadProgress};
pub use platform::{Platform, detect_platform};
