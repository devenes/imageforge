# Contributing to ImageForge

Thank you for your interest in contributing to ImageForge! We welcome bug reports, feature suggestions, documentation enhancements, and pull requests.

ImageForge is a local-first, privacy-focused macOS application built with **Tauri 2.x**, **Svelte 5**, and **Rust**.

---

## Code of Conduct

We are dedicated to providing a friendly, safe, and welcoming environment for all contributors, regardless of experience level, background, or identity. Please treat fellow contributors with respect, empathy, and constructive feedback.

---

## Development Prerequisites

Before contributing code, ensure you have installed:

- **macOS 11 (Big Sur)** or newer (Apple Silicon or Intel)
- **Xcode Command Line Tools**: `xcode-select --install`
- **Homebrew**: `https://brew.sh`
- **Rust toolchain** (1.98.1 or newer, pinned in `rust-toolchain.toml`):
  ```bash
  rustup toolchain install 1.98.1
  rustup component add clippy rustfmt
  ```
- **Node.js**: v20 or newer
- **pnpm**: v9 or newer (`npm install -g pnpm`)

### System Libraries

Install the required native image codec libraries via Homebrew:

```bash
brew install jpeg-turbo webp libheif cmake nasm pkgconf
```

---

## Local Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/devenes/imageforge.git
   cd imageforge
   ```

2. **Install frontend dependencies:**
   ```bash
   pnpm install
   ```

3. **Configure environment:**
   Ensure `pkg-config` can locate Homebrew libraries:
   ```bash
   export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
   ```
   *(Tip: Add this line to your `~/.zshrc` or shell profile).*

4. **Run in development mode:**
   ```bash
   pnpm tauri dev
   ```

---

## Quality & Testing Guidelines

All pull requests must pass our automated quality gates before merge. You can run all checks locally:

### 1. Frontend Checks
```bash
# Type check Svelte and TypeScript files
pnpm check

# Verify production build succeeds
pnpm build
```

### 2. Rust Backend Checks
```bash
# Verify formatting conforms to style guidelines
cargo fmt --all -- --check

# Run Clippy lints with zero tolerance for warnings
cargo clippy --all-targets -- -D warnings

# Run all unit and integration tests
cargo test --all-targets
```

> [!NOTE]
> Integration tests automatically generate synthetic fixtures for all 4 supported image formats and test them through the full compression pipeline.

---

## Core Principles & Invariants

When submitting changes, ensure the following core architectural invariants are preserved:

1. **Dimension Invariant**: Output images must never have altered pixel dimensions.
2. **Non-Destructive Operations**: Source images must never be modified or overwritten. Outputs use collision-safe naming (`-compressed`, `-compressed-1`, etc.).
3. **Local-First Privacy**: Never introduce network calls, analytics, telemetry, or remote telemetry tracking.
4. **Clean Error Handling**: Always surface friendly, readable error messages in the UI (`AppError::user_friendly_message()`) and preserve raw technical details in tracing logs.
5. **Atomic File Writes**: All file outputs must be written to temporary scratch files (`.tmp_*`) and committed via atomic filesystem rename.

---

## Pull Request Workflow

1. Create a descriptive feature branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. Commit your changes with clear, semantic commit messages (e.g. `feat:`, `fix:`, `docs:`, `test:`, `refactor:`).
3. Ensure all tests, linter checks, and builds pass locally.
4. Push your branch to GitHub and open a Pull Request using the provided PR template.
5. Provide context, screenshots, or screen recordings where relevant.
