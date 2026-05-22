# Kiri

Kiri is a small Rust CLI that helps you see through the fog of local development ports. The public command is `ports`.

Kiri shows the process, PID, project, detected framework, uptime, and status behind each listening port. It also supports Docker host-port mapping, safe explicit kill, process listing, log lookup, clean, watch, and readable color-controlled terminal output.

## Why This Exists

Kiri is a Rust migration and refactor of the existing JavaScript project `port-whisperer`. The source project is the behavior baseline: Kiri preserves the core CLI workflows around developer-port discovery, `--all`, port details, Docker mapping, framework detection, logs, clean, watch, and safe process termination.

This Rust version intentionally changes a few product decisions:

- The public command is `ports`.
- CLI only for now.
- macOS is the first fully implemented platform.
- Display is redesigned as a width-aware, high-contrast, color-controlled CLI table.
- Port details do not ask interactively whether to kill a process; killing must be explicit.

## Current Features

- `ports` command.
- Default view shows developer-relevant listening ports only.
- `--all` shows every listening TCP port found by the platform collector.
- `ports <port>` shows a detail page for one listening port.
- `ports kill <target...>` terminates listeners by port, range, or PID fallback.
- `ports ps` shows developer-related running processes.
- `ports logs <port|pid>` shows process log output when log files or system logs are available.
- `ports clean` lists orphaned/zombie developer processes and asks before killing.
- `ports watch` streams port start/stop events.
- `-f` / `--force` uses `SIGKILL`; default kill uses `SIGTERM`.
- Docker host-port mapping from running containers.
- Docker image identification for common services such as PostgreSQL, Redis, MySQL, MongoDB, nginx, LocalStack, RabbitMQ, Kafka, Elasticsearch, and MinIO.
- Non-Docker framework detection from command lines, process names, `package.json`, and common project files.
- High-contrast, width-aware terminal table output.
- `--color auto|always|never` for terminal and scripting use cases.

## Install

Release packaging is planned. After Kiri is published, install it with npm:

```bash
npm install -g kiri
```

Or with Homebrew:

```bash
brew install kiri
```

If Kiri is distributed through a Homebrew tap instead of Homebrew core, use the tap name:

```bash
brew install <tap-owner>/tap/kiri
```

Kiri has not been published to npm or Homebrew yet. These are the intended user-facing install commands for the first public release, not a claim that the package or formula is already available.

After installation, verify the command is available:

```bash
which ports
ports --color never
```

The command `ports` can conflict with an older npm install of `port-whisperer` or another local tool. If your system already has a `ports` command, the one that runs depends on `PATH` order. Use `which ports` to check what will execute.

## Usage

Show developer-relevant listening ports:

```bash
ports
```

Show all listening ports:

```bash
ports --all
```

Show details for one port:

```bash
ports 3000
```

`ports <port>` only displays details. It does not ask whether to kill the process.

Show developer-related running processes:

```bash
ports ps
ports ps --all
```

Show process logs by port or PID:

```bash
ports logs 3000
ports logs 3000 --lines 10
ports logs 3000 --lines=10
ports logs 3000 --err
ports logs 3000 --follow
```

If multiple log files are found, Kiri asks you to choose in a real terminal. In non-interactive output, it selects the best match by deterministic priority.

Clean orphaned/zombie developer processes:

```bash
ports clean
```

`ports clean` lists candidates and asks for confirmation before sending any signal.

Watch port changes:

```bash
ports watch
```

Kill must be explicit:

```bash
ports kill 3000
```

Kill multiple targets:

```bash
ports kill 3000 5173 8080
```

Kill a port range:

```bash
ports kill 3000-3010
```

Force kill:

```bash
ports kill --force 3000
ports kill -f 3000
```

If a numeric target is not a listening port and the PID exists, Kiri falls back to killing that PID.

## Color Modes

Kiri defaults to `--color auto`.

```bash
ports --color auto
ports --color always
ports --color never
```

- `auto`: color is enabled for a real terminal and disabled for non-TTY output.
- `always`: always emits ANSI color codes, useful for visual checks and snapshot-style verification.
- `never`: emits plain text with no ANSI color codes, useful for scripts, logs, and pipes.

The table avoids forced white primary text so it remains readable on both light and dark terminal backgrounds.

## Kill Safety

`ports kill` resolves targets before sending signals:

- A matching listening port is preferred.
- Port ranges are expanded one port at a time.
- Empty ports inside a range are counted but do not stop the whole command.
- If a numeric target is not a listening port, Kiri checks whether it is an existing PID.
- Invalid targets, reversed ranges, oversized ranges, and ports outside `1-65535` return clear errors.

Details mode never performs kill. Use `ports kill <target>` when you intend to terminate a process.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | Implemented first and currently the supported platform |
| Linux | Module structure exists; real collection is still TODO |
| Windows | Module structure exists; real collection is still TODO |

On macOS, Kiri uses:

- `lsof -iTCP -sTCP:LISTEN -P -n` for listening TCP ports.
- `ps -p <pidList> -o pid=,ppid=,stat=,rss=,lstart=,command=` for process details.
- `ps -eo pid=,pcpu=,pmem=,rss=,lstart=,command=` for `ports ps`.
- `lsof -a -d cwd -p <pidList>` for current working directories.
- `lsof -p <pid>` plus `tail` and macOS `log show` / `log stream` for `ports logs`.
- `docker ps --format "{{.Ports}}\t{{.Names}}\t{{.Image}}"` for Docker host-port mappings when Docker is available.

Docker is optional. If Docker is unavailable or no containers are running, Kiri silently continues without Docker mappings.

## Not Supported Yet

This first CLI release does not implement:

- TUI.
- Desktop app.
- Tauri wrapper.
- Complete Linux or Windows support.
- Docker logs or process tree views.
- Published npm package or Homebrew formula. Those release packages are planned but not available yet.
- Secondary command aliases; the public command is `ports`.

## Differences From port-whisperer

Kiri keeps the source project's useful behavior but changes implementation and product boundaries:

- Rust CLI instead of JavaScript runtime implementation.
- Public command is `ports`.
- macOS is implemented first; Linux and Windows are not claimed as complete.
- Port detail view is read-only and never prompts to kill.
- Kill is only performed through explicit `ports kill <target...>`.
- `logs` supports interactive multi-file selection in a real terminal and deterministic selection in non-interactive output.
- Color output is controlled with `--color auto|always|never`.
- Terminal rendering is width-aware and avoids low-contrast primary data.

## Development Checks

For maintainers and contributors working from source:

```bash
cargo fmt
cargo test
cargo run --bin ports -- --color never
cargo run --bin ports -- --all --color never
cargo run --bin ports -- ps --color never
```
