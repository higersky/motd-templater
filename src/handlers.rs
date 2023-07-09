use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;
use lazy_static::lazy_static;
use procfs::process::Process;
use sysinfo::{CpuRefreshKind, DiskExt, System, SystemExt};

lazy_static! {
    static ref SYSINFO: System = {
        let mut sys = System::new();
        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());
        sys.refresh_memory();
        sys.refresh_disks_list();
        sys
    };
}

fn login_user() -> Result<String> {
    let mut p = Process::myself()?;
    loop {
        let parent = p.status()?.ppid;
        if let Ok(newp) = Process::new(parent) {
            let cmd = newp
                .cmdline()?
                .first()
                .map(|x| x.to_owned())
                .with_context(|| "Cannot get process cmdline")?;
            if cmd.starts_with("sshd") {
                let username = cmd
                    .split_ascii_whitespace()
                    .skip(1)
                    .next()
                    .and_then(|x| x.split("@").next())
                    .map(|x| x.trim().to_owned())
                    .with_context(|| "Unknown login user")?;
                if username.is_empty() {
                    anyhow::bail!("Unknown login user");
                } else {
                    return Ok(username);
                }
            }
            p = newp;
        } else {
            break;
        }
    }
    Ok(whoami::username())
}

fn load1() -> Result<String> {
    fs::read_to_string("/proc/loadavg")?
        .split_ascii_whitespace()
        .next()
        .map(|x| x.to_string())
        .with_context(|| "Failed to parse /proc/loadavg")
}

fn load5() -> Result<String> {
    fs::read_to_string("/proc/loadavg")?
        .split_ascii_whitespace()
        .nth(1)
        .map(|x| x.to_string())
        .with_context(|| "Failed to parse /proc/loadavg")
}

fn load15() -> Result<String> {
    fs::read_to_string("/proc/loadavg")?
        .split_ascii_whitespace()
        .nth(2)
        .map(|x| x.to_string())
        .with_context(|| "Failed to parse /proc/loadavg")
}

fn hostname() -> Result<String> {
    SYSINFO
        .host_name()
        .with_context(|| "Failed to get hostname")
}

fn kernel_version() -> Result<String> {
    SYSINFO
        .kernel_version()
        .with_context(|| "Failed to get kernel version")
}

fn memory_usage() -> Result<String> {
    Ok(format!(
        "{:.1}",
        SYSINFO.used_memory() as f64 / SYSINFO.total_memory() as f64 * 100.0
    ))
}

fn swap_usage() -> Result<String> {
    Ok(format!(
        "{:.1}",
        SYSINFO.used_swap() as f64 / SYSINFO.total_swap() as f64 * 100.0
    ))
}

fn cpu_cores() -> Result<String> {
    Ok(format!("{}", SYSINFO.cpus().len()))
}

fn root_disk_usage() -> Result<String> {
    for disk in SYSINFO.disks() {
        if disk.mount_point() == Path::new("/") {
            return Ok(format!(
                "{:.0}",
                100.0 - disk.available_space() as f64 / disk.total_space() as f64 * 100.0
            ));
        }
    }
    anyhow::bail!("Failed to get disk usage of /")
}

fn data_disk_usage() -> Result<String> {
    for disk in SYSINFO.disks() {
        if disk.mount_point() == Path::new("/data") {
            return Ok(format!(
                "{:.0}",
                100.0 - disk.available_space() as f64 / disk.total_space() as f64 * 100.0
            ));
        }
    }
    anyhow::bail!("Failed to get disk usage of /data")
}

fn cuda_version() -> Result<String> {
    // CUDA 11 or newer
    fs::read_to_string("/usr/local/cuda/version.json")
        .with_context(|| "Failed to read /usr/local/cuda/version.json")
        .and_then(|s| {
            serde_json::from_str::<serde_json::Value>(&s)
                .with_context(|| "Failed to parse /usr/local/cuda/version.json")
        })
        .and_then(|v| {
            v.get("cuda")
                .and_then(|x| x.get("version"))
                .and_then(|x| x.as_str())
                .map(|x| x.to_owned())
                .with_context(|| "Failed to parse version field in /usr/local/cuda/version.json")
        })
        .or_else(|_| {
            // Fallback: CUDA 10 or older
            fs::read_to_string("/usr/local/cuda/version.txt")
                .with_context(|| "Failed to read /usr/local/cuda/version.txt")
                .and_then(|s| {
                    s.strip_prefix("CUDA Version")
                        .map(|x| x.trim().to_owned())
                        .with_context(|| "Failed to parse cuda version in version.txt")
                })
        })
}

macro_rules! build_builtins {
    ($($x:ident), *) => {
        {
            let mut m: HashMap<String, fn() -> Result<String>> = HashMap::new();
            $(
                m.insert(stringify!($x).to_string(), $x);
            )*

            m
        }
    }
}

pub fn build_builtins() -> HashMap<String, fn() -> Result<String>> {
    build_builtins![
        load1,
        load5,
        load15,
        hostname,
        kernel_version,
        memory_usage,
        swap_usage,
        cpu_cores,
        root_disk_usage,
        data_disk_usage,
        cuda_version,
        login_user
    ]
}

pub fn warn_color(s: &str) -> String {
    let split = s.split_once(|c| !(char::is_numeric(c) || c == '.'));
    let f = if let Some((prefix, _)) = split {
        prefix.parse::<f64>()
    } else {
        s.parse::<f64>()
    };

    if let Ok(x) = f {
        if x > 90.0 {
            s.bright_red()
        } else if x > 85.0 {
            s.red()
        } else if x > 75.0 {
            s.yellow()
        } else {
            s.green()
        }
    } else {
        s.white()
    }
    .to_string()
}

pub fn underline(s: &str) -> String {
    s.underline().to_string()
}

pub fn bold(s: &str) -> String {
    s.bold().to_string()
}

/// Add a percent symbol % to string
pub fn percent(s: &str) -> String {
    format!("{s}%")
}

macro_rules! build_modifiers {
    ($($x:ident), *) => {
        {
            let mut m: HashMap<String, fn(&str) -> String> = HashMap::new();
            $(
                m.insert(stringify!($x).to_string(), $x);
            )*

            m
        }
    }
}

pub fn build_modifiers() -> HashMap<String, fn(&str) -> String> {
    build_modifiers![warn_color, bold, underline, percent]
}
