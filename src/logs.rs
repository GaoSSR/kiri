use crate::kill::{resolve_target, SystemResolver};
use crate::platform;
use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRequest {
    pub target: u32,
    pub follow: bool,
    pub lines: usize,
    pub stderr_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogCommandOutcome {
    pub exit_code: i32,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFile {
    pub path: PathBuf,
    pub fd: LogFd,
    pub kind: LogFileKind,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFd {
    Stdout,
    Stderr,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFileKind {
    Redirect,
    LogFile,
    Framework,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogParseError {
    MissingTarget,
    MissingLinesValue,
    InvalidLines(String),
    InvalidTarget(String),
    UnknownArgument(String),
}

impl LogParseError {
    fn message(&self) -> String {
        match self {
            Self::MissingTarget => usage_text(),
            Self::MissingLinesValue => "  x --lines requires a positive number\n".to_string(),
            Self::InvalidLines(value) => {
                format!("  x \"{value}\" is not a valid --lines value\n")
            }
            Self::InvalidTarget(value) => format!("  x \"{value}\" is not a valid port/PID\n"),
            Self::UnknownArgument(value) => format!("  x Unknown logs argument: {value}\n"),
        }
    }
}

pub fn run_logs_command(args: &[String]) -> LogCommandOutcome {
    let request = match parse_logs_args(args) {
        Ok(request) => request,
        Err(error) => {
            return LogCommandOutcome {
                exit_code: 1,
                output: error.message(),
            };
        }
    };

    execute_logs_command(&request, &SystemResolver)
}

fn execute_logs_command(request: &LogRequest, resolver: &SystemResolver) -> LogCommandOutcome {
    let Some(resolved) = resolve_target(request.target, resolver) else {
        let output = if request.target <= u16::MAX as u32 {
            format!(
                "  x No listener on :{} and no process with PID {}\n",
                request.target, request.target
            )
        } else {
            format!("  x No process with PID {}\n", request.target)
        };
        return LogCommandOutcome {
            exit_code: 1,
            output,
        };
    };

    let label = resolved
        .port
        .map(|port| format!(":{port}"))
        .unwrap_or_else(|| format!("PID {}", resolved.pid));
    let process_name = resolved.process_name.as_deref().unwrap_or("unknown");
    let mut output = format!(
        "Kiri - logs for {label} ({process_name}, PID {})\n\n",
        resolved.pid
    );
    let log_files = get_process_log_files(resolved.pid);

    if request.stderr_only {
        let Some(file) = stderr_log_file(&log_files) else {
            output.push_str(&format!(
                "No stderr redirect found for PID {}\n",
                resolved.pid
            ));
            return LogCommandOutcome {
                exit_code: 1,
                output,
            };
        };
        output.push_str(&format!("Tailing stderr: {}\n\n", file.path.display()));
        return run_tail(file, request, output);
    }

    if let Some(file) = select_log_file(&log_files) {
        output.push_str(&format!(
            "Tailing {}: {}\n\n",
            log_file_label(file),
            file.path.display()
        ));
        return run_tail(file, request, output);
    }

    if let Some(system_log) = get_system_log_command(resolved.pid, request.follow) {
        output.push_str("No log files found. Falling back to system log...\n\n");
        return run_system_log_command(system_log, resolved.pid, request.follow, output);
    }

    output.push_str(&format!(
        "No log files or system log found for PID {}.\n",
        resolved.pid
    ));
    LogCommandOutcome {
        exit_code: 1,
        output,
    }
}

pub fn parse_logs_args(args: &[String]) -> Result<LogRequest, LogParseError> {
    let mut follow = false;
    let mut lines = 50usize;
    let mut stderr_only = false;
    let mut target = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "-f" || arg == "--follow" {
            follow = true;
            index += 1;
        } else if arg == "--err" {
            stderr_only = true;
            index += 1;
        } else if arg == "--lines" {
            let Some(value) = args.get(index + 1) else {
                return Err(LogParseError::MissingLinesValue);
            };
            lines = parse_lines(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--lines=") {
            lines = parse_lines(value)?;
            index += 1;
        } else if arg.starts_with('-') {
            return Err(LogParseError::UnknownArgument(arg.clone()));
        } else if target.is_none() {
            target = Some(parse_target(arg)?);
            index += 1;
        } else {
            return Err(LogParseError::UnknownArgument(arg.clone()));
        }
    }

    Ok(LogRequest {
        target: target.ok_or(LogParseError::MissingTarget)?,
        follow,
        lines,
        stderr_only,
    })
}

fn parse_lines(value: &str) -> Result<usize, LogParseError> {
    let lines = value
        .parse::<usize>()
        .map_err(|_| LogParseError::InvalidLines(value.to_string()))?;
    if lines == 0 {
        return Err(LogParseError::InvalidLines(value.to_string()));
    }
    Ok(lines)
}

fn parse_target(value: &str) -> Result<u32, LogParseError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(LogParseError::InvalidTarget(value.to_string()));
    }
    let target = value
        .parse::<u32>()
        .map_err(|_| LogParseError::InvalidTarget(value.to_string()))?;
    if target == 0 {
        return Err(LogParseError::InvalidTarget(value.to_string()));
    }
    Ok(target)
}

pub fn get_process_log_files(pid: u32) -> Vec<LogFile> {
    let mut files = Vec::new();
    let mut pipe_ids = HashSet::new();

    if cfg!(any(target_os = "macos", target_os = "linux")) {
        if let Ok(output) = Command::new("lsof").args(["-p", &pid.to_string()]).output() {
            if output.status.success() {
                let raw = String::from_utf8_lossy(&output.stdout);
                files.extend(parse_lsof_log_files(&raw));
                pipe_ids = parse_lsof_pipe_ids(&raw);
            }
        }
    }

    if !pipe_ids.is_empty() {
        if let Ok(output) = Command::new("lsof").output() {
            if output.status.success() {
                let raw = String::from_utf8_lossy(&output.stdout);
                files.extend(parse_pipe_writer_log_files(&raw, &pipe_ids));
            }
        }
    }

    if let Some(cwd) = platform::batch_cwd(&[pid]).get(&pid).cloned() {
        files.extend(common_framework_logs(&cwd));
    }

    sort_and_deduplicate_log_files(files)
}

pub fn parse_lsof_log_files(raw: &str) -> Vec<LogFile> {
    let mut files = Vec::new();

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let fd = parts[3];
        let file_type = parts[4];
        let path = parts[8..].join(" ");

        if (fd == "1w" || fd == "2w") && file_type == "REG" {
            files.push(LogFile {
                path: PathBuf::from(&path),
                fd: if fd == "1w" {
                    LogFd::Stdout
                } else {
                    LogFd::Stderr
                },
                kind: LogFileKind::Redirect,
                priority: 1,
            });
            continue;
        }

        if file_type == "REG" && fd.ends_with('w') && is_log_like_path(&path) {
            files.push(LogFile {
                path: PathBuf::from(&path),
                fd: LogFd::File,
                kind: LogFileKind::LogFile,
                priority: 2,
            });
        }
    }

    sort_and_deduplicate_log_files(files)
}

fn parse_lsof_pipe_ids(raw: &str) -> HashSet<String> {
    raw.lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 || parts[4] != "PIPE" {
                return None;
            }

            matches!(fd_number(parts[3]), Some(1 | 2)).then(|| parts[5].to_string())
        })
        .collect()
}

