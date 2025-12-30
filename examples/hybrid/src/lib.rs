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
struct GuestImpl;

#[cfg(target_family = "wasm")]
impl GuestApi for GuestImpl {
    fn init() -> Result<(), String> {
        // Log initialization via host API
        tairitsu::core::host_api::log("info", "Guest module initialized");
        
        // Try to execute a command on the host
        match tairitsu::core::host_api::execute("get_info", "{}") {
            Ok(response) => {
                tairitsu::core::host_api::log("info", &format!("Host info: {}", response));
                Ok(())
            }
            Err(e) => {
                tairitsu::core::host_api::log("error", &format!("Failed to get host info: {}", e));
                Err(e)
            }
        }
    }
    
    fn handle_command(command: String, payload: String) -> Result<String, String> {
        // Log the received command
        tairitsu::core::host_api::log("info", &format!("Received command: {} with payload: {}", command, payload));
        
        // Handle different commands
        match command.as_str() {
            "greet" => {
                let response = format!("Hello from WASM guest! You said: {}", payload);
                Ok(response)
            }
            "compute" => {
                // Simulate some computation
                let result = payload.len() * 42;
                Ok(format!("Computed result: {}", result))
            }
            "call_host" => {
                // Demonstrate calling back to the host
                match tairitsu::core::host_api::execute("echo", &payload) {
                    Ok(host_response) => {
                        Ok(format!("Host echoed: {}", host_response))
                    }
                    Err(e) => Err(format!("Host call failed: {}", e))
                }
            }
            _ => {
                Err(format!("Unknown command: {}", command))
            }
        }
    }
}

#[cfg(target_family = "wasm")]
export!(GuestImpl);
