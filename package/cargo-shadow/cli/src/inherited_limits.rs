//! What:
//! The resource limits of the control group that started a run,
//! read so the scope of the run can carry them.
//! Why:
//! The scope is made under the manager's own slice,
//! not under the caller's control group,
//! so the limits of the caller's cgroup (`MemoryMax`,
//! `CPUQuota`,
//! `TasksMax`,
//! `IOWeight`) would not reach it:
//! a build under a capped `systemd-run --user --scope` ran uncapped (rule RXI:
//! every command that is not trivial runs under
//! limits,
//! in every agent).
//! The scope carries,
//! for each of the four limits,
//! the tightest finite value on the cgroups the run leaves:
//! the cgroups on the way from the caller's own up to,
//! and not including,
//! the deepest ancestor it shares with the slice.
//! Limits of the shared ancestors and above keep applying to the scope by ancestry on their own.
//! A limit nobody sets stays absent:
//! no size is ever invented.
//! A limit file that cannot be read is an error,
//! because reading it as "no limit" could loosen the run;
//! the discovery of the route turns that error into the weaker path,
//! which keeps the process in the caller's own cgroup,
//! so the limits still hold.
//! This is a port of the four limits of `package/wg/cli-quicker-exempt/src/inherited_limits.rs` (the precedent),
//! not a dependency on it:
//! that crate is a binary without a library,
//! and each crate here stands on its own (its own `[workspace]`,
//! published separately).
//! `memory.high`,
//! `memory.swap.max` and the per-device `io.max` limits of the precedent are not carried.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export interface InheritedLimits { cpuQuotaPerSecUsec?: number; memoryMax?: number; tasksMax?: number; ioWeight?: number }
//! export function ownCgroup(procSelfCgroup: string): string  // throws
//! export function readInheritedLimits(root: string, origin: string, destination: string): InheritedLimits  // throws
//! export function callerLimits(ownCgroupFile: string, root: string, sliceDir: string): InheritedLimits  // throws
//! ```

/// The `InvalidData` error that reports a control file whose text is not what the kernel documents.
use std::io;
/// The control files read:
/// borrowed from the caller,
/// and owned when joined under the slice directory.
use std::path::{Path, PathBuf};

/// The kernel's default I/O weight,
/// which a cgroup that sets nothing has,
/// and so a weight with nothing to carry.
const DEFAULT_IO_WEIGHT: u64 = 100;

/// CPU time per second,
/// in microseconds,
/// that `CPUQuota=100%` means.
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

/// The unit of `CPUQuota=`:
/// one hundredth of a percent,
/// which is the finest step `systemd` reads,
/// in microseconds of CPU time per second.
const MICROSECONDS_PER_PERMYRIAD: u64 = MICROSECONDS_PER_SECOND / 10_000;

/// The limits the new scope must carry.
/// Each is the smallest finite value found on the cgroups the run leaves,
/// and absent when none of them sets one.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InheritedLimits {
    /// CPU time per second in microseconds,
    /// from `cpu.max`.
    pub(crate) cpu_quota_per_sec_usec: Option<u64>,
    /// Hard memory limit in bytes,
    /// from `memory.max`.
    pub(crate) memory_max: Option<u64>,
    /// Task count limit,
    /// from `pids.max`.
    pub(crate) tasks_max: Option<u64>,
    /// I/O weight,
    /// from the `default` line of `io.weight`.
    /// Absent when no cgroup on the way sets a weight below the kernel default.
    pub(crate) io_weight: Option<u64>,
}

