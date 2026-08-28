# Development Guide

This guide covers setting up your local environment for developing and testing `@valen-run/heic`.

## Prerequisites

- **Rust** (1.75 or later)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-unknown-unknown
  ```
- **Node.js** (v18+) & **pnpm** (v9+)
- **wasm-pack** (optional, for compiling WASM binaries locally)
  ```bash
  cargo install wasm-pack
  ```

---

## Local Build & Test Commands

### 1. Rust Workspace
```bash
# Check code without building
cargo check --workspace --all-targets

# Run tests across all crates
cargo test --workspace

# Lint with Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check
```

### 2. TypeScript Package
```bash
# Install dependencies
pnpm install

# Typecheck TypeScript files
pnpm run typecheck

# Run TypeScript tests
pnpm run test:js
```

### 3. Packaging Verification
```bash
# Verify npm package output contents
cd packages/heic
pnpm pack --dry-run
```

---

## Questions & Support

For developer inquiries, questions, or contribution feedback, please contact [**hi@valen.run**](mailto:hi@valen.run).

