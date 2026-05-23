use crate::kill::{resolve_target, SystemResolver};
use crate::platform;
use std::collections::HashSet;
use std::io::{self, IsTerminal, Write};
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
    .filter_map(|relative| {
        let path = cwd.join(relative);
        path.exists().then_some(LogFile {
            path,
            fd: LogFd::File,
            kind: LogFileKind::Framework,
            priority: 3,
        })
    })
    .collect()
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
        print!("{output}");
        let status = Command::new(command)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        return LogCommandOutcome {
            exit_code: status.ok().and_then(|status| status.code()).unwrap_or(1),
            output: String::new(),
        };
    }

    match Command::new(command).args(args).output() {
        Ok(result) => {
            output.push_str(&String::from_utf8_lossy(&result.stdout));
            output.push_str(&String::from_utf8_lossy(&result.stderr));
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
            output.push_str(&stdout);
            output.push_str(&stderr);

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
}
