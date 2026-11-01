//! What:
//! Forward-slash path arithmetic for repository-relative paths.
//! Why:
//! Manifest paths and `path = "../x"` dependency values are compared as plain text,
//! never through the file
//! system,
//! so the library stays free of file-system access and behaves the same on every platform.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { posix } from 'node:path';
//! posix.normalize(posix.join(dir, relative));
//! ```

/// What:
/// `pub fn join_normalized(base: &str, relative: &str) -> String` joins two forward-slash paths and
/// resolves `.` and `..` segments,
/// like `posix.normalize(posix.join(base, relative))`.
/// A `..` that climbs
/// above the start is kept,
/// so an escaping path never matches a repository directory by accident.
/// Why:
/// The internal-crate lookup keys are normalized crate directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// posix.normalize(posix.join(base, relative))
/// ```
pub fn join_normalized(base: &str, relative: &str) -> String {
    let combined: String = format!("{base}/{relative}");
    let absolute: bool = combined.starts_with('/');
    let mut kept: Vec<&str> = Vec::new();
    for segment in combined.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            let can_pop: bool = matches!(kept.last(), Some(last) if *last != "..");
            if can_pop {
                kept.pop();
            } else if !absolute {
                kept.push("..");
            }
            continue;
        }
        kept.push(segment);
    }
    let joined: String = kept.join("/");
    if absolute {
        return format!("/{joined}");
    }
    if joined.is_empty() {
        return ".".to_string();
    }
    return joined;
}
