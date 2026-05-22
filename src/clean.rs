use crate::kill::{KillSignal, ProcessKiller, SystemKiller};
use crate::model::{PortInfo, ProcessStatus};
use crate::scanner::get_listening_ports;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanCommandOutcome {
    pub exit_code: i32,
    pub output: String,
}

pub fn run_clean_command() -> CleanCommandOutcome {
    let candidates = find_clean_candidates(&get_listening_ports(false));
    if candidates.is_empty() {
        return CleanCommandOutcome {
            exit_code: 0,
            output: "No orphaned or zombie developer processes found.\n".to_string(),
        };
    }

    let mut output = render_clean_candidates(&candidates);
    print!("{output}");
    print!("Kill all? [y/N] ");
    let _ = io::stdout().flush();

    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() || !is_yes(&answer) {
        output.clear();
        output.push_str("Aborted.\n");
        return CleanCommandOutcome {
            exit_code: 0,
            output,
        };
    }

    execute_clean(&candidates, &SystemKiller)
}

pub fn find_clean_candidates(ports: &[PortInfo]) -> Vec<PortInfo> {
    ports
        .iter()
        .filter(|port| matches!(port.status, ProcessStatus::Orphaned | ProcessStatus::Zombie))
        .cloned()
        .collect()
}

pub fn execute_clean<K>(candidates: &[PortInfo], killer: &K) -> CleanCommandOutcome
where
    K: ProcessKiller,
{
    let mut output = String::new();
    let mut killed = 0usize;
    let mut failed = 0usize;

    for candidate in candidates {
        let pid = candidate.process.pid;
        if killer.kill(pid, KillSignal::Term) {
            output.push_str(&format!("OK cleaned :{} PID {}\n", candidate.port, pid));
            killed += 1;
        } else {
            output.push_str(&format!(
                "x failed to clean :{} PID {}. Try: sudo kill -9 {}\n",
                candidate.port, pid, pid
            ));
            failed += 1;
        }
    }

    output.push_str(&format!(
        "Cleaned {killed} process{}",
        if killed == 1 { "" } else { "es" }
    ));
    if failed > 0 {
        output.push_str(&format!(
            ", failed {failed} process{}",
            if failed == 1 { "" } else { "es" }
        ));
    }
    output.push('\n');

    CleanCommandOutcome {
        exit_code: if failed > 0 { 1 } else { 0 },
        output,
    }
}

pub fn render_clean_candidates(candidates: &[PortInfo]) -> String {
    let mut output = format!(
        "Found {} orphaned/zombie developer process{}:\n",
        candidates.len(),
        if candidates.len() == 1 { "" } else { "es" }
    );
    for candidate in candidates {
        output.push_str(&format!(
            "- :{} {} (PID {}, {})\n",
            candidate.port,
            candidate.process.name,
            candidate.process.pid,
            candidate.status.as_str()
        ));
    }
    output
}

fn is_yes(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("y") || value.trim().eq_ignore_ascii_case("yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ProcessInfo, RawListenerEntry};
    use std::cell::RefCell;

    #[derive(Default)]
    struct RecordingKiller {
        killed: RefCell<Vec<u32>>,
    }

    impl ProcessKiller for RecordingKiller {
        fn kill(&self, pid: u32, _signal: KillSignal) -> bool {
            self.killed.borrow_mut().push(pid);
            true
        }
    }

    #[test]
    fn filters_only_orphaned_and_zombie_candidates() {
        let ports = vec![
            port_info(3000, 10, ProcessStatus::Healthy),
            port_info(3001, 11, ProcessStatus::Orphaned),
            port_info(3002, 12, ProcessStatus::Zombie),
        ];

        let candidates = find_clean_candidates(&ports);

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.process.pid)
                .collect::<Vec<_>>(),
            vec![11, 12]
        );
    }

    #[test]
    fn rendering_candidates_does_not_kill_before_confirmation() {
        let killer = RecordingKiller::default();
        let candidates = vec![port_info(3001, 11, ProcessStatus::Orphaned)];

        let output = render_clean_candidates(&candidates);

        assert!(output.contains(":3001"));
        assert!(killer.killed.borrow().is_empty());
    }

    #[test]
    fn execute_clean_kills_only_given_candidates() {
        let killer = RecordingKiller::default();
        let candidates = vec![port_info(3001, 11, ProcessStatus::Orphaned)];

        let outcome = execute_clean(&candidates, &killer);

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(*killer.killed.borrow(), vec![11]);
    }

    fn port_info(port: u16, pid: u32, status: ProcessStatus) -> PortInfo {
        let mut info = PortInfo::from(RawListenerEntry {
            port,
            pid,
            process_name: "node".to_string(),
        });
        info.process = ProcessInfo {
            pid,
            name: "node".to_string(),
            command: "node server.js".to_string(),
            ppid: Some(1),
            stat: None,
            rss_kb: None,
        };
        info.status = status;
        info
    }
}
