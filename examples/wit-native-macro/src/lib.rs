//! Macro-based WASM Guest Library
//!
//! This module demonstrates using Tairitsu's macro system combined with
//! WIT-based exports for bidirectional host-guest communication.

#![cfg_attr(not(target_family = "wasm"), allow(dead_code))]

// For WASM targets, use WIT bindings
#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    inline: "
        package tairitsu:core;

        world tairitsu {
            import host-api;
            export guest-api;
        }

        interface host-api {
            log: func(message: string);
            execute-command: func(command: string, args: list<string>) -> result<string, string>;
        }

        interface guest-api {
            init: func() -> result<_, string>;
            process: func(input: string) -> result<string, string>;
            get-info: func() -> (name: string, version: string, features: list<string>);
        }
    ",
});

// For non-WASM targets (native/testing), provide simple implementations
#[cfg(not(target_family = "wasm"))]
pub mod guest {
    /// Initialize the guest module
    pub fn init() -> Result<(), String> {
        println!("[Guest Native - Macro] Initializing...");
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

#[cfg(target_family = "wasm")]
impl Guest {
    /// Initialize the guest module
    pub fn init() -> Result<(), String> {
        Ok(())
    }

    /// Process a request
    pub fn process(input: String) -> Result<String, String> {
        // In a real implementation, you could use the macro-generated
        // command types here for type-safe dispatch
        Ok(format!("Processed from WASM guest (macro): {}", input))
    }

    /// Get guest information
    pub fn get_info() -> GuestInfo {
        GuestInfo {
            name: "tairitsu-macro-wasm-guest".to_string(),
            version: "0.1.0".to_string(),
            features: vec!["wit-native-macro".to_string(), "wasm".to_string()],
        }
    }
}

// Type that matches WIT definition
#[cfg(target_family = "wasm")]
pub struct GuestInfo {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}
