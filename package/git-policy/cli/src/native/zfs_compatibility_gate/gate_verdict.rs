//! What:
//!  The check itself:
//! the qualified target,
//! the device,
//! the identity read,
//! the binding's layout comparison,
//! the second identity read and its one retry,
//! and the
//!  verdict this process remembers.
//! Why:
//!  The order is the design's,
//! and each step is a refusal a caller can act on:
//! a missing
//!  device is refused before any file is read,
//! a module that moved under the check is
//!  retried once and refused only when the retry's own two reads differ too,
//! and
//!  the verdict is compared with the module's identity before it is reused,
//! so
//!  a reload is noticed between two uses in one long process.
//!  The comparison and the cache are handed in,
//! so a control can script both
//!  without this machine's kernel files or device.
//!  Nothing is written to disk:
//! the cross-process cache is a question the design holds
//!  open until a cost is measured,
//! so the verdict lives in this process's memory alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = beforeRequests(); // the process's one answer, rechecked each use
//! ```

use super::gate_paths::GatePaths;
use super::gate_target::{
    current_target,
    target_refusal,
};
use super::module_identity::ModuleIdentity;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityRefusal as Refusal,
    CompatibilityStep as Step,
    Compatible as LayoutCompatible,
    LayoutRequest,
    ZfsDevice,
    check_module_layout,
};
use std::sync::{
    Mutex,
    MutexGuard,
    OnceLock,
    PoisonError,
};

/// The most attempts one check makes:
/// the first,
/// and the one retry the design allows.
pub const ATTEMPTS: usize = 2;

/// What:
///  What a compatible verdict read,
/// for the words a person reads.
#[derive(Clone, Debug)]
pub struct Facts {
    /// The process's own target.
    pub target: &'static str,
    /// The module's release string,
    /// which is reported and never compared with a rule.
    pub version: Option<String>,
    /// The identity attributes the module does not carry.
    pub absent: Vec<&'static str>,
    /// The bytes of the kernel's type information.
    pub base_btf_bytes: usize,
    /// The bytes of the module's type information.
    pub module_btf_bytes: usize,
    /// The two files' header lengths.
    pub header_bytes: (usize, usize),
}

/// What:
///  The gate's answer.
#[derive(Clone, Debug)]
pub enum Verdict {
    /// The loaded module's declaration matches,
    /// so raw requests may be made.
    Compatible(Facts),
    /// The gate refused,
    /// so every raw request stays disabled.
    Refused(Refusal),
}

/// What:
///  The gate one invocation runs before its first request.
/// Why:
///  The command line takes its gate as a value,
/// so a control can answer without this
///  machine's kernel files or device and the release path's own gate is the
///  only one that reads them.
pub trait BeforeRequests {
    /// The gate's answer.
    fn verdict(&self) -> Verdict;
}

/// The gate the two installed programs run.
pub struct LiveGate;

impl BeforeRequests for LiveGate {
    /// The live check,
    /// remembered for this process.
    fn verdict(&self) -> Verdict {
        return before_requests();
    }
}

/// What:
///  One comparison:
/// two files into a layout pass or the first refusal.
/// Why:
///  The live comparison reads this machine's two files,
/// so a control hands in
///  a scripted one and the order,
/// the retry and the cache stay the same code.
pub type Comparison<'a> = &'a dyn Fn(&GatePaths) -> Result<LayoutCompatible, Refusal>;

/// The verdict this process remembers,
/// with the identity it was made from.
struct Remembered {
    /// The answer.
    verdict: Verdict,
    /// The module that answered.
    identity: ModuleIdentity,
}

/// What:
///  The verdict one process remembers,
///  rechecked against the module's identity before a reuse.
/// Why:
///  The rule is one parse per process,
/// so the cache is a value a control can
///  construct instead of a process-wide static,
/// and only the live gate takes the
///  static one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class VerdictCache { answer(paths, compare): Verdict }
/// ```
pub struct VerdictCache {
    /// The one remembered answer,
    /// or nothing before the first check.
    slot: Mutex<Option<Remembered>>,
}

