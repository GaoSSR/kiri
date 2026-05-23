use crate::model::{RawListenerEntry, RawProcessEntry, RawProcessInfo};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn get_listening_ports_raw() -> Vec<RawListenerEntry> {
    let output = match Command::new("ss").args(["-H", "-ltnp"]).output() {
        Ok(output) => output,
        Err(_) => return get_lsof_listening_ports_raw(),
    };

    if !output.status.success() {
        return get_lsof_listening_ports_raw();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_ss_listeners(&raw)
}

fn get_lsof_listening_ports_raw() -> Vec<RawListenerEntry> {
    let output = match Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_lsof_listeners(&raw)
}

pub fn parse_ss_listeners(raw: &str) -> Vec<RawListenerEntry> {
    let mut seen_ports = HashSet::new();
    let mut entries = Vec::new();

    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let port = match parse_port_from_address(parts[3]) {
            Some(port) => port,
            None => continue,
        };
        if !seen_ports.insert(port) {
            continue;
        }

        let Some(pid) = parse_pid_from_ss_line(line) else {
            continue;
        };

        entries.push(RawListenerEntry {
            port,
            pid,
            process_name: parse_process_name_from_ss_line(line)
                .unwrap_or_else(|| format!("pid-{pid}")),
        });
    }

    entries.sort_by_key(|entry| entry.port);
    entries
}

pub fn parse_lsof_listeners(raw: &str) -> Vec<RawListenerEntry> {
    let mut seen_ports = HashSet::new();
    let mut entries = Vec::new();

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let process_name = parts[0];
        let pid = match parts[1].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        let port = match parse_port_from_address(parts[8]) {
            Some(port) => port,
            None => continue,
        };

        if !seen_ports.insert(port) {
            continue;
        }

        entries.push(RawListenerEntry {
            port,
            pid,
            process_name: process_name.to_string(),
        });
    }

    entries.sort_by_key(|entry| entry.port);
    entries
}

pub fn batch_process_info(pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    if pids.is_empty() {
        return HashMap::new();
    }

    let pid_list = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let output = match Command::new("ps")
        .env("LC_ALL", "C")
        .args([
            "-p",
            &pid_list,
            "-o",
            "pid=,ppid=,stat=,rss=,lstart=,command=",
        ])
        .output()
    {
        Ok(output) => output,
        Err(_) => return HashMap::new(),
    };

    if !output.status.success() {
        return HashMap::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_process_info(&raw)
}

pub fn parse_process_info(raw: &str) -> HashMap<u32, RawProcessInfo> {
    let mut processes = HashMap::new();

    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let pid = match parts[0].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        let ppid = match parts[1].parse::<u32>() {
            Ok(ppid) => ppid,
            Err(_) => continue,
        };
        let rss_kb = match parts[3].parse::<u64>() {
            Ok(rss_kb) => rss_kb,
            Err(_) => continue,
        };

        let Some((lstart, command)) = parse_lstart_and_command(&parts[4..]) else {
            continue;
        };

        processes.insert(
            pid,
            RawProcessInfo {
                ppid,
                stat: parts[2].to_string(),
                rss_kb,
                lstart,
                command,
            },
        );
    }

    processes
}

pub fn batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    let mut cwd = HashMap::new();

    for pid in pids {
        let Ok(path) = std::fs::read_link(format!("/proc/{pid}/cwd")) else {
            continue;
        };
        cwd.insert(*pid, path);
    }

    cwd
}

pub fn get_all_processes_raw() -> Vec<RawProcessEntry> {
    let output = match Command::new("ps")
        .env("LC_ALL", "C")
        .args(["-eo", "pid=,pcpu=,pmem=,rss=,lstart=,command="])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_all_processes(&raw, std::process::id())
}

pub fn parse_all_processes(raw: &str, current_pid: u32) -> Vec<RawProcessEntry> {
    let mut processes = Vec::new();
    let mut seen = HashSet::new();

    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid = match parts[0].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        if pid <= 1 || pid == current_pid || !seen.insert(pid) {
            continue;
        }

        let cpu = match parts[1].parse::<f64>() {
            Ok(cpu) => cpu,
            Err(_) => continue,
        };
        let mem_percent = match parts[2].parse::<f64>() {
            Ok(mem_percent) => mem_percent,
            Err(_) => continue,
        };
        let rss_kb = match parts[3].parse::<u64>() {
            Ok(rss_kb) => rss_kb,
            Err(_) => continue,
        };
        let Some((lstart, command)) = parse_lstart_and_command(&parts[4..]) else {
            continue;
        };
        if command.is_empty() {
            continue;
        }

        processes.push(RawProcessEntry {
            pid,
            process_name: process_name_from_command(&command),
            cpu,
            mem_percent,
            rss_kb,
            lstart,
            command,
        });
    }

    processes
}

