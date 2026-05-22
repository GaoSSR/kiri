<h4 align="right"><a href="./README.md">简体中文</a> | <strong><a href="./README.en.md">English</a></strong></h4>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/kiri-logo.png">
  <source media="(prefers-color-scheme: light)" srcset="assets/kiri-logo.png">
  <img align="right" src="assets/kiri-logo.png" alt="Kiri logo" width="360">
</picture>

<p>
  <a href="https://github.com/GaoSSR/kiri">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="assets/kiri-wordmark-dark.svg">
      <source media="(prefers-color-scheme: light)" srcset="assets/kiri-wordmark-light.svg">
      <img src="assets/kiri-wordmark-light.svg" alt="Kiri" height="64" style="width: auto;">
    </picture>
  </a>
  <br />
</p>

### High-performance local development port management CLI, powered by Rust

Kiri is a high-performance CLI for managing local development ports, powered by Rust. It helps you quickly see which local services are running, which ports they use, and handle the process behind a port when needed.

<p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="macOS supported" src="https://img.shields.io/badge/macOS-supported-brightgreen" />
  <img alt="Command: ports" src="https://img.shields.io/badge/command-ports-8A2BE2" />
  <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
</p>

<br clear="right" />

---

## Core Usage

- **View local development services and their ports:** `ports`
- **Quickly kill the process / PID behind a port:** `ports kill <port>`
- **View logs for the process listening on a port:** `ports logs <port|pid>`
- **View all ports:** `ports --all`

## Install

Kiri is preparing its first public release. The commands below describe the planned release channels after GitHub Release artifacts, npm packaging, and the Homebrew tap are published.

```bash
# Install script
curl -fsSL https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.sh | bash

# npm
npm install -g @gaossr/kiri

# Homebrew
brew install gaossr/tap/kiri
```

Windows PowerShell is planned, but Windows runtime support is not available until the Windows collector and release artifacts ship:

```powershell
irm https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.ps1 | iex
```

Current support is macOS first. Linux and Windows have distribution plans, but their complete platform collectors and release artifacts are not ready yet.

## Commands

```bash
ports                       # view local development services and their ports
ports --all                 # show all listening ports
ports <port>                # show details for one port
ports ps                    # show developer-related running processes
ports ps --all              # show all processes
ports logs <port|pid>       # view logs for the process listening on a port
ports logs 3000 --lines 10  # show last 10 lines
ports logs 3000 --err       # stderr only
ports logs 3000 --follow    # follow logs
ports clean                 # ask before cleaning orphaned/zombie dev processes
ports watch                 # stream port start/stop events
ports kill 3000             # quickly kill the process / PID behind a port
ports kill 3000-3010        # terminate listeners across a range
ports kill --force 3000     # use SIGKILL instead of SIGTERM
```

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | Current primary supported platform |
| Linux | Planned; collector and release artifacts are not complete |
| Windows | Planned; PowerShell installer currently reports unsupported until collector and artifacts ship |

On macOS, Kiri uses `lsof`, `ps`, `tail`, macOS `log` commands, and optional Docker metadata. Docker is optional; if Docker is unavailable or no containers are running, Kiri continues without Docker mappings.

## Development

For maintainers and contributors working from source:

```bash
cargo fmt
cargo test
cargo run --bin ports
cargo run --bin ports -- --all
cargo run --bin ports -- ps
```
