//! What:
//!  How the conversion fence makes one Git run,
//!  and the two runs that read the blob sizes it decides with.
//! Why:
//!   Every run of the fence names a failure the same way.
//! The map both size reads answer is one type here.
//! The parser is one function,
//!  because the read the ladder starts before its source check
//!  and the read the fence makes where it stands answer the same question
//!  (`worktree_clone_conversion_size_read.rs`,
//!  `worktree_clone_conversion_fence.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = runIn(context, root, ['ls-tree', '-r', '-l', '-z', commit], null, []);
//! const sizes = parseSizes(output.stdout);
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The blob size of every path of one commit's tree,
/// as the fence's `git ls-tree` reports it.
/// Why:
///   Both places the sizes are read answer this one map:
/// the read the ladder starts before the source check
///  (`worktree_clone_conversion_size_read.rs`),
///  and the one the fence makes where it stands
/// (`worktree_clone_conversion_fence.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BlobSizes = Map<Uint8Array, number>;
/// ```
pub type BlobSizes = HashMap<Vec<u8>, u64>;

/// What:
///  One Git run's failure as the reason the rung ends,
/// with Git's own words.
/// Why:
///   The rung's answer is a reason string,
/// and a run that cannot start or that fails says which run and what Git said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = (what, output) => `${what} ended with ${output.code} (${output.stderr.trim()})`;
/// ```
pub(crate) fn run_reason(what: &str, output: &GitOutput) -> String {
    return format!("{what} ended with {:?} ({})", output.code, String::from_utf8_lossy(output.stderr.as_slice()).trim());
}

/// What:
///  `run_reason` for the size read the ladder may start before the source check.
/// Why:
///   That read's output is parsed by `parse_sizes` here,
/// so a run that ended badly has to be named here too;
/// one sentence names it in both places.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadable = (output) => `the sizes of the committed blobs cannot be read: ${runReason('`git ls-tree`', output)}`;
/// ```
pub(crate) fn unreadable_sizes(output: &GitOutput) -> String {
    return format!("the sizes of the committed blobs cannot be read: {}", run_reason("`git ls-tree`", output));
}

/// What:
///  One Git run the fence could not make,
/// as the reason the fence reads with it.
/// Why:
///   A run that cannot start and a read that never answered are the same thing to the fence:
/// no output to decide with,
///  and the reason names the run and the system's words.
/// One formatter keeps the two sentences the same
///  (`worktree_clone_conversion_size_read.rs`,
///  the join's deadline).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const didNotAnswer = (error) => `a Git run of the fence could not start (${error})`;
/// ```
pub(crate) fn a_git_run_that_did_not_answer(error: &std::io::Error) -> String {
    return format!("a Git run of the fence could not start ({error})");
}

/// What:
///  Run one Git command of the fence,
/// with optional standard input and environment,
/// and give its output.
/// Why:
///   Every read of the fence is one step of the add's own record,
/// and a run that cannot start answers a reason rather than a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runIn(context, cwd, arguments, input, environment): GitOutput;
/// ```
pub(crate) fn run_in(context: &GitContext, cwd: &Path, arguments: &[OsString], input: Option<Vec<u8>>, environment: Vec<(OsString, OsString)>) -> Result<GitOutput, String> {
    let request: GitRequest = GitRequest {
        cwd: cwd.to_path_buf(),
        arguments: arguments.to_vec(),
        without_prefix: true,
        environment,
        input,
        ..GitRequest::default()
    };
    return run_git(context, &request).map_err(|error| return a_git_run_that_did_not_answer(&error));
}

/// What:
///  A command line's words as the owned strings a request carries.
/// Why:
///   One request takes an owned argument a caller built (`--prefix`),
/// so every request of the fence is built the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (list: string[]) => list.map((word) => word);
/// ```
pub(crate) fn words(list: &[&str]) -> Vec<OsString> {
    return list.iter().map(|word| return OsString::from(word)).collect();
}

/// What:
///  The blob size of every path of the commit's tree,
/// as one `git ls-tree -r -l -z` reports them.
/// Why:
///   The size is what proves that a file whose line-ending rule only strips carriage returns still holds its blob's bytes.
/// This is the read the fence makes where it stands;
///  rung 1's read may have started earlier
/// (`worktree_clone_conversion_size_read.rs`),
///  and both parse the same bytes with `parse_sizes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blobSizes(context, root, commit): Map<string, number>;
/// ```
pub(crate) fn blob_sizes(context: &GitContext, root: &Path, commit: &str) -> Result<BlobSizes, String> {
    let output: GitOutput = run_in(context, root, &words(&["ls-tree", "-r", "-l", "-z", commit]), None, Vec::new())?;
    if output.code != Some(0) {
        return Err(unreadable_sizes(&output));
    }
    return Ok(parse_sizes(output.stdout.as_slice()));
}

/// What:
///  The output of one `git ls-tree -r -l -z`,
/// as the size of every path it lists.
/// Why:
///   One parser for the two places the read runs,
/// so a size that cannot be listed (a partial clone prints `BAD`)
///  is missing from the map in both of them,
/// and a missing size only ever sends a path to Git's writing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSizes(stdout): Map<string, number> { return stdout.split('\0').flatMap(parseRecord); }
/// ```
pub(crate) fn parse_sizes(stdout: &[u8]) -> BlobSizes {
    let mut sizes: BlobSizes = HashMap::new();
    for record in stdout.split(|byte| return *byte == 0) {
        // `mode SP type SP object SP size TAB path`
        let Some(tab) = record.iter().position(|byte| return *byte == b'\t') else { continue };
        // The size is right-aligned, so the words between the spaces hold empty ones that are not fields.
        let mut fields = record[..tab].split(|byte| return *byte == b' ').filter(|word| return !word.is_empty());
        let (Some(_mode), Some(_kind), Some(_object), Some(size)) = (fields.next(), fields.next(), fields.next(), fields.next()) else { continue };
        let Ok(size_text) = std::str::from_utf8(size) else { continue };
        let Ok(size_value) = size_text.parse::<u64>() else { continue };
        sizes.insert(record[tab + 1..].to_vec(), size_value);
    }
    return sizes;
}