fn parse_pipe_writer_log_files(raw: &str, pipe_ids: &HashSet<String>) -> Vec<LogFile> {
    if pipe_ids.is_empty() {
        return Vec::new();
    }

    let pipe_reader_pids: HashSet<&str> = raw
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 || parts[4] != "PIPE" {
                return None;
            }

            let endpoint = parts.iter().find_map(|part| part.strip_prefix("->"))?;
            pipe_ids.contains(endpoint).then_some(parts[1])
        })
        .collect();

    let mut files = Vec::new();
    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 || parts[4] != "REG" || !pipe_reader_pids.contains(parts[1]) {
            continue;
        }

        let fd = parts[3];
        let Some(path) = regular_file_path(&parts) else {
            continue;
        };

        if fd.contains('w') && is_log_like_path(&path) {
            files.push(LogFile {
                path: PathBuf::from(path),
                fd: LogFd::File,
                kind: LogFileKind::Redirect,
                priority: 1,
            });
        }
    }

    sort_and_deduplicate_log_files(files)
}

fn regular_file_path(parts: &[&str]) -> Option<String> {
    if parts.len() >= 9 {
        Some(parts[8..].join(" "))
    } else if parts.len() >= 8 {
        Some(parts[7..].join(" "))
    } else {
        None
    }
}

fn fd_number(fd: &str) -> Option<u32> {
    let digits: String = fd.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    (!digits.is_empty())
        .then(|| digits.parse::<u32>().ok())
        .flatten()
}

pub fn is_log_like_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".log")
        || lower.contains("/log/")
        || lower.contains("/logs/")
        || lower.contains("\\log\\")
        || lower.contains("\\logs\\")
        || lower.contains("/tmp/")
        || lower.contains("nohup.out")
        || lower.contains("stdout")
        || lower.contains("stderr")
}

fn common_framework_logs(cwd: &Path) -> Vec<LogFile> {
    let mut files = Vec::new();
    let component_name = cwd.file_name().and_then(|name| name.to_str());

    for root in log_search_roots(cwd) {
        files.extend(fixed_framework_log_paths(&root));
        if let Some(component_name) = component_name {
            files.extend(component_log_paths(&root, component_name));
            files.extend(component_rotated_log_paths(
                &root.join(".dev-logs"),
                component_name,
            ));
            files.extend(component_rotated_log_paths(
                &root.join("logs"),
                component_name,
            ));
            files.extend(component_rotated_log_paths(
                &root.join("log"),
                component_name,
            ));
        }
        files.extend(log_files_in_dir(&root.join(".dev-logs"), 5));
        files.extend(log_files_in_dir(&root.join("logs"), 5));
        files.extend(log_files_in_dir(&root.join("log"), 5));
    }

    sort_and_deduplicate_log_files(files)
}

fn log_search_roots(cwd: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut current = Some(cwd);

    while let Some(path) = current {
        roots.push(path.to_path_buf());
        if roots.len() >= 4 || is_home_like_root(path) {
            break;
        }
        current = path.parent();
    }

    roots
}

fn is_home_like_root(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name == "Users" || name == "home"
}

fn fixed_framework_log_paths(root: &Path) -> Vec<LogFile> {
    [
        ".next/server.log",
        "logs/development.log",
        "log/development.log",
        "tmp/pids/server.log",
        "storage/logs/laravel.log",
        "npm-debug.log",
        "yarn-error.log",
    ]
    .into_iter()
    .filter_map(|relative| framework_log_file(root.join(relative), 4))
    .collect()
}

