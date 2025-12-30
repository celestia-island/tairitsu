//! Tairitsu - Generic WASM Component Runtime Engine
//!
//! This framework provides Docker-like image/container mechanisms for running WASM components.
//! It does not prescribe any specific WIT interface - users define their own.
//!
//! ## Architecture
//!
//! - [`Image`] - Represents a compiled WASM component (like a Docker image)
//! - [`Container`] - Represents a running instance of an Image (like a Docker container)
//! - [`Registry`] - Manages multiple Images and Containers (like a Docker daemon)
//! - [`WitInterface`] - User-defined WIT interface trait
//! - [`ContainerBuilder`] - Builder pattern for creating containers with custom WIT bindings
//!
//! ## Quick Start
//!
//! ```rust,no_run,ignore
//! use tairitsu::{Container, GuestInstance, Image};
//! use bytes::Bytes;
//!
//! // 1. Define your WIT interface (use wit-bindgen to generate bindings)
//! // wit-bindgen generates your WIT code...
//!
//! // 2. Create a WASM image
//! let wasm_binary = std::fs::read("my_component.wasm")?;
//! let image = Image::new(Bytes::from(wasm_binary))?;
//!
//! // 3. Create container (user handles WIT binding)
//! let container = Container::builder(image)?
//!     .with_guest_initializer(|ctx| {
//!         // Register your WIT interface
//!         MyWit::add_to_linker(ctx.linker, |state| &mut state.my_data)?;
//!         let instance = MyWit::instantiate(ctx.store, ctx.component, ctx.linker)?;
//!         Ok(GuestInstance::new(instance))
//!     })?
//!     .build()?;
//! ```
//!
//! ## Helper Macros
//!
//! ```rust,no_run
//! use tairitsu::wit_interface;
//!
//! // Automatically generate WIT interface code
//! wit_interface! {
//!     interface filesystem {
//!         read: func(path: String) -> Result<Vec<u8>, String>;
//!         write: func(path: String, data: Vec<u8>) -> Result<(), String>;
//!     }
//! }
//! ```

pub mod container;
mod image;
pub mod registry;
pub mod wit_helper;
pub mod wit_registry;

pub use container::{Container, GuestHandlerContext, GuestInstance, HostState, HostStateImpl};
pub use image::Image;
pub use registry::Registry;
pub use wit_helper::GuestInfo;
pub use wit_registry::{
    CompositeWitInterface, WitCommand, WitCommandDispatcher, WitCommandHandler, WitInterface,
};

// Re-export common types
pub use anyhow::{Error, Result};

// Re-export procedural macros
pub use tairitsu_macros::{export_wit, wit_guest_impl, wit_interface, WitCommand};

// Re-export wasmtime types for user convenience
pub use wasmtime::{Engine, Store};
pub use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

// Helper macros are automatically available through the wit_helper module
// Usage: tairitsu::impl_wit_interface!(), tairitsu::simple_handler!(), etc.
