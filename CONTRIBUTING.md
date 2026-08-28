# Contributing to @valen-run/heic

Thank you for your interest in contributing to `@valen-run/heic`!

## Code of Conduct

Please be respectful and constructive in all interactions within issues, pull requests, and discussions.

## Development Setup

See [Development Guide](docs/development.md) for full instructions on setting up your local Rust and WebAssembly toolchain.

### Quick Commands

```bash
# Run Rust tests
cargo test --workspace

# Check formatting and clippy
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Run TypeScript checks
pnpm run typecheck
```

## Pull Request Guidelines

1. Ensure all CI checks pass.
2. Maintain crate boundaries and do not introduce unintended dependencies.
3. Write unit and regression tests for bug fixes or new features.
4. Keep the public API backwards-compatible and well-documented.