fn component_log_paths(root: &Path, component_name: &str) -> Vec<LogFile> {
    [
        root.join(".dev-logs").join(format!("{component_name}.log")),
        root.join("logs").join(format!("{component_name}.log")),
        root.join("log").join(format!("{component_name}.log")),
    ]
    .into_iter()
    .filter_map(|path| {
        let priority = if is_non_empty_file(&path) { 3 } else { 6 };
        framework_log_file(path, priority)
    })
    .collect()
}

fn component_rotated_log_paths(dir: &Path, component_name: &str) -> Vec<LogFile> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let prefix = format!("{component_name}-");
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            (file_name.starts_with(&prefix) && is_non_empty_file(&path))
                .then_some(path)
                .and_then(|path| framework_log_file(path, 4))
        })
        .collect()
}

fn log_files_in_dir(dir: &Path, priority: u8) -> Vec<LogFile> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| framework_log_file(entry.path(), priority))
        .collect()
}

fn framework_log_file(path: PathBuf, priority: u8) -> Option<LogFile> {
    if !path.is_file() || !is_log_like_path(&path.to_string_lossy()) {
        return None;
    }

    Some(LogFile {
        path,
        fd: LogFd::File,
        kind: LogFileKind::Framework,
        priority,
    })
}

fn is_non_empty_file(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
}

fn sort_and_deduplicate_log_files(mut files: Vec<LogFile>) -> Vec<LogFile> {
    files.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| log_fd_rank(a.fd).cmp(&log_fd_rank(b.fd)))
            .then_with(|| a.path.cmp(&b.path))
    });
    let mut seen = HashSet::new();
    files
        .into_iter()
        .filter(|file| seen.insert(file.path.clone()))
        .collect()
}

fn select_log_file(log_files: &[LogFile]) -> Option<&LogFile> {
    if should_prompt_log_file_selection(log_files) {
        prompt_log_file_selection(log_files).or_else(|| auto_select_log_file(log_files))
    } else {
        auto_select_log_file(log_files)
    }
}

fn should_prompt_log_file_selection(log_files: &[LogFile]) -> bool {
    log_files.len() > 1 && io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn prompt_log_file_selection(log_files: &[LogFile]) -> Option<&LogFile> {
    println!("Multiple log files found:");
    for (index, file) in log_files.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            index + 1,
            file.path.display(),
            log_file_label(file)
        );
    }
    print!("Select log file [1]: ");
    let _ = io::stdout().flush();

    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return None;
    }

    select_log_file_by_number(log_files, &answer)
}

fn select_log_file_by_number<'a>(log_files: &'a [LogFile], answer: &str) -> Option<&'a LogFile> {
    let trimmed = answer.trim();
    if trimmed.is_empty() {
        return log_files.first();
    }

    let number = trimmed.parse::<usize>().ok()?;
    number.checked_sub(1).and_then(|index| log_files.get(index))
}

fn auto_select_log_file(log_files: &[LogFile]) -> Option<&LogFile> {
    log_files.first()
}

fn stderr_log_file(log_files: &[LogFile]) -> Option<&LogFile> {
    log_files.iter().find(|file| file.fd == LogFd::Stderr)
}

fn log_fd_rank(fd: LogFd) -> u8 {
    match fd {
        LogFd::Stdout => 0,
        LogFd::Stderr => 1,
        LogFd::File => 2,
    }
}

fn run_tail(file: &LogFile, request: &LogRequest, output: String) -> LogCommandOutcome {
    let args = if request.follow {
        vec![
            "-f".to_string(),
            "-n".to_string(),
            request.lines.to_string(),
            file.path.display().to_string(),
        ]
    } else {
        vec![
            "-n".to_string(),
            request.lines.to_string(),
            file.path.display().to_string(),
        ]
    };
    run_command("tail".to_string(), &args, request.follow, output)
}

fn run_command(
    command: String,
    args: &[String],
    follow: bool,
    mut output: String,
) -> LogCommandOutcome {
    if follow {
        return run_streaming_command(command, args, output);
    }

    match Command::new(command).args(args).output() {
        Ok(result) => {
            output.push_str(&colorize_log_output(&String::from_utf8_lossy(
                &result.stdout,
            )));
            output.push_str(&colorize_log_output(&String::from_utf8_lossy(
                &result.stderr,
            )));
            LogCommandOutcome {
                exit_code: if result.status.success() { 0 } else { 1 },
                output,
            }
        }
        Err(error) => {
            output.push_str(&format!("Failed to run log command: {}\n", error));
            LogCommandOutcome {
                exit_code: 1,
                output,
            }
        }
    }
}

fn run_streaming_command(
    command: String,
    args: &[String],
    mut output: String,
) -> LogCommandOutcome {
    print!("{output}");
    let _ = io::stdout().flush();

    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            output.push_str(&format!("Failed to run log command: {error}\n"));
            eprintln!("Failed to run log command: {error}");
            return LogCommandOutcome {
                exit_code: 1,
                output: String::new(),
            };
        }
    };

    let Some(stdout) = child.stdout.take() else {
        eprintln!("Failed to read log command output");
        return LogCommandOutcome {
            exit_code: 1,
            output: String::new(),
        };
    };

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                print!("{}", colorize_log_output(&line));
                let _ = io::stdout().flush();
            }
            Err(error) => {
                eprintln!("Failed to read log output: {error}");
                let _ = child.kill();
                let _ = child.wait();
                return LogCommandOutcome {
                    exit_code: 1,
                    output: String::new(),
                };
            }
        }
    }

    let exit_code = child
        .wait()
        .ok()
        .and_then(|status| status.code())
        .unwrap_or(1);
    LogCommandOutcome {
        exit_code,
        output: String::new(),
    }
}

