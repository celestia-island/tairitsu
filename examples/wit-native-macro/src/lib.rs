//! Approach A: WASM Guest - Demonstrates type-safe commands across WASM boundary
//! Note: The macro-generated enums are primarily for the host side.
//! The guest uses WIT-bindgen for type safety and serde for serialization.

#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
});

#[cfg(target_family = "wasm")]
use exports::tairitsu::core::guest_api::Guest as GuestApi;

#[cfg(target_family = "wasm")]
use serde::{Deserialize, Serialize};

// Define the command enums (matching the host side)
#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum GuestCommands {
    Greet(String),
    Compute(String),
    CallHost(String),
    Custom { name: String, data: String },
}

#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum HostCommands {
    GetInfo,
    Echo(String),
    Custom { name: String, data: String },
}

#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum HostResponse {
    Info {
        name: String,
        version: String,
        status: String,
    },
    Text(String),
}

#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GuestResponse {
    Text(String),
}

#[cfg(target_family = "wasm")]
struct GuestImpl;

#[cfg(target_family = "wasm")]
impl GuestApi for GuestImpl {
    fn init() -> Result<(), String> {
        // Log initialization via host API
        tairitsu::core::host_api::log("info", "Guest module initialized (Approach A - Macros)");

        // Call host using type-safe command
        tairitsu::core::host_api::log(
            "info",
            "Testing macro-generated type-safe command...",
        );

        // Execute GetInfo command on host
        let cmd = HostCommands::GetInfo;
        let cmd_json = serde_json::to_string(&cmd).map_err(|e| format!("Serialization error: {}", e))?;

        match tairitsu::core::host_api::execute(&cmd_json, "") {
            Ok(response_json) => {
                match serde_json::from_str::<HostResponse>(&response_json) {
                    Ok(HostResponse::Info { name, version, status }) => {
                        tairitsu::core::host_api::log(
                            "info",
                            &format!("Host info: {} v{} ({})", name, version, status),
                        );
                    }
                    Ok(HostResponse::Text(text)) => {
                        tairitsu::core::host_api::log(
                            "info",
                            &format!("Host response: {}", text),
                        );
                    }
                    Err(e) => {
                        tairitsu::core::host_api::log("error", &format!("Failed to parse host response: {}", e));
                    }
                }
            }
            Err(e) => {
                tairitsu::core::host_api::log("error", &format!("Failed to get host info: {}", e));
            }
        }

        Ok(())
    }

    fn handle_command(command: String, _payload: String) -> Result<String, String> {
        tairitsu::core::host_api::log(
            "info",
            &format!("Guest received command: '{}'", command),
        );

        // Deserialize the command
        let cmd: GuestCommands = serde_json::from_str(&command)
            .map_err(|e| format!("Failed to deserialize command: {}", e))?;

        // Handle commands using pattern matching
        let response = match cmd {
            GuestCommands::Greet(name) => {
                GuestResponse::Text(format!(
                    "Hello from Approach A (macro-generated)! You said: {}",
                    name
                ))
            }

            GuestCommands::Compute(input) => {
                let word_count = input.split_whitespace().count();
                let char_count = input.chars().count();
                GuestResponse::Text(format!(
                    "Computation result: {} words, {} characters (Approach A)",
                    word_count, char_count
                ))
            }

            GuestCommands::CallHost(message) => {
                // Demonstrate nested bidirectional call
                tairitsu::core::host_api::log(
                    "info",
                    "Guest is calling back to host during command processing...",
                );

                // Call the host's echo command with the message
                let host_cmd = HostCommands::Echo(message);
                let host_cmd_json = serde_json::to_string(&host_cmd)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                match tairitsu::core::host_api::execute(&host_cmd_json, "") {
                    Ok(host_response_json) => {
                        match serde_json::from_str::<HostResponse>(&host_response_json) {
                            Ok(HostResponse::Text(text)) => {
                                GuestResponse::Text(format!(
                                    "Host echoed back (via macro-generated commands): {}",
                                    text
                                ))
                            }
                            Ok(HostResponse::Info { .. }) => {
                                GuestResponse::Text("Unexpected Info response from Echo".to_string())
                            }
                            Err(e) => return Err(format!("Failed to parse host response: {}", e)),
                        }
                    }
                    Err(e) => return Err(format!("Failed to call host: {}", e)),
                }
            }

            GuestCommands::Custom { name, data } => {
                GuestResponse::Text(format!("Custom command '{}' with data: {}", name, data))
            }
        };

        // Serialize the response
        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    }
}

#[cfg(target_family = "wasm")]
export!(GuestImpl);
