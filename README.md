<h1 align="center">
  <img src="assets/kiri-logo.png" alt="Kiri" width="360" />
  <br />
  Kiri
</h1>

<p align="center">
  <strong>See through the fog of local development ports.</strong>
</p>

<p align="center">
  <a href="#english">English</a> · <a href="#简体中文">简体中文</a>
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="macOS supported" src="https://img.shields.io/badge/macOS-supported-brightgreen" />
  <img alt="Command: ports" src="https://img.shields.io/badge/command-ports-8A2BE2" />
  <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
</p>

## English

### What is Kiri?

Kiri is a local development port inspector. It shows the dev servers, databases,
containers, processes, and logs running behind your listening ports.

The product is **Kiri**. The command you type is:

```bash
ports
```

Kiri is built for the moment when a port is occupied, a dev server is still
running, or a container is exposing something you forgot about. It keeps the
default view focused on developer-relevant listeners, while `ports --all` stays
available when you need the full system view.

### Install

Kiri is preparing its first public release. The commands below describe the
planned release channels after GitHub Release artifacts, npm packaging, and the
Homebrew tap are published.

```bash
# Install script
curl -fsSL https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.sh | bash

# npm
npm install -g @gaossr/kiri

# Homebrew
brew install gaossr/tap/kiri
```

Windows PowerShell is planned, but Windows runtime support is not available
until the Windows collector and release artifacts ship:

```powershell
irm https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.ps1 | iex
```

Current support is macOS first. Linux and Windows have distribution plans, but
their complete platform collectors and release artifacts are not ready yet.

### Quick Start

```bash
ports
ports --all
ports 3000
ports ps
ports logs 3000
ports kill 3000
```

### Commands

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

### Why Kiri?

- **See the owner behind a port.** Resolve ports to processes, projects,
  frameworks, and Docker mappings.
- **Keep the default view useful.** `ports` focuses on common development
  listeners; `ports --all` shows everything.
- **Inspect before killing.** `ports <port>` is read-only. Termination only
  happens through `ports kill`.
- **Find logs without guessing.** `ports logs <port|pid>` resolves likely log
  files and supports interactive selection in a real terminal.
- **Stay readable in terminals.** Table rendering is width-aware and keeps
  primary data high-contrast.

### Safety

`ports kill` resolves targets before sending signals:

- A matching listening port is preferred.
- Port ranges are expanded one port at a time.
- Empty ports inside a range are counted but do not stop the whole command.
- If a numeric target is not a listening port, Kiri checks whether it is an
  existing PID.
- Invalid targets, reversed ranges, oversized ranges, and ports outside
  `1-65535` return clear errors.

`ports clean` lists orphaned or zombie developer-process candidates and asks for
confirmation before sending any signal.

Details mode never performs kill. Use `ports kill <target>` when you intend to
terminate a process.

### Platform Support

| Platform | Status |
| --- | --- |
| macOS | Current primary supported platform |
| Linux | Planned; collector and release artifacts are not complete |
| Windows | Planned; PowerShell installer currently reports unsupported until collector and artifacts ship |

On macOS, Kiri uses `lsof`, `ps`, `tail`, macOS `log` commands, and optional
Docker metadata. Docker is optional; if Docker is unavailable or no containers
are running, Kiri continues without Docker mappings.

### Acknowledgements

Kiri was originally inspired by port-whisperer.

### Development

For maintainers and contributors working from source:

```bash
cargo fmt
cargo test
cargo run --bin ports -- --color never
cargo run --bin ports -- --all --color never
cargo run --bin ports -- ps --color never
```

## 简体中文

### Kiri 是什么？

Kiri 是一个本地开发端口检查工具。它帮你看清楚每个监听端口背后运行的开发服务、数据库、容器、进程和日志。

产品名是 **Kiri**，真正输入的命令是：

```bash
ports
```