/// The methods of the type above.
impl InheritedLimits {
    /// What:
    /// `pub(crate) fn properties(&self) -> Vec<String>` is the `Name=value` text of each limit that is set,
    /// in the order `MemoryMax`,
    /// `CPUQuota`,
    /// `TasksMax`,
    /// `IOWeight`.
    /// Why:
    /// `systemd-run --property=` takes these spellings;
    /// `CPUQuotaPerSecUSec=` is refused there (probed on 2026-11-07:
    /// "Unknown assignment"),
    /// so the quota is a percent with two decimals,
    /// rounded down so the scope is never looser than the caller.
    /// An absent limit gives no property.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// properties(): string[]
    /// ```
    pub(crate) fn properties(&self) -> Vec<String> {
        let mut properties: Vec<String> = Vec::new();
        if let Some(bytes) = self.memory_max {
            properties.push(format!("MemoryMax={bytes}"));
        }
        if let Some(quota) = self.cpu_quota_per_sec_usec {
            // A quota below one step is still a cap: it is carried as one step, never dropped.
            let permyriad: u64 = (quota / MICROSECONDS_PER_PERMYRIAD).max(1);
            properties.push(format!("CPUQuota={}.{:02}%", permyriad / 100, permyriad % 100));
        }
        if let Some(tasks) = self.tasks_max {
            properties.push(format!("TasksMax={tasks}"));
        }
        if let Some(weight) = self.io_weight {
            properties.push(format!("IOWeight={weight}"));
        }
        return properties;
    }

    /// What:
    /// `fn tighten(&mut self, other: &InheritedLimits)` keeps the smaller finite value of each limit and the lighter I/O weight.
    /// Why:
    /// The tightest cgroup on the way is the one that bounds the caller.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tighten(other: InheritedLimits): void
    /// ```
    fn tighten(&mut self, other: &InheritedLimits) {
        self.cpu_quota_per_sec_usec = smaller(self.cpu_quota_per_sec_usec, other.cpu_quota_per_sec_usec);
        self.memory_max = smaller(self.memory_max, other.memory_max);
        self.tasks_max = smaller(self.tasks_max, other.tasks_max);
        self.io_weight = lighter(self.io_weight, other.io_weight);
    }
}

/// What:
/// `fn smaller(left: Option<u64>, right: Option<u64>) -> Option<u64>` is the smaller of two optional limits,
/// where absent means unlimited.
/// Why:
/// Absent never wins against a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function smaller(left?: number, right?: number): number | undefined
/// ```
fn smaller(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    return match (left, right) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, other) => other,
    };
}

/// What:
/// `fn lighter(left: Option<u64>, right: Option<u64>) -> Option<u64>` is the lighter of two optional I/O weights,
/// where absent means the kernel default and a result at the default is absent again.
/// Why:
/// A weight above the default is never carried:
/// the scope may be throttled more than its origin but never less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lighter(left?: number, right?: number): number | undefined
/// ```
fn lighter(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    let weight: u64 = left.unwrap_or(DEFAULT_IO_WEIGHT).min(right.unwrap_or(DEFAULT_IO_WEIGHT));
    return if weight == DEFAULT_IO_WEIGHT { None } else { Some(weight) };
}

/// What:
/// `fn invalid(text: String) -> io::Error` is an `InvalidData` error.
/// Why:
/// Every parse failure of a limit file is the same kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invalid(text: string): Error
/// ```
fn invalid(text: String) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidData, text);
}

/// What:
/// `fn parse_limit(text: &str) -> io::Result<Option<u64>>` parses a cgroup limit file:
/// `max` is unlimited and a number is the limit.
/// Why:
/// `memory.max` and `pids.max` use this form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLimit(text: string): number | undefined  // throws
/// ```
fn parse_limit(text: &str) -> io::Result<Option<u64>> {
    let trimmed: &str = text.trim();
    if trimmed == "max" {
        return Ok(None);
    }
    return match trimmed.parse::<u64>() {
        Ok(number) => Ok(Some(number)),
        Err(_error) => Err(invalid(format!("not a cgroup limit: {trimmed:?}"))),
    };
}

