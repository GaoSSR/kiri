# DevPorts

DevPorts is a Rust CLI for inspecting local listening ports during development. It shows the process, PID, project, detected framework, uptime, and status behind each listening port, with Docker host-port mapping and readable terminal output.

The primary command is `devports`. The short command `ports` is also installed and runs the same logic.

## Why This Exists

DevPorts is a Rust migration and refactor of the existing JavaScript project `port-whisperer`. The source project is the behavior baseline, not just inspiration: DevPorts preserves the core CLI workflows around developer-port discovery, `--all`, port details, Docker mapping, framework detection, and safe process termination.

This first Rust version intentionally changes the delivery shape:

- CLI only for now.
- macOS is the first fully implemented platform.
- Display is redesigned as a width-aware, high-contrast, color-controlled CLI table.
- Port details do not ask interactively whether to kill a process; killing must be explicit.

## Current Features

- `devports` and `ports` commands.
- Default view shows developer-relevant listening ports only.
- `--all` shows every listening TCP port found by the platform collector.
- `devports <port>` shows a detail page for one listening port.
- `devports kill <target...>` terminates listeners by port, range, or PID fallback.
- `-f` / `--force` uses `SIGKILL`; default kill uses `SIGTERM`.
- Docker host-port mapping from running containers.
- Docker image identification for common services such as PostgreSQL, Redis, MySQL, MongoDB, nginx, LocalStack, RabbitMQ, Kafka, Elasticsearch, and MinIO.
- Non-Docker framework detection from command lines, process names, `package.json`, and common project files.
- High-contrast, width-aware terminal table output.
- `--color auto|always|never` for terminal and scripting use cases.

## Install

Release packaging is planned. After DevPorts is published, install it with npm:

```bash
npm install -g devports
```

Or with Homebrew:

```bash
brew install devports
```

If DevPorts is distributed through a Homebrew tap instead of Homebrew core, use the tap name:

```bash
brew install <tap-owner>/tap/devports
```

DevPorts has not been published to npm or Homebrew yet. These are the intended user-facing install commands for the first public release, not a claim that the package or formula is already available.

After installation, verify both binaries are available:

```bash
which devports
which ports
devports --color never
ports --color never
```

If both npm and Homebrew versions are installed, `PATH` order decides which `devports` or `ports` binary runs.

The short command `ports` can also conflict with an older npm install of `port-whisperer`. If your system already has a `ports` command, the one that runs depends on `PATH` order. Use `which ports` to check what will execute. The primary `devports` command avoids that common name collision.

## Usage

Show developer-relevant listening ports:

```bash
devports
ports
```

Show all listening ports:

```bash
devports --all
ports --all
```

Show details for one port:

```bash
devports 3000
ports 3000
```

`devports <port>` only displays details. It does not ask whether to kill the process.

Kill must be explicit:

```bash
devports kill 3000
ports kill 3000
```

Kill multiple targets:

```bash
devports kill 3000 5173 8080
```

Kill a port range:

```bash
devports kill 3000-3010
```

Force kill:

```bash
devports kill --force 3000
devports kill -f 3000
```

If a numeric target is not a listening port and the PID exists, DevPorts falls back to killing that PID.

## Color Modes

DevPorts defaults to `--color auto`.

```bash
devports --color auto
devports --color always
devports --color never
```

- `auto`: color is enabled for a real terminal and disabled for non-TTY output.
- `always`: always emits ANSI color codes, useful for visual checks and snapshot-style verification.
- `never`: emits plain text with no ANSI color codes, useful for scripts, logs, and pipes.

The table avoids forced white primary text so it remains readable on both light and dark terminal backgrounds.

## Kill Safety

`devports kill` resolves targets before sending signals:

- A matching listening port is preferred.
- Port ranges are expanded one port at a time.
- Empty ports inside a range are counted but do not stop the whole command.
- If a numeric target is not a listening port, DevPorts checks whether it is an existing PID.
- Invalid targets, reversed ranges, oversized ranges, and ports outside `1-65535` return clear errors.

Details mode never performs kill. Use `devports kill <target>` when you intend to terminate a process.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | Implemented first and currently the supported platform |
| Linux | Module structure exists; real collection is still TODO |
| Windows | Module structure exists; real collection is still TODO |

On macOS, DevPorts uses:

- `lsof -iTCP -sTCP:LISTEN -P -n` for listening TCP ports.
- `ps -p <pidList> -o pid=,ppid=,stat=,rss=,lstart=,command=` for process details.
- `lsof -a -d cwd -p <pidList>` for current working directories.
- `docker ps --format "{{.Ports}}\t{{.Names}}\t{{.Image}}"` for Docker host-port mappings when Docker is available.

Docker is optional. If Docker is unavailable or no containers are running, DevPorts silently continues without Docker mappings.

## Not Supported Yet

This first CLI release does not implement:

- TUI.
- Desktop app.
- Tauri wrapper.
- Complete Linux or Windows support.
- `ps`, `logs`, `clean`, or `watch` commands from `port-whisperer`.
- Docker logs or process tree views.
- Published npm package or Homebrew formula. Those release packages are planned but not available yet.

## Differences From port-whisperer

DevPorts keeps the core behavior but changes the implementation and first-release boundary:

- Rust CLI instead of JavaScript/npm package.
- `devports` is the primary command; `ports` is kept as the short alias.
- macOS is implemented first; Linux and Windows are not claimed as complete.
- Port detail view is read-only and never prompts to kill.
- Kill is only performed through explicit `devports kill <target...>`.
- Color output is controlled with `--color auto|always|never`.
- Terminal rendering is width-aware and avoids low-contrast primary data.
- `ps`, `logs`, `clean`, and `watch` are intentionally left out of this version.

## Development Checks

For maintainers and contributors working from source:

```bash
cargo fmt
cargo test
cargo run --bin devports -- --color never
cargo run --bin devports -- --all --color never
cargo run --bin ports -- --color never
```
