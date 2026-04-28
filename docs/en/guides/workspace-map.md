# Workspace Map

## Top-level

```text
packages/   Core Rust crates
docs/       Multilingual docs
examples/   Integration examples
scripts/    WIT and WebIDL pipeline scripts
tests/      Test assets
```

## Key crates
- runtime: container lifecycle and component invocation
- macros: rsx!, wit_interface!, wit_world!
- vdom: platform-neutral virtual DOM and events
- web: web and wit-bindings backends
- browser-worlds: WIT worlds
- browser-wit-resolver: WIT fetch/cache/resolve
- packager: CLI and wit subcommands