fn run_system_log_command(
    system_log: SystemLogCommand,
    pid: u32,
    follow: bool,
    mut output: String,
) -> LogCommandOutcome {
    let (command, args) = system_log.command_and_args();
    if follow {
        return run_command(command, &args, follow, output);
    }

    match Command::new(command).args(args).output() {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            output.push_str(&colorize_log_output(&stdout));
            output.push_str(&colorize_log_output(&stderr));

            if result.status.success() && system_log_output_has_records(&stdout) {
                return LogCommandOutcome {
                    exit_code: 0,
                    output,
                };
            }

            if result.status.success() {
                output.push_str(&format!(
                    "No system log entries found for PID {pid}.\n\
Tip: if the process logs through a pipe, restart it with stdout/stderr redirected to a file or use tee output detection.\n"
                ));
            }

            LogCommandOutcome {
                exit_code: 1,
                output,
            }
        }
        Err(error) => {
            output.push_str(&format!("Failed to run system log command: {}\n", error));
            LogCommandOutcome {
                exit_code: 1,
                output,
            }
        }
    }
}

fn system_log_output_has_records(output: &str) -> bool {
    output.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with("Timestamp ")
    })
}

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BOLD: &str = "\x1b[1m";
const LOG_TIMESTAMP: &str = "\x1b[38;5;45m";
const LOG_GREEN: &str = "\x1b[38;2;21;128;61m";
const LOG_FRAMEWORK_RED: &str = "\x1b[38;2;239;68;68m";
const LOG_INFO: &str = LOG_GREEN;
const LOG_WARN: &str = "\x1b[38;5;214m";
const LOG_ERROR: &str = LOG_FRAMEWORK_RED;
const LOG_DEBUG: &str = "\x1b[38;5;141m";
const LOG_TRACE: &str = "\x1b[38;5;244m";
const LOG_THREAD: &str = "\x1b[38;5;51m";
const LOG_SOURCE: &str = LOG_GREEN;
const LOG_KEY: &str = "\x1b[38;5;75m";
const LOG_VALUE: &str = LOG_GREEN;
const LOG_TRACE_ID: &str = "\x1b[38;5;177m";
const LOG_SEPARATOR: &str = "\x1b[38;5;245m";
const LOG_PID: &str = "\x1b[38;5;208m";
const LOG_HTTP_METHOD: &str = "\x1b[38;5;81m";
const LOG_HTTP_STATUS_OK: &str = LOG_GREEN;
const LOG_HTTP_STATUS_WARN: &str = "\x1b[38;5;214m";
const LOG_HTTP_STATUS_ERROR: &str = LOG_FRAMEWORK_RED;

fn colorize_log_output(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let mut output = raw
        .lines()
        .map(colorize_log_line)
        .collect::<Vec<_>>()
        .join("\n");
    if raw.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn colorize_log_line(line: &str) -> String {
    if !is_structured_log_line(line) {
        return line.to_string();
    }

    if is_json_log_line(line.trim()) {
        return colorize_json_log_line(line);
    }

    let mut output = String::with_capacity(line.len() + 64);
    let mut current = String::new();
    let mut current_is_whitespace = None;

    for ch in line.chars() {
        let is_whitespace = ch.is_whitespace();
        match current_is_whitespace {
            Some(kind) if kind == is_whitespace => current.push(ch),
            Some(true) => {
                output.push_str(&current);
                current.clear();
                current.push(ch);
                current_is_whitespace = Some(is_whitespace);
            }
            Some(false) => {
                output.push_str(&colorize_log_token(&current));
                current.clear();
                current.push(ch);
                current_is_whitespace = Some(is_whitespace);
            }
            None => {
                current.push(ch);
                current_is_whitespace = Some(is_whitespace);
            }
        }
    }

    match current_is_whitespace {
        Some(true) => output.push_str(&current),
        Some(false) => output.push_str(&colorize_log_token(&current)),
        None => {}
    }

    output
}

fn is_structured_log_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if is_json_log_line(trimmed) {
        return true;
    }

    let mut has_timestamp = false;
    let mut level_index = None;
    let mut key_value_count = 0usize;
    let mut has_http_token = false;

    for (index, token) in line.split_whitespace().enumerate() {
        has_timestamp |= is_timestamp_token(token);
        if level_index.is_none() && log_level_color(token).is_some() {
            level_index = Some(index);
        }
        has_http_token |= is_http_method(token) || http_status_color(token).is_some();

        if let Some((key, value)) = token.split_once('=') {
            if !key.is_empty() && !value.is_empty() && is_log_key(key) {
                key_value_count += 1;
                if key.eq_ignore_ascii_case("level") || key.eq_ignore_ascii_case("severity") {
                    level_index = level_index.or(Some(index));
                }
                if key.eq_ignore_ascii_case("time") || key.eq_ignore_ascii_case("timestamp") {
                    has_timestamp = true;
                }
            }
        }
    }

    let has_level_context = level_index.is_some_and(|index| {
        index == 0 || (index <= 3 && (has_timestamp || has_http_token || key_value_count > 0))
    });
    has_level_context || key_value_count >= 2 || (has_timestamp && has_http_token)
}