当你遇到端口被占用、开发服务忘记关闭、容器暴露了端口却不知道来源时，Kiri 会把这些信息整理成更容易读的终端视图。默认 `ports` 只展示开发相关监听端口；需要完整系统视图时再使用 `ports --all`。

### 安装

Kiri 正在准备第一次公开发布。下面命令是 GitHub Release artifacts、npm 包和 Homebrew tap 发布后的规划安装入口。

```bash
# 安装脚本
curl -fsSL https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.sh | bash

# npm
npm install -g @gaossr/kiri

# Homebrew
brew install gaossr/tap/kiri
```

Windows PowerShell 入口已规划，但在 Windows collector 和 release artifact 交付前，脚本会明确提示暂不支持：

```powershell
irm https://raw.githubusercontent.com/GaoSSR/kiri/main/scripts/install.ps1 | iex
```

当前真实支持平台是 macOS。Linux 和 Windows 已有分发规划，但完整平台采集逻辑和发布产物还没有完成。

### 快速开始

```bash
ports
ports --all
ports 3000
ports ps
ports logs 3000
ports kill 3000
```

### 命令

```bash
ports                       # 展示开发相关监听端口
ports --all                 # 展示所有监听端口
ports <port>                # 查看单个端口详情
ports ps                    # 展示开发相关运行进程
ports ps --all              # 展示所有进程
ports logs <port|pid>       # 查看已解析进程的日志
ports logs 3000 --lines 10  # 只看最后 10 行
ports logs 3000 --err       # 只看 stderr
ports logs 3000 --follow    # 持续跟随日志
ports clean                 # 清理孤儿或僵尸开发进程前先询问
ports watch                 # 监听端口启动和停止事件
ports kill 3000             # 终止监听指定端口的进程
ports kill 3000-3010        # 终止一个端口范围内的监听进程
ports kill --force 3000     # 使用 SIGKILL 而不是 SIGTERM
ports --color never         # 输出无颜色文本，便于脚本处理
```

### 为什么需要 Kiri？

- **知道端口背后是谁。** 把端口解析到进程、项目、框架和 Docker 映射。
- **默认视图更贴近开发场景。** `ports` 聚焦常见开发监听端口，`ports --all` 保留完整视图。
- **先查看，再终止。** `ports <port>` 只读；只有 `ports kill` 才会发送信号。
- **不用猜日志位置。** `ports logs <port|pid>` 会解析可能的日志文件，并在真实终端里支持交互选择。
- **终端输出可读。** 表格会根据宽度调整，并保证主要数据有足够对比度。

### 安全边界

`ports kill` 会先解析目标，再发送信号：

- 优先匹配监听端口。
- 端口范围会逐个展开处理。
- 范围里的空端口会被计数，但不会让整个命令失败。
- 如果数字目标不是监听端口，Kiri 会继续检查它是否是一个现有 PID。
- 非法目标、反向范围、过大范围、超出 `1-65535` 的端口都会返回清晰错误。

`ports clean` 会先列出疑似孤儿或僵尸开发进程，并在发送任何信号前请求确认。

详情模式永远不会执行终止操作。你确实想结束进程时，请使用 `ports kill <target>`。

### 平台支持

| 平台 | 状态 |
| --- | --- |
| macOS | 当前主要真实支持平台 |
| Linux | 已规划；collector 和 release artifacts 尚未完成 |
| Windows | 已规划；PowerShell 安装入口当前会提示 unsupported，等待 collector 和 artifacts |

在 macOS 上，Kiri 使用 `lsof`、`ps`、`tail`、macOS `log` 命令，并在 Docker 可用时读取容器端口映射。Docker 是可选项；如果 Docker 不可用或没有运行容器，Kiri 会继续正常工作。

### 致谢

Kiri was originally inspired by port-whisperer.

### 开发

维护者和贡献者可以从源码运行检查：

```bash
cargo fmt
cargo test
cargo run --bin ports -- --color never
cargo run --bin ports -- --all --color never
cargo run --bin ports -- ps --color never
```
