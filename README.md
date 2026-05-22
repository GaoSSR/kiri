<p align="center">
  <img src="assets/kiri-logo.png" alt="Kiri logo" width="128" />
</p>

<h1 align="center">Kiri</h1>

<p align="center">
  <strong>See through the fog of local development ports.</strong>
</p>

<p align="center">
  <img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue" />
  <img alt="macOS supported first" src="https://img.shields.io/badge/macOS-supported-brightgreen" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="Command: ports" src="https://img.shields.io/badge/command-ports-8A2BE2" />
</p>

Kiri shows the local dev servers, databases, containers, processes, and logs running behind your ports. The project is Kiri; the command you type is `ports`.

## Install

Kiri is not published yet. These are the intended installation channels for the first public release.

```bash
# macOS / Linux shell
curl -fsSL https://raw.githubusercontent.com/gaossr/kiri/main/scripts/install.sh | bash

# npm
npm install -g @gaossr/kiri

# Homebrew
brew install gaossr/tap/kiri
```

Windows PowerShell entry is planned, but Windows runtime support is not available until the Windows collector and release artifact ship:

```powershell
irm https://raw.githubusercontent.com/gaossr/kiri/main/scripts/install.ps1 | iex
```

After installation, run:

```bash
ports
```

Kiri currently supports macOS first. Linux and Windows distribution will be added after their platform collectors and release artifacts are implemented.

The command `ports` can conflict with an older npm install of `port-whisperer` or another local tool. If your system already has a `ports` command, the one that runs depends on `PATH` order. Use `which ports` to check what will execute.

## Quick Start

```bash
ports
ports --all
ports 3000
ports logs 3000
ports kill 3000
```

`ports <port>` only displays details. It does not ask whether to kill the process. Killing must be explicit through `ports kill <target>`.

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
ports --color never         # plain output for scripts
```

If multiple log files are found, Kiri asks you to choose in a real terminal. In non-interactive output, it selects the best match by deterministic priority.

## Safety

`ports kill` resolves targets before sending signals:

- A matching listening port is preferred.
- Port ranges are expanded one port at a time.
- Empty ports inside a range are counted but do not stop the whole command.
- If a numeric target is not a listening port, Kiri checks whether it is an existing PID.
- Invalid targets, reversed ranges, oversized ranges, and ports outside `1-65535` return clear errors.

`ports clean` lists orphaned/zombie developer-process candidates and asks for confirmation before sending any signal.

Details mode never performs kill. Use `ports kill <target>` when you intend to terminate a process.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | Supported first and currently the real implementation target |
| Linux | Module structure exists; collector and release artifacts are still TODO |
| Windows | Module structure exists; collector and release artifacts are still TODO; PowerShell entry currently reports unsupported |

On macOS, Kiri uses:

- `lsof -iTCP -sTCP:LISTEN -P -n` for listening TCP ports.
- `ps -p <pidList> -o pid=,ppid=,stat=,rss=,lstart=,command=` for process details.
- `ps -eo pid=,pcpu=,pmem=,rss=,lstart=,command=` for `ports ps`.
- `lsof -a -d cwd -p <pidList>` for current working directories.
- `lsof -p <pid>` plus `tail` and macOS `log show` / `log stream` for `ports logs`.
- `docker ps --format "{{.Ports}}\t{{.Names}}\t{{.Image}}"` for Docker host-port mappings when Docker is available.

Docker is optional. If Docker is unavailable or no containers are running, Kiri silently continues without Docker mappings.

## Differences From port-whisperer

Kiri is a Rust migration and refactor of the JavaScript project `port-whisperer`. The source project is the behavior baseline, but Kiri intentionally changes a few product boundaries:

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
