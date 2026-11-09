//! What:
//!  Which branches a `git branch` region deletes.
//! Why:
//!  Deleting a branch takes its commits out of every reference's reach,
//!  and a forced deletion (`-D`) does
//!       that to a branch nothing else contains.
//!  The owner chose to let Git run as it always did and record what the deletion cost,
//!  so this file
//!       names the branches Git 2.56.0 would delete and no more.
//!  A form that creates or reads writes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchDeletions(['-D', 'gone']) === ['gone']
//! ```

use std::ffi::OsString;

/// What:
///  The first byte of a long option,
///  and the separator between a name and its value.
/// Why:
///   Options are compared as bytes and never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DASH = 0x2d; const EQUALS = 0x3d;
/// ```
const DASH: u8 = b'-';

/// What:
///  The names a `git branch` region deletes,
///  or nothing when the region deletes no branch.
///  `Option<Vec<OsString>>` is "the names,
/// or nothing".
/// Why:
///   `-d` (`--delete`),
///  `-D` and `--delete --force` delete local branches,
///  `-d` refusing one that is not
///       merged;
///  `-m`,
///  `-c` and every reading form write nothing,
///  so only these spellings are read.
///  `-d` may sit in a cluster (`-rd`),
///  the long form may be abbreviated to four letters,
///  and the names
///       follow the options or a `--`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function branchDeletions(region: string[]): string[] | undefined;
/// ```
pub fn branch_deletions(region: &[OsString]) -> Option<Vec<OsString>> {
    let mut deletes: bool = false;
    let mut options_end: bool = false;
    let mut names: Vec<OsString> = Vec::<OsString>::new();
    for token in region {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        if !options_end && bytes.starts_with(&[DASH]) {
            if spells_delete(bytes) {
                deletes = true;
            }
            continue;
        }
        names.push(token.clone());
    }
    if !deletes {
        return None;
    }
    return Some(names);
}

/// What:
///  Whether one option token is a deletion spelling.
/// Why:
///   `-d` and `-D` are the short forms,
///  alone or inside a cluster,
///  and `--delete` is the long one,
///  read in its exact spelling only:
///  every
///       abbreviation of it is ambiguous with `--delete-merged` in Git 2.56.0,
///  and `--delete-merged` itself
///       deletes branches that a merged branch already contains,
///  so it cannot lose a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spellsDelete(token: Buffer): boolean;
/// ```
fn spells_delete(token: &[u8]) -> bool {
    if token.starts_with(b"--") {
        return token == b"--delete";
    }
    // A short cluster such as `-rd`: every letter after the dash is a flag of its own.
    return token[1..].iter().any(is_delete_letter);
}

/// What:
///  Whether one letter of a short cluster is either deletion flag.
/// Why:
///   A named function keeps the scan free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (letter: number) => letter === 0x64 || letter === 0x44
/// ```
fn is_delete_letter(letter: &u8) -> bool {
    return *letter == b'd' || *letter == b'D';
}

/// The spellings,
///  the clusters and the forms that delete nothing stay out of the release executable.
#[cfg(test)]
#[path = "command_branch_delete_tests.rs"]
mod tests;
