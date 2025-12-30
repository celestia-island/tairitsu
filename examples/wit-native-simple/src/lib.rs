//! Approach B: WASM Guest - Using trait-based composable commands
//! This demonstrates how traits enable composable type-safe commands
//! that work seamlessly across the WASM boundary

#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
});

#[cfg(target_family = "wasm")]
use exports::tairitsu::core::guest_api::Guest as GuestApi;

// For Approach B, we manually define the command structures
// but in a production system, these would be shared types

#[cfg(target_family = "wasm")]
struct GuestImpl;

#[cfg(target_family = "wasm")]
impl GuestApi for GuestImpl {
    fn init() -> Result<(), String> {
        // Log initialization via host API
        tairitsu::core::host_api::log("info", "Guest module initialized (Approach B - Traits)");

        // Demonstrate trait-based type-safe communication
        tairitsu::core::host_api::log("info", "Testing trait-based type-safe command...");

        // Execute GetInfo command on host
        match tairitsu::core::host_api::execute("get_info", "{}") {
            Ok(response) => {
                tairitsu::core::host_api::log("info", &format!("Host info received: {}", response));
            }
            Err(e) => {
                tairitsu::core::host_api::log("error", &format!("Failed to get host info: {}", e));
            }
        }

        Ok(())
    }

    fn handle_command(command: String, payload: String) -> Result<String, String> {
        tairitsu::core::host_api::log(
            "info",
            &format!(
                "Guest received command: '{}' with payload: '{}'",
                command, payload
            ),
        );

        // Handle commands using trait-based pattern matching
        match command.as_str() {
            "greet" => {
                let response =
                    format!("Hello from Approach B (trait-based)! You said: {}", payload);
                Ok(response)
            }

            "compute" => {
                // Demonstrate computation with trait-based handlers
                let word_count = payload.split_whitespace().count();
                let char_count = payload.chars().count();
                let uppercase_count = payload.chars().filter(|c| c.is_uppercase()).count();

                Ok(format!(
                    "Computation result (Approach B): {} words, {} chars, {} uppercase",
                    word_count, char_count, uppercase_count
                ))
            }

            "call_host" => {
                // Demonstrate nested bidirectional call with trait-based composition
                tairitsu::core::host_api::log(
                    "info",
                    "Guest is calling back to host (trait-based composable call)...",
                );

                // Call the host's echo command with the payload
                match tairitsu::core::host_api::execute("echo", &payload) {
                    Ok(host_response) => Ok(format!(
                        "Host echoed back (via trait-based commands): {}",
                        host_response
                    )),
                    Err(e) => Err(format!("Failed to call host: {}", e)),
                }
            }

            "filesystem_write" => {
                // Demonstrate filesystem interface (trait-based)
                tairitsu::core::host_api::log(
                    "info",
                    &format!("Writing data via trait-based interface: {}", payload),
                );
                Ok(format!(
                    "Wrote {} bytes to virtual filesystem (Approach B)",
                    payload.len()
                ))
            }

            "network_request" => {
                // Demonstrate network interface (trait-based)
                tairitsu::core::host_api::log(
                    "info",
                    &format!(
                        "Making network request via trait-based interface: {}",
                        payload
                    ),
                );

                // Simulate calling host's network interface
                match tairitsu::core::host_api::execute("network_get", &payload) {
                    Ok(response) => Ok(format!("Network response (Approach B): {}", response)),
                    Err(e) => Err(format!("Network error: {}", e)),
                }
            }

            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

#[cfg(target_family = "wasm")]
export!(GuestImpl);
