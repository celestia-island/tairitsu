//! WIT-based registry system for composable type-safe command handling
//!
//! This module provides a trait-based system for registering and composing
//! multiple WIT interface implementations without runtime serialization.

use std::any::Any;
use std::collections::HashMap;

/// Trait for WIT command types that can be dispatched
pub trait WitCommand: Send + Sync + 'static {
    /// The response type for this command
    type Response: Send + Sync + 'static;

    /// Get the command name for routing
    fn command_name(&self) -> &'static str;

    /// Convert to Any for dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

/// Trait for WIT command handlers
pub trait WitCommandHandler<C: WitCommand>: Send + Sync {
    /// Execute a command and return its response
    fn execute(&mut self, command: &C) -> Result<C::Response, String>;
}

/// Dynamic command dispatcher using trait objects
pub struct WitCommandDispatcher {
    handlers: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl WitCommandDispatcher {
    /// Create a new command dispatcher
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific command type
    pub fn register<C: WitCommand>(
        &mut self,
        command_name: &'static str,
        handler: Box<dyn WitCommandHandler<C>>,
    ) {
        self.handlers.insert(command_name, Box::new(handler));
    }

    /// Dispatch a command to its registered handler
    pub fn dispatch<C: WitCommand>(&mut self, command: &C) -> Result<C::Response, String> {
        let name = command.command_name();

        let handler = self
            .handlers
            .get_mut(name)
            .ok_or_else(|| format!("No handler registered for command: {}", name))?;

        let handler = handler
            .downcast_mut::<Box<dyn WitCommandHandler<C>>>()
            .ok_or_else(|| format!("Handler type mismatch for command: {}", name))?;

        handler.execute(command)
    }
}

impl Default for WitCommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to define a WIT command enum from interface functions
#[macro_export]
macro_rules! define_wit_commands {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $({ $($field:ident: $field_ty:ty),* $(,)? })?
            ),* $(,)?
        }

        response => $response_ty:ty
    ) => {
        $(#[$enum_meta])*
        $vis enum $enum_name {
            $(
                $(#[$variant_meta])*
                $variant $({ $($field: $field_ty),* })?,
            )*
        }

        impl $crate::wit_registry::WitCommand for $enum_name {
            type Response = $response_ty;

            fn command_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// Composable WIT interface trait
pub trait WitInterface: Send + Sync {
    /// Get the interface name
    fn interface_name(&self) -> &'static str;

    /// Register handlers with a dispatcher
    fn register_handlers(&self, dispatcher: &mut WitCommandDispatcher);
}

/// Composite WIT interface that combines multiple interfaces
pub struct CompositeWitInterface {
    interfaces: Vec<Box<dyn WitInterface>>,
}

impl CompositeWitInterface {
    /// Create a new composite interface
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }

    /// Add an interface to the composite
    pub fn add_interface(&mut self, interface: Box<dyn WitInterface>) {
        self.interfaces.push(interface);
    }

    /// Register all handlers from all interfaces
    pub fn register_all(&self, dispatcher: &mut WitCommandDispatcher) {
        for interface in &self.interfaces {
            interface.register_handlers(dispatcher);
        }
    }
}

impl Default for CompositeWitInterface {
    fn default() -> Self {
        Self::new()
    }
}
