//! Browser management module

mod cache;
mod downloader;
mod platform;

pub use cache::BrowserCache;
pub use downloader::{BrowserDownloader, DownloadProgress, CHROME_VERSION};
pub use platform::{Platform, detect_platform};
