//! WIT binding helper macros
//!
//! This module provides macros that simplify WIT interface definitions, reducing user code.

/// Helper macro to implement `WitInterface` trait for a single WIT interface
///
/// # Example
/// ```ignore
/// use tairitsu::wit_registry::{WitInterface, WitCommandDispatcher};
/// use tairitsu::impl_wit_interface;
///
/// struct FilesystemInterface;
///
/// impl_wit_interface!(FilesystemInterface, "filesystem", {
///     fn register_handlers(&self, dispatcher: &mut WitCommandDispatcher) {
///         dispatcher.register("fs-read", Box::new(handler));
///     }
/// });
/// ```
#[macro_export]
macro_rules! impl_wit_interface {
    (
        $impl_type:ty,
        $name:expr,
        fn register_handlers(&self, $dispatcher:ident: &mut $dispatcher_ty:ty) $block:block
    ) => {
        impl $crate::wit_registry::WitInterface for $impl_type {
            fn interface_name(&self) -> &'static str {
                $name
            }

            fn register_handlers(&self, $dispatcher: &mut $dispatcher_ty) $block
        }
    };
}

/// Macro to quickly create simple command handlers
///
/// # Example
/// ```ignore
/// use tairitsu::simple_handler;
///
/// let handler = simple_handler!(MyCommand, |cmd: &MyCommand| {
///     match cmd {
///         MyCommand::Foo => Ok(()),
///     }
/// });
/// ```
#[macro_export]
macro_rules! simple_handler {
    ($command_type:ty, $handler:expr) => {
        {
            struct Handler;

            impl $crate::wit_registry::WitCommandHandler<$command_type> for Handler {
                fn execute(&mut self, command: &$command_type) -> Result<
                    <$command_type as $crate::wit_registry::WitCommand>::Response,
                    String,
                > {
                    $handler(command)
                }
            }

            Box::new(Handler) as Box<dyn $crate::wit_registry::WitCommandHandler<$command_type>>
        }
    };
}

/// Macro to quickly create stateful command handlers
///
/// # Example
/// ```ignore
/// use tairitsu::stateful_handler;
///
/// struct MyState { counter: i32 }
///
/// let handler = stateful_handler!(MyState, MyCommand, |state, cmd| {
///     match cmd {
///         MyCommand::Increment => {
///             state.counter += 1;
///             Ok(state.counter)
///         }
///     }
/// });
/// ```
#[macro_export]
macro_rules! stateful_handler {
    ($state_type:ty, $command_type:ty, $handler:expr) => {
        {
            struct Handler(std::sync::Arc<std::sync::Mutex<$state_type>>);

            impl $crate::wit_registry::WitCommandHandler<$command_type> for Handler {
                fn execute(&mut self, command: &$command_type) -> Result<
                    <$command_type as $crate::wit_registry::WitCommand>::Response,
                    String,
                > {
                    let mut state = self.0.lock().unwrap();
                    $handler(&mut state, command)
                }
            }

            let state = std::sync::Arc::new(std::sync::Mutex::new(state));
            Box::new(Handler(state)) as Box<dyn $crate::wit_registry::WitCommandHandler<$command_type>>
        }
    };
}
