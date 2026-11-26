//! What:
//!  The shape of one finding of a layout rule before it gets a severity and a rule name.
//! Why:
//!  Both layout rules report a file,
//!  a line and a message,
//!  and the step that runs them adds the severity and the diagnostic code.
//!  A finding also carries the key that an allow-list entry writes to tolerate it,
//!  and a finding of a loop carries none,
//!  because a loop is tolerated one use at a time and never as a whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LayoutFinding = { file: Path; line: number; message: string; help?: string; key: string };
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
    /// The stable name of this one finding,
    ///  which an allow-list entry writes to tolerate it;
    ///  empty for a finding no list may tolerate,
    ///  such as a loop,
    ///  which is tolerated by listing the uses inside it.
    pub key: String,
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
pub fn shown(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    return match path.strip_prefix(base) {
        Ok(below) if below.components().next().is_some() => below.to_string_lossy().into_owned(),
        _ => path.to_string_lossy().into_owned(),
    };
}

/// What:
///  A path as one finding's key writes it:
///  relative to the package folder when it is inside it,
///  and always with `/` between segments, whatever the platform's own separator is.
/// Why:
///  A list is written on one machine and read on another,
///  and this crate ships binaries for every platform,
///  so a key built from the platform's separator would match on the machine that wrote the line
///  and nowhere else.
///  A file name that holds a backslash is written as a slash here;
///  an allow-list entry refuses a backslash for that reason,
///  so a list line and a key can never disagree about a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // keyPath(p, base) = shown(p, base).replaceAll('\\', '/')
/// ```
pub fn key_path(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    return shown(path, base).replace('\\', "/");
}

/// What:
///  Make every non-empty key name exactly one finding,
///  by numbering a finding that would share an earlier finding's key.
/// Why:
///  One line of a list tolerates every finding whose key it holds,
///  so two findings with one key would let a line that tolerates today's finding
///  also tolerate a second one added later.
///  The first finding keeps the plain key and a later one gains `#2`, `#3` and so on in the rule's own order,
///  which is document order within a file,
///  so an entry written before the numbering existed still names its finding
///  and an edit above a finding never renumbers it.
///  Only findings a rule cannot tell apart by their own words reach this state,
///  such as two `impl` blocks of one type or two globs of one module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // for each finding: n = ++taken[key]; if (n > 1) finding.key = `${key}#${n}`
/// ```
pub fn unique_keys(findings: &mut [LayoutFinding]) {
    // How many findings have already taken each key, in the order the rule produced them.
    let mut taken: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::<String, usize>::new();
    for finding in findings.iter_mut() {
        if finding.key.is_empty() {
            continue;
        }
        let base: String = finding.key.clone();
        let count: usize = match taken.get(&base) {
            Some(seen) => *seen + 1,
            None => 1,
        };
        taken.insert(base.clone(), count);
        if count > 1 {
            finding.key = format!("{base}#{count}");
        }
    }
    return;
}

/// Controls of the path display stay outside release artifacts.
#[cfg(test)]
#[path = "layout_finding_tests.rs"]
mod tests;