fn colorize_log_token(token: &str) -> String {
    if is_timestamp_token(token) {
        return ansi_color(token, LOG_TIMESTAMP);
    }

    if let Some(color) = log_level_color(token) {
        return ansi_bold_color(token, color);
    }

    if token.starts_with("[traceId:") || token.starts_with("traceId:") {
        return ansi_color(token, LOG_TRACE_ID);
    }

    if token.starts_with('[') && token.ends_with(']') {
        return ansi_color(token, LOG_THREAD);
    }

    if token == "---" {
        return ansi_color(token, LOG_SEPARATOR);
    }

    if token.bytes().all(|byte| byte.is_ascii_digit()) {
        if let Some(color) = http_status_color(token) {
            return ansi_bold_color(token, color);
        }
        return ansi_color(token, LOG_PID);
    }

    if is_http_method(token) {
        return ansi_bold_color(token, LOG_HTTP_METHOD);
    }

    if let Some((key, value)) = token.split_once('=') {
        if !key.is_empty() && is_log_key(key) {
            return format!("{}={}", ansi_color(key, LOG_KEY), colorize_log_value(value));
        }
    }

    if is_logger_token(token) {
        return ansi_color(token, LOG_SOURCE);
    }

    colorize_log_value(token)
}

fn colorize_log_value(value: &str) -> String {
    let unquoted = value.trim_matches(|ch| ch == '"' || ch == '\'');
    if let Some(color) = log_level_color(unquoted) {
        return ansi_bold_color(value, color);
    }
    if let Some(color) = http_status_color(unquoted) {
        return ansi_bold_color(value, color);
    }

    match unquoted.to_ascii_uppercase().as_str() {
        "SUCCESS" | "OK" | "TRUE" | "READY" => ansi_bold_color(value, LOG_VALUE),
        "FAILED" | "FAILURE" | "ERROR" | "FALSE" => ansi_bold_color(value, LOG_ERROR),
        _ => value.to_string(),
    }
}

fn is_timestamp_token(token: &str) -> bool {
    let bytes = token.as_bytes();
    is_iso_timestamp_token(bytes) || is_date_token(bytes) || is_time_token(bytes)
}

fn is_iso_timestamp_token(bytes: &[u8]) -> bool {
    bytes.len() >= 19
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && matches!(bytes.get(10), Some(b'T' | b' '))
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':')
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn is_date_token(bytes: &[u8]) -> bool {
    bytes.len() == 10
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && matches!(bytes[4], b'-' | b'/')
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && matches!(bytes[7], b'-' | b'/')
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn is_time_token(bytes: &[u8]) -> bool {
    bytes.len() >= 8
        && bytes[0..2].iter().all(u8::is_ascii_digit)
        && bytes[2] == b':'
        && bytes[3..5].iter().all(u8::is_ascii_digit)
        && bytes[5] == b':'
        && bytes[6..8].iter().all(u8::is_ascii_digit)
}

fn log_level_color(token: &str) -> Option<&'static str> {
    match token
        .trim_matches(|ch: char| !ch.is_ascii_alphabetic())
        .to_ascii_uppercase()
        .as_str()
    {
        "ERROR" | "FATAL" => Some(LOG_ERROR),
        "WARN" | "WARNING" => Some(LOG_WARN),
        "INFO" => Some(LOG_INFO),
        "DEBUG" => Some(LOG_DEBUG),
        "TRACE" => Some(LOG_TRACE),
        _ => None,
    }
}

fn is_http_method(token: &str) -> bool {
    matches!(
        token.trim_matches('"'),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
    )
}

fn http_status_color(token: &str) -> Option<&'static str> {
    if token.len() != 3 || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    match token.parse::<u16>().ok()? {
        100..=399 => Some(LOG_HTTP_STATUS_OK),
        400..=499 => Some(LOG_HTTP_STATUS_WARN),
        500..=599 => Some(LOG_HTTP_STATUS_ERROR),
        _ => None,
    }
}

fn is_json_log_line(line: &str) -> bool {
    line.starts_with('{')
        && line.ends_with('}')
        && serde_json::from_str::<serde_json::Value>(line).is_ok()
}

fn colorize_json_log_line(line: &str) -> String {
    let mut output = String::with_capacity(line.len() + 64);
    let mut index = 0usize;

    while index < line.len() {
        let rest = &line[index..];
        if rest.starts_with('"') {
            let Some(end) = json_string_end(line, index) else {
                output.push_str(rest);
                break;
            };
            let segment = &line[index..=end];
            let after = skip_ascii_spaces(line, end + 1);

            if line.as_bytes().get(after) == Some(&b':') {
                output.push_str(&ansi_color(segment, LOG_KEY));
            } else {
                output.push_str(&colorize_log_value(segment));
            }
            index = end + 1;
            continue;
        }

        if let Some((token, end)) = json_scalar_token(line, index) {
            output.push_str(&colorize_log_value(token));
            index = end;
            continue;
        }

        output.push(rest.chars().next().unwrap());
        index += rest.chars().next().unwrap().len_utf8();
    }

    output
}

fn json_string_end(line: &str, start: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut escaped = false;
    for (offset, byte) in bytes[start + 1..].iter().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if *byte == b'\\' {
            escaped = true;
            continue;
        }
        if *byte == b'"' {
            return Some(start + 1 + offset);
        }
    }
    None
}