/// One process's verdict,
/// kept until the module's identity moves.
impl Default for VerdictCache {
    fn default() -> Self {
        return Self::new();
    }
}

impl VerdictCache {
    /// An empty cache.
    pub fn new() -> Self {
        return VerdictCache {
            slot: Mutex::new(None),
        };
    }

    /// The gate's answer,
    /// reused only while the module's identity holds.
    pub fn answer(
        &self,
        paths: &GatePaths,
        compare: Comparison<'_>,
    ) -> Verdict {
        let mut slot: MutexGuard<'_, Option<Remembered>> =
            self.slot.lock().unwrap_or_else(PoisonError::into_inner);
        let now: ModuleIdentity = ModuleIdentity::read(paths);
        if let Some(remembered) = slot.as_ref()
            && remembered.identity.same_module_as(&now)
        {
            return remembered.verdict.clone();
        }
        let verdict: Verdict = check_with(paths, compare);
        *slot = Some(Remembered {
            verdict: verdict.clone(),
            identity: now,
        });
        return verdict;
    }
}

/// The one remembered answer of this process.
static REMEMBERED: OnceLock<VerdictCache> = OnceLock::new();

/// The gate's answer for this process,
/// rechecked against the module's identity before a reuse.
pub fn before_requests() -> Verdict {
    return before_requests_with(&GatePaths::live(), &compare, remembered_cache());
}

/// The gate's answer for one set of paths,
/// one comparison and one cache.
pub fn before_requests_with(
    paths: &GatePaths,
    compare: Comparison<'_>,
    cache: &VerdictCache,
) -> Verdict {
    return cache.answer(paths, compare);
}

/// The cache the live gate remembers its one answer in.
fn remembered_cache() -> &'static VerdictCache {
    return REMEMBERED.get_or_init(VerdictCache::new);
}

/// Check one set of paths,
/// in the design's order.
pub fn check(paths: &GatePaths) -> Verdict {
    return check_with(paths, &compare);
}

/// Check one set of paths against one comparison,
/// in the design's order.
pub fn check_with(
    paths: &GatePaths,
    compare: Comparison<'_>,
) -> Verdict {
    if let Some(refusal) = target_refusal(current_target()) {
        return Verdict::Refused(refusal);
    }
    if let Err(refusal) = open_the_device(paths) {
        return Verdict::Refused(refusal);
    }
    for _attempt in 0..ATTEMPTS {
        let first: ModuleIdentity = ModuleIdentity::read(paths);
        let answer: Result<LayoutCompatible, Refusal> = compare(paths);
        let second: ModuleIdentity = ModuleIdentity::read(paths);
        if !first.same_as(&second) {
            continue;
        }
        let compatible: LayoutCompatible = match answer {
            Ok(compatible) => compatible,
            Err(refusal) => return Verdict::Refused(refusal),
        };
        return Verdict::Compatible(Facts {
            target: current_target(),
            version: first.version.clone(),
            absent: first.absent_attributes(),
            base_btf_bytes: compatible.base_bytes,
            module_btf_bytes: compatible.module_bytes,
            header_bytes: (compatible.base_header_bytes, compatible.module_header_bytes),
        });
    }
    return Verdict::Refused(ModuleIdentity::changed_under_the_check());
}

/// Open the device the way a request does,
/// and close it again,
/// so a missing or unopenable device refuses before any file is read.
fn open_the_device(paths: &GatePaths) -> Result<(), Refusal> {
    return match ZfsDevice::open_at(paths.device.as_path()) {
        Ok(device) => {
            drop(device);
            Ok(())
        }
        Err(error) => Err(Refusal {
            step: Step::Device,
            file: Some(paths.device.clone()),
            detail: format!("it could not be opened, and every raw request needs it: {error}"),
        }),
    };
}

/// Compare the two files with the declaration this crate compiles.
fn compare(paths: &GatePaths) -> Result<LayoutCompatible, Refusal> {
    return check_module_layout(&LayoutRequest {
        base_btf: paths.base_btf.as_path(),
        module_btf: paths.module_btf.as_path(),
    });
}
