//! What:
//!  The paths a dirty worktree has uncommitted work in,
//! the paths a fast-forward would change,
//! and the overlap
//!  between them.
//! Why:
//!  After an auto-push publishes a merge,
//! local `main` can follow it even in a worktree that holds uncommitted
//!  work,
//! provided the move changes none of the files that work touches.
//! Another session's staged renames then
//!  never keep this branch behind the remote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // overlappingPaths(statusPaths(statusZ), diffPaths(diffZ))
//! ```

/// What:
///  The paths named by `git status --porcelain=v1 -z`:
/// each entry's path,
/// and the original path of a rename or
///  copy.
///  `&[u8]` borrows Git's output;
/// `Vec<Vec<u8>>` is an owned list of byte paths.
/// Why:
///   A staged rename touches both its names,
/// and a move that changes either one would overwrite the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusPaths(output: Uint8Array): Uint8Array[];
/// ```
pub fn status_paths(output: &[u8]) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    let mut entries = non_empty_fields(output).into_iter();
    while let Some(entry) = entries.next() {
        // An entry is two status letters, a space, then the path.
        if entry.len() < 4 {
            continue;
        }
        paths.push(entry[3..].to_vec());
        // A rename or copy is followed by its original path, which carries no status letters.
        let renamed: bool = matches!(entry[0], b'R' | b'C') || matches!(entry[1], b'R' | b'C');
        if renamed && let Some(original) = entries.next() {
            paths.push(original.to_vec());
        }
    }
    return paths;
}

/// What:
///  The paths of `git diff --name-only -z`.
/// Why:
///   They are the files a fast-forward from one commit to the other writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diffPaths(output: Uint8Array): Uint8Array[];
/// ```
pub fn diff_paths(output: &[u8]) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for path in non_empty_fields(output) {
        paths.push(path.to_vec());
    }
    return paths;
}

/// Whether `byte` is the NUL that ends each field of Git's `-z` output.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// The fields of NUL-separated output,
/// leaving out the empty ones.
fn non_empty_fields(output: &[u8]) -> Vec<&[u8]> {
    let mut fields: Vec<&[u8]> = Vec::new();
    for field in output.split(is_nul) {
        if !field.is_empty() {
            fields.push(field);
        }
    }
    return fields;
}

/// What:
///  The dirty paths a move would also change,
/// without repeats.
/// Why:
///   Exactly these paths decide whether the move may run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overlappingPaths(dirty: Uint8Array[], incoming: Uint8Array[]): Uint8Array[];
/// ```
pub fn overlapping_paths(dirty: &[Vec<u8>], incoming: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut overlap: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in dirty {
        if incoming.contains(path) && !overlap.contains(path) {
            overlap.push(path.clone());
        }
    }
    return overlap;
}

/// The most paths a report names before it counts the rest.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const LISTED_LIMIT: usize = 5;

/// What:
///  The overlapping paths as text for the report:
/// the first few by name,
/// then a count.
/// Why:
///   The person must see which files held the branch back,
/// and a report is not a place for a hundred names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listed(paths: Uint8Array[]): string;
/// ```
pub fn listed(paths: &[Vec<u8>]) -> String {
    let mut names: Vec<String> = Vec::new();
    for path in paths.iter().take(LISTED_LIMIT) {
        names.push(String::from_utf8_lossy(path).into_owned());
    }
    if paths.len() > LISTED_LIMIT {
        names.push(format!("and {} more", paths.len() - LISTED_LIMIT));
    }
    return names.join(", ");
}

/// Status entries,
/// renames,
/// copies,
/// the overlap and the report text.
#[cfg(test)]
#[path = "auto_push_dirty_paths_tests.rs"]
mod tests;