fn skip_ascii_spaces(line: &str, mut index: usize) -> usize {
    while matches!(line.as_bytes().get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    index
}

fn json_scalar_token(line: &str, start: usize) -> Option<(&str, usize)> {
    let byte = *line.as_bytes().get(start)?;
    if !(byte.is_ascii_digit() || byte == b'-' || matches!(byte, b't' | b'f' | b'n')) {
        return None;
    }

    let mut end = start;
    while let Some(byte) = line.as_bytes().get(end) {
        if matches!(byte, b',' | b'}' | b']' | b' ' | b'\t') {
            break;
        }
        end += 1;
    }

    (end > start).then_some((&line[start..end], end))
}

fn is_log_key(key: &str) -> bool {
    key.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn is_logger_token(token: &str) -> bool {
    if token.contains('=') || !token.contains('.') || token.ends_with('.') {
        return false;
    }

    if !token.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'$' | b'-' | b'/' | b':')
    }) {
        return false;
    }

    let segments = token
        .split('.')
        .filter(|segment| !segment.is_empty())
        .count();
    segments >= 2
}

fn ansi_color(value: &str, color: &str) -> String {
    format!("{color}{value}{ANSI_RESET}")
}

fn ansi_bold_color(value: &str, color: &str) -> String {
    format!("{ANSI_BOLD}{color}{value}{ANSI_RESET}")
}

