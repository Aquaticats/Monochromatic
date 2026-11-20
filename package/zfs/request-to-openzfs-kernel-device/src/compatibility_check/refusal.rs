//! What:
//!  One refusal of the start-time compatibility gate:
//! the step that failed,
//! the file it
//!  read,
//! and the detail that names what differs,
//! or why the step could not run.
//! Why:
//!  A refusal is what disables every raw request,
//! reads included,
//! and it is also
//!  what the caller and `doctor` print,
//! so it carries the facts a person needs to tell a
//!  missing file from a module whose layout moved:
//! a step that could not run,
//! a file,
//!  and the expected and actual values.
//!  The words a person reads are the RENDERING of these facts,
//! so the gate's
//!  two programs do not word the same failure twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { step: Step; file?: string; detail: string };
//! ```

/// The formatting traits a refusal's own words use.
use std::fmt;
/// The path types a step's file is taken as and kept as.
use std::path::{Path, PathBuf};

/// What:
///  The step of the gate that refused.
/// Why:
///   The step is the first thing a reader needs:
/// a missing `/dev/zfs`,
/// a module that changed
///  under the check,
/// a file that could not be read,
/// a header or a type entry the reader cannot
///  represent,
/// a type whose shape moved,
/// a member,
/// an enumerator or a base type size.
///  `Device` and `Identity` are the caller's two steps,
/// which the binding never performs itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'Device' | 'Identity' | 'BaseBtf' | 'ModuleBtf' | 'Header' | 'Type' | 'Member' | 'Enum' | 'BaseType';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// `/dev/zfs` is missing or the caller cannot open it.
    Device,
    /// The module changed between the two identity reads.
    Identity,
    /// The kernel's own type information could not be read.
    BaseBtf,
    /// The loaded module's type information could not be read.
    ModuleBtf,
    /// A BTF header,
    /// a section or a type entry is not one this reader represents.
    Header,
    /// A compared type's own size or kind differs.
    Type,
    /// A member is missing,
    /// extra or misplaced,
    /// or its type size differs.
    Member,
    /// An enumerator is missing or carries another value.
    Enum,
    /// A base integer size a member uses differs.
    BaseType,
}

/// The step in the words a refusal prints.
impl Step {
    /// The step as a reader meets it.
    pub fn as_str(self) -> &'static str {
        return match self {
            Step::Device => "device",
            Step::Identity => "identity",
            Step::BaseBtf => "base type information",
            Step::ModuleBtf => "module type information",
            Step::Header => "header",
            Step::Type => "type",
            Step::Member => "member",
            Step::Enum => "enumerator",
            Step::BaseType => "base type",
        };
    }
}

/// What:
///  Why the gate refuses:
/// the step,
/// the file it read when the step read one,
///  and the detail.
/// Why:
///   Every refusal a caller prints is this one value,
/// so a missing device,
/// a short
///  read and a moved field cannot be worded three different ways;
/// the detail names
///  the field path and both values where a comparison refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = { step: Step.Member, file: moduleBtf, detail: 'zfs_cmd.zc_objset_stats.dds_flags: expected 1 byte, found none' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// The step that failed.
    pub step: Step,
    /// The file the step read,
    /// or nothing for a step that reads none.
    pub file: Option<PathBuf>,
    /// The field path with both values,
    /// or why the step could not run.
    pub detail: String,
}

/// The words a refusal prints.
impl fmt::Display for Refusal {
    /// The step,
    /// the file it read and the detail,
    /// in one line a person can act on.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(path): Option<&Path> = self.file.as_deref() else {
            return write!(formatter, "{} refused: {}", self.step.as_str(), self.detail);
        };
        return write!(
            formatter,
            "{} refused whilst reading {}: {}",
            self.step.as_str(),
            path.display(),
            self.detail
        );
    }
}

/// One refusal built by a step that read `file`.
pub fn refusal_at(step: Step, file: &Path, detail: String) -> Refusal {
    return Refusal {
        step,
        file: Some(file.to_path_buf()),
        detail,
    };
}
