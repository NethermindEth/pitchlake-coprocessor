# Installation Guide

This guide walks you through installing all dependencies required for Pitchlake Coprocessor development.

## Table of Contents
- [System Requirements](#system-requirements)
- [Quick Installation](#quick-installation)
- [Manual Installation](#manual-installation)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Operating System
- Linux (Ubuntu 20.04+, Debian 11+, etc.)
- macOS (11.0+)
- Windows with WSL2

### Hardware
- CPU: 4+ cores recommended (8+ for parallel proof generation)
- RAM: 16GB minimum, 32GB recommended for full proof generation
- Disk: 20GB+ free space for toolchains, build cache, and proof artifacts
- Internet connection for downloading dependencies

## Quick Installation

The fastest way to set up your development environment:

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
$HOME/.risc0/bin/rzup install

# Install StarkNet tools (optional, for contract development)
asdf plugin add scarb
asdf plugin add starknet-foundry
asdf install

# Clone repository (if not already cloned)
# git clone https://github.com/your-org/pitchlake-coprocessor.git
# cd pitchlake-coprocessor

# Build all methods
cargo build --release
```

**Estimated time:** 10-20 minutes (depending on your internet speed and CPU)

## Manual Installation

If you prefer to install dependencies manually or need to troubleshoot issues, follow these steps:

### 1. Rust Toolchain

Pitchlake Coprocessor uses Rust stable with rustfmt and clippy.

#### Installation

```bash
# Install rustup (Rust version manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source cargo environment
source $HOME/.cargo/env

# Install stable toolchain (if not default)
rustup toolchain install stable
rustup default stable

# Install required components
rustup component add rustfmt clippy
```

#### Verification

```bash
rustc --version    # Should show 1.70.0+
cargo --version
rustfmt --version
cargo clippy --version
```

### 2. RISC Zero Toolchain

RISC Zero is used for zero-knowledge proof generation. Install via `rzup`:

#### Installation

```bash
# Install rzup
curl -L https://risczero.com/install | bash

# Source cargo environment (if not already sourced)
source $HOME/.cargo/env

# Install RISC Zero toolchain
$HOME/.risc0/bin/rzup install
```

#### Verification

```bash
cargo risczero --version
```

**Troubleshooting:**
- If `rzup` is not found, ensure `$HOME/.risc0/bin` is in your PATH
- You may need to restart your terminal after installation

### 3. asdf Version Manager (Optional)

asdf is used to manage StarkNet tool versions for contract development. Skip this if you only need to generate proofs.

#### Installation

**macOS/Linux:**
```bash
# Clone asdf repository
git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.14.0

# Add to your shell profile
# For bash (~/.bashrc):
echo '. "$HOME/.asdf/asdf.sh"' >> ~/.bashrc

# For zsh (~/.zshrc):
echo '. "$HOME/.asdf/asdf.sh"' >> ~/.zshrc

# Restart your terminal or source your profile
source ~/.bashrc  # or source ~/.zshrc
```

#### Verification

```bash
asdf --version  # Should show v0.14.0
```

**Documentation:** https://asdf-vm.com/guide/getting-started.html

### 4. StarkNet Tools (Optional)

Required only for contract development and testing. Skip if you're only generating proofs.

#### Installation

```bash
# Add asdf plugins
asdf plugin add scarb
asdf plugin add starknet-foundry

# Install versions from .tool-versions file
cd /path/to/pitchlake-coprocessor
asdf install

# Set global versions
asdf global scarb 2.12.1
asdf global starknet-foundry 0.49.0
```

#### Verification

```bash
scarb --version              # Should show 2.12.1
snforge --version            # Should show 0.49.0
```

### 5. Build the Project

Build all methods to verify installation:

```bash
cd /path/to/pitchlake-coprocessor

# Build all projects in release mode
cargo build --release

# Or build individual methods:
cargo build -p mock-proof-composition --release
cargo build -p proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods --release
```

**Build time:** 5-15 minutes for full release build

## Verification

Verify your complete installation:

```bash
# Check Rust installation
rustc --version
cargo --version
rustfmt --version
cargo clippy --version

# Check RISC Zero installation
cargo risczero --version

# Check StarkNet tools (if installed)
asdf --version
scarb --version
snforge --version

# Test build
cd /path/to/pitchlake-coprocessor
cargo build --release
```

### Expected Output

```
rustc 1.70.0+
cargo 1.70.0+
rustfmt 1.7.0+
clippy 0.1.70+
cargo-risczero 2.3.2
v0.14.0
scarb 2.12.1
snforge 0.49.0
```

## Troubleshooting

### Rust/Cargo Issues

**Problem:** `cargo: command not found`
```bash
# Source cargo environment
source $HOME/.cargo/env

# Add to shell profile permanently
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

**Problem:** Compilation errors
```bash
# Update Rust to latest stable
rustup update stable

# Clean build cache
cargo clean
```

### RISC Zero Issues

**Problem:** `cargo risczero: command not found`
```bash
# Reinstall RISC Zero toolchain
$HOME/.risc0/bin/rzup install

# Verify installation directory
ls -la $HOME/.risc0/bin/
```

**Problem:** rzup installation fails
```bash
# Manual installation
cargo install cargo-risczero
cargo risczero install
```

### asdf Issues

**Problem:** `asdf: command not found`
```bash
# Verify asdf installation
ls -la ~/.asdf

# Source asdf in current shell
source ~/.asdf/asdf.sh

# Add to shell profile (if not already added)
echo '. "$HOME/.asdf/asdf.sh"' >> ~/.bashrc
```

**Problem:** Plugin installation fails
```bash
# Update asdf plugins
asdf plugin update --all

# Remove and re-add plugin
asdf plugin remove scarb
asdf plugin add scarb
asdf install
```

### Build Issues

**Problem:** Out of memory during compilation
```bash
# Reduce parallel compilation jobs
export CARGO_BUILD_JOBS=2
cargo build --release
```

**Problem:** Disk space errors
```bash
# Clean build artifacts
cargo clean
rm -rf target/

# Check disk space
df -h
```

**Problem:** Linker errors on Linux
```bash
# Install build essentials
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

**Problem:** OpenBLAS errors (if using common crate with "original" feature)
```bash
# Install OpenBLAS
# Ubuntu/Debian:
sudo apt-get install libopenblas-dev

# macOS:
brew install openblas
```

### Platform-Specific Issues

**macOS:**
- Ensure Xcode Command Line Tools are installed: `xcode-select --install`
- If using Apple Silicon (M1/M2), ensure Rosetta 2 is installed for compatibility

**Linux:**
- Install build essentials: `sudo apt-get install build-essential`
- Install pkg-config: `sudo apt-get install pkg-config`

**Windows WSL2:**
- Ensure WSL2 is properly configured
- Use Ubuntu 20.04+ for best compatibility
- Follow Linux installation instructions within WSL

## Next Steps

Once installation is complete, proceed to:
- [Quick Start Guide](quickstart.md) - Run your first proof in 5 minutes
- [Testing Guide](testing.md) - Learn how to run and write tests
- [Running Methods](../guides/running-methods.md) - Execute individual methods

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [RISC Zero Documentation](https://dev.risczero.com/)
- [StarkNet Documentation](https://docs.starknet.io/)
- [asdf Documentation](https://asdf-vm.com/)
