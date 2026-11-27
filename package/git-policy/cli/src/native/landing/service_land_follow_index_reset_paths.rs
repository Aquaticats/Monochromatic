//! What:
//!  The paths a reset installs, as the bytes `git reset` reads them:
//! a pathspec that names one path from the top of the worktree,
//! a NUL-separated list of them,
//! and the names that differ between two commits.
//! Why:
//!  A commit Git built in a scratch repository carries no list of changes,
//! so the entries come from the repository itself,
//! as bytes,
//! so a name that is not UTF-8,
//! a long list and a gitlink need no special case
//! (`service_land_follow_index_reset.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const input = pathspecInput(derivedNames(git, repository, old, new)); // :(top,literal)<path> per name, NUL-separated
//! ```

use crate::service_land_follow_index_reset_git_questions::{
    classify,
    run,
    run_awaiting,
};
use crate::service_protocol::{
    FailureCode,
    is_zero_object,
};
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
use std::path::Path;

/// A path in the form `reset` reads literally from the top of the repository.
pub(super) fn literal(path: &[u8]) -> Vec<u8> {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(path);
    return spec;
}

/// What:
///  Whether a byte is the NUL that separates the names of a pathspec list.
/// Why:
///  `pathspec_input` splits a NUL-separated list into names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZeroByte = (byte: number) => byte === 0;
/// ```
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether a name from a NUL-separated list is not empty.
/// Why:
///  Two NULs in a row leave an empty name that names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyName = (name: Uint8Array) => name.length > 0;
/// ```
fn is_not_empty_name(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  A path as a pathspec that matches that path from the top of the worktree and nothing else.
/// Why:
///  `reset` is given one such spec per path the batch changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const topLiteralSpec = (path: string) => `:(top,literal)${path}`;
/// ```
pub(super) fn top_literal_spec(path: &String) -> String {
    return format!(":(top,literal){path}");
}

/// The standard input of `reset --pathspec-from-file=- --pathspec-file-nul` for a list of NUL-separated names.
pub(super) fn pathspec_input(names: &[u8]) -> Vec<u8> {
    let mut input: Vec<u8> = Vec::new();
    for name in names.split(is_zero_byte).filter(is_not_empty_name) {
        input.extend(literal(name));
        input.push(0);
    }
    return input;
}

/// The names (bytes,
/// NUL-separated) of the paths that differ between `old` and `new`.
pub(crate) fn derived_names(
    git: &GitContext,
    repository: &Path,
    old: &str,
    new: &str,
) -> Result<Vec<u8>, FailureCode> {
    let output: GitOutput = run(
        git,
        repository,
        &derived_names_arguments(old, new),
        None,
        None,
    )?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// What:
///  `derived_names` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the paths a batch changed are read when the index is
///  updated from the landing.
/// The arguments and the failure a refused step stands for are the synchronous face's,
/// so the two
///  read one list of names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = await derivedNamesAwaiting(git, repository, old, new);
/// ```
pub(crate) async fn derived_names_awaiting(
    git: &GitContext,
    repository: &Path,
    old: &str,
    new: &str,
) -> Result<Vec<u8>, FailureCode> {
    let output: GitOutput = run_awaiting(
        git,
        repository,
        &derived_names_arguments(old, new),
        None,
        None,
    )
    .await?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// The arguments of the question `derived_names` asks Git.
fn derived_names_arguments<'a>(
    old: &'a str,
    new: &'a str,
) -> Vec<&'a str> {
    let mut arguments: Vec<&str> = vec!["diff-tree", "-r", "--name-only", "-z", "--no-renames"];
    if is_zero_object(old) {
        // A branch that did not exist: the commit is a root commit, and `--root` lists everything it holds.
        arguments.push("--root");
        arguments.push(new);
    } else {
        arguments.push(old);
        arguments.push(new);
    }
    return arguments;
}
