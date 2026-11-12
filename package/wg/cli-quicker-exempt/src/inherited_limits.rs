//! Reads the resource limits a process loses when it leaves its cgroup,
//! so the scope that adopts it can carry them.
//! Moving a process for routing must never loosen its CPU or memory limits:
//! a build under a `CPUQuota=` cap or a memory cap stays capped after the move.
//! Limits of cgroups above the common ancestor of the old and new place keep applying on their own.
//! The scope is shared by every process moved from the same origin,
//! so together they stay within the origin's cap;
//! the origin keeps its own cap for the processes that stayed.
//! The I/O weight is copied when it is below the kernel default,
//! and so are the per-device limits of `io.max`,
//! keyed by device.
//! CPU weight and cpusets are not copied.

/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The per-device limits keyed by device,
/// in a stable order.
use std::collections::BTreeMap;
/// Syscall and parse failures.
use std::io;
/// One place that names the `io::ErrorKind` a failure carries.
use crate::io_error_kind::{invalid_data};
/// Cgroup directories.
use std::path::Path;

/// The kernel's default I/O weight,
/// which a cgroup that sets nothing has,
/// and so a weight with nothing to carry.
const DEFAULT_IO_WEIGHT: u64 = 100;
/// The value of a cgroup limit property that means no limit.
const UNLIMITED: u64 = u64::MAX;

/// The four limits `io.max` can set for one block device,
/// each absent when it is `max`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeviceLimits {
    /// Read bytes per second,
    /// `rbps`,
    /// the value of `IOReadBandwidthMax`.
    pub read_bandwidth: Option<u64>,
    /// Write bytes per second,
    /// `wbps`,
    /// the value of `IOWriteBandwidthMax`.
    pub write_bandwidth: Option<u64>,
    /// Read operations per second,
    /// `riops`,
    /// the value of `IOReadIOPSMax`.
    pub read_iops: Option<u64>,
    /// Write operations per second,
    /// `wiops`,
    /// the value of `IOWriteIOPSMax`.
    pub write_iops: Option<u64>,
}

/// Merging.
impl DeviceLimits {
    /// Keeps the smaller finite value of each limit.
    fn tighten(&mut self, other: &DeviceLimits) {
        self.read_bandwidth = smaller(self.read_bandwidth, other.read_bandwidth);
        self.write_bandwidth = smaller(self.write_bandwidth, other.write_bandwidth);
        self.read_iops = smaller(self.read_iops, other.read_iops);
        self.write_iops = smaller(self.write_iops, other.write_iops);
    }

    /// The four values in the order of the manager's property names.
    fn values(&self) -> [Option<u64>; 4] {
        return [self.read_bandwidth, self.write_bandwidth, self.read_iops, self.write_iops];
    }
}

/// The limits the new scope must carry.
/// Each is the smallest finite value found on the cgroups the process leaves,
/// and absent when none of them sets one.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InheritedLimits {
    /// CPU time per second in microseconds,
    /// from `cpu.max`,
    /// the value of `CPUQuotaPerSecUSec`.
    pub cpu_quota_per_sec_usec: Option<u64>,
    /// Hard memory limit in bytes,
    /// from `memory.max`.
    pub memory_max: Option<u64>,
    /// Memory throttling limit in bytes,
    /// from `memory.high`.
    pub memory_high: Option<u64>,
    /// Swap limit in bytes,
    /// from `memory.swap.max`.
    pub memory_swap_max: Option<u64>,
    /// Task count limit,
    /// from `pids.max`.
    pub tasks_max: Option<u64>,
    /// I/O weight,
    /// from the `default` line of `io.weight`,
    /// the value of `IOWeight`.
    /// Absent when no cgroup on the way sets a weight below the kernel default.
    /// The manager writes it to `io.bfq.weight` as well,
    /// which the BFQ scheduler of this host reads.
    pub io_weight: Option<u64>,
    /// Per-device limits of `io.max`,
    /// by the `major:minor` the file names the device with.
    /// A device with no finite limit is not listed.
    pub io_devices: BTreeMap<String, DeviceLimits>,
}

