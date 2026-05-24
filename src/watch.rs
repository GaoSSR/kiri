use crate::model::PortInfo;
use crate::scanner::get_listening_ports;
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    New(Box<PortInfo>),
    Removed(u16),
}

pub fn run_watch_command() -> i32 {
    let running = Arc::new(AtomicBool::new(true));
    let handler_running = Arc::clone(&running);
    let _ = ctrlc::set_handler(move || {
        handler_running.store(false, Ordering::SeqCst);
    });

    println!("Kiri - watching for port changes");
    println!("Press Ctrl+C to stop\n");

    let mut previous = watched_ports(&get_listening_ports(false));
    while running.load(Ordering::SeqCst) {
        let current = get_listening_ports(false);
        let Some(events) =
            watch_events_after_scan(&previous, &current, running.load(Ordering::SeqCst))
        else {
            break;
        };
        for event in events {
            println!("{}", render_watch_event(&event));
        }
        previous = watched_ports(&current);
        sleep(Duration::from_secs(2));
    }

    println!("\nStopped watching.");
    0
}

fn watched_ports(ports: &[PortInfo]) -> HashSet<u16> {
    ports.iter().map(|port| port.port).collect()
}

fn watch_events_after_scan(
    previous: &HashSet<u16>,
    current: &[PortInfo],
    keep_running: bool,
) -> Option<Vec<WatchEvent>> {
    keep_running.then(|| diff_ports(previous, current))
}

pub fn diff_ports(previous: &HashSet<u16>, current: &[PortInfo]) -> Vec<WatchEvent> {
    let current_set = current.iter().map(|port| port.port).collect::<HashSet<_>>();
    let mut events = Vec::new();

    for port in current {
        if !previous.contains(&port.port) {
            events.push(WatchEvent::New(Box::new(port.clone())));
        }
    }

    let mut removed = previous
        .iter()
        .copied()
        .filter(|port| !current_set.contains(port))
        .collect::<Vec<_>>();
    removed.sort_unstable();
    events.extend(removed.into_iter().map(WatchEvent::Removed));

    events
}

fn render_watch_event(event: &WatchEvent) -> String {
    match event {
        WatchEvent::New(info) => {
            let project = info
                .project_name
                .as_deref()
                .map(|project| format!(" [{project}]"))
                .unwrap_or_default();
            let framework = info
                .framework
                .as_deref()
                .map(|framework| format!(" {framework}"))
                .unwrap_or_default();
            format!(
                "NEW :{} <- {}{}{}",
                info.port, info.process.name, project, framework
            )
        }
        WatchEvent::Removed(port) => format!("REMOVED :{port}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ProcessInfo, ProcessStatus, RawListenerEntry};

    #[test]
    fn detects_new_ports() {
        let previous = HashSet::new();
        let current = vec![port_info(3000, 10, "node")];

        let events = diff_ports(&previous, &current);

        assert_eq!(events, vec![WatchEvent::New(Box::new(current[0].clone()))]);
    }

    #[test]
    fn detects_removed_ports() {
        let previous = HashSet::from([3000, 5173]);
        let current = vec![port_info(5173, 10, "node")];

        let events = diff_ports(&previous, &current);

        assert_eq!(events, vec![WatchEvent::Removed(3000)]);
    }

    #[test]
    fn detects_no_changes() {
        let previous = HashSet::from([5173]);
        let current = vec![port_info(5173, 10, "node")];

        let events = diff_ports(&previous, &current);

        assert!(events.is_empty());
    }

    #[test]
    fn initial_snapshot_prevents_existing_ports_from_being_reported_as_new() {
        let current = vec![port_info(5173, 10, "node"), port_info(8080, 11, "java")];
        let previous = watched_ports(&current);

        let events = diff_ports(&previous, &current);

        assert!(events.is_empty());
    }

    #[test]
    fn stop_signal_after_scan_suppresses_interrupted_empty_snapshot() {
        let previous = HashSet::from([5173, 8080]);
        let current = Vec::new();

        let events = watch_events_after_scan(&previous, &current, false);

        assert_eq!(events, None);
    }

    fn port_info(port: u16, pid: u32, process_name: &str) -> PortInfo {
        let mut info = PortInfo::from(RawListenerEntry {
            port,
            pid,
            process_name: process_name.to_string(),
        });
        info.process = ProcessInfo {
            pid,
            name: process_name.to_string(),
            command: format!("{process_name} server.js"),
            ppid: None,
            stat: None,
            rss_kb: None,
        };
        info.status = ProcessStatus::Healthy;
        info
    }
}
