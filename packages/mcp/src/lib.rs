//! tairitsu MCP server — vision analysis + browser automation tools.
//!
//! Provides two tools:
//! - `analyze_screenshot`: capture a screenshot from a running shirabe instance
//!   and analyze it with a vision LLM from provider-registry.
//! - `list_vision_models`: list all vision-capable models from provider-registry.

pub mod client;
pub mod registry;
pub mod vision;

pub use vision::{do_analyze, do_list_models, ModelInfo};
