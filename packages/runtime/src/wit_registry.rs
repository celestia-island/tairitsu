//! WIT interface registration system
//!
//! This module provides a trait-based system for registering and composing multiple
//! WIT interface implementations without runtime serialization.

use std::any::Any;
use std::collections::HashMap;

/// Trait for WIT command types
///
/// Users need to implement this trait for their command types
pub trait WitCommand: Send + Sync + 'static {
    /// Response type for the command
    type Response: Send + Sync + 'static;

    /// Get command name for routing
    fn command_name(&self) -> &'static str;

    /// Convert to Any to support dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

/// Trait for WIT command handlers
///
/// Users need to implement this trait for each command type
pub trait WitCommandHandler<C: WitCommand>: Send + Sync {
    /// Execute command and return response
    fn execute(&mut self, command: &C) -> Result<C::Response, String>;
}

/// Dynamic command dispatcher using trait objects
pub struct WitCommandDispatcher {
    handlers: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl WitCommandDispatcher {
    /// Create new command dispatcher
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register handler for specific command type
    pub fn register<C: WitCommand>(
        &mut self,
        command_name: &'static str,
        handler: Box<dyn WitCommandHandler<C>>,
    ) {
        self.handlers.insert(command_name, Box::new(handler));
    }

    /// Dispatch command to its registered handler
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

/// Composable WIT interface trait
///
/// Users can implement this trait for each WIT interface
pub trait WitInterface: Send + Sync {
    /// Get interface name
    fn interface_name(&self) -> &'static str;

    /// Register handlers with dispatcher
    fn register_handlers(&self, dispatcher: &mut WitCommandDispatcher);
}

/// Compose multiple WIT interfaces
pub struct CompositeWitInterface {
    interfaces: Vec<Box<dyn WitInterface>>,
}

impl CompositeWitInterface {
    /// Create new composite interface
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }

    /// Add interface to composite
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
