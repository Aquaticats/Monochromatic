//! What:
//!  The questions the restore asks of the stage:
//! the words of each Git run there,
//! one captured read under the new worktree's Git directory,
//! and the ignored paths the stage holds under the rules it has now.
//! Why:
//!   Every run names the stage directory itself,
//! so none can run in the main checkout,
//! and the ignored paths are what the restore carries through the clean and the reset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const carried = listedIgnored({ context, target, stage }); // throws the reason
//! ```

use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use crate::worktree_ladder_target::{
    Target,
    checkout_environment,
};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  Whether a byte is the NUL that ends each name in Git's `-z` output.
/// Why:
///   The untracked and the ignored names are split on it,
/// and a path may hold any other byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
pub(super) fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The words of the query that lists the stage's ignored paths under the stage's own rules,
/// which are the source's until the
///  reset brings the new commit's.
/// Why:
///   `--directory` collapses a whole ignored directory into one name,
/// so a carried `node_modules` costs one
///  entry rather than every file in it,
/// and every name ends with `/` when it is a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IGNORED_WORDS = ['ls-files', '--others', '--ignored', '--exclude-standard', '--directory', '--no-empty-directory', '-z']
/// ```
const IGNORED_WORDS: &[&str] = &[
    "ls-files",
    "--others",
    "--ignored",
    "--exclude-standard",
    "--directory",
    "--no-empty-directory",
    "-z",
];

/// What:
///  Whether a name split out of `-z` output holds anything.
/// Why:
///   The output ends with a NUL,
/// so the split leaves an empty last piece that names no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.length > 0
/// ```
pub(super) fn is_nonempty_name(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  The words of a Git run in the stage:
/// `-C <stage>` and then `words`.
/// Why:
///   Every run of the restore names the stage directory itself,
/// so none can run in the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-C', stage, ...words]
/// ```
pub(super) fn arguments_in_stage(
    stage: &Path,
    words: &[&str],
) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = vec![OsString::from("-C"), stage.as_os_str().to_os_string()];
    arguments.extend(words.iter().copied().map(OsString::from));
    return arguments;
}

/// What:
///  Run one captured Git read in the stage under the new worktree's Git directory.
/// Why:
///   The proof reads the stage the way the restore wrote it,
/// never the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = await runIn(stage, admin, ['diff-files', '--quiet']);
/// ```
pub(super) fn ask(
    context: &GitContext,
    target: &Target,
    work_tree: &Path,
    arguments: &[&str],
) -> Result<(Option<i32>, Vec<u8>), String> {
    // The index is named through the request's own field as well as the
    // environment, so the wrapper's in-process readers answer from the same file
    // the started Git reads (`GitRequest::index`), which is what makes the
    // private name the one file every reader of a restore agrees on.
    let request: GitRequest = GitRequest {
        cwd: work_tree.to_path_buf(),
        arguments: arguments.iter().copied().map(OsString::from).collect(),
        without_prefix: true,
        index: target.index.clone(),
        environment: checkout_environment(target.admin.as_path(), work_tree),
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(format!(
                "`git {}` could not start ({error})",
                arguments.join(" ")
            ));
        }
    };
    return Ok((output.code, output.stdout));
}

/// What:
///  The ignored paths of the stage,
/// listed under the rules the stage has now,
/// as the boundaries of a whole ignored
///  directory collapse them.
/// Why:
///   The restore carries them through the clean and the reset,
/// so it must know them before either
///  runs;
/// a query that fails ends the rung instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carried = listedIgnored({ context, target, stage }); // throws the reason
/// ```
pub(super) fn listed_ignored(
    context: &GitContext,
    target: &Target,
    stage: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    let (listed, names) = ask(context, target, stage, IGNORED_WORDS)?;
    if listed != Some(0) {
        return Err(String::from(
            "Git could not list the ignored files of the snapshot",
        ));
    }
    return Ok(names
        .split(is_nul_byte)
        .filter(is_nonempty_name)
        .map(<[u8]>::to_vec)
        .collect());
}
