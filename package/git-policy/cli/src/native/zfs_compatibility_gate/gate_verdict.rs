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
use super::gate_target::{current_target, target_refusal};
use super::module_identity::ModuleIdentity;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    Compatible as LayoutCompatible, CompatibilityRefusal as Refusal,
    CompatibilityStep as Step, LayoutRequest, check_module_layout,
};
use std::sync::{Mutex, MutexGuard, OnceLock};

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

/// The verdict this process remembers,
/// with the identity it was made from.
struct Remembered {
    /// The answer.
    verdict: Verdict,
    /// The module that answered.
    identity: ModuleIdentity,
}

/// The one remembered answer of this process.
static REMEMBERED: OnceLock<Mutex<Option<Remembered>>> = OnceLock::new();

/// The gate's answer for this process,
/// rechecked against the module's identity before a reuse.
pub fn before_requests() -> Verdict {
    let cache: &Mutex<Option<Remembered>> = REMEMBERED.get_or_init(|| {
        return Mutex::new(None);
    });
    let mut slot: MutexGuard<'_, Option<Remembered>> = cache.lock().unwrap_or_else(|poisoned| {
        return poisoned.into_inner();
    });
    let paths: GatePaths = GatePaths::live();
    let now: ModuleIdentity = ModuleIdentity::read(&paths);
    if let Some(remembered) = slot.as_ref()
        && remembered.identity.same_as(&now)
    {
        return remembered.verdict.clone();
    }
    let verdict: Verdict = check(&paths);
    *slot = Some(Remembered {
        verdict: verdict.clone(),
        identity: now,
    });
    return verdict;
}

/// Check one set of paths,
/// in the design's order.
pub fn check(paths: &GatePaths) -> Verdict {
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

/// Open the device the way a request would,
/// and close it again,
/// so a missing or unopenable device refuses before any file is read.
fn open_the_device(paths: &GatePaths) -> Result<(), Refusal> {
    let answer: std::io::Result<std::fs::File> = std::fs::OpenOptions::new()
        .read(true)
        .open(paths.device.as_path());
    return match answer {
        Ok(file) => {
            drop(file);
            Ok(())
        }
        Err(error) => Err(Refusal {
            step: Step::Device,
            file: Some(paths.device.clone()),
            detail: format!(
                "it could not be opened, and every raw request needs it: {error}"
            ),
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
