use crate::model::PortInfo;
use crate::scanner::get_port_details;
use std::process::{Command, Stdio};

const MAX_RANGE_SPAN: u32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
}

impl KillSignal {
    fn name(self) -> &'static str {
        match self {
            Self::Term => "SIGTERM",
            Self::Kill => "SIGKILL",
        }
    }

    fn kill_flag(self) -> &'static str {
        match self {
            Self::Term => "-TERM",
            Self::Kill => "-KILL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillRequest {
    pub signal: KillSignal,
    pub targets: Vec<KillTarget>,
    pub has_range: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KillTarget {
    pub value: u32,
    pub from_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KillParseError {
    MissingTarget,
    InvalidTarget(String),
    InvalidRangeOrder(String),
    RangeTooLarge(String),
    RangeOutOfBounds(String),
}

impl KillParseError {
    fn message(&self) -> String {
        match self {
            Self::MissingTarget => usage_text(),
            Self::InvalidTarget(value) => format!("  x \"{value}\" is not a valid port/PID\n"),
            Self::InvalidRangeOrder(value) => {
                format!("  x Invalid range: {value} (start must be less than end)\n")
            }
            Self::RangeTooLarge(value) => {
                format!("  x Range too large: {value} (max 1000 ports)\n")
            }
            Self::RangeOutOfBounds(value) => {
                format!("  x Invalid range: {value} (ports must be 1-65535)\n")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KillVia {
    Port,
    Pid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    pub pid: u32,
    pub via: KillVia,
    pub port: Option<u16>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillCommandOutcome {
    pub exit_code: i32,
    pub output: String,
}

pub trait TargetResolver {
    fn listener_on_port(&self, port: u16) -> Option<PortInfo>;
    fn pid_exists(&self, pid: u32) -> bool;
}

pub trait ProcessKiller {
    fn kill(&self, pid: u32, signal: KillSignal) -> bool;
}

pub struct SystemResolver;

impl TargetResolver for SystemResolver {
    fn listener_on_port(&self, port: u16) -> Option<PortInfo> {
        get_port_details(port)
    }

    fn pid_exists(&self, pid: u32) -> bool {
        pid_exists(pid)
    }
}

pub struct SystemKiller;

impl ProcessKiller for SystemKiller {
    fn kill(&self, pid: u32, signal: KillSignal) -> bool {
        kill_process(pid, signal)
    }
}

pub fn run_kill_command(args: &[String]) -> KillCommandOutcome {
    execute_kill_command(args, &SystemResolver, &SystemKiller)
}

pub fn execute_kill_command<R, K>(args: &[String], resolver: &R, killer: &K) -> KillCommandOutcome
where
    R: TargetResolver,
    K: ProcessKiller,
{
    let request = match parse_kill_args(args) {
        Ok(request) => request,
        Err(error) => {
            return KillCommandOutcome {
                exit_code: 1,
                output: error.message(),
            };
        }
    };

    let mut output = String::new();
    let mut any_failed = false;
    let mut killed = 0usize;
    let mut empty = 0usize;

    for target in &request.targets {
        let Some(resolved) = resolve_target(target.value, resolver) else {
            if target.from_range {
                empty += 1;
                continue;
            }

            if target.value <= u16::MAX as u32 {
                output.push_str(&format!(
                    "  x No listener on :{} and no process with PID {}\n",
                    target.value, target.value
                ));
            } else {
                output.push_str(&format!("  x No process with PID {}\n", target.value));
            }
            any_failed = true;
            continue;
        };

        let label = resolved_label(&resolved);
        output.push_str(&format!("  Killing {label}\n"));
        if killer.kill(resolved.pid, request.signal) {
            output.push_str(&format!("  OK Sent {} to {label}\n", request.signal.name()));
            killed += 1;
        } else {
            output.push_str(&format!(
                "  x Failed. Try: sudo kill{} {}\n",
                if request.signal == KillSignal::Kill {
                    " -9"
                } else {
                    ""
                },
                resolved.pid
            ));
            any_failed = true;
        }
    }

    if request.has_range {
        let mut parts = Vec::new();
        if killed > 0 {
            parts.push(format!("{killed} killed"));
        }
        if empty > 0 {
            parts.push(format!("{empty} empty"));
        }
        if any_failed {
            parts.push("some failed".to_string());
        }
        output.push_str(&format!("  Range summary: {}\n", parts.join(", ")));
    }

    KillCommandOutcome {
        exit_code: if any_failed { 1 } else { 0 },
        output,
    }
}

pub fn parse_kill_args(args: &[String]) -> Result<KillRequest, KillParseError> {
    let mut signal = KillSignal::Term;
    let mut raw_targets = Vec::new();

    for arg in args {
        if arg == "-f" || arg == "--force" {
            signal = KillSignal::Kill;
        } else {
            raw_targets.push(arg.as_str());
        }
    }

    if raw_targets.is_empty() {
        return Err(KillParseError::MissingTarget);
    }

    let mut targets = Vec::new();
    let mut has_range = false;

    for raw in raw_targets {
        if let Some((start, end)) = parse_range(raw)? {
            has_range = true;
            for value in start..=end {
                targets.push(KillTarget {
                    value,
                    from_range: true,
                });
            }
        } else {
            targets.push(KillTarget {
                value: parse_number(raw)?,
                from_range: false,
            });
        }
    }

    Ok(KillRequest {
        signal,
        targets,
        has_range,
    })
}

fn parse_range(raw: &str) -> Result<Option<(u32, u32)>, KillParseError> {
    let Some((start_raw, end_raw)) = raw.split_once('-') else {
        return Ok(None);
    };

    let start =
        parse_range_number(start_raw).map_err(|_| KillParseError::InvalidTarget(raw.into()))?;
    let end = parse_range_number(end_raw).map_err(|_| KillParseError::InvalidTarget(raw.into()))?;

    if start > end {
        return Err(KillParseError::InvalidRangeOrder(raw.into()));
    }
    if end - start > MAX_RANGE_SPAN {
        return Err(KillParseError::RangeTooLarge(raw.into()));
    }
    if start < 1 || end > u16::MAX as u32 {
        return Err(KillParseError::RangeOutOfBounds(raw.into()));
    }

    Ok(Some((start, end)))
}

fn parse_number(raw: &str) -> Result<u32, KillParseError> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(KillParseError::InvalidTarget(raw.into()));
    }

    let value = raw
        .parse::<u32>()
        .map_err(|_| KillParseError::InvalidTarget(raw.into()))?;
    if value < 1 {
        return Err(KillParseError::InvalidTarget(raw.into()));
    }

    Ok(value)
}

fn parse_range_number(raw: &str) -> Result<u32, KillParseError> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(KillParseError::InvalidTarget(raw.into()));
    }

    raw.parse::<u32>()
        .map_err(|_| KillParseError::InvalidTarget(raw.into()))
}

pub fn resolve_target<R>(value: u32, resolver: &R) -> Option<ResolvedTarget>
where
    R: TargetResolver,
{
    if value < 1 {
        return None;
    }

    if value <= u16::MAX as u32 {
        if let Some(info) = resolver.listener_on_port(value as u16) {
            return Some(ResolvedTarget {
                pid: info.process.pid,
                via: KillVia::Port,
                port: Some(value as u16),
                process_name: Some(info.process.name),
            });
        }
    }

    if resolver.pid_exists(value) {
        return Some(ResolvedTarget {
            pid: value,
            via: KillVia::Pid,
            port: None,
            process_name: None,
        });
    }

    None
}

pub fn pid_exists(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn kill_process(pid: u32, signal: KillSignal) -> bool {
    Command::new("kill")
        .args([signal.kill_flag(), &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn resolved_label(resolved: &ResolvedTarget) -> String {
    match resolved.via {
        KillVia::Port => format!(
            ":{} - {} (PID {})",
            resolved.port.unwrap_or_default(),
            resolved.process_name.as_deref().unwrap_or("unknown"),
            resolved.pid
        ),
        KillVia::Pid => format!("PID {}", resolved.pid),
    }
}

fn usage_text() -> String {
    [
        "  Usage: devports kill [-f|--force] <port|pid|range> [port|pid|range...]",
        "  Kills listener on port (1-65535), or process by PID. Use -f for SIGKILL.",
        "  Ranges: devports kill 3000-3010",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ProcessInfo, ProcessStatus};
    use std::collections::{HashMap, HashSet};
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Default)]
    struct MockResolver {
        ports: HashMap<u16, PortInfo>,
        pids: HashSet<u32>,
    }

    impl TargetResolver for MockResolver {
        fn listener_on_port(&self, port: u16) -> Option<PortInfo> {
            self.ports.get(&port).cloned()
        }

        fn pid_exists(&self, pid: u32) -> bool {
            self.pids.contains(&pid)
        }
    }

    #[derive(Default)]
    struct MockKiller {
        failures: HashSet<u32>,
    }

    impl ProcessKiller for MockKiller {
        fn kill(&self, pid: u32, _signal: KillSignal) -> bool {
            !self.failures.contains(&pid)
        }
    }

    #[test]
    fn resolve_target_prefers_listener_port_when_port_matches() {
        let mut resolver = MockResolver::default();
        resolver.ports.insert(3000, port_info(3000, 4242, "node"));
        resolver.pids.insert(3000);

        let resolved = resolve_target(3000, &resolver).expect("target should resolve");

        assert_eq!(resolved.pid, 4242);
        assert_eq!(resolved.via, KillVia::Port);
        assert_eq!(resolved.port, Some(3000));
    }

    #[test]
    fn resolve_target_falls_back_to_pid_when_no_port_listener_exists() {
        let mut resolver = MockResolver::default();
        resolver.pids.insert(4242);

        let resolved = resolve_target(4242, &resolver).expect("pid should resolve");

        assert_eq!(resolved.pid, 4242);
        assert_eq!(resolved.via, KillVia::Pid);
        assert_eq!(resolved.port, None);
    }

    #[test]
    fn rejects_invalid_targets_and_missing_args() {
        assert!(matches!(
            parse_kill_args(&[]),
            Err(KillParseError::MissingTarget)
        ));
        assert!(matches!(
            parse_kill_args(&strings(["abc"])),
            Err(KillParseError::InvalidTarget(_))
        ));
        assert!(matches!(
            parse_kill_args(&strings(["0"])),
            Err(KillParseError::InvalidTarget(_))
        ));
    }

    #[test]
    fn parses_ranges_and_rejects_invalid_ranges() {
        let request = parse_kill_args(&strings(["3000-3002"])).unwrap();
        assert_eq!(
            request.targets,
            vec![
                KillTarget {
                    value: 3000,
                    from_range: true
                },
                KillTarget {
                    value: 3001,
                    from_range: true
                },
                KillTarget {
                    value: 3002,
                    from_range: true
                }
            ]
        );
        assert!(request.has_range);

        assert!(matches!(
            parse_kill_args(&strings(["3002-3000"])),
            Err(KillParseError::InvalidRangeOrder(_))
        ));
        assert!(matches!(
            parse_kill_args(&strings(["3000-4001"])),
            Err(KillParseError::RangeTooLarge(_))
        ));
        assert!(matches!(
            parse_kill_args(&strings(["65535-65536"])),
            Err(KillParseError::RangeOutOfBounds(_))
        ));
        assert!(matches!(
            parse_kill_args(&strings(["0-2"])),
            Err(KillParseError::RangeOutOfBounds(_))
        ));
    }

    #[test]
    fn counts_empty_ports_inside_ranges_without_failing() {
        let resolver = MockResolver::default();
        let killer = MockKiller::default();

        let outcome = execute_kill_command(&strings(["3000-3001"]), &resolver, &killer);

        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.output.contains("Range summary: 2 empty"));
    }

    #[test]
    fn parses_force_flags() {
        assert_eq!(
            parse_kill_args(&strings(["-f", "3000"])).unwrap().signal,
            KillSignal::Kill
        );
        assert_eq!(
            parse_kill_args(&strings(["--force", "3000"]))
                .unwrap()
                .signal,
            KillSignal::Kill
        );
    }

    #[test]
    fn executes_multiple_targets() {
        let mut resolver = MockResolver::default();
        resolver.ports.insert(3000, port_info(3000, 1111, "node"));
        resolver.ports.insert(5173, port_info(5173, 2222, "node"));
        let killer = MockKiller::default();

        let outcome = execute_kill_command(&strings(["3000", "5173"]), &resolver, &killer);

        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.output.contains("Sent SIGTERM to :3000"));
        assert!(outcome.output.contains("Sent SIGTERM to :5173"));
    }

    #[test]
    #[cfg(unix)]
    fn kill_process_terminates_only_spawned_child() {
        let mut child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("sleep should spawn");
        let pid = child.id();

        assert!(pid_exists(pid));
        assert!(kill_process(pid, KillSignal::Term));

        let mut exited = false;
        for _ in 0..20 {
            if child.try_wait().unwrap().is_some() {
                exited = true;
                break;
            }
            sleep(Duration::from_millis(100));
        }

        if !exited {
            let _ = kill_process(pid, KillSignal::Kill);
            let _ = child.wait();
        }

        assert!(exited, "spawned child should terminate after SIGTERM");
    }

    fn port_info(port: u16, pid: u32, process_name: &str) -> PortInfo {
        let mut info = PortInfo::from(crate::model::RawListenerEntry {
            port,
            pid,
            process_name: process_name.to_string(),
        });
        info.process = ProcessInfo {
            pid,
            name: process_name.to_string(),
            command: String::new(),
            ppid: None,
            stat: None,
            rss_kb: None,
        };
        info.status = ProcessStatus::Healthy;
        info
    }

    fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
}
