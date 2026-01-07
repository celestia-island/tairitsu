// Guest WASM Component Implementation
//
// This file implements the WIT interfaces defined in tairitsu.wit

use wasmtime_wasi::WasiCtx;

// ============================================================================
// Calculator Interface Implementation
// ============================================================================

#[allow(dead_code)]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[allow(dead_code)]
pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[allow(dead_code)]
pub fn mul(a: i32, b: i32) -> i32 {
    a * b
}

#[allow(dead_code)]
pub fn div(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// ============================================================================
// String Operations Interface Implementation
// ============================================================================

#[allow(dead_code)]
pub fn to_upper(text: String) -> String {
    text.to_uppercase()
}

#[allow(dead_code)]
pub fn to_lower(text: String) -> String {
    text.to_lowercase()
}

#[allow(dead_code)]
pub fn reverse(text: String) -> String {
    text.chars().rev().collect()
}

#[allow(dead_code)]
pub fn length(text: String) -> u32 {
    text.len() as u32
}

// ============================================================================
// Data Processing Interface Implementation
// ============================================================================

#[allow(dead_code)]
pub fn process_numbers(numbers: Vec<u32>) -> Vec<u32> {
    numbers.into_iter().map(|n| n * 2).collect()
}

#[allow(dead_code)]
pub fn transform(input: String, multiplier: u32) -> String {
    format!("{} (x{})", input, multiplier)
}

// ============================================================================
// Host Logger Import (Optional)
// ============================================================================

pub struct HostLogger;

impl HostLogger {
    #[allow(dead_code)]
    pub fn log(_level: String, _message: String) {
        // Placeholder for host logger implementation
    }
}