/// What:
/// `fn parse_cpu_max(text: &str) -> io::Result<Option<u64>>` parses `cpu.max`,
/// `max <period>` or `<quota> <period>`,
/// into microseconds of CPU time per second.
/// Why:
/// The file holds a quota per period,
/// and a period is not always a second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCpuMax(text: string): number | undefined  // throws
/// ```
fn parse_cpu_max(text: &str) -> io::Result<Option<u64>> {
    let mut words = text.split_whitespace();
    let (Some(quota), Some(period)) = (words.next(), words.next()) else {
        return Err(invalid(format!("not a cpu.max line: {:?}", text.trim())));
    };
    let Some(quota_value) = parse_limit(quota)? else {
        return Ok(None);
    };
    let Ok(period_value) = period.parse::<u64>() else {
        return Err(invalid(format!("not a cpu.max period: {period:?}")));
    };
    if period_value == 0 {
        return Err(invalid("a cpu.max period of zero".to_string()));
    }
    return Ok(Some(quota_value.saturating_mul(MICROSECONDS_PER_SECOND) / period_value));
}

/// What:
/// `fn parse_io_weight(text: &str) -> io::Result<Option<u64>>` parses `io.weight`:
/// the `default` line holds the weight of the cgroup,
/// and lines for single devices are not read.
/// Why:
/// The kernel default is no weight to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseIoWeight(text: string): number | undefined  // throws
/// ```
fn parse_io_weight(text: &str) -> io::Result<Option<u64>> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("default ") else {
            continue;
        };
        return match rest.trim().parse::<u64>() {
            Ok(weight) if weight == DEFAULT_IO_WEIGHT => Ok(None),
            Ok(weight) => Ok(Some(weight)),
            Err(_error) => Err(invalid(format!("not an io.weight default: {:?}", rest.trim()))),
        };
    }
    return Err(invalid("an io.weight without a default line".to_string()));
}

/// What:
/// `fn read_limit_file(directory, name, parse) -> io::Result<Option<u64>>` reads one limit file of one cgroup directory,
/// where a missing file means the controller sets no limit there.
/// Why:
/// A controller that is not enabled in a cgroup has no file;
/// any other failure is an error,
/// never "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLimitFile(directory: string, name: string, parse: (text: string) => number | undefined): number | undefined  // throws
/// ```
fn read_limit_file(directory: &Path, name: &str, parse: fn(&str) -> io::Result<Option<u64>>) -> io::Result<Option<u64>> {
    let path: PathBuf = directory.join(name);
    let text: String = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io::Error::new(error.kind(), format!("{}: {error}", path.display()))),
    };
    return parse(&text).map_err(|error| { return io::Error::new(error.kind(), format!("{}: {error}", path.display())); });
}

/// What:
/// `fn read_directory_limits(directory: &Path) -> io::Result<InheritedLimits>` reads the four limits set on one cgroup directory.
/// Why:
/// One directory is one step of the walk up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readDirectoryLimits(directory: string): InheritedLimits  // throws
/// ```
fn read_directory_limits(directory: &Path) -> io::Result<InheritedLimits> {
    return Ok(InheritedLimits {
        cpu_quota_per_sec_usec: read_limit_file(directory, "cpu.max", parse_cpu_max)?,
        memory_max: read_limit_file(directory, "memory.max", parse_limit)?,
        tasks_max: read_limit_file(directory, "pids.max", parse_limit)?,
        io_weight: read_limit_file(directory, "io.weight", parse_io_weight)?,
    });
}

/// What:
/// `fn components(cgroup: &str) -> io::Result<Vec<&str>>` splits a cgroup path into its components.
/// Why:
/// A `..` component names a place outside the mount (a cgroup namespace that does not contain the process),
/// and joining it would read another directory than the cgroup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function components(cgroup: string): string[]  // throws
/// ```
fn components(cgroup: &str) -> io::Result<Vec<&str>> {
    let parts: Vec<&str> = cgroup.split('/').filter(|part| { return !part.is_empty(); }).collect();
    if parts.iter().any(|part| { return *part == ".." || *part == "."; }) {
        return Err(invalid(format!("the cgroup path {cgroup:?} is not below the mount")));
    }
    return Ok(parts);
}

