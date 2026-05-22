use crate::dev_process::is_developer_process;
use crate::framework::{detect_framework, detect_framework_from_command};
use crate::model::{ProcessListInfo, RawProcessEntry};
use crate::platform;
use crate::scanner::find_project_root;
use chrono::{Local, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn get_all_processes(show_all: bool) -> Vec<ProcessListInfo> {
    let entries = platform::get_all_processes_raw();
    let cwd_map = cwd_for_non_docker_processes(&entries);
    let mut processes = enrich_process_entries(entries, &cwd_map);

    if !show_all {
        processes.retain(|process| is_developer_process(&process.process_name, &process.command));
        processes = collapse_docker_processes(processes);
    }

    sort_processes_by_cpu(&mut processes);
    processes
}

fn cwd_for_non_docker_processes(entries: &[RawProcessEntry]) -> HashMap<u32, PathBuf> {
    let pids = entries
        .iter()
        .filter(|entry| !is_docker_process_name(&entry.process_name))
        .map(|entry| entry.pid)
        .collect::<Vec<_>>();

    platform::batch_cwd(&pids)
}

pub fn enrich_process_entries(
    entries: Vec<RawProcessEntry>,
    cwd_map: &HashMap<u32, PathBuf>,
) -> Vec<ProcessListInfo> {
    entries
        .into_iter()
        .map(|entry| {
            let mut process = ProcessListInfo {
                pid: entry.pid,
                process_name: entry.process_name.clone(),
                command: entry.command.clone(),
                description: summarize_command(&entry.command, &entry.process_name),
                cpu: entry.cpu,
                rss_kb: entry.rss_kb,
                memory: (entry.rss_kb > 0).then(|| format_memory(entry.rss_kb)),
                cwd: None,
                project_name: None,
                framework: detect_framework_from_command(&entry.command, &entry.process_name)
                    .map(ToOwned::to_owned),
                uptime: format_uptime_from_lstart(&entry.lstart),
            };

            if let Some(cwd) = cwd_map.get(&entry.pid) {
                let project_root = find_project_root(cwd);
                if process.framework.is_none() {
                    process.framework = detect_framework(&project_root).map(ToOwned::to_owned);
                }
                process.project_name = project_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(ToOwned::to_owned);
                process.cwd = Some(project_root);
            }

            process
        })
        .collect()
}

pub fn collapse_docker_processes(processes: Vec<ProcessListInfo>) -> Vec<ProcessListInfo> {
    let mut docker = Vec::new();
    let mut non_docker = Vec::new();

    for process in processes {
        if is_docker_process_name(&process.process_name) {
            docker.push(process);
        } else {
            non_docker.push(process);
        }
    }

    if let Some(first) = docker.first() {
        let total_cpu = docker.iter().map(|process| process.cpu).sum::<f64>();
        let total_rss = docker.iter().map(|process| process.rss_kb).sum::<u64>();
        non_docker.push(ProcessListInfo {
            pid: first.pid,
            process_name: "Docker".to_string(),
            command: String::new(),
            description: format!(
                "{} process{}",
                docker.len(),
                if docker.len() == 1 { "" } else { "es" }
            ),
            cpu: total_cpu,
            rss_kb: total_rss,
            memory: (total_rss > 0).then(|| format_memory(total_rss)),
            cwd: None,
            project_name: None,
            framework: Some("Docker".to_string()),
            uptime: first.uptime.clone(),
        });
    }

    non_docker
}

pub fn sort_processes_by_cpu(processes: &mut [ProcessListInfo]) {
    processes.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.pid.cmp(&b.pid))
    });
}

pub fn summarize_command(command: &str, process_name: &str) -> String {
    let mut meaningful = Vec::new();

    for (index, part) in command.split_whitespace().enumerate() {
        if index == 0 || part.starts_with('-') {
            continue;
        }

        if part.contains('/') {
            meaningful.push(
                Path::new(part)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(part)
                    .to_string(),
            );
        } else {
            meaningful.push(part.to_string());
        }

        if meaningful.len() >= 3 {
            break;
        }
    }

    if meaningful.is_empty() {
        process_name.to_string()
    } else {
        meaningful.join(" ")
    }
}

fn is_docker_process_name(process_name: &str) -> bool {
    process_name.starts_with("com.docke")
        || process_name.starts_with("Docker")
        || process_name == "docker"
        || process_name == "docker-sandbox"
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn filters_dev_processes_and_keeps_non_dev_outside_callers() {
        assert!(is_developer_process("node", "node server.js"));
        assert!(!is_developer_process(
            "Slack",
            "/Applications/Slack.app/Slack"
        ));
    }

    #[test]
    fn enriches_processes_with_project_framework_and_description() {
        let project = unique_temp_dir("kiri-process-project");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"dependencies":{"vite":"latest"}}"#,
        )
        .unwrap();

        let entries = vec![RawProcessEntry {
            pid: 101,
            process_name: "node".to_string(),
            cpu: 2.5,
            mem_percent: 1.0,
            rss_kb: 4096,
            lstart: "May 20 14:12:44 2026".to_string(),
            command: "/usr/local/bin/node /repo/app/node_modules/.bin/vite --host".to_string(),
        }];
        let mut cwd = HashMap::new();
        cwd.insert(101, project.clone());

        let processes = enrich_process_entries(entries, &cwd);

        assert_eq!(processes[0].process_name, "node");
        assert_eq!(processes[0].description, "vite");
        assert_eq!(
            processes[0].project_name,
            project
                .file_name()
                .and_then(|v| v.to_str())
                .map(str::to_string)
        );
        assert_eq!(processes[0].framework.as_deref(), Some("Vite"));
        assert_eq!(processes[0].memory.as_deref(), Some("4.0 MB"));
    }

    #[test]
    fn collapses_docker_processes_into_one_summary() {
        let processes = vec![
            process_info(10, "com.docker.backend", 1.0, 1024),
            process_info(11, "docker-sandbox", 2.5, 2048),
            process_info(12, "node", 0.5, 4096),
        ];

        let collapsed = collapse_docker_processes(processes);

        assert_eq!(collapsed.len(), 2);
        let docker = collapsed
            .iter()
            .find(|process| process.process_name == "Docker")
            .expect("docker summary should exist");
        assert_eq!(docker.description, "2 processes");
        assert_eq!(docker.framework.as_deref(), Some("Docker"));
        assert_eq!(docker.cpu, 3.5);
        assert_eq!(docker.memory.as_deref(), Some("3.0 MB"));
    }

    #[test]
    fn sorts_processes_by_cpu_descending_then_pid() {
        let mut processes = vec![
            process_info(12, "node", 1.0, 1024),
            process_info(10, "python3", 2.0, 1024),
            process_info(11, "java", 2.0, 1024),
        ];

        sort_processes_by_cpu(&mut processes);

        assert_eq!(
            processes
                .iter()
                .map(|process| process.pid)
                .collect::<Vec<_>>(),
            vec![10, 11, 12]
        );
    }

    fn process_info(pid: u32, process_name: &str, cpu: f64, rss_kb: u64) -> ProcessListInfo {
        ProcessListInfo {
            pid,
            process_name: process_name.to_string(),
            command: process_name.to_string(),
            description: process_name.to_string(),
            cpu,
            rss_kb,
            memory: Some(format_memory(rss_kb)),
            cwd: None,
            project_name: None,
            framework: None,
            uptime: None,
        }
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}"))
    }
}
