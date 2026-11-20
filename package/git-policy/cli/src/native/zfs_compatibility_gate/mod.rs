//! What:
//!  The start-time ZFS compatibility gate both executables run before their
//!  first ioctl:
//!  the qualified target,
//!  the device check,
//!  the module's identity read,
//!  the binding's layout comparison,
//!  the second identity read with its one retry,
//!  the verdict remembered per process,
//!  and the words the caller and `doctor` print.
//! Why:
//!  A raw request built for another `zfs_cmd_t` writes another request's bytes
//!  into a kernel that reads them as its own,
//!  so nothing is sent before the loaded module's
//!  declaration matches the one this crate compiles,
//!  and a missing device,
//!  a missing or
//!  unreadable file,
//!  a short read or any difference disables every raw request,
//!  reads
//!  included,
//!  with no force flag and no environment override
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`).
//!  The binding holds the reader,
//!  the expectations and the comparison;
//!  this folder holds the
//!  paths,
//!  the read order,
//!  the module's identity,
//!  the target allowlist,
//!  the per-process
//!  cache and the words,
//!  as the decision record places them.
//!  Nothing about the module's release is a rule:
//!  by the project's answer of
//!  2026-11-19,
//!  "Any version,
//!  layout decides",
//!  the layout comparison and the request
//!  numbers decide,
//!  and the release string is identity and reporting only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = beforeRequests(); // { compatible } | { refusal }
//! if (verdict.refusal) return reportRefusal(verdict.refusal);
//! ```

/// The device, identity and type-information paths one check reads.
pub mod gate_paths;
/// The target allowlist the managed route is qualified for.
pub mod gate_target;
/// The loaded module's identity and each file's own facts.
pub mod module_identity;
/// The check, its one retry and the per-process cache.
pub mod gate_verdict;
/// The words the caller and `doctor` print.
pub mod gate_words;

/// The paths one check reads,
/// the live set and a test's own.
pub use gate_paths::GatePaths;
/// The qualified targets and the target check.
pub use gate_target::{QUALIFIED_TARGETS, target_refusal};
/// The identity one check compares and reports.
pub use module_identity::{FileFacts, ModuleIdentity};
/// The check,
/// its verdict and the process's one answer.
pub use gate_verdict::{BeforeRequests, Facts, LiveGate, Verdict, before_requests, check};
/// The lines a person reads.
pub use gate_words::{TESTED_MODULE_VERSION, compatible_line, refusal_line, verdict_line};

/// The target allowlist and the target check.
#[cfg(test)]
#[path = "gate_target_tests.rs"]
mod gate_target_tests;
/// The identity read and its comparison.
#[cfg(test)]
#[path = "module_identity_tests.rs"]
mod module_identity_tests;
/// The check, its retry and its cache.
#[cfg(test)]
#[path = "gate_verdict_tests.rs"]
mod gate_verdict_tests;
/// The words the caller and `doctor` print.
#[cfg(test)]
#[path = "gate_words_tests.rs"]
mod gate_words_tests;
