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

/// Reading the limit files of the cgroups a process leaves.
mod cgroup_limit_reading;
/// Reading the text of one cgroup limit file.
mod limit_file_parsing;

/// Reading the limits of every cgroup the process leaves.
pub use cgroup_limit_reading::read_inherited_limits;
/// The parsers of the limit files,
/// which a test reads on its own.
#[cfg(test)]
pub use limit_file_parsing::{
    parse_cpu_max,
    parse_io_max,
    parse_io_weight,
    parse_limit,
};

/// The per-device limits keyed by device,
/// in a stable order.
use std::collections::BTreeMap;

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
    fn tighten(
        &mut self,
        other: &DeviceLimits,
    ) {
        self.read_bandwidth = smaller(self.read_bandwidth, other.read_bandwidth);
        self.write_bandwidth = smaller(self.write_bandwidth, other.write_bandwidth);
        self.read_iops = smaller(self.read_iops, other.read_iops);
        self.write_iops = smaller(self.write_iops, other.write_iops);
    }

    /// The four values in the order of the manager's property names.
    fn values(&self) -> [Option<u64>; 4] {
        return [
            self.read_bandwidth,
            self.write_bandwidth,
            self.read_iops,
            self.write_iops,
        ];
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
                    properties[slot]
                        .1
                        .push((format!("/dev/block/{device}"), number));
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
            (
                "CPUQuotaPerSecUSec",
                self.cpu_quota_per_sec_usec.unwrap_or(UNLIMITED),
            ),
            ("MemoryMax", self.memory_max.unwrap_or(UNLIMITED)),
            ("MemoryHigh", self.memory_high.unwrap_or(UNLIMITED)),
            ("MemorySwapMax", self.memory_swap_max.unwrap_or(UNLIMITED)),
            ("TasksMax", self.tasks_max.unwrap_or(UNLIMITED)),
            ("IOWeight", self.io_weight.unwrap_or(DEFAULT_IO_WEIGHT)),
        ];
    }

    /// Keeps the smaller finite value of each limit.
    fn tighten(
        &mut self,
        other: &InheritedLimits,
    ) {
        self.cpu_quota_per_sec_usec =
            smaller(self.cpu_quota_per_sec_usec, other.cpu_quota_per_sec_usec);
        self.memory_max = smaller(self.memory_max, other.memory_max);
        self.memory_high = smaller(self.memory_high, other.memory_high);
        self.memory_swap_max = smaller(self.memory_swap_max, other.memory_swap_max);
        self.tasks_max = smaller(self.tasks_max, other.tasks_max);
        self.io_weight = lighter(self.io_weight, other.io_weight);
        for (device, limits) in &other.io_devices {
            self.io_devices
                .entry(device.clone())
                .or_default()
                .tighten(limits);
        }
    }
}

/// The smaller of two optional limits,
/// where absent means unlimited.
fn smaller(
    left: Option<u64>,
    right: Option<u64>,
) -> Option<u64> {
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
fn lighter(
    left: Option<u64>,
    right: Option<u64>,
) -> Option<u64> {
    let weight = left
        .unwrap_or(DEFAULT_IO_WEIGHT)
        .min(right.unwrap_or(DEFAULT_IO_WEIGHT));
    return if weight == DEFAULT_IO_WEIGHT {
        None
    } else {
        Some(weight)
    };
}
