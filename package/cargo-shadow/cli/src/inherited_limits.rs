//! What:
//! The resource limits of the control group that started a run (red stub: the types and entry points exist, and read nothing).
//! Why:
//! The tests of `inherited_limits_tests.rs` need the names to compile and fail on their assertions; the next commit fills it in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export interface InheritedLimits { cpuQuotaPerSecUsec?: number; memoryMax?: number; tasksMax?: number; ioWeight?: number }
//! ```

use std::io;
use std::path::Path;

/// The limits the new scope must carry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InheritedLimits {
    /// CPU time per second in microseconds.
    pub(crate) cpu_quota_per_sec_usec: Option<u64>,
    /// Hard memory limit in bytes.
    pub(crate) memory_max: Option<u64>,
    /// Task count limit.
    pub(crate) tasks_max: Option<u64>,
    /// I/O weight below the kernel default.
    pub(crate) io_weight: Option<u64>,
}

/// The methods of the type above.
impl InheritedLimits {
    /// The `Name=value` text of each limit that is set (red stub: none).
    pub(crate) fn properties(&self) -> Vec<String> {
        return Vec::new();
    }
}

/// The limits of the cgroups a run leaves (red stub: none).
pub(crate) fn read_inherited_limits(_cgroup_root: &Path, _origin: &str, _destination: &str) -> io::Result<InheritedLimits> {
    return Ok(InheritedLimits::default());
}

/// The cgroup v2 path out of the text of `/proc/<pid>/cgroup` (red stub: empty).
pub(crate) fn own_cgroup(_text: &str) -> io::Result<String> {
    return Ok(String::new());
}

/// The limits the scopes of a slice must carry for a caller (red stub: none).
pub(crate) fn caller_limits(_own_cgroup_file: &Path, _cgroup_root: &Path, _slice_dir: &Path) -> io::Result<InheritedLimits> {
    return Ok(InheritedLimits::default());
}
