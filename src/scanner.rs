use crate::dev_process::is_developer_process;
use crate::docker::{batch_docker_info, detect_framework_from_image, DockerInfo};
use crate::framework::{detect_framework, detect_framework_from_command};
use crate::model::{PortInfo, ProcessStatus, RawListenerEntry, RawProcessInfo};
use crate::platform;
use chrono::{Local, NaiveDateTime, TimeZone};
use std::collections::{HashMap, HashSet};
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
    let has_docker = entries.iter().any(|entry| {
        let name = entry.process_name.to_ascii_lowercase();
        name.starts_with("com.docke") || name == "docker"
    });
    let docker_map = if has_docker {
        batch_docker_info()
    } else {
        HashMap::new()
    };

    enrich_entries(entries, &process_map, &cwd_map, &docker_map)
}

fn enrich_entries(
    entries: Vec<RawListenerEntry>,
    process_map: &HashMap<u32, RawProcessInfo>,
    cwd_map: &HashMap<u32, PathBuf>,
    docker_map: &HashMap<u16, DockerInfo>,
) -> Vec<PortInfo> {
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

                if info.framework.is_none() {
                    info.framework =
                        detect_framework_from_command(&process.command, &info.process.name)
                            .map(ToOwned::to_owned);
                }
            }

            if let Some(docker) = docker_map.get(&info.port) {
                info.process.name = "docker".to_string();
                info.project_name = Some(docker.name.clone());
                info.framework = Some(detect_framework_from_image(&docker.image).to_string());
                info.docker_container = Some(docker.name.clone());
                info.docker_image = Some(docker.image.clone());
                info.cwd = None;
            } else if let Some(cwd) = cwd_map.get(&info.process.pid) {
                let project_root = find_project_root(cwd);
                if info.framework.is_none() {
                    info.framework = detect_framework(&project_root).map(ToOwned::to_owned);
                }
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
        let root = unique_temp_dir("kiri-package-root");
        let nested = root.join("apps/web/src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();

        let found = find_project_root(&nested);

        assert_eq!(found, root);
    }

    #[test]
    fn find_project_root_supports_cargo_marker() {
        let root = unique_temp_dir("kiri-cargo-root");
        let nested = root.join("src/bin");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"sample\"\n").unwrap();

        let found = find_project_root(&nested);

        assert_eq!(found, root);
    }

    #[test]
    fn find_project_root_returns_original_cwd_when_no_marker_exists() {
        let cwd = unique_temp_dir("kiri-no-marker");
        fs::create_dir_all(&cwd).unwrap();

        let found = find_project_root(&cwd);

        assert_eq!(found, cwd);
    }

    #[test]
    fn docker_mapping_overrides_process_project_and_framework() {
        let project = unique_temp_dir("kiri-docker-project");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"dependencies":{"next":"latest"}}"#,
        )
        .unwrap();

        let entries = vec![RawListenerEntry {
            port: 5432,
            pid: 900,
            process_name: "com.docker.backend".to_string(),
        }];
        let mut processes = HashMap::new();
        processes.insert(
            900,
            RawProcessInfo {
                ppid: 1,
                stat: "S".to_string(),
                rss_kb: 2048,
                lstart: "May 22 09:00:00 2026".to_string(),
                command: "/Applications/Docker.app/Contents/MacOS/com.docker.backend".to_string(),
            },
        );
        let mut cwd = HashMap::new();
        cwd.insert(900, project);
        let mut docker = HashMap::new();
        docker.insert(
            5432,
            DockerInfo {
                name: "backend-postgres-1".to_string(),
                image: "postgres:16".to_string(),
            },
        );

        let ports = enrich_entries(entries, &processes, &cwd, &docker);
        let info = &ports[0];

        assert_eq!(info.process.name, "docker");
        assert_eq!(info.project_name.as_deref(), Some("backend-postgres-1"));
        assert_eq!(info.framework.as_deref(), Some("PostgreSQL"));
        assert_eq!(info.docker_image.as_deref(), Some("postgres:16"));
        assert!(info.cwd.is_none());
    }

    #[test]
    fn non_docker_listener_detects_framework_from_command() {
        let entries = vec![RawListenerEntry {
            port: 5173,
            pid: 902,
            process_name: "node".to_string(),
        }];
        let mut processes = HashMap::new();
        processes.insert(
            902,
            RawProcessInfo {
                ppid: 100,
                stat: "S".to_string(),
                rss_kb: 2048,
                lstart: "May 22 09:00:00 2026".to_string(),
                command: "pnpm vite --host 0.0.0.0".to_string(),
            },
        );
        let cwd = HashMap::new();
        let docker = HashMap::new();

        let ports = enrich_entries(entries, &processes, &cwd, &docker);
        let info = &ports[0];

        assert_eq!(info.framework.as_deref(), Some("Vite"));
    }

    #[test]
    fn non_docker_listener_detects_framework_from_project_root() {
        let project = unique_temp_dir("kiri-non-docker-package-project");
        let nested = project.join("apps/web/src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"dependencies":{"next":"latest"}}"#,
        )
        .unwrap();

        let entries = vec![RawListenerEntry {
            port: 3000,
            pid: 903,
            process_name: "node".to_string(),
        }];
        let processes = HashMap::new();
        let mut cwd = HashMap::new();
        cwd.insert(903, nested);
        let docker = HashMap::new();

        let ports = enrich_entries(entries, &processes, &cwd, &docker);
        let info = &ports[0];

        assert_eq!(info.framework.as_deref(), Some("Next.js"));
    }

    #[test]
    fn non_docker_listener_keeps_existing_cwd_project_behavior() {
        let project = unique_temp_dir("kiri-non-docker-project");
        let nested = project.join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(project.join("Cargo.toml"), "[package]\nname = \"sample\"\n").unwrap();

        let entries = vec![RawListenerEntry {
            port: 8080,
            pid: 901,
            process_name: "java".to_string(),
        }];
        let processes = HashMap::new();
        let mut cwd = HashMap::new();
        cwd.insert(901, nested);
        let docker = HashMap::new();

        let ports = enrich_entries(entries, &processes, &cwd, &docker);
        let info = &ports[0];

        assert_eq!(info.process.name, "java");
        assert_eq!(
            info.project_name.as_deref(),
            project.file_name().and_then(|name| name.to_str())
        );
        assert_eq!(info.framework.as_deref(), Some("Rust"));
        assert!(info.docker_image.is_none());
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}"))
    }
}
