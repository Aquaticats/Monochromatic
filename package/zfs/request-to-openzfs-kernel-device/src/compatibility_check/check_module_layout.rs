//! What:
//!  The one call the root-run handler makes before its first ioctl,
//!  and the wrapper makes when its request route lands:
//! read the kernel's
//!  base type information and the loaded module's own,
//! compare the module's declared
//!  layout and request numbers with this crate's,
//! and answer a pass or the first refusal.
//! Why:
//!  The caller owns the device check,
//! the module's identity,
//! the target allowlist and
//!  the words it prints;
//! this call owns the two files and what they declare,
//! so every caller refuses on the same evidence,
//! and no caller reads the module's release
//!  string as a rule:
//! what each program prints stays that program's own.
//!  A missing or unreadable file,
//! a short read,
//! a header or a type entry this reader cannot
//!  represent,
//! or any difference disables every raw request,
//! reads included,
//! and there
//!  is no force flag and no environment override.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkModuleLayout({ baseBtf, moduleBtf }); // { compatible } | { refusal }
//! ```

/// The bounded read of each file and its bounds.
use super::bounded_read::{BASE_BTF_BOUND, MODULE_BTF_BOUND, ReadRequest, read_bounded};
/// The section cut of each file.
use super::btf_header::{Sections, read_sections};
/// The one adapter that compares the two files through the declaration.
use super::btf_rs_declaration::compare_declared_layout;
/// The one refusal value a caller renders.
use super::refusal::{Refusal, Step};
/// The borrowed path type of one call.
use std::path::Path;

/// What:
///  The two files the comparison reads.
/// Why:
///   The paths are the caller's:
/// the wrapper and the root handler name them,
/// because the
///  binding performs no write and owns no configuration,
/// and a test names copies of them
///  instead of the live ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LayoutRequest<'a> = { baseBtf: &'a Path; moduleBtf: &'a Path };
/// ```
pub struct LayoutRequest<'a> {
    /// The kernel's own type information,
    /// which a module's file extends.
    pub base_btf: &'a Path,
    /// The loaded module's type information.
    pub module_btf: &'a Path,
}

/// What:
///  What a pass read,
/// so a caller can print it and compare it before a reuse.
/// Why:
///   `doctor` prints each file and its size beside the verdict,
/// and the
///  per-process cache re-reads each file's own facts,
/// so a pass states what it saw rather than
///  only that it saw something.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Compatible = { baseBytes: number; moduleBytes: number; baseHeaderBytes: number; moduleHeaderBytes: number };
/// ```
pub struct Compatible {
    /// The bytes the base file held.
    pub base_bytes: usize,
    /// The bytes the module file held.
    pub module_bytes: usize,
    /// The base file's header length,
    /// with the sections counted from its end.
    pub base_header_bytes: usize,
    /// The module file's header length.
    pub module_header_bytes: usize,
}

/// Read the two files and compare the module's declaration with this crate's.
///
/// # Errors
/// A refusal naming the step,
/// the file and the detail of the first difference or of the read that failed.
pub fn check_module_layout(request: &LayoutRequest<'_>) -> Result<Compatible, Refusal> {
    let base_bytes: Vec<u8> = read_bounded(&ReadRequest {
        path: request.base_btf,
        bound: BASE_BTF_BOUND,
        step: Step::BaseBtf,
    })?;
    let base: Sections = read_sections(&base_bytes, request.base_btf)?;
    let module_bytes: Vec<u8> = read_bounded(&ReadRequest {
        path: request.module_btf,
        bound: MODULE_BTF_BOUND,
        step: Step::ModuleBtf,
    })?;
    let module: Sections = read_sections(&module_bytes, request.module_btf)?;
    compare_declared_layout(&base_bytes, &module_bytes, request.base_btf, request.module_btf)?;
    return Ok(Compatible {
        base_bytes: base_bytes.len(),
        module_bytes: module_bytes.len(),
        base_header_bytes: base.header_bytes,
        module_header_bytes: module.header_bytes,
    });
}
