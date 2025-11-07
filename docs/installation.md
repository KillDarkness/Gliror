# Installation

This guide covers how to install and set up GLIROR on various platforms.

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- OpenSSL development libraries (for HTTPS support)

## Installing via Cargo (Recommended)

The easiest way to install GLIROR is using Cargo:

```bash
cargo install gliror
```

This will download, compile, and install GLIROR to your Cargo bin directory (usually `~/.cargo/bin`), which should be in your PATH.

## Installing from Source

### 1. Clone the Repository

```bash
git clone https://github.com/KillDarkness/Gliror.git
cd Gliror
```

### 2. Build the Project

```bash
cargo build --release
```

### 3. Run GLIROR

```bash
# Direct execution
cargo run --release -- [options]

# Or use the compiled binary
./target/release/gliror [options]
```

### 4. Install System-Wide from Source

```bash
cargo install --path .
```

## Installing Rust

If you don't have Rust installed:

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload your shell
source ~/.bashrc

# Verify installation
rustc --version
cargo --version
```

## Platform-Specific Instructions

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
```

### CentOS/RHEL/Fedora

```bash
sudo dnf install gcc pkgconfig openssl-devel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
```

## Verifying Installation

Test your installation:

```bash
gliror --version
```

You should see output similar to: `GLIROR 1.0.3`

## Building as a System Binary

To install GLIROR system-wide:

```bash
cargo install --path .
```

This will place the binary in your Cargo bin directory (usually `~/.cargo/bin`), which should be in your PATH.