use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawListenerEntry {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProcessInfo {
    pub ppid: u32,
    pub stat: String,
    pub rss_kb: u64,
    pub lstart: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub ppid: Option<u32>,
    pub stat: Option<String>,
    pub rss_kb: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Healthy,
    Orphaned,
    Zombie,
    Unknown,
}

impl ProcessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Orphaned => "orphaned",
            Self::Zombie => "zombie",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    pub port: u16,
    pub process: ProcessInfo,
    pub status: ProcessStatus,
    pub cwd: Option<PathBuf>,
    pub project_name: Option<String>,
    pub framework: Option<String>,
    pub docker_image: Option<String>,
    pub docker_container: Option<String>,
    pub memory: Option<String>,
    pub uptime: Option<String>,
    pub start_time: Option<String>,
}

impl From<RawListenerEntry> for PortInfo {
    fn from(entry: RawListenerEntry) -> Self {
        Self {
            port: entry.port,
            process: ProcessInfo {
                pid: entry.pid,
                name: entry.process_name,
                command: String::new(),
                ppid: None,
                stat: None,
                rss_kb: None,
            },
            status: ProcessStatus::Healthy,
            cwd: None,
            project_name: None,
            framework: None,
            docker_image: None,
            docker_container: None,
            memory: None,
            uptime: None,
            start_time: None,
        }
    }
}