/// Merging and rendering.
impl InheritedLimits {
    /// The unit properties in the order they are passed to the manager,
    /// each as a name and a value of D-Bus type `t`.
    pub fn properties(&self) -> Vec<(&'static str, u64)> {
        let mut properties: Vec<(&'static str, u64)> = Vec::new();
        for (name, value) in [
            ("CPUQuotaPerSecUSec", self.cpu_quota_per_sec_usec),
            ("MemoryMax", self.memory_max),
            ("MemoryHigh", self.memory_high),
            ("MemorySwapMax", self.memory_swap_max),
            ("TasksMax", self.tasks_max),
            ("IOWeight", self.io_weight),
        ] {
            if let Some(number) = value {
                properties.push((name, number));
            }
        }
        return properties;
    }

    /// The per-device properties in the form the manager takes them:
    /// an array of device path and number for each of `IOReadBandwidthMax`,
    /// `IOWriteBandwidthMax`,
    /// `IOReadIOPSMax`
    /// and `IOWriteIOPSMax`.
    /// The device is spelled `/dev/block/<major>:<minor>`,
    /// which the manager resolves to the device whatever its name.
    /// All four are returned,
    /// empty when no device has a limit of that kind:
    /// a live scope that is sent an empty array loses the limits of that kind.
    pub fn device_properties(&self) -> Vec<(&'static str, Vec<(String, u64)>)> {
        let mut properties: Vec<(&'static str, Vec<(String, u64)>)> = vec![
            ("IOReadBandwidthMax", Vec::new()),
            ("IOWriteBandwidthMax", Vec::new()),
            ("IOReadIOPSMax", Vec::new()),
            ("IOWriteIOPSMax", Vec::new()),
        ];
        for (device, limits) in &self.io_devices {
            for (slot, value) in limits.values().into_iter().enumerate() {
                if let Some(number) = value {
                    properties[slot].1.push((format!("/dev/block/{device}"), number));
                }
            }
        }
        return properties;
    }

    /// Every limit property for a live scope,
    /// with the unlimited value for a limit that is absent and the default for the I/O weight,
    /// because the manager keeps the old value of a property that is not sent.
    pub fn all_properties(&self) -> Vec<(&'static str, u64)> {
        return vec![
            ("CPUQuotaPerSecUSec", self.cpu_quota_per_sec_usec.unwrap_or(UNLIMITED)),
            ("MemoryMax", self.memory_max.unwrap_or(UNLIMITED)),
            ("MemoryHigh", self.memory_high.unwrap_or(UNLIMITED)),
            ("MemorySwapMax", self.memory_swap_max.unwrap_or(UNLIMITED)),
            ("TasksMax", self.tasks_max.unwrap_or(UNLIMITED)),
            ("IOWeight", self.io_weight.unwrap_or(DEFAULT_IO_WEIGHT)),
        ];
    }

    /// Keeps the smaller finite value of each limit.
    fn tighten(&mut self, other: &InheritedLimits) {
        self.cpu_quota_per_sec_usec = smaller(self.cpu_quota_per_sec_usec, other.cpu_quota_per_sec_usec);
        self.memory_max = smaller(self.memory_max, other.memory_max);
        self.memory_high = smaller(self.memory_high, other.memory_high);
        self.memory_swap_max = smaller(self.memory_swap_max, other.memory_swap_max);
        self.tasks_max = smaller(self.tasks_max, other.tasks_max);
        self.io_weight = lighter(self.io_weight, other.io_weight);
        for (device, limits) in &other.io_devices {
            self.io_devices.entry(device.clone()).or_default().tighten(limits);
        }
    }
}

/// The smaller of two optional limits,
/// where absent means unlimited.
fn smaller(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    return match (left, right) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, other) => other,
    };
}

/// The lighter of two optional I/O weights,
/// where absent means the kernel default,
/// and a result at the default is absent again.
/// A weight above the default is never carried:
/// the scope may be throttled more than its origin but never less.
fn lighter(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    let weight = left.unwrap_or(DEFAULT_IO_WEIGHT).min(right.unwrap_or(DEFAULT_IO_WEIGHT));
    return if weight == DEFAULT_IO_WEIGHT { None } else { Some(weight) };
}

/// Reports a nonempty run of digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_digit_byte);
}

/// Reports an ASCII digit byte.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Reports a `major:minor` device number.
fn is_device_number(word: &str) -> bool {
    return match word.split_once(':') {
        Some((major, minor)) => is_digits(major) && is_digits(minor),
        None => false,
    };
}

