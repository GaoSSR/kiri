#[cfg(target_os = "macos")]
pub mod darwin;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::model::{RawListenerEntry, RawProcessInfo};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_listening_ports_raw() -> Vec<RawListenerEntry> {
    platform_get_listening_ports_raw()
}

pub fn batch_process_info(pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    platform_batch_process_info(pids)
}

pub fn batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    platform_batch_cwd(pids)
}

#[cfg(target_os = "macos")]
fn platform_get_listening_ports_raw() -> Vec<RawListenerEntry> {
    darwin::get_listening_ports_raw()
}

#[cfg(target_os = "macos")]
fn platform_batch_process_info(pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    darwin::batch_process_info(pids)
}

#[cfg(target_os = "macos")]
fn platform_batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    darwin::batch_cwd(pids)
}

#[cfg(target_os = "linux")]
fn platform_get_listening_ports_raw() -> Vec<RawListenerEntry> {
    linux::get_listening_ports_raw()
}

#[cfg(target_os = "linux")]
fn platform_batch_process_info(pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    linux::batch_process_info(pids)
}

#[cfg(target_os = "linux")]
fn platform_batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    linux::batch_cwd(pids)
}

#[cfg(target_os = "windows")]
fn platform_get_listening_ports_raw() -> Vec<RawListenerEntry> {
    windows::get_listening_ports_raw()
}

#[cfg(target_os = "windows")]
fn platform_batch_process_info(pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    windows::batch_process_info(pids)
}

#[cfg(target_os = "windows")]
fn platform_batch_cwd(pids: &[u32]) -> HashMap<u32, PathBuf> {
    windows::batch_cwd(pids)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_get_listening_ports_raw() -> Vec<RawListenerEntry> {
    Vec::new()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_batch_process_info(_pids: &[u32]) -> HashMap<u32, RawProcessInfo> {
    HashMap::new()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_batch_cwd(_pids: &[u32]) -> HashMap<u32, PathBuf> {
    HashMap::new()
}
