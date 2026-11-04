//! What:
//!  The object name of the empty tree under each object format Git supports,
//!  known without asking Git.
//! Why:
//!  The invocation capture needs the empty tree as the baseline of an unborn branch.
//!  Git hashes the framed bytes `tree 0` followed by a NUL byte,
//!  so the name depends on the object format alone and never on the repository's contents.
//!  Spawning `git hash-object -t tree --stdin` for a constant cost a process on every commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const EMPTY_TREE = { sha1: '4b825d...', sha256: '6ef19b...' } as const;
//! ```

/// The empty tree under SHA-1,
///  the object format of nearly every repository.
pub const SHA1_EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// The empty tree under SHA-256.
pub const SHA256_EMPTY_TREE: &str = "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321";

/// What:
///  The empty tree's object name for an object format as `git rev-parse --show-object-format` spells it,
///  or `None` for a format this table does not know.
/// Why:
///   `None` sends the caller to ask Git,
///  so a future object format is answered by Git and never guessed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emptyTreeName(objectFormat: string): string | undefined;
/// ```
pub fn empty_tree_name(object_format: &str) -> Option<&'static str> {
    return match object_format {
        "sha1" => Some(SHA1_EMPTY_TREE),
        "sha256" => Some(SHA256_EMPTY_TREE),
        _ => None,
    };
}

/// Empty-tree controls stay out of the release executable.
#[cfg(test)]
#[path = "empty_tree_tests.rs"]
mod tests;
