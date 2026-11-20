//! What:
//!  The one target the managed route is qualified for,
//! and the refusal every other
//!  target gets before anything is read.
//! Why:
//!  The proof the gate makes is compiled for one architecture,
//! and the design refuses
//!  `linux-arm64` until a guest qualifies it;
//! BTF records no architecture,
//! so the
//!  compiled target is the only evidence there is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!QUALIFIED_TARGETS.includes(process.arch)) return refusalForTarget(process.arch);
//! ```

use monochromatic_zfs_request_to_openzfs_kernel_device::{CompatibilityRefusal as Refusal, CompatibilityStep as Step};

/// The targets the managed route is qualified for.
pub const QUALIFIED_TARGETS: [&str; 1] = ["x86_64"];

/// The target this binary compiled for.
pub fn current_target() -> &'static str {
    return std::env::consts::ARCH;
}

/// The refusal a target gets,
/// or nothing when it is qualified.
pub fn target_refusal(target: &str) -> Option<Refusal> {
    if QUALIFIED_TARGETS.contains(&target) {
        return None;
    }
    return Some(Refusal {
        step: Step::Type,
        file: None,
        detail: format!(
            "this binary runs on {target}, and the managed ZFS route is qualified for {} only until a guest run qualifies another target; BTF records no architecture, so the compiled target is the only evidence the gate has",
            QUALIFIED_TARGETS.join(", ")
        ),
    });
}
