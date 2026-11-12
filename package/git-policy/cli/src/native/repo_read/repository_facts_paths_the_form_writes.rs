//! What:
//!  The paths one guarded form writes into the worktree,
//!  asked of the tree or the index the form names,
//!  and the worktree entries that stand where those paths go.
//! Why:
//!  A command that would overwrite or delete a file Git did not put there is refused,
//!  and only Git knows which paths a tree or the index holds:
//!  `git checkout other -- dir` reaches every path of `other` below `dir`,
//!  which a directory name or a glob in this process could only guess.
//!  The worktree entries are the untracked ones a form did not create,
//!  and the comparison is Git's own answer read twice:
//!  the paths the form writes,
//!  and every entry Git does not track.
//!  `with_ancestors` and `replaced_answers` are the same readings that judge an ignored file,
//!  because a tree that needs a directory where the worktree holds a file deletes that file,
//!  and a tree that holds a file where the worktree holds a directory deletes everything inside it.
//!  `--prefix` moves every written path,
//!  as `read-tree --prefix` and `checkout-index --prefix` do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // entriesInTheWay(['b.txt'], ['b.txt', 'nested']) === ['b.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The captured-query runner's output type,
///  and the NUL-separated reading of an answer.
use crate::git_metadata::MetadataOutput;
/// The expression Git resolves for a shorthand only `switch` and `checkout` read.
use crate::command_discard::previous_branch_expression;
/// The reading of one path list,
///  the path relation a written path needs,
///  and every entry that stands where one is written.
use crate::rule_worktree_dirty::{nul_separated_paths, replaced_answers, with_ancestors};
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};

/// What:
///  Every path of one revision's tree,
///  or the paths of it the given pathspecs match when there are any,
///  as Git printed them;
///  nothing when Git cannot resolve the revision.
///  `Result<Option<Vec<Vec<u8>>>, String>` is "the paths,
///  no such revision,
///  or why Git could not be asked".
/// Why:
///   `git ls-tree -r -z --name-only` prints each path of the tree,
///  and Git itself matches the pathspecs,
///  so a directory,
///  a glob and a literal name reach exactly what the form reaches.
///  A revision that does not resolve writes no path:
///  Git refuses that command itself,
///  so nothing is destroyed and nothing is refused here for it.
///  `-` is the previous branch only to `switch` and `checkout`,
///  so it is read as `@{-1}` before the tree is listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treePaths(facts, revision: string, pathspecs: string[]): Promise<Buffer[] | undefined>; // throws
/// ```
pub(super) fn tree_paths(
    facts: &mut GitFacts,
    revision: Option<&OsStr>,
    pathspecs: &[OsString],
) -> Result<Option<Vec<Vec<u8>>>, String> {
    // `-` is the previous branch only to `switch` and `checkout`, so it is read as the expression Git resolves
    //  wherever an object name is read; `@{-N}` needs no such reading.
    let named: OsString = match revision {
        Some(written) => match previous_branch_expression(written.as_encoded_bytes()) {
            Some(expression) => OsString::from(String::from_utf8_lossy(expression).into_owned()),
            None => written.to_os_string(),
        },
        None => OsString::from("HEAD"),
    };
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-tree", "-r", "-z", "--name-only"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(named);
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query(arguments, "the tree this command writes")?;
    if !output.success {
        return Ok(None);
    }
    return Ok(Some(nul_separated_paths(output.stdout.as_slice())));
}

/// What:
///  Every path the index holds,
///  or the paths of it the given pathspecs match when there are any.
/// Why:
///  A `restore` without a source and a `checkout` without a revision write the index's content over the
///  worktree,
///  so the index is the tree those forms write;
///  a query that failed is a failure and never an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexPaths(facts, pathspecs: string[]): Promise<Buffer[]>; // throws
/// ```
pub(super) fn index_paths(facts: &mut GitFacts, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-files", "-z", "--cached", "--full-name"] {
        arguments.push(OsString::from(token));
    }
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query(arguments, "the paths the index holds")?;
    if !output.success {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("monochromatic-git-policy-cli could not read the paths the index holds: {detail}"));
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}

/// What:
///  The worktree entries that stand where `written` goes,
///  each once and in a settled order.
///  `written` borrows the paths the form writes,
///  `untracked` every entry Git does not track.
/// Why:
///   An entry at a written path is replaced by the tree's content;
///  one below a written path is deleted with the directory a file must replace,
///  and one above a written path is a file that must become a directory.
///  `with_ancestors` and `replaced_answers` ask exactly those three questions,
///  and a nested repository arrives as the directory entry Git prints for it,
///  so a refusal names it whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entriesInTheWay(written: Buffer[], untracked: Buffer[]): string[];
/// ```
pub(crate) fn entries_in_the_way(written: &[Vec<u8>], untracked: &[Vec<u8>]) -> Vec<String> {
    if written.is_empty() {
        return Vec::<String>::new();
    }
    let asked: Vec<Vec<u8>> = with_ancestors(written);
    let mut named: Vec<String> = Vec::<String>::new();
    for path in replaced_answers(asked.as_slice(), written, untracked) {
        named.push(String::from_utf8_lossy(path.as_slice()).into_owned());
    }
    named.sort();
    named.dedup();
    return named;
}

/// What:
///  Every path of `paths` under `prefix`,
///  or the same paths when the prefix is empty.
/// Why:
///  `read-tree --prefix` and `checkout-index --prefix` write the tree or the index below that directory,
///  so the worktree entries that would be replaced stand below it as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withPrefix(prefix: Buffer, paths: Buffer[]): Buffer[];
/// ```
pub(crate) fn with_prefix(prefix: &[u8], paths: &[Vec<u8>]) -> Vec<Vec<u8>> {
    if prefix.is_empty() {
        return paths.to_vec();
    }
    let mut moved: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(paths.len());
    for path in paths {
        let mut under: Vec<u8> = prefix.to_vec();
        under.extend_from_slice(path);
        moved.push(under);
    }
    return moved;
}
