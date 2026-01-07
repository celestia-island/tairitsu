//! Macro-based WASM Guest Library
//!
//! This module demonstrates using Tairitsu's macro system combined with
//! WIT-based exports for bidirectional host-guest communication.

#![cfg_attr(not(target_family = "wasm"), allow(dead_code))]

// For WASM targets, use WIT bindings with proper Component Model export
#[cfg(target_family = "wasm")]
mod guest {

use super::Guest;

    // Generate WIT bindings from wit/tairitsu.wit
    // This generates the bindings and the export!() macro
    wit_bindgen::generate!({
        path: "wit",
        world: "tairitsu",
    });

    // Export the Guest implementation using the generated macro
    export!(Guest);
}

// For non-WASM targets (native/testing), provide simple implementations
#[cfg(not(target_family = "wasm"))]
pub mod guest {
    use log::info;

    /// Initialize the guest module
    pub fn init() -> Result<(), String> {
        info!("[Guest Native - Macro] Initializing...");
        Ok(())
    }

    /// Process a request
    pub fn process(input: String) -> Result<String, String> {
        Ok(format!("Processed (macro): {}", input))
    }

    /// Get guest information
    pub fn get_info() -> tairitsu::GuestInfo {
        tairitsu::GuestInfo::new("tairitsu-macro-guest", "0.1.0").with_feature("wit-native-macro")
    }
}

// WASM guest implementation
#[cfg(target_family = "wasm")]
pub struct Guest;

// Implement the generated guest_api::Guest trait
#[cfg(target_family = "wasm")]
impl guest::exports::tairitsu::core::guest_api::Guest for Guest {
    /// Initialize the guest module
    fn init() -> Result<(), String> {
        Ok(())
    }

    /// Process a request
    fn process(input: String) -> Result<String, String> {
        Ok(format!("Processed from WASM guest (macro): {}", input))
    }

    /// Get guest name
    fn getname() -> String {
        "tairitsu-macro-wasm-guest".to_string()
    }

    /// Get guest version
    fn getversion() -> String {
        "0.1.0".to_string()
    }

    /// Get guest features
    fn getfeatures() -> Vec<String> {
        vec!["wit-native-macro".to_string(), "wasm".to_string()]
    }

    /// Shutdown the guest module
    fn shutdown() -> Result<(), String> {
        Ok(())
    }

    /// Handle a notification
    fn notify(_event: String, _data: String) -> Result<(), String> {
        Ok(())
    }
}
