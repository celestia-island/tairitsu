# Contributing to Tairitsu

Thank you for your interest in contributing to Tairitsu!

## Getting Started

### Prerequisites

- Rust 1.76+ (stable)
- Node.js 18+ (for web examples)
- Python 3.11+ (for WebIDL scripts)
- Just (command runner)

### Setup

```bash
# Clone the repository
git clone https://github.com/tairitsu/tairitsu.git
cd tairitsu

# Install dependencies
just init

# Run tests
just test

# Build examples
just build-examples
```

## Development Workflow

### 1. Fork and Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Follow our coding conventions:
- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Write tests for new features
- Update documentation

### 3. Commit

Use conventional commit format:

```
feat: add new feature
fix: resolve bug
docs: update documentation
test: add tests
perf: performance improvement
refactor: code refactoring
```

### 4. Pull Request

- Describe your changes
- Link related issues
- Ensure CI passes
- Request review from maintainers

## Coding Conventions

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` default settings
- Fix `clippy` warnings
- Document public APIs with rustdoc

```rust
/// Brief description.
///
/// Longer explanation with examples.
///
/// # Examples
///
/// ```
/// use tairitsu::function;
/// let result = function();
/// ```
pub fn function() -> Result<()> {
    // ...
}
```

### WIT Definitions

- Follow WIT naming conventions
- Document all interfaces and types
- Include examples in comments

### Tests

- Unit tests in `src/*.rs` files
- Integration tests in `tests/` directories
- E2E tests in `packages/e2e/tests/`

## Project Structure

```
tairitsu/
├── packages/           # Core packages
│   ├── vdom/          # Virtual DOM
│   ├── web/           # Platform implementations
│   ├── hooks/         # State management
│   ├── macros/        # Procedural macros
│   └── ...
├── examples/          # Example applications
├── docs/             # Documentation
├── scripts/          # Build and utility scripts
└── packages/browser-worlds/  # WIT definitions
```

## Testing

### Running Tests

```bash
# All tests
just test

# Unit tests only
cargo test

# E2E tests
just test-e2e

# Specific package
cargo test -p tairitsu-vdom
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = feature();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        // ...
    }
}
```

## Documentation

### Rust Docs

```bash
# Build and open docs
cargo doc --open
```

### Website Docs

Documentation in `docs/` follows:
- Markdown format
- Multi-language support
- Code examples with syntax highlighting
- Diagrams in Mermaid

## Release Process

Releases are managed by maintainers:

1. Bump version: `just version bump [major|minor|patch]`
2. Update CHANGELOG.md
3. Create git tag
4. Publish to crates.io
5. Build release binaries
6. Update website

## Community

### Channels

- **GitHub**: Issues and PRs
- **Discord**: Chat with contributors
- **Discussions**: Q&A and ideas

### Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

## Getting Help

- Check [documentation](docs/)
- Search [existing issues](https://github.com/tairitsu/tairitsu/issues)
- Ask in [Discussions](https://github.com/tairitsu/tairitsu/discussions)
- Join our [Discord](https://discord.gg/tairitsu)

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Website contributors page

Thank you for contributing! 🎉
