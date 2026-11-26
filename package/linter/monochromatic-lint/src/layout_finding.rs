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
///  the walk from the package folder when it is outside (a module a `#[path]` names elsewhere),
///  and always with `/` between segments,
/// whatever the platform's own separator is.
/// Why:
///  A list is written on one machine and read on another,
///  and this crate ships binaries for every platform,
///  so a key built from the platform's separator would match on the machine that wrote the line
///  and nowhere else.
///  An absolute path would make the line match only one checkout,
///  so a file outside the package is named by the folders between it and the package:
///  `../outside x.rs` reads the same on every machine that holds the repository.
///  The package folder itself is written `.`,
///  because the empty path would end a key in a space that the list reader trims away,
///  and no line could hold it.
///  A file name that holds a backslash is written as a slash here;
///  an allow-list entry refuses a backslash for that reason,
///  so a list line and a key can never disagree about a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // keyPath(p, base) = shown(p, base).replaceAll('\\', '/'), or '.' for the base folder
/// ```
pub fn key_path(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    let written: String = match path.strip_prefix(base) {
        Ok(below) if below.components().next().is_some() => below.to_string_lossy().into_owned(),
        _ => climbing_path(path, base),
    };
    let slashed: String = written.replace('\\', "/");
    // What: the one spelling of the package folder itself.
    // Why: a key that ends in an empty path holds a trailing space, which the list reader trims away.
    if slashed.is_empty() {
        return String::from(".");
    }
    return slashed;
}

/// What:
///  How one folder is written from another:
///  one `..` for each folder they do not share, then the folders of `path` below that.
/// Why:
///  A key must read the same in every checkout,
///  and only the distance between two folders of one package is the same everywhere.
///  A relative path and an absolute one share no folder,
///  so they keep the plain spelling the caller gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function climbingPath(path: string, base: string): string;
/// ```
fn climbing_path(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    if path.is_absolute() != base.is_absolute() {
        return path.to_string_lossy().into_owned();
    }
    // What: the folders of each path, so the ones they share can be counted once.
    // Why: each folder below the shared part becomes one `..` of the answer.
    let base_parts: Vec<std::path::Component<'_>> = base.components().collect::<Vec<_>>();
    let path_parts: Vec<std::path::Component<'_>> = path.components().collect::<Vec<_>>();
    let mut shared: usize = 0;
    while shared < base_parts.len()
        && shared < path_parts.len()
        && base_parts[shared] == path_parts[shared]
    {
        shared += 1;
    }
    let mut parts: Vec<String> = Vec::<String>::new();
    for _ in shared..base_parts.len() {
        parts.push(String::from(".."));
    }
    for part in &path_parts[shared..] {
        parts.push(part.as_os_str().to_string_lossy().into_owned());
    }
    return parts.join("/");
}

/// What:
///  Make every non-empty key name exactly one finding,
///  by numbering a finding that would share an earlier finding's key.
/// Why:
///  One line of a list tolerates every finding whose key it holds,
///  so two findings with one key would let a line that tolerates today's finding
///  also tolerate a second one added later.
///  A rule tells its findings apart by what each one holds where it can;
///  only findings that hold the same words reach this state,
///  such as two empty `impl` blocks of one type or two glob re-exports of one module.
///  The first keeps the plain key and a later one gains `#2`,
/// `#3` and so on,
/// in the rule's own order,
///  so an entry written before the numbering existed still names its finding,
///  and only an identical finding added above renumbers the ones below it.
///  No line number enters a key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // for each finding: n = ++taken[key]; if (n > 1) finding.key = numbered(key, n)
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
            finding.key = numbered(base.as_str(), count);
        }
    }
    return;
}

/// What:
///  The key of the `count`th finding of a shape whose own words an earlier finding already holds.
/// Why:
///  The number goes into the key's first word,
/// right after the shape it repeats,
///  because no literal key holds `#` there:
///  every key a rule prints starts with a shape word and a space,
///  so `foreign-name#2 src/lib.rs x` can never be read as a doc entry named `x#2`,
///  which prints `foreign-name src/lib.rs x#2`.
///  Two findings therefore never share a key,
///  and a line written for the first cannot tolerate the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numbered = (base, count) => base.replace(' ', `#${count} `)
/// ```
fn numbered(
    base: &str,
    count: usize,
) -> String {
    return match base.split_once(' ') {
        Some((shape, rest)) => format!("{shape}#{count} {rest}"),
        None => format!("{base}#{count}"),
    };
}

/// Controls of the path display stay outside release artifacts.
#[cfg(test)]
#[path = "layout_finding_tests.rs"]
mod tests;
