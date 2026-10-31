//! What:
//!  Glob expansion over the virtual view,
//! one path segment at a time.
//! Why:
//!  `package/*/*/Cargo.toml` must match exactly two levels and never walk `target/` or `node_modules/`;
//! expanding
//!  segment by segment lists only the directories the pattern names,
//! and every listing is recorded for revalidation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function expandGlob(view: View, pattern: string, want: EntryKind): string[]
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `Glob` from `globset`.
use globset::Glob;

/// What:
///  Whether a segment has glob syntax.
/// Why:
///  Literal segments need no listing to match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isPattern(segment: string): boolean
/// ```
fn is_pattern(segment: &str) -> bool {
    return segment.contains(['*', '?', '[', ']', '{', '}']);
}

/// What:
///  Expand a pattern into the sorted repo-relative paths whose last segment has kind `want`.
///  Hidden names (starting with `.`) match only a segment that starts with `.` itself,
/// as shell globs do.
/// Why:
///  Deterministic,
/// bounded-depth discovery;
/// `**` is not supported on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expandGlob(view: View, pattern: string, want: EntryKind): string[]
/// ```
pub fn expand_glob(view: &mut View, pattern: &str, want: EntryKind) -> Result<Vec<String>, Diagnostic> {
    let segments: Vec<&str> = pattern.split('/').collect();
    if pattern.is_empty() || segments.iter().any(|segment| return segment.is_empty() || *segment == "**") {
        let message = format!("the glob {pattern:?} is empty or uses an unsupported ** or empty segment");
        return Err(Diagnostic::new("glob-invalid", &message));
    }
    let mut current: Vec<String> = vec![String::new()];
    for (index, segment) in segments.iter().enumerate() {
        let is_last = index + 1 == segments.len();
        let needed = if is_last { want } else { EntryKind::Directory };
        let mut next: Vec<String> = Vec::new();
        let matcher = if is_pattern(segment) {
            match Glob::new(segment) {
                Ok(glob) => Some(glob.compile_matcher()),
                Err(error) => {
                    let message = format!("the glob segment {segment:?} is invalid: {error}");
                    return Err(Diagnostic::new("glob-invalid", &message));
                }
            }
        } else {
            None
        };
        if matcher.is_none() && !is_last {
            for prefix in &current {
                next.push(join(prefix, segment));
            }
            current = next;
            continue;
        }
        for prefix in &current {
            for (name, kind) in view.list(prefix)? {
                if kind != needed {
                    continue;
                }
                let matches = match &matcher {
                    Some(compiled) => compiled.is_match(&name) && (!name.starts_with('.') || segment.starts_with('.')),
                    None => name == *segment,
                };
                if matches {
                    next.push(join(prefix, &name));
                }
            }
        }
        current = next;
    }
    current.sort();
    current.dedup();
    return Ok(current);
}

/// What:
///  Join a directory prefix and a name with `/`,
/// treating the empty prefix as the root.
/// Why:
///  Keeps repo-relative paths free of a leading slash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function join(prefix: string, name: string): string
/// ```
fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        return name.to_string();
    }
    return format!("{prefix}/{name}");
}
