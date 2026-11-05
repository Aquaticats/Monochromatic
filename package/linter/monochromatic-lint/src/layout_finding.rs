//! What:
//!  The shape of one finding of a layout rule before it gets a severity and a rule name.
//! Why:
//!  Both layout rules report a file,
//!  a line and a message,
//!  and the step that runs them adds the severity and the diagnostic code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LayoutFinding = { file: Path; line: number; message: string; help?: string };
//! ```

/// Import the native path type.
use std::path::PathBuf;

/// What:
///  A finding of a layout rule.
/// Why:
///  A line of zero means the finding is about a folder or a link and has no line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LayoutFinding = { file: Path; line: number; message: string; help?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutFinding {
    /// The file the finding is reported on:
    ///  the file holding the proving reference,
    ///  or the link or folder the finding is about.
    pub file: PathBuf,
    /// One-based line,
    ///  or zero for no line.
    pub line: usize,
    /// What is wrong,
    ///  naming every folder or file involved.
    pub message: String,
    /// What to do about it.
    pub help: Option<String>,
}

/// What:
///  A path as findings show it:
///  relative to the package folder when it is inside it.
/// Why:
///  Absolute paths of a checkout would make every message long and different on every machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shown(path: Path, base: Path): string;
/// ```
pub fn shown(path: &std::path::Path, base: &std::path::Path) -> String {
    return match path.strip_prefix(base) {
        Ok(below) if below.components().next().is_some() => below.to_string_lossy().into_owned(),
        _ => path.to_string_lossy().into_owned(),
    };
}
