// Guest WASM Component Implementation
//
// This file implements the WIT interfaces defined in tairitsu.wit

use wasmtime_wasi::WasiCtx;

// ============================================================================
// Calculator Interface Implementation
// ============================================================================

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

pub fn mul(a: i32, b: i32) -> i32 {
    a * b
}

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

pub fn to_upper(text: String) -> String {
    text.to_uppercase()
}

pub fn to_lower(text: String) -> String {
    text.to_lowercase()
}

pub fn reverse(text: String) -> String {
    text.chars().rev().collect()
}

pub fn length(text: String) -> u32 {
    text.len() as u32
}

// ============================================================================
// Data Processing Interface Implementation
// ============================================================================

pub fn process_numbers(numbers: Vec<u32>) -> Vec<u32> {
    numbers.into_iter().map(|n| n * 2).collect()
}

pub fn transform(input: String, multiplier: u32) -> String {
    format!("{} (x{})", input, multiplier)
}

// ============================================================================
// Host Logger Import (Optional)
// ============================================================================

pub struct HostLogger;

impl HostLogger {
    pub fn log(_level: String, _message: String) {
        // HostLogger.log is intentionally a no-op in this demo example;
        // real host logging would forward to the host via a WIT import.
    }
}