fn log_file_label(file: &LogFile) -> &'static str {
    match file.fd {
        LogFd::Stdout => "stdout",
        LogFd::Stderr => "stderr",
        LogFd::File => "log",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SystemLogCommand {
    program: String,
    args: Vec<String>,
}

impl SystemLogCommand {
    fn command_and_args(self) -> (String, Vec<String>) {
        (self.program, self.args)
    }
}

fn get_system_log_command(pid: u32, follow: bool) -> Option<SystemLogCommand> {
    if cfg!(target_os = "macos") {
        let mut args = if follow {
            vec!["stream".to_string()]
        } else {
            vec!["show".to_string()]
        };
        args.extend([
            "--predicate".to_string(),
            format!("processID == {pid}"),
            "--style".to_string(),
            "compact".to_string(),
        ]);
        if !follow {
            args.extend(["--last".to_string(), "1m".to_string()]);
        }
        Some(SystemLogCommand {
            program: "log".to_string(),
            args,
        })
    } else if cfg!(target_os = "linux") {
        let mut args = vec![format!("_PID={pid}"), "--no-pager".to_string()];
        if follow {
            args.push("-f".to_string());
        } else {
            args.extend(["-n".to_string(), "50".to_string()]);
        }
        Some(SystemLogCommand {
            program: "journalctl".to_string(),
            args,
        })
    } else {
        None
    }
}

fn usage_text() -> String {
    [
        "  Usage: ports logs <port|pid> [-f|--follow] [--lines N] [--lines=N] [--err]",
        "  Shows log output for a process resolved by listening port or PID.",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_follow_and_stderr_flags() {
        assert_eq!(
            parse_logs_args(&strings(["3000", "--lines", "5", "-f", "--err"])).unwrap(),
            LogRequest {
                target: 3000,
                follow: true,
                lines: 5,
                stderr_only: true,
            }
        );
        assert_eq!(
            parse_logs_args(&strings(["3000", "--lines=10", "--follow"])).unwrap(),
            LogRequest {
                target: 3000,
                follow: true,
                lines: 10,
                stderr_only: false,
            }
        );
    }

    #[test]
    fn rejects_invalid_logs_arguments() {
        assert!(matches!(
            parse_logs_args(&[]),
            Err(LogParseError::MissingTarget)
        ));
        assert!(matches!(
            parse_logs_args(&strings(["abc"])),
            Err(LogParseError::InvalidTarget(_))
        ));
        assert!(matches!(
            parse_logs_args(&strings(["3000", "--lines=0"])),
            Err(LogParseError::InvalidLines(_))
        ));
        assert!(matches!(
            parse_logs_args(&strings(["3000", "--lines"])),
            Err(LogParseError::MissingLinesValue)
        ));
        assert!(matches!(
            parse_logs_args(&strings(["3000", "--bogus"])),
            Err(LogParseError::UnknownArgument(_))
        ));
    }

    #[test]
    fn identifies_log_like_paths() {
        assert!(is_log_like_path("/tmp/next.output"));
        assert!(is_log_like_path("/repo/log/development.log"));
        assert!(is_log_like_path("/repo/logs/app.txt"));
        assert!(is_log_like_path("nohup.out"));
        assert!(is_log_like_path("stderr"));
        assert!(!is_log_like_path("/repo/src/main.rs"));
    }

    #[test]
    fn parses_lsof_stdout_stderr_and_log_files() {
        let raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
node    42872 user    1w   REG   1,18      640 1234 /tmp/dev.stdout
node    42872 user    2w   REG   1,18      640 1235 /tmp/dev.stderr
node    42872 user   11w   REG   1,18      640 1236 /repo/log/app.log
node    42872 user   12r   REG   1,18      640 1237 /repo/src/main.rs
";

        let files = parse_lsof_log_files(raw);

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].fd, LogFd::Stdout);
        assert_eq!(files[1].fd, LogFd::Stderr);
        assert_eq!(files[2].fd, LogFd::File);
    }

    #[test]
    fn parses_pipe_endpoint_written_by_tee_to_log_file() {
        let target_raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
java    68388 user    1   PIPE 0xaaaa    16384      ->0xbbbb
java    68388 user    2   PIPE 0xaaaa    16384      ->0xbbbb
";
        let peer_raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
tee     68361 user    0   PIPE 0xbbbb    16384      ->0xaaaa
tee     68361 user    1u   CHR   16,34    0t57550   /dev/ttys034
tee     68361 user    3w   REG   1,15    116166    /private/tmp/nori-backend.log
";

        let pipe_ids = parse_lsof_pipe_ids(target_raw);
        let files = parse_pipe_writer_log_files(peer_raw, &pipe_ids);

        assert_eq!(pipe_ids, HashSet::from(["0xaaaa".to_string()]));
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].path,
            PathBuf::from("/private/tmp/nori-backend.log")
        );
        assert_eq!(files[0].fd, LogFd::File);
        assert_eq!(files[0].kind, LogFileKind::Redirect);
    }

    #[test]
    fn ignores_pipe_writers_for_unrelated_pipe_ids() {
        let pipe_ids = HashSet::from(["0xaaaa".to_string()]);
        let peer_raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
tee     68361 user    0   PIPE 0xbbbb    16384      ->0xcccc
tee     68361 user    3w   REG   1,15    116166    /private/tmp/nori-backend.log
";

        let files = parse_pipe_writer_log_files(peer_raw, &pipe_ids);

        assert!(files.is_empty());
    }

    #[test]
    fn ignores_pipe_writers_without_log_like_output_files() {
        let pipe_ids = HashSet::from(["0xaaaa".to_string()]);
        let peer_raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
tee     68361 user    0   PIPE 0xbbbb    16384      ->0xaaaa
tee     68361 user    3w   REG   1,15    116166    /Users/me/output.txt
";

        let files = parse_pipe_writer_log_files(peer_raw, &pipe_ids);

        assert!(files.is_empty());
    }

    #[test]
    fn detects_system_log_output_with_records_after_header() {
        assert!(!system_log_output_has_records(
            "Timestamp               Ty Process[PID:TID]\n"
        ));
        assert!(system_log_output_has_records(
            "Timestamp               Ty Process[PID:TID]\n2026-05-23 event\n"
        ));
    }

    #[test]
    fn colorizes_timestamped_key_value_log_lines() {
        let line = "2026-05-23T18:45:04.859+08:00  INFO [traceId:dc1f5ec6cda04ca5b904b56c1e640c12] 15224 --- [nio-8080-exec-1] c.n.a.c.api.ConversationController method=GET status=200 result=SUCCESS";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("2026-05-23T18:45:04.859+08:00"));
        assert!(colored.contains("INFO"));
        assert!(colored.contains("traceId:dc1f5ec6cda04ca5b904b56c1e640c12"));
        assert!(colored.contains("method"));
        assert!(colored.contains("SUCCESS"));
    }

    #[test]
    fn log_palette_uses_muted_green_for_info_source_and_success_values() {
        let line = "2026-05-23T18:45:04.859+08:00  INFO [traceId:-] 15224 --- [main] c.n.a.Service status=200 result=SUCCESS";

        let colored = colorize_log_line(line);

        assert!(colored.contains("\x1b[38;2;21;128;61m"));
        assert!(!colored.contains("\x1b[38;5;82m"));
        assert!(!colored.contains("\x1b[38;5;120m"));
    }

    #[test]
    fn log_palette_uses_framework_red_for_error_levels_and_5xx_statuses() {
        let line = "2026-05-23T18:45:04.859+08:00 ERROR [traceId:-] 15224 --- [main] c.n.a.Service status=500 result=FAILED";

        let colored = colorize_log_line(line);

        assert!(colored.contains("\x1b[38;2;239;68;68m"));
        assert!(!colored.contains("\x1b[38;5;196m"));
    }

    #[test]
    fn logger_detection_does_not_color_plain_words_with_trailing_periods() {
        let line = "2026-05-23T18:45:04.859+08:00  INFO [traceId:] 15224 --- [main] c.n.a.Service : Graceful shutdown complete.";

        let colored = colorize_log_line(line);

        assert!(colored.contains("\x1b[38;2;21;128;61mc.n.a.Service\x1b[0m"));
        assert!(colored.contains("complete."));
        assert!(!colored.contains("\x1b[38;2;21;128;61mcomplete.\x1b[0m"));
    }

    #[test]
    fn colorizes_node_style_lowercase_log_lines() {
        let line = "2026-05-23T18:45:04.859Z info vite server listening port=5173 status=ready";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("info"));
        assert!(colored.contains("port"));
        assert!(colored.contains("ready"));
    }

    #[test]
    fn colorizes_uvicorn_style_log_lines_without_timestamp() {
        let line = "INFO:     127.0.0.1:53415 - \"GET /docs HTTP/1.1\" 200 OK";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("INFO:"));
        assert!(colored.contains("GET"));
        assert!(colored.contains("200"));
    }

    #[test]
    fn colorizes_go_log_lines() {
        let line = "2026/05/23 18:45:04 WARN server request method=POST path=/api/users status=201 duration=12ms";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("WARN"));
        assert!(colored.contains("method"));
        assert!(colored.contains("201"));
    }

    #[test]
    fn colorizes_logfmt_lines() {
        let line =
            "time=2026-05-23T18:45:04.859Z level=warn msg=\"slow request\" method=GET status=504";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("level"));
        assert!(colored.contains("warn"));
        assert!(colored.contains("status"));
    }

    #[test]
    fn colorizes_json_log_lines() {
        let line = "{\"time\":\"2026-05-23T18:45:04.859Z\",\"level\":\"error\",\"msg\":\"failed\",\"status\":500}";

        let colored = colorize_log_line(line);

        assert_ne!(colored, line);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("\"level\""));
        assert!(colored.contains("\"error\""));
        assert!(colored.contains("\"status\""));
    }

    #[test]
    fn leaves_plain_log_text_uncolored() {
        let line = "server started without structured fields or information markers";

        assert_eq!(colorize_log_line(line), line);
    }

    #[test]
    fn leaves_plain_sentence_with_info_word_uncolored() {
        let line = "server info message without timestamp or structured fields";

        assert_eq!(colorize_log_line(line), line);
    }

    #[test]
    fn colorizes_only_structured_lines_in_multiline_log_output() {
        let raw = "server started\n2026-05-23T18:45:04.859+08:00  WARN [traceId:-] 15224 --- [main] app.Service status=500\n";

        let colored = colorize_log_output(raw);

        assert!(colored.starts_with("server started\n"));
        assert!(colored.contains("\x1b["));
        assert!(colored.ends_with('\n'));
    }

    #[test]
    fn keeps_first_duplicate_log_path() {
        let raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
node    42872 user    1w   REG   1,18      640 1234 /tmp/dev.log
node    42872 user    2w   REG   1,18      640 1234 /tmp/dev.log
";

        let files = parse_lsof_log_files(raw);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].fd, LogFd::Stdout);
    }

    #[test]
    fn selects_log_file_by_number_for_interactive_choices() {
        let files = sample_log_files();

        let selected = select_log_file_by_number(&files, "2").unwrap();

        assert_eq!(selected.path, PathBuf::from("/tmp/app.stderr"));
        assert!(select_log_file_by_number(&files, "9").is_none());
        assert!(select_log_file_by_number(&files, "abc").is_none());
    }

    #[test]
    fn empty_log_selection_defaults_to_first_file() {
        let files = sample_log_files();

        let selected = select_log_file_by_number(&files, "").unwrap();

        assert_eq!(selected.path, PathBuf::from("/tmp/app.stdout"));
    }

    #[test]
    fn non_interactive_auto_selection_uses_deterministic_first_file() {
        let files = sample_log_files();

        let selected = auto_select_log_file(&files).unwrap();

        assert_eq!(selected.path, PathBuf::from("/tmp/app.stdout"));
    }

    #[test]
    fn stderr_flag_prefers_stderr_redirect() {
        let files = sample_log_files();

        let selected = stderr_log_file(&files).unwrap();

        assert_eq!(selected.fd, LogFd::Stderr);
        assert_eq!(selected.path, PathBuf::from("/tmp/app.stderr"));
    }

    #[test]
    fn common_framework_logs_discovers_matching_dev_log_in_project_parent() {
        let root = temp_test_dir("matching-dev-log");
        let backend = root.join("backend");
        let log_dir = root.join(".dev-logs");
        std::fs::create_dir_all(&backend).unwrap();
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(log_dir.join("frontend.log"), "frontend").unwrap();
        std::fs::write(log_dir.join("backend-20260520.log"), "old").unwrap();
        std::fs::write(log_dir.join("backend.log"), "current").unwrap();

        let logs = common_framework_logs(&backend);

        assert_eq!(
            logs.first().map(|file| file.path.as_path()),
            Some(log_dir.join("backend.log").as_path())
        );
        assert!(logs
            .iter()
            .any(|file| file.path == log_dir.join("frontend.log")));
    }

    #[test]
    fn common_framework_logs_prefers_non_empty_component_rotation_over_empty_current_log() {
        let root = temp_test_dir("non-empty-rotation");
        let backend = root.join("backend");
        let log_dir = root.join(".dev-logs");
        std::fs::create_dir_all(&backend).unwrap();
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(log_dir.join("backend.log"), "").unwrap();
        std::fs::write(log_dir.join("backend-20260520.log"), "current output").unwrap();
        std::fs::write(log_dir.join("frontend.log"), "frontend").unwrap();

        let logs = common_framework_logs(&backend);

        assert_eq!(
            logs.first().map(|file| file.path.as_path()),
            Some(log_dir.join("backend-20260520.log").as_path())
        );
    }

    #[test]
    fn common_framework_logs_ignores_non_log_files_in_dev_log_dirs() {
        let root = temp_test_dir("ignore-non-log");
        let backend = root.join("backend");
        let log_dir = root.join(".dev-logs");
        std::fs::create_dir_all(&backend).unwrap();
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(log_dir.join("backend.txt"), "not a log").unwrap();

        let logs = common_framework_logs(&backend);

        assert!(logs.is_empty());
    }

    #[test]
    fn common_framework_logs_keeps_existing_framework_paths() {
        let root = temp_test_dir("existing-framework-paths");
        let log_dir = root.join("logs");
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(log_dir.join("development.log"), "rails").unwrap();

        let logs = common_framework_logs(&root);

        assert!(logs
            .iter()
            .any(|file| file.path == log_dir.join("development.log")));
    }

    fn sample_log_files() -> Vec<LogFile> {
        vec![
            LogFile {
                path: PathBuf::from("/tmp/app.stdout"),
                fd: LogFd::Stdout,
                kind: LogFileKind::Redirect,
                priority: 1,
            },
            LogFile {
                path: PathBuf::from("/tmp/app.stderr"),
                fd: LogFd::Stderr,
                kind: LogFileKind::Redirect,
                priority: 1,
            },
            LogFile {
                path: PathBuf::from("/repo/log/app.log"),
                fd: LogFd::File,
                kind: LogFileKind::LogFile,
                priority: 2,
            },
        ]
    }

    fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("kiri-logs-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }
}