fn parse_port_from_address(address: &str) -> Option<u16> {
    let port_text = if let Some((_, port)) = address.rsplit_once("]:") {
        port
    } else {
        address.rsplit_once(':')?.1
    };
    if port_text.is_empty() || !port_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let port = port_text.parse::<u32>().ok()?;
    if port == 0 || port > u16::MAX as u32 {
        return None;
    }

    Some(port as u16)
}

fn parse_pid_from_ss_line(line: &str) -> Option<u32> {
    let after = line.split("pid=").nth(1)?;
    let pid_text = after
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    pid_text.parse::<u32>().ok()
}

fn parse_process_name_from_ss_line(line: &str) -> Option<String> {
    let start = line.find("users:((\"")? + "users:((\"".len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn process_name_from_command(command: &str) -> String {
    let first = command.split_whitespace().next().unwrap_or(command);
    Path::new(first)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(first)
        .to_string()
}

fn parse_lstart_and_command(parts: &[&str]) -> Option<(String, String)> {
    if parts.len() < 6 {
        return None;
    }

    let day_name = parts[0];
    let month = parts[1];
    let day = parts[2];
    let time = parts[3];
    let year = parts[4];
    if is_weekday(day_name)
        && english_month(month).is_some()
        && is_day(day)
        && is_time(time)
        && is_year(year)
    {
        return Some((format!("{month} {day} {time} {year}"), parts[5..].join(" ")));
    }

    None
}

fn is_weekday(value: &str) -> bool {
    matches!(value, "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun")
}

fn english_month(value: &str) -> Option<&'static str> {
    match value {
        "Jan" => Some("Jan"),
        "Feb" => Some("Feb"),
        "Mar" => Some("Mar"),
        "Apr" => Some("Apr"),
        "May" => Some("May"),
        "Jun" => Some("Jun"),
        "Jul" => Some("Jul"),
        "Aug" => Some("Aug"),
        "Sep" => Some("Sep"),
        "Oct" => Some("Oct"),
        "Nov" => Some("Nov"),
        "Dec" => Some("Dec"),
        _ => None,
    }
}

fn is_day(value: &str) -> bool {
    value.parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
}

fn is_year(value: &str) -> bool {
    value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_time(value: &str) -> bool {
    let mut parts = value.split(':');
    let Some(hour) = parts.next() else {
        return false;
    };
    let Some(minute) = parts.next() else {
        return false;
    };
    let Some(second) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    hour.parse::<u8>().is_ok_and(|value| value < 24)
        && minute.parse::<u8>().is_ok_and(|value| value < 60)
        && second.parse::<u8>().is_ok_and(|value| value < 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ss_ipv4_and_ipv6_listeners() {
        let raw = "\
LISTEN 0 4096 127.0.0.1:5432 0.0.0.0:* users:((\"postgres\",pid=1200,fd=7))
LISTEN 0 4096 [::1]:3000 [::]:* users:((\"node\",pid=1300,fd=23))
";

        let entries = parse_ss_listeners(raw);

        assert_eq!(
            entries,
            vec![
                RawListenerEntry {
                    port: 3000,
                    pid: 1300,
                    process_name: "node".to_string(),
                },
                RawListenerEntry {
                    port: 5432,
                    pid: 1200,
                    process_name: "postgres".to_string(),
                },
            ]
        );
    }

    #[test]
    fn ignores_ss_lines_without_pid_or_valid_port() {
        let raw = "\
LISTEN 0 4096 127.0.0.1:* 0.0.0.0:* users:((\"node\",pid=1300,fd=23))
LISTEN 0 4096 127.0.0.1:70000 0.0.0.0:* users:((\"node\",pid=1300,fd=23))
LISTEN 0 4096 127.0.0.1:3000 0.0.0.0:*
";

        assert!(parse_ss_listeners(raw).is_empty());
    }

    #[test]
    fn parses_process_info_and_all_processes() {
        let raw = "\
42872 1 S 184320 Mon May 20 14:12:44 2026 node /repo/app/server.js
";

        let processes = parse_process_info(raw);
        let process = processes.get(&42872).expect("process should parse");

        assert_eq!(process.ppid, 1);
        assert_eq!(process.stat, "S");
        assert_eq!(process.rss_kb, 184320);
        assert_eq!(process.lstart, "May 20 14:12:44 2026");
        assert_eq!(process.command, "node /repo/app/server.js");

        let all = parse_all_processes(
            "42872 12.5 1.0 184320 Mon May 20 14:12:44 2026 /usr/bin/node /repo/app/server.js",
            99999,
        );
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].process_name, "node");
        assert_eq!(all[0].cpu, 12.5);
    }
}