/// What:
/// `pub(crate) fn read_inherited_limits(cgroup_root: &Path, origin: &str, destination: &str) -> io::Result<InheritedLimits>`
/// reads the limits of every cgroup the run leaves:
/// those on the way from `origin` up to,
/// but not including,
/// the deepest ancestor it shares with `destination`.
/// Why:
/// The limits of the shared ancestors keep applying to the scope by ancestry.
/// A limit file that cannot be read is an error:
/// guessing "no limit" could loosen the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readInheritedLimits(cgroupRoot: string, origin: string, destination: string): InheritedLimits  // throws
/// ```
pub(crate) fn read_inherited_limits(cgroup_root: &Path, origin: &str, destination: &str) -> io::Result<InheritedLimits> {
    let origin_parts: Vec<&str> = components(origin)?;
    let destination_parts: Vec<&str> = components(destination)?;
    let mut shared: usize = 0;
    while shared < origin_parts.len() && shared < destination_parts.len() && origin_parts[shared] == destination_parts[shared] {
        shared += 1;
    }
    let mut limits: InheritedLimits = InheritedLimits::default();
    for depth in (shared + 1)..=origin_parts.len() {
        let mut directory: PathBuf = cgroup_root.to_path_buf();
        for part in &origin_parts[..depth] {
            directory.push(part);
        }
        limits.tighten(&read_directory_limits(&directory)?);
    }
    return Ok(limits);
}

/// What:
/// `pub(crate) fn own_cgroup(text: &str) -> io::Result<String>` reads the cgroup v2 path out of the text of `/proc/<pid>/cgroup`.
/// Why:
/// On a unified hierarchy the file has the line `0::<path>`;
/// a host without one (a cgroup v1 hybrid) gives no path to read limits from.
/// A trailing ` (deleted)` is the kernel's note on a removed cgroup and is not part of the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ownCgroup(text: string): string  // throws
/// ```
pub(crate) fn own_cgroup(text: &str) -> io::Result<String> {
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("0::") {
            return Ok(path.strip_suffix(" (deleted)").unwrap_or(path).to_string());
        }
    }
    return Err(invalid("no `0::` line, so no cgroup v2 path".to_string()));
}

/// What:
/// `pub(crate) fn caller_limits(own_cgroup_file: &Path, cgroup_root: &Path, slice_dir: &Path) -> io::Result<InheritedLimits>`
/// reads the limits the scopes of the slice at `slice_dir` must carry for the process whose cgroup file is `own_cgroup_file`.
/// Why:
/// The destination of a scope is its slice,
/// as the manager reported it (already joined to `cgroup_root` by the discovery).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function callerLimits(ownCgroupFile: string, cgroupRoot: string, sliceDir: string): InheritedLimits  // throws
/// ```
pub(crate) fn caller_limits(own_cgroup_file: &Path, cgroup_root: &Path, slice_dir: &Path) -> io::Result<InheritedLimits> {
    let text: String = std::fs::read_to_string(own_cgroup_file).map_err(|error| { return io::Error::new(error.kind(), format!("{}: {error}", own_cgroup_file.display())); })?;
    let origin: String = own_cgroup(&text).map_err(|error| { return invalid(format!("{}: {error}", own_cgroup_file.display())); })?;
    let relative: &Path = slice_dir.strip_prefix(cgroup_root).map_err(|_error| { return invalid(format!("{} is not below {}", slice_dir.display(), cgroup_root.display())); })?;
    let destination: &str = relative.to_str().ok_or_else(|| { return invalid(format!("{} is not UTF-8", relative.display())); })?;
    return read_inherited_limits(cgroup_root, &origin, destination);
}
