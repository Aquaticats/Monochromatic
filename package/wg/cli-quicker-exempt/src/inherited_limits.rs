//! Reads the resource limits a process loses when it leaves its cgroup,
//! so the scope that adopts it can carry them.
//! Moving a process for routing must never loosen its CPU or memory limits:
//! a build under a `CPUQuota=` cap or a memory cap stays capped after the move.
//! Limits of cgroups above the common ancestor of the old and new place keep applying on their own.
//! The cap becomes the moved processes' own cap,
//! no longer shared with the processes that stayed in the old cgroup.
//! Weights,
//! I/O limits and cpusets are not copied.

/// Syscall and parse failures.
use std::io;
/// Cgroup directories.
use std::path::Path;

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
        ] {
            if let Some(number) = value {
                properties.push((name, number));
            }
        }
        return properties;
    }

    /// Keeps the smaller finite value of each limit.
    fn tighten(&mut self, other: &InheritedLimits) {
        self.cpu_quota_per_sec_usec = smaller(self.cpu_quota_per_sec_usec, other.cpu_quota_per_sec_usec);
        self.memory_max = smaller(self.memory_max, other.memory_max);
        self.memory_high = smaller(self.memory_high, other.memory_high);
        self.memory_swap_max = smaller(self.memory_swap_max, other.memory_swap_max);
        self.tasks_max = smaller(self.tasks_max, other.tasks_max);
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

/// Parses a cgroup limit file:
/// `max` is unlimited and a number is the limit.
pub fn parse_limit(text: &str) -> io::Result<Option<u64>> {
    let trimmed = text.trim();
    if trimmed == "max" {
        return Ok(None);
    }
    return match trimmed.parse::<u64>() {
        Ok(number) => Ok(Some(number)),
        Err(_error) => Err(io::Error::new(io::ErrorKind::InvalidData, format!("not a cgroup limit: {trimmed:?}"))),
    };
}

/// Parses `cpu.max`,
/// `max <period>` or `<quota> <period>`,
/// into microseconds of CPU time per second.
pub fn parse_cpu_max(text: &str) -> io::Result<Option<u64>> {
    let mut words = text.split_whitespace();
    let (Some(quota), Some(period)) = (words.next(), words.next()) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("not a cpu.max line: {:?}", text.trim())));
    };
    let Some(quota_value) = parse_limit(quota)? else {
        return Ok(None);
    };
    let Ok(period_value) = period.parse::<u64>() else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("not a cpu.max period: {period:?}")));
    };
    if period_value == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a cpu.max period of zero"));
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
        Err(error) => return Err(io::Error::new(error.kind(), format!("{}: {error}", path.display()))),
    };
    return match parse(&text) {
        Ok(limit) => Ok(limit),
        Err(error) => Err(io::Error::new(error.kind(), format!("{}: {error}", path.display()))),
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
