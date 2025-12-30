//! Guest side implementation - runs in WASM
//! This demonstrates how the guest can import host APIs and export its own APIs

#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
});

#[cfg(target_family = "wasm")]
use exports::tairitsu::core::guest_api::Guest as GuestApi;

#[cfg(target_family = "wasm")]
use serde::{Deserialize, Serialize};

// Re-define the command enums for the guest side
// In a real application, these would be shared via a common crate

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
#[serde(tag = "type", content = "data")]
enum GuestCommands {
    Greet(String),
    Compute(String),
    CallHost(String),
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
        tairitsu::core::host_api::log("info", "Guest module initialized");

        // Try to execute a typed command on the host
        let cmd = HostCommands::GetInfo;
        let cmd_json =
            serde_json::to_string(&cmd).map_err(|e| format!("Serialization error: {}", e))?;

        match tairitsu::core::host_api::execute(&cmd_json, "") {
            Ok(response_json) => {
                match serde_json::from_str::<HostResponse>(&response_json) {
                    Ok(HostResponse::Info {
                        name,
                        version,
                        status,
                    }) => {
                        tairitsu::core::host_api::log(
                            "info",
                            &format!("Host info: {} v{} ({})", name, version, status),
                        );
                    }
                    Ok(HostResponse::Text(text)) => {
                        tairitsu::core::host_api::log("info", &format!("Host response: {}", text));
                    }
                    Err(e) => {
                        tairitsu::core::host_api::log(
                            "warn",
                            &format!("Failed to parse response: {}", e),
                        );
                    }
                }
                Ok(())
            }
            Err(e) => {
                tairitsu::core::host_api::log("error", &format!("Failed to get host info: {}", e));
                Err(e)
            }
        }
    }

    fn handle_command(command: String, _payload: String) -> Result<String, String> {
        // Try to parse as a typed command
        let cmd: GuestCommands = serde_json::from_str(&command)
            .map_err(|_| format!("Failed to deserialize command: {}", command))?;

        tairitsu::core::host_api::log("info", &format!("Received typed command: {:?}", cmd));

        let response = match cmd {
            GuestCommands::Greet(msg) => {
                GuestResponse::Text(format!("Hello from WASM guest! You said: {}", msg))
            }
            GuestCommands::Compute(payload) => {
                // Simulate some computation
                let result = payload.len() * 42;
                GuestResponse::Text(format!("Computed result: {}", result))
            }
            GuestCommands::CallHost(payload) => {
                // Demonstrate calling back to the host with typed command
                let host_cmd = HostCommands::Echo(payload);
                let host_cmd_json = serde_json::to_string(&host_cmd)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                match tairitsu::core::host_api::execute(&host_cmd_json, "") {
                    Ok(host_response_json) => {
                        match serde_json::from_str::<HostResponse>(&host_response_json) {
                            Ok(HostResponse::Text(text)) => {
                                GuestResponse::Text(format!("Host echoed: {}", text))
                            }
                            Ok(HostResponse::Info { name, .. }) => {
                                GuestResponse::Text(format!("Got info from {}", name))
                            }
                            Err(e) => {
                                return Err(format!("Failed to parse host response: {}", e));
                            }
                        }
                    }
                    Err(e) => return Err(format!("Host call failed: {}", e)),
                }
            }
            GuestCommands::Custom { name, data } => {
                GuestResponse::Text(format!("Custom command '{}': {}", name, data))
            }
        };

        serde_json::to_string(&response).map_err(|e| format!("Response serialization error: {}", e))
    }
}

#[cfg(target_family = "wasm")]
export!(GuestImpl);
