//! What:
//!  The start-time compatibility gate's layout half:
//! the bounded read of the
//!  kernel's base type information and the ZFS module's own,
//! the pre-scan that runs
//!  before any type is decoded,
//! the `btf-rs` 2.0.1 adapter that reads the two files
//!  behind the declaration,
//! the expected layout compiled from this crate's `#[repr(C)]` type,
//!  the comparison that refuses the first difference,
//! and the one call the root-run handler makes
//!  before its first ioctl,
//! which the wrapper gains when its request route lands.
//! Why:
//!  The kernel copies its whole `zfs_cmd_t` in and out of one ioctl,
//! so a module whose
//!  field moved,
//! grew or vanished reads another request's bytes and writes another
//!  request's fields.
//! Nothing in userspace can see that without a device,
//! so a program that sends a raw request compares the loaded module's declared
//!  layout with the one this crate compiles,
//! before its first ioctl,
//! and disables every raw request,
//! reads included,
//!  when the comparison refuses
//! (`doc/decision/cli-git-zfs-runtime-layout-checker.md`).
//!  The gate proves no module release:
//! by the project's answer of 2026-11-19,
//! "Any version,
//!  layout decides",
//! the layout comparison and the request numbers decide.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `refusal.rs`:
//!   one refusal's step,
//!   the file it read and the detail it names.
//! - `bounded_read.rs`:
//!   the one bounded read of a file root controls,
//!   and its page,
//!   base and module bounds.
//! - `btf_header.rs`:
//!   the type information's header,
//!   its sections and the checks that run before any type is decoded.
//! - `declaration.rs`:
//!   the reader-neutral declaration of one module's types,
//!   the three duties every reader owes it,
//!   and the vocabulary everything after it reads.
//! - `expectations.rs`:
//!   the layout this crate's type compiles,
//!   and the two records compared member by member.
//! - `expected_requests.rs`:
//!   the request numbers this crate can send and the range's two endpoints.
//! - `mismatch.rs`:
//!   the step and the detail of one difference.
//! - `comparison.rs`:
//!   the fixed-order walk that compares the module's declaration and stops at
//!   the first difference.
//! - `comparison_requests.rs`:
//!   the request numbers and the range's two endpoints.
//! - `btf_rs_declaration.rs`:
//!   the one adapter that reads the two files with `btf-rs` 2.0.1 and maps
//!   them onto the declaration.
//! - `check_module_layout.rs`:
//!   the entry point the root-run handler calls,
//!   and what a pass read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkModuleLayout({ baseBtf: '/sys/kernel/btf/vmlinux', moduleBtf: '/sys/kernel/btf/zfs' });
//! if (!verdict.compatible) rawRequestsDisabled(verdict.refusal);
//! ```

/// The step a refusal names.
mod refusal;
/// The one bounded read of a file root controls.
mod bounded_read;
/// The type information's header and its sections.
mod btf_header;
/// The reader-neutral declaration everything after it reads.
mod declaration;
/// The layout this crate's type compiles.
mod expectations;
/// The request numbers this crate can send.
mod expected_requests;
/// The one difference value both walks return.
mod mismatch;
/// The walk that compares the module's declaration with the expectation.
mod comparison;
/// The request numbers compared with the module's `enum zfs_ioc`.
mod comparison_requests;
/// The one adapter that reads the two files with `btf-rs` 2.0.1.
mod btf_rs_declaration;
/// The entry point the root-run handler calls.
mod check_module_layout;

/// The step a refusal names,
/// its file and its detail.
pub use refusal::{Refusal, Step};
/// The page,
/// base and module bounds one read accepts.
pub use bounded_read::{BASE_BTF_BOUND, IDENTITY_FILE_BOUND, MODULE_BTF_BOUND};
/// The entry point,
/// the two files it reads and what a pass reports.
pub use check_module_layout::{Compatible, LayoutRequest, check_module_layout};

/// The bounded read's own decisions.
#[cfg(test)]
#[path = "bounded_read_tests.rs"]
mod bounded_read_tests;
/// The header and section checks.
#[cfg(test)]
#[path = "btf_header_tests.rs"]
mod btf_header_tests;
/// The adapter's array sizes and the reader's own refusals.
#[cfg(test)]
#[path = "btf_rs_declaration_tests.rs"]
mod btf_rs_declaration_tests;
/// The expectation table's own checks.
#[cfg(test)]
#[path = "expectations_tests.rs"]
mod expectations_tests;
/// The comparison's decisions and refusals.
#[cfg(test)]
#[path = "comparison_tests.rs"]
mod comparison_tests;
/// The three duties every reader owes the declaration.
#[cfg(test)]
#[path = "declaration_contract_tests.rs"]
mod declaration_contract_tests;
/// The entry point's files,
/// bounds and pass.
#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
/// The BTF bytes one test starts from,
/// built by a writer of this folder's own.
#[cfg(test)]
mod blob_fixture;
/// The approved module's declaration,
/// written as data a test can move one field in.
#[cfg(test)]
mod approved_module_fixture;
/// The gate's one live arm in the throwaway guest.
#[cfg(test)]
#[path = "live_gate_tests.rs"]
mod live_gate_tests;
