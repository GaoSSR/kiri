use crate::model::{RawListenerEntry, RawProcessInfo};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;

pub fn get_listening_ports_raw() -> Vec<RawListenerEntry> {
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

pub fn parse_lsof_listeners(raw: &str) -> Vec<RawListenerEntry> {
    let mut seen_ports = HashSet::new();
    let mut entries = Vec::new();

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let process_name = parts[0];
        let pid = match parts[1].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        let port = match parse_port_from_name_field(parts[8]) {
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

        processes.insert(
            pid,
            RawProcessInfo {
                ppid,
                stat: parts[2].to_string(),
                rss_kb,
                lstart: format!("{} {} {} {}", parts[5], parts[6], parts[7], parts[8]),
                command: parts[9..].join(" "),
            },
        );
    }

    processes
}

pub fn batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    if pids.is_empty() {
        return HashMap::new();
    }

    let pid_list = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let output = match Command::new("lsof")
        .args(["-a", "-d", "cwd", "-p", &pid_list])
        .output()
    {
        Ok(output) => output,
        Err(_) => return HashMap::new(),
    };

    if !output.status.success() {
        return HashMap::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_cwd_output(&raw)
}

pub fn parse_cwd_output(raw: &str) -> HashMap<u32, PathBuf> {
    let mut cwd = HashMap::new();

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid = match parts[1].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        let path = parts[8..].join(" ");
        if path.starts_with('/') {
            cwd.insert(pid, PathBuf::from(path));
        }
    }

    cwd
}

fn parse_port_from_name_field(name_field: &str) -> Option<u16> {
    let (_, port_text) = name_field.rsplit_once(':')?;
    if port_text.is_empty() || !port_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let port = port_text.parse::<u32>().ok()?;
    if port == 0 || port > u16::MAX as u32 {
        return None;
    }

    Some(port as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_listen_lines_with_port_pid_and_process_name() {
        let raw = "\
COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node    42872 user   23u  IPv6 0x123456789abcdef0      0t0  TCP *:3000 (LISTEN)
";

        let entries = parse_lsof_listeners(raw);

        assert_eq!(
            entries,
            vec![RawListenerEntry {
                port: 3000,
                pid: 42872,
                process_name: "node".to_string(),
            }]
        );
    }

    #[test]
    fn ignores_incomplete_lines_and_lines_without_ports() {
        let raw = "\
COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node
node    42872 user   23u  IPv6 0x123456789abcdef0      0t0  TCP *:* (LISTEN)
node    42872 user   23u  IPv6 0x123456789abcdef0      0t0  TCP localhost (LISTEN)
";

        let entries = parse_lsof_listeners(raw);

        assert!(entries.is_empty());
    }

    #[test]
    fn keeps_first_entry_when_multiple_listeners_report_same_port() {
        let raw = "\
COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node    42872 user   23u  IPv6 0x123456789abcdef0      0t0  TCP *:3000 (LISTEN)
python3 51000 user   24u  IPv4 0x123456789abcdef1      0t0  TCP 127.0.0.1:3000 (LISTEN)
";

        let entries = parse_lsof_listeners(raw);

        assert_eq!(
            entries,
            vec![RawListenerEntry {
                port: 3000,
                pid: 42872,
                process_name: "node".to_string(),
            }]
        );
    }

    #[test]
    fn parses_process_info_lines_with_expected_fields() {
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
    }

    #[test]
    fn ignores_incomplete_process_info_lines() {
        let raw = "\
42872 1 S 184320 Mon May 20
not-a-pid 1 S 184320 Mon May 20 14:12:44 2026 node
";

        let processes = parse_process_info(raw);

        assert!(processes.is_empty());
    }

    #[test]
    fn preserves_process_command_when_it_contains_spaces() {
        let raw = "\
51000 42872 S 2048 Fri May 22 09:01:02 2026 /usr/local/bin/node /repo/app/node_modules/.bin/vite --host 0.0.0.0
";

        let processes = parse_process_info(raw);
        let process = processes.get(&51000).expect("process should parse");

        assert_eq!(
            process.command,
            "/usr/local/bin/node /repo/app/node_modules/.bin/vite --host 0.0.0.0"
        );
    }

    #[test]
    fn parses_cwd_lines_with_absolute_path() {
        let raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
node    42872 user  cwd    DIR   1,18      640 1234 /Users/dev/project
";

        let cwd = parse_cwd_output(raw);

        assert_eq!(
            cwd.get(&42872).map(|path| path.as_path()),
            Some(std::path::Path::new("/Users/dev/project"))
        );
    }

    #[test]
    fn ignores_cwd_lines_without_absolute_paths_or_enough_fields() {
        let raw = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
node
node    42872 user  cwd    DIR   1,18      640 1234 relative/project
";

        let cwd = parse_cwd_output(raw);

        assert!(cwd.is_empty());
    }
}
