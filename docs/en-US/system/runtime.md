# Runtime and Container Model

The runtime crate provides:
- Image: wasm component binary abstraction
- Container: executable instance with store/linker/state
- ContainerBuilder: host linker and guest initializer composition

Two invocation paths are supported:
- Typed compile-time bindings
- Dynamic invocation (RON and canonical binary)
