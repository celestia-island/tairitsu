//! SSR stub implementations for browser WIT interfaces
//!
//! This module provides stub implementations for all browser WIT interfaces
//! that are NOT manually implemented. Stubs return appropriate default values
//! or errors to indicate browser-only operations.
//!
//! Manually implemented interfaces (in linker.rs):
//! - document, node, element, style, console, window, platform-helpers, event-target

use crate::host_state::SsrHostState;
use anyhow::Result;
use wasmtime::component::Linker;

/// Register all stub implementations with the linker
///
/// This function registers stub implementations for all browser interfaces
/// that are not manually implemented in linker.rs.
pub fn register_all_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Register stub implementations for interfaces that aren't manually implemented
    // This is a simplified version that handles common interfaces

    // Animation and timing interfaces
    register_animation_stubs(linker)?;
    register_animation_event_stubs(linker)?;

    // Credential management
    register_credential_stubs(linker)?;
    register_credentials_container_stubs(linker)?;
    register_credential_user_data_stubs(linker)?;
    register_federated_credential_stubs(linker)?;
    register_password_credential_stubs(linker)?;

    // Crypto interfaces
    register_crypto_stubs(linker)?;
    register_crypto_key_stubs(linker)?;
    register_subtle_crypto_stubs(linker)?;

    // History and navigation
    register_history_stubs(linker)?;
    register_location_stubs(linker)?;

    // Media interfaces
    register_audio_decoder_stubs(linker)?;
    register_audio_encoder_stubs(linker)?;
    register_video_decoder_stubs(linker)?;
    register_video_encoder_stubs(linker)?;

    // Storage
    register_storage_stubs(linker)?;
    register_storage_event_stubs(linker)?;

    Ok(())
}

// ============================================================================
// Stub registration functions for individual interfaces
// ============================================================================

fn stub_error(interface: &str, function: &str) -> String {
    format!("{}.{} is not available in SSR. Use use_effect() for browser-only operations.", interface, function)
}

fn register_animation_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Animation interface has no functions in the WIT file
    Ok(())
}

fn register_animation_event_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // AnimationEvent interface has no functions in the WIT file
    Ok(())
}

fn register_credential_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/credential@0.2.0")?;
    instance.func_wrap(
        "get-id",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    instance.func_wrap(
        "get-type",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    instance.func_wrap(
        "is-conditional-mediation-available",
        |_caller, (): ()| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_credentials_container_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/credentials-container@0.2.0")?;
    instance.func_wrap(
        "get",
        |_caller, (_self, _options): (u64, Option<u64>)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    instance.func_wrap(
        "store",
        |_caller, (_self, _credential): (u64, u64)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    instance.func_wrap(
        "create",
        |_caller, (_self, _options): (u64, Option<u64>)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    instance.func_wrap(
        "prevent-silent-access",
        |_caller, (_self,): (u64,)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_credential_user_data_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/credential-user-data@0.2.0")?;
    instance.func_wrap(
        "get-name",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    instance.func_wrap(
        "get-icon-url",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    Ok(())
}

fn register_federated_credential_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/federated-credential@0.2.0")?;
    instance.func_wrap(
        "get-provider",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    instance.func_wrap(
        "get-protocol",
        |_caller, (_self,): (u64,)| -> Result<(Option<String>,), wasmtime::Error> {
            Ok((None,))
        },
    )?;
    Ok(())
}

fn register_password_credential_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/password-credential@0.2.0")?;
    instance.func_wrap(
        "get-password",
        |_caller, (_self,): (u64,)| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    Ok(())
}

fn register_crypto_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Crypto interface has no functions in the WIT file
    Ok(())
}

fn register_crypto_key_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // CryptoKey interface has no functions in the WIT file
    Ok(())
}

fn register_subtle_crypto_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // SubtleCrypto interface has no functions in the WIT file
    Ok(())
}

fn register_history_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/history@0.2.0")?;
    instance.func_wrap(
        "get-length",
        |_caller, (): ()| -> Result<(u32,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    instance.func_wrap(
        "get-scroll-restoration",
        |_caller, (): ()| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    instance.func_wrap(
        "set-scroll-restoration",
        |_caller, (_value,): (u64,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    instance.func_wrap(
        "get-state",
        |_caller, (): ()| -> Result<(String,), wasmtime::Error> {
            Ok((String::new(),))
        },
    )?;
    instance.func_wrap(
        "go",
        |_caller, (_delta,): (Option<i32>,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    instance.func_wrap(
        "back",
        |_caller, (): ()| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    instance.func_wrap(
        "forward",
        |_caller, (): ()| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    instance.func_wrap(
        "push-state",
        |_caller, (_data, _unused, _url): (String, String, Option<String>)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    instance.func_wrap(
        "replace-state",
        |_caller, (_data, _unused, _url): (String, String, Option<String>)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;
    Ok(())
}

fn register_location_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Location interface has no functions in the WIT file
    Ok(())
}

fn register_audio_decoder_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/audio-decoder@0.2.0")?;
    instance.func_wrap(
        "get-state",
        |_caller, (_self,): (u64,)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_audio_encoder_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/audio-encoder@0.2.0")?;
    instance.func_wrap(
        "get-state",
        |_caller, (_self,): (u64,)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_video_decoder_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/video-decoder@0.2.0")?;
    instance.func_wrap(
        "get-state",
        |_caller, (_self,): (u64,)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_video_encoder_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    let mut instance = linker.instance("tairitsu-browser:full/video-encoder@0.2.0")?;
    instance.func_wrap(
        "get-state",
        |_caller, (_self,): (u64,)| -> Result<(u64,), wasmtime::Error> {
            Ok((0,))
        },
    )?;
    Ok(())
}

fn register_storage_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Storage interface has no functions in the WIT file
    Ok(())
}

fn register_storage_event_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {
    // StorageEvent interface has no functions in the WIT file
    Ok(())
}
