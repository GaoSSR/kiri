use crate::model::{RawListenerEntry, RawProcessEntry, RawProcessInfo};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_listening_ports_raw() -> Vec<RawListenerEntry> {
    // TODO(phase-2): Port the Linux collector from port-whisperer src/platform/linux.js.
    Vec::new()
}

pub fn batch_process_info(_pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    // TODO(phase-3): Port Linux process enrichment from port-whisperer src/platform/linux.js.
    HashMap::new()
}

pub fn batch_cwd(_pids: &[u32]) -> HashMap<u32, PathBuf> {
    // TODO(phase-3): Port Linux cwd enrichment from port-whisperer src/platform/linux.js.
    HashMap::new()
}

pub fn get_all_processes_raw() -> Vec<RawProcessEntry> {
    // TODO(phase-10): Port Linux process listing after Linux support is made real.
    Vec::new()
}
