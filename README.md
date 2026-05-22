<h1 align="center">
  <img src="assets/kiri-logo.png" alt="Kiri" width="420" />
  <br />
  Kiri
</h1>

<p align="center">
  <strong>管理本地开发端口：看进程、查对应日志、关占用。</strong>
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

Kiri 是一个面向本地开发的端口管理 CLI。它帮你快速看清本机启动了哪些开发服务，按端口找到背后的进程和项目，查看监听端口所对应进程的日志，或者直接结束占用端口的进程。

产品名是 **Kiri**，真正输入的命令是：

```bash
ports
```

日常开发里最常见的问题不是“端口号是多少”，而是“谁占了这个端口、它在打什么日志、我能不能安全关掉它”。Kiri 把这些操作收在一个 `ports` 命令里。默认 `ports` 聚焦开发相关监听端口；需要完整系统视图时使用 `ports --all`。

## 常用场景

```bash
ports            # 看本地开发启了几个服务
ports logs 3000  # 查看监听 3000 端口的进程日志
ports kill 3000  # 结束占用 3000 端口的进程
ports ps         # 看本地开发启了哪些进程
```

## 安装

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

## 快速开始

```bash
ports
ports logs 3000
ports kill 3000
ports ps
ports --all
ports 3000
```

## 命令

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
```

## 为什么需要 Kiri？

- **快速看清本地开发状态：** `ports` 直接列出当前开发相关监听端口、进程、项目、框架、运行时长和状态。
- **监听端口对应进程的日志：** `ports logs <port|pid>` 会解析对应进程的日志位置，并在真实终端里支持交互选择。
- **快速 Kill 掉端口所对应的进程：** `ports kill <port>` 先解析监听者，再终止占用端口的进程，不需要手动查 PID。
- **查看端口所对应的进程/PID：** Kiri 会把端口映射到进程、PID、项目、框架和 Docker 容器信息。
- **保留完整视图：** 默认 `ports` 聚焦开发场景，`ports --all` 用来查看所有监听端口。

## 安全边界

`ports kill` 会先解析目标，再发送信号：

- 优先匹配监听端口。
- 端口范围会逐个展开处理。
- 范围里的空端口会被计数，但不会让整个命令失败。
- 如果数字目标不是监听端口，Kiri 会继续检查它是否是一个现有 PID。
- 非法目标、反向范围、过大范围、超出 `1-65535` 的端口都会返回清晰错误。

`ports clean` 会先列出疑似孤儿或僵尸开发进程，并在发送任何信号前请求确认。

详情模式永远不会执行终止操作。你确实想结束进程时，请使用 `ports kill <target>`。

## 平台支持

| 平台 | 状态 |
| --- | --- |
| macOS | 当前主要真实支持平台 |
| Linux | 已规划；collector 和 release artifacts 尚未完成 |
| Windows | 已规划；PowerShell 安装入口当前会提示 unsupported，等待 collector 和 artifacts |

在 macOS 上，Kiri 使用 `lsof`、`ps`、`tail`、macOS `log` 命令，并在 Docker 可用时读取容器端口映射。Docker 是可选项；如果 Docker 不可用或没有运行容器，Kiri 会继续正常工作。

## 致谢

Kiri was originally inspired by port-whisperer.

## 开发

维护者和贡献者可以从源码运行检查：

```bash
cargo fmt
cargo test
cargo run --bin ports
cargo run --bin ports -- --all
cargo run --bin ports -- ps
```
