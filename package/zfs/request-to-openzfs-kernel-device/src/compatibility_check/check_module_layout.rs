//! What:
//!  The one call both executables make before their first ioctl:
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
//! so both
//!  programs refuse on the same evidence and neither reads the module's release
//!  string as a rule.
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
/// The indexed type entries.
use super::btf_types::TypeInformation;
/// The walk that compares the declared layout.
use super::comparison::compare;
/// The walk that compares the request numbers.
use super::comparison_requests::compare_requests;
/// The one difference value either walk returns.
use super::mismatch::Mismatch;
/// The one refusal value both programs render.
use super::refusal::{Refusal, Step, refusal_at};
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
    let base: Sections<'_> = read_sections(&base_bytes, request.base_btf)?;
    let module_bytes: Vec<u8> = read_bounded(&ReadRequest {
        path: request.module_btf,
        bound: MODULE_BTF_BOUND,
        step: Step::ModuleBtf,
    })?;
    let module: Sections<'_> = read_sections(&module_bytes, request.module_btf)?;
    let base_types: TypeInformation<'_> =
        TypeInformation::alone(base.types, base.strings, request.base_btf)?;
    let module_types: TypeInformation<'_> =
        TypeInformation::over_base(module.types, module.strings, &base_types, request.module_btf)?;
    compare(&module_types).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, request.module_btf, mismatch.detail);
    })?;
    compare_requests(&module_types).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, request.module_btf, mismatch.detail);
    })?;
    return Ok(Compatible {
        base_bytes: base_bytes.len(),
        module_bytes: module_bytes.len(),
        base_header_bytes: base.header_bytes,
        module_header_bytes: module.header_bytes,
    });
}
