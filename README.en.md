<h1 align="center">
  <img src="assets/kiri-logo.png" alt="Kiri" width="420" />
  <br />
  Kiri
</h1>

<p align="center">
  <strong>Manage local development ports: inspect processes, view matching logs, stop port owners.</strong>
</p>

<p align="center">
  <a href="./README.md">简体中文</a> · <a href="./README.en.md">English</a>
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="macOS supported" src="https://img.shields.io/badge/macOS-supported-brightgreen" />
  <img alt="Command: ports" src="https://img.shields.io/badge/command-ports-8A2BE2" />
  <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
</p>

Kiri is a local development port management CLI. It helps you see which dev services are running, resolve a port to its process and project, view logs for the process listening on that port, and stop the process that owns the port.

The product is **Kiri**. The command you type is:

```bash
ports
```

In daily development, the real question is rarely just "which port is open?" It is "who owns this port, what is it logging, and can I stop it safely?" Kiri puts those actions behind one `ports` command. The default `ports` view stays focused on developer-relevant listeners, while `ports --all` is available when you need the full system view.

## Common Workflows

```bash
ports            # see how many local dev services are running
ports logs 3000  # view logs for the process listening on port 3000
ports kill 3000  # stop the process occupying port 3000
ports ps         # see which local dev processes are running
```

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

## Quick Start

```bash
ports
ports logs 3000
ports kill 3000
ports ps
ports --all
ports 3000
```

## Commands

```bash
ports                       # show developer-relevant listening ports
ports --all                 # show all listening ports
ports <port>                # show details for one port
ports ps                    # show developer-related running processes
ports ps --all              # show all processes
ports logs <port|pid>       # show logs for a resolved process
ports logs 3000 --lines 10  # show last 10 lines
ports logs 3000 --err       # stderr only
ports logs 3000 --follow    # follow logs
ports clean                 # ask before cleaning orphaned/zombie dev processes
ports watch                 # stream port start/stop events
ports kill 3000             # terminate listener on a port
ports kill 3000-3010        # terminate listeners across a range
ports kill --force 3000     # use SIGKILL instead of SIGTERM
```

## Why Kiri?

- **See local development at a glance:** `ports` lists developer-relevant listeners with process, project, framework, uptime, and status.
- **View logs for the process listening on a port:** `ports logs <port|pid>` resolves likely log files for the owning process and supports interactive selection in a real terminal.
- **Quickly kill the process behind a port:** `ports kill <port>` resolves the listener first, then stops the process occupying that port without making you look up the PID manually.
- **View the process/PID behind a port:** Kiri maps ports to processes, PIDs, projects, frameworks, and Docker container metadata.
- **Keep the full view available:** `ports` stays focused on development work, while `ports --all` shows every listening port.

## Safety

`ports kill` resolves targets before sending signals:

- A matching listening port is preferred.
- Port ranges are expanded one port at a time.
- Empty ports inside a range are counted but do not stop the whole command.
- If a numeric target is not a listening port, Kiri checks whether it is an existing PID.
- Invalid targets, reversed ranges, oversized ranges, and ports outside `1-65535` return clear errors.

`ports clean` lists orphaned or zombie developer-process candidates and asks for confirmation before sending any signal.

Details mode never performs kill. Use `ports kill <target>` when you intend to terminate a process.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | Current primary supported platform |
| Linux | Planned; collector and release artifacts are not complete |
| Windows | Planned; PowerShell installer currently reports unsupported until collector and artifacts ship |

On macOS, Kiri uses `lsof`, `ps`, `tail`, macOS `log` commands, and optional Docker metadata. Docker is optional; if Docker is unavailable or no containers are running, Kiri continues without Docker mappings.

## Acknowledgements

Kiri was originally inspired by port-whisperer.

## Development

For maintainers and contributors working from source:

```bash
cargo fmt
cargo test
cargo run --bin ports
cargo run --bin ports -- --all
cargo run --bin ports -- ps
```
