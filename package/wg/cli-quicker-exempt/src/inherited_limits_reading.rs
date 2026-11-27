//! Reading the limit files of the cgroups a process leaves,
//! and merging them into the limits its new scope must carry.

/// The parser of each file these readings read.
use super::limit_file_parsing::{
    parse_cpu_max,
    parse_io_max,
    parse_io_weight,
    parse_limit,
};
/// The records these readings build.
use super::{
    DeviceLimits,
    InheritedLimits,
};
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The per-device limits keyed by device.
use std::collections::BTreeMap;
/// Syscall and parse failures.
use std::io;
/// Cgroup directories.
use std::path::Path;

/// Reads the `io.max` file of one cgroup directory,
/// where a missing file means the controller sets no limit there.
fn read_io_max(directory: &Path) -> io::Result<BTreeMap<String, DeviceLimits>> {
    let path = directory.join("io.max");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("{}: {error}", quote_name(&path.display().to_string())),
            ));
        }
    };
    return match parse_io_max(&text) {
        Ok(devices) => Ok(devices),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("{}: {error}", quote_name(&path.display().to_string())),
        )),
    };
}

/// Reads one limit file of one cgroup directory,
/// where a missing file means the controller sets no limit there.
fn read_limit_file(
    directory: &Path,
    name: &str,
    parse: fn(&str) -> io::Result<Option<u64>>,
) -> io::Result<Option<u64>> {
    let path = directory.join(name);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("{}: {error}", quote_name(&path.display().to_string())),
            ));
        }
    };
    return match parse(&text) {
        Ok(limit) => Ok(limit),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("{}: {error}", quote_name(&path.display().to_string())),
        )),
    };
}

/// Reads the limits set on one cgroup directory.
fn read_directory_limits(directory: &Path) -> io::Result<InheritedLimits> {
    return Ok(InheritedLimits {
        cpu_quota_per_sec_usec: read_limit_file(directory, "cpu.max", parse_cpu_max)?,
        memory_max: read_limit_file(directory, "memory.max", parse_limit)?,
        memory_high: read_limit_file(directory, "memory.high", parse_limit)?,
        memory_swap_max: read_limit_file(directory, "memory.swap.max", parse_limit)?,
        tasks_max: read_limit_file(directory, "pids.max", parse_limit)?,
        io_weight: read_limit_file(directory, "io.weight", parse_io_weight)?,
        io_devices: read_io_max(directory)?,
    });
}

/// Reports a nonempty path component.
fn is_component(part: &&str) -> bool {
    return !part.is_empty();
}

/// Splits a cgroup path into its components.
fn components(cgroup: &str) -> Vec<&str> {
    return cgroup.split('/').filter(is_component).collect();
}

/// Reads the limits of every cgroup the process leaves.
/// These are the cgroups on the way from `origin` up to,
/// but not including,
/// the deepest ancestor it shares with `destination`.
/// A limit file that cannot be read is an error:
/// guessing "no limit" could loosen the process.
pub fn read_inherited_limits(
    cgroup_root: &Path,
    origin: &str,
    destination: &str,
) -> io::Result<InheritedLimits> {
    let origin_parts = components(origin);
    let destination_parts = components(destination);
    let mut shared = 0;
    while shared < origin_parts.len()
        && shared < destination_parts.len()
        && origin_parts[shared] == destination_parts[shared]
    {
        shared += 1;
    }
    let mut limits = InheritedLimits::default();
    for depth in (shared + 1)..=origin_parts.len() {
        let mut directory = cgroup_root.to_path_buf();
        for part in &origin_parts[..depth] {
            directory.push(part);
        }
        limits.tighten(&read_directory_limits(&directory)?);
    }
    return Ok(limits);
}
