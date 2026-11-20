//! What:
//!  The start-time compatibility gate's layout half:
//! the bounded reader of the
//!  kernel's base type information and the ZFS module's own,
//! the pre-scan that runs before
//!  any type is decoded,
//! the expected layout compiled from this crate's `#[repr(C)]` type,
//!  the comparison that refuses the first difference,
//! and the one call both executables make
//!  before their first ioctl.
//! Why:
//!  The kernel copies its whole `zfs_cmd_t` in and out of one ioctl,
//! so a module whose
//!  field moved,
//! grew or vanished reads another request's bytes and writes another
//!  request's fields.
//! Nothing in userspace can see that without a device,
// so both
//!  executables compare the loaded module's declared layout with the one this crate
//!  compiles,
//! before their first ioctl,
//! and disable every raw request,
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
//!   its three sections and the checks that run before any type is decoded.
//! - `btf_entry.rs`:
//!   the format's kind numbers,
//!   the bytes each kind's entry holds and one decoded entry.
//! - `btf_decode.rs`:
//!   the bytes each kind's entry holds after its head,
//!   and the decode of one entry.
//! - `btf_types.rs`:
//!   the decoded type entries,
//!   the walk that indexes them and the lookups the comparison asks.
//! - `btf_records.rs`:
//!   one record's member list and one enumeration's values,
//!   as the module declares them.
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
//! - `check_module_layout.rs`:
//!   the entry point both executables call,
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
/// The format's kind numbers and one decoded entry.
mod btf_entry;
/// The decode of one entry's kind-specific fields.
mod btf_decode;
/// The decoded type entries and the lookups the comparison asks.
mod btf_types;
/// One record's members and one enumeration's values.
mod btf_records;
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
/// The entry point both executables call.
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
/// The type-entry walk and the lookups.
#[cfg(test)]
#[path = "btf_types_tests.rs"]
mod btf_types_tests;
/// The member list and enumeration values.
#[cfg(test)]
#[path = "btf_records_tests.rs"]
mod btf_records_tests;
/// The expectation table's own checks.
#[cfg(test)]
#[path = "expectations_tests.rs"]
mod expectations_tests;
/// The comparison's decisions and refusals.
#[cfg(test)]
#[path = "comparison_tests.rs"]
mod comparison_tests;
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