/// Parses `io.max`:
/// one line per device,
/// the `major:minor` and then `key=value` words,
/// where `max` is no limit.
/// A key this parser does not know is skipped,
/// so a newer kernel's extra key does not stop every move,
/// and a value that is neither `max` nor a number is an error.
pub fn parse_io_max(text: &str) -> io::Result<BTreeMap<String, DeviceLimits>> {
    let mut devices: BTreeMap<String, DeviceLimits> = BTreeMap::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let Some(device) = words.next() else {
            continue;
        };
        if !is_device_number(device) {
            return Err(invalid_data(format!("not an io.max device: {device:?}")));
        }
        let mut limits = DeviceLimits::default();
        for word in words {
            let Some((key, value)) = word.split_once('=') else {
                return Err(invalid_data(format!("not an io.max setting: {word:?}")));
            };
            let parsed = parse_limit(value)?;
            match key {
                "rbps" => limits.read_bandwidth = parsed,
                "wbps" => limits.write_bandwidth = parsed,
                "riops" => limits.read_iops = parsed,
                "wiops" => limits.write_iops = parsed,
                _unknown => {}
            }
        }
        if limits != DeviceLimits::default() {
            devices.insert(device.to_string(), limits);
        }
    }
    return Ok(devices);
}

/// Reads the `io.max` file of one cgroup directory,
/// where a missing file means the controller sets no limit there.
fn read_io_max(directory: &Path) -> io::Result<BTreeMap<String, DeviceLimits>> {
    let path = directory.join("io.max");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(io::Error::new(error.kind(), format!("{}: {error}", quote_name(&path.display().to_string())))),
    };
    return match parse_io_max(&text) {
        Ok(devices) => Ok(devices),
        Err(error) => Err(io::Error::new(error.kind(), format!("{}: {error}", quote_name(&path.display().to_string())))),
    };
}

/// Parses `io.weight`:
/// the `default` line holds the weight of the cgroup,
/// and lines for single devices are not read.
/// The kernel default is no weight to carry.
pub fn parse_io_weight(text: &str) -> io::Result<Option<u64>> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("default ") else {
            continue;
        };
        return match rest.trim().parse::<u64>() {
            Ok(weight) if weight == DEFAULT_IO_WEIGHT => Ok(None),
            Ok(weight) => Ok(Some(weight)),
            Err(_error) => Err(invalid_data(format!("not an io.weight default: {:?}", rest.trim()))),
        };
    }
    return Err(invalid_data("an io.weight without a default line"));
}

/// Parses a cgroup limit file:
/// `max` is unlimited and a number is the limit.
pub fn parse_limit(text: &str) -> io::Result<Option<u64>> {
    let trimmed = text.trim();
    if trimmed == "max" {
        return Ok(None);
    }
    return match trimmed.parse::<u64>() {
        Ok(number) => Ok(Some(number)),
        Err(_error) => Err(invalid_data(format!("not a cgroup limit: {trimmed:?}"))),
    };
}

/// Parses `cpu.max`,
/// `max <period>` or `<quota> <period>`,
/// into microseconds of CPU time per second.
pub fn parse_cpu_max(text: &str) -> io::Result<Option<u64>> {
    let mut words = text.split_whitespace();
    let (Some(quota), Some(period)) = (words.next(), words.next()) else {
        return Err(invalid_data(format!("not a cpu.max line: {:?}", text.trim())));
    };
    let Some(quota_value) = parse_limit(quota)? else {
        return Ok(None);
    };
    let Ok(period_value) = period.parse::<u64>() else {
        return Err(invalid_data(format!("not a cpu.max period: {period:?}")));
    };
    if period_value == 0 {
        return Err(invalid_data("a cpu.max period of zero"));
    }
    return Ok(Some(quota_value.saturating_mul(1_000_000) / period_value));
}

/// Reads one limit file of one cgroup directory,
/// where a missing file means the controller sets no limit there.
fn read_limit_file(directory: &Path, name: &str, parse: fn(&str) -> io::Result<Option<u64>>) -> io::Result<Option<u64>> {
    let path = directory.join(name);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io::Error::new(error.kind(), format!("{}: {error}", quote_name(&path.display().to_string())))),
    };
    return match parse(&text) {
        Ok(limit) => Ok(limit),
        Err(error) => Err(io::Error::new(error.kind(), format!("{}: {error}", quote_name(&path.display().to_string())))),
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
pub fn read_inherited_limits(cgroup_root: &Path, origin: &str, destination: &str) -> io::Result<InheritedLimits> {
    let origin_parts = components(origin);
    let destination_parts = components(destination);
    let mut shared = 0;
    while shared < origin_parts.len() && shared < destination_parts.len() && origin_parts[shared] == destination_parts[shared] {
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
