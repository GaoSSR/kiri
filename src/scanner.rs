use crate::dev_process::is_developer_process;
use crate::model::{PortInfo, ProcessStatus, RawListenerEntry};
use crate::platform;
use chrono::{Local, NaiveDateTime, TimeZone};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn get_listening_ports(show_all: bool) -> Vec<PortInfo> {
    let ports = collect_ports();
    ports
        .into_iter()
        .filter(|info| show_all || is_developer_process(&info.process.name, &info.process.command))
        .collect()
}

pub fn get_port_details(port: u16) -> Option<PortInfo> {
    collect_ports().into_iter().find(|info| info.port == port)
}

fn collect_ports() -> Vec<PortInfo> {
    let entries = platform::get_listening_ports_raw();
    let pids = unique_pids(&entries);
    let process_map = platform::batch_process_info(&pids);
    let cwd_map = platform::batch_cwd(&pids);

    let mut ports: Vec<PortInfo> = entries
        .into_iter()
        .map(|entry| {
            let mut info = PortInfo::from(entry);

            if let Some(process) = process_map.get(&info.process.pid) {
                info.process.command = process.command.clone();
                info.process.ppid = Some(process.ppid);
                info.process.stat = Some(process.stat.clone());
                info.process.rss_kb = Some(process.rss_kb);
                info.start_time = Some(process.lstart.clone());

                if process.stat.contains('Z') {
                    info.status = ProcessStatus::Zombie;
                } else if process.ppid == 1
                    && is_developer_process(&info.process.name, &process.command)
                {
                    info.status = ProcessStatus::Orphaned;
                }

                if process.rss_kb > 0 {
                    info.memory = Some(format_memory(process.rss_kb));
                }

                info.uptime = format_uptime_from_lstart(&process.lstart);
            }

            if let Some(cwd) = cwd_map.get(&info.process.pid) {
                let project_root = find_project_root(cwd);
                info.project_name = project_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(ToOwned::to_owned);
                info.cwd = Some(project_root);
            }

            info
        })
        .collect();

    ports.sort_by_key(|info| info.port);
    ports
}

fn unique_pids(entries: &[RawListenerEntry]) -> Vec<u32> {
    let mut seen = HashSet::new();
    let mut pids = Vec::new();

    for entry in entries {
        if seen.insert(entry.pid) {
            pids.push(entry.pid);
        }
    }

    pids
}

pub fn find_project_root(cwd: &Path) -> PathBuf {
    const MARKERS: &[&str] = &[
        "package.json",
        "Cargo.toml",
        "go.mod",
        "pyproject.toml",
        "Gemfile",
        "pom.xml",
        "build.gradle",
    ];

    let original = cwd.to_path_buf();
    let mut current = cwd.to_path_buf();

    for _ in 0..15 {
        if MARKERS.iter().any(|marker| current.join(marker).exists()) {
            return current;
        }

        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent.to_path_buf();
    }

    original
}

fn format_memory(rss_kb: u64) -> String {
    if rss_kb > 1_048_576 {
        format!("{:.1} GB", rss_kb as f64 / 1_048_576_f64)
    } else if rss_kb > 1024 {
        format!("{:.1} MB", rss_kb as f64 / 1024_f64)
    } else {
        format!("{rss_kb} KB")
    }
}

fn format_uptime_from_lstart(lstart: &str) -> Option<String> {
    let start = NaiveDateTime::parse_from_str(lstart, "%b %e %H:%M:%S %Y").ok()?;
    let start = Local.from_local_datetime(&start).single()?;
    let elapsed = Local::now().signed_duration_since(start);
    if elapsed.num_seconds() < 0 {
        return None;
    }

    let seconds = elapsed.num_seconds();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        Some(format!("{}d {}h", days, hours % 24))
    } else if hours > 0 {
        Some(format!("{}h {}m", hours, minutes % 60))
    } else if minutes > 0 {
        Some(format!("{}m {}s", minutes, seconds % 60))
    } else {
        Some(format!("{seconds}s"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn find_project_root_walks_up_to_nearest_marker() {
        let root = unique_temp_dir("devports-package-root");
        let nested = root.join("apps/web/src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();

        let found = find_project_root(&nested);

        assert_eq!(found, root);
    }

    #[test]
    fn find_project_root_supports_cargo_marker() {
        let root = unique_temp_dir("devports-cargo-root");
        let nested = root.join("src/bin");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"sample\"\n").unwrap();

        let found = find_project_root(&nested);

        assert_eq!(found, root);
    }

    #[test]
    fn find_project_root_returns_original_cwd_when_no_marker_exists() {
        let cwd = unique_temp_dir("devports-no-marker");
        fs::create_dir_all(&cwd).unwrap();

        let found = find_project_root(&cwd);

        assert_eq!(found, cwd);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}"))
    }
}
