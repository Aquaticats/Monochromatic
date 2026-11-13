//! What:
//!  The paths one guarded form writes into the worktree,
//!  asked of the tree or the index the form names.
//! Why:
//!  Refusing a form that would write over a file nothing else holds needs the written paths first,
//!  and only Git knows which paths a pathspec reaches in a tree:
//!  `git checkout other -- dir` writes every path of `other` below `dir`,
//!  which a glob or a directory name would have to be re-read to guess.
//!  Every directory on the way to a written path is asked about as well,
//!  because a tree creates the directories it needs,
//!  and a worktree that holds a file where one of them must go loses that file.
//!  A form that reads its pathspecs from a file names paths no token does, so the file is read
//!  here, the way Git reads it.
//!  A revision Git cannot resolve writes no path,
//!  which is the reading `git checkout -f <a path>` needs:
//!  Git writes that path from the index and no tree at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsTheTreeWrites(facts, { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }) === ['b.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The written-path reading this fact answers.
use crate::command_written_paths::{PathspecTarget, WrittenPaths};
/// The captured-query runner's output type,
///  and the NUL-separated reading of an answer.
use crate::git_metadata::MetadataOutput;
use crate::rule_worktree_dirty::nul_separated_paths;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Every path at which the form writes a file,
///  `Result<Vec<Vec<u8>>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   A query that failed says nothing,
///  and a safety check must never read a failure as "nothing is written".
///  The directories on the way to a path are not listed:
///  Git creates the ones it needs,
///  so a directory there loses nothing,
///  while a file at one of those paths would lose its bytes and is found as the entry above the
///  written path
///  (`repository_facts_paths_in_the_way.rs` says how).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsTheTreeWrites(facts, written): Promise<Buffer[]>; // throws
/// ```
pub(super) fn paths_the_tree_writes(facts: &mut GitFacts, written: &WrittenPaths) -> Result<Vec<Vec<u8>>, String> {
    return match written {
        WrittenPaths::Tree { revision, .. } => tree_paths(facts, revision.as_deref(), &[]),
        WrittenPaths::MatchingTree { revision, pathspecs } => tree_paths(facts, revision.as_deref(), pathspecs.as_slice()),
        WrittenPaths::Index { pathspecs } => index_paths(facts, pathspecs.as_slice()),
        WrittenPaths::IndexUnderPrefix { pathspecs, prefix } => index_under_prefix_paths(facts, pathspecs, prefix),
        WrittenPaths::Paths { paths } => Ok(paths.iter().map(|path| return path.as_encoded_bytes().to_vec()).collect()),
        WrittenPaths::ReadTree { revisions, prefix, .. } => read_tree_paths(facts, revisions.as_slice(), prefix.as_deref()),
        WrittenPaths::PathspecFile { target, file, nul_separated, directory } => {
            let at: Option<&Path> = directory.as_ref().map(|named| return Path::new(named));
            let pathspecs: Vec<OsString> = pathspecs_in_the_file(file, *nul_separated, at)?;
            if pathspecs.is_empty() {
                Ok(Vec::<Vec<u8>>::new())
            } else {
                match target {
                    PathspecTarget::Tree(revision) => tree_paths(facts, revision.as_deref(), pathspecs.as_slice()),
                    PathspecTarget::Index => index_paths(facts, pathspecs.as_slice()),
                }
            }
        }
    };
}

/// What:
///  The pathspecs the file a form names holds,
///  each as its own token,
///  `Result<Vec<OsString>, String>` is "the pathspecs, or why the file could not be read".
///  `&OsStr` borrows the file's name, `&Path` the directory a relative name is read from.
/// Why:
///  `--pathspec-from-file` names paths no token of the command does,
///  so a reading that ignores it asks Git about nothing and the file the command writes over is
///  never seen.
///  Git separates the entries with line ends, or with NUL bytes when `--pathspec-file-nul` is
///  written, and a line ending may carry the carriage return of a CRLF file.
///  A file that cannot be read is a failure, never an empty answer:
///  Git refuses the command for the same reason, and a guard must not read that as "nothing is
///  written".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathspecsInTheFile(file, nulSeparated, directory): string[]; // throws
/// ```
fn pathspecs_in_the_file(file: &OsStr, nul_separated: bool, directory: Option<&Path>) -> Result<Vec<OsString>, String> {
    // `-` is standard input to Git, which this guard cannot read without consuming what Git needs.
    if file.as_encoded_bytes() == b"-" {
        return Err(String::from(
            "Git could not read the pathspecs `--pathspec-from-file` names, because `-` means standard input, which the guard cannot read before Git does",
        ));
    }
    let named: PathBuf = PathBuf::from(file);
    let absolute: PathBuf = if named.is_absolute() {
        named
    } else {
        let base: PathBuf = match directory {
            Some(given) => given.to_path_buf(),
            None => match std::env::current_dir() {
                Ok(here) => here,
                Err(error) => {
                    return Err(format!("Git could not read the pathspecs `{}` names: {error}", named.display()));
                }
            },
        };
        base.join(named.as_path())
    };
    let bytes: Vec<u8> = match std::fs::read(absolute.as_path()) {
        Ok(read) => read,
        Err(error) => {
            return Err(format!("Git could not read the pathspecs `{}` names: {error}", absolute.display()));
        }
    };
    let separator: u8 = if nul_separated { 0 } else { b'\n' };
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    for entry in bytes.split(|byte| return *byte == separator) {
        let trimmed: &[u8] = entry.strip_suffix(b"\r").unwrap_or(entry);
        if trimmed.is_empty() {
            continue;
        }
        pathspecs.push(path_from_bytes(trimmed));
    }
    return Ok(pathspecs);
}

/// What:
///  One path as the operating system takes it,
///  from the bytes a file or Git printed.
/// Why:
///  A name that is not UTF-8 is still a name;
///  converting it through `String` would replace its bytes and look up a file that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromBytes = (bytes) => bytes; // a Buffer is already the name
/// ```
pub(super) fn path_from_bytes(bytes: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return OsStr::from_bytes(bytes).to_os_string();
    }
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(bytes).into_owned());
    }
}

/// What:
///  The paths of one revision's tree that the given pathspecs match,
///  or every path of it when there are none.
/// Why:
///   `git ls-tree -r -z --name-only` prints each path of the tree,
///  and Git itself matches the pathspecs,
///  so a directory, a glob and a literal name reach exactly what the form reaches.
///  A revision that does not resolve writes no path;
///  a query that could not run at all is a failure.
///  `-` is the previous branch only to `switch` and `checkout`,
///  so it is read as `@{-1}` before the tree is listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treePaths(facts, revision: string | undefined, pathspecs: string[]): Promise<Buffer[]>; // throws
/// ```
fn tree_paths(facts: &mut GitFacts, revision: Option<&OsStr>, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
    // `-` is the previous branch only to `switch` and `checkout`, so it is read as the expression Git resolves
    //  wherever an object name is read; `@{-N}` needs no such reading.
    let named: OsString = match revision {
        Some(written) => match crate::command_discard::previous_branch_expression(written.as_encoded_bytes()) {
            Some(expression) => OsString::from(String::from_utf8_lossy(expression).into_owned()),
            None => written.to_os_string(),
        },
        None => OsString::from("HEAD"),
    };
    return paths_of_a_named_tree(facts, named.as_os_str(), pathspecs);
}

/// What:
///  The paths of every tree a `git read-tree` form names,
///  each below the directory its `--prefix` writes it in.
///  `&[OsString]` borrows the names the form carries and the prefix it may name.
/// Why:
///   Every path a merge or a reset can write comes from one of the trees it names,
///  and `--prefix` moves each of them below a directory the form names,
///  which is where the worktree entry Git writes over really stands.
///  A revision here is read as Git reads a tree name:
///  `-` is no previous branch to `read-tree`, so it is not read as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreePaths(facts, ['HEAD', 'other'], 'sub/') ==> ['sub/a.txt', 'sub/b.txt']
/// ```
fn read_tree_paths(facts: &mut GitFacts, revisions: &[OsString], prefix: Option<&OsStr>) -> Result<Vec<Vec<u8>>, String> {
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for revision in revisions {
        for path in paths_of_a_named_tree(facts, revision.as_os_str(), &[])? {
            paths.push(under_the_prefix(path, prefix));
        }
    }
    return Ok(paths);
}

/// What:
///  One path as the form writes it:
///  below the prefix when it names one,
///  and unchanged when it names none.
/// Why:
///  `read-tree --prefix=sub/ -u other` writes `other`'s `a.txt` at `sub/a.txt`,
///  so the entry in the way is the one below the prefix, not the tree's own path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const underThePrefix = (path, prefix) => prefix ? Buffer.concat([prefix, path]) : path;
/// ```
fn under_the_prefix(path: Vec<u8>, prefix: Option<&OsStr>) -> Vec<u8> {
    let Some(named) = prefix else {
        return path;
    };
    let mut shifted: Vec<u8> = named.as_encoded_bytes().to_vec();
    shifted.extend(path);
    return shifted;
}

/// What:
///  The paths of one named tree that the given pathspecs match,
///  or every path of it when there are none,
///  read with `git ls-tree -r -z --name-only`.
/// Why:
///   The name is already resolved to what Git reads,
///  and a query that could not run at all is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsOfANamedTree(facts, 'other', ['b.txt']): Promise<Buffer[]>; // throws
/// ```
fn paths_of_a_named_tree(facts: &mut GitFacts, named: &OsStr, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-tree", "-r", "-z", "--name-only"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(named.to_os_string());
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query(arguments, "the tree this command writes")?;
    if !output.success {
        return Ok(Vec::<Vec<u8>>::new());
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  each below the directory a `git checkout-index --prefix` names.
///  `&OsString` borrows them and the directory.
/// Why:
///  `git checkout-index --prefix=<directory>/ --all` writes the index's content there,
///  so the worktree entry a form would write over stands at that shifted path,
///  and no query of the index knows the directory's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexUnderPrefixPaths(facts, pathspecs, prefix): Promise<Buffer[]>; // throws
/// ```
fn index_under_prefix_paths(facts: &mut GitFacts, pathspecs: &[OsString], prefix: &OsStr) -> Result<Vec<Vec<u8>>, String> {
    let mut shifted: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in index_paths(facts, pathspecs)? {
        shifted.push(under_the_prefix(path, Some(prefix)));
    }
    return Ok(shifted);
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  or every path of it when there are none.
/// Why:
///  A restore without a source writes the index's content over the worktree,
///  and a query that failed is a failure rather than an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexPaths(facts, pathspecs: string[]): Promise<Buffer[]>; // throws
/// ```
fn index_paths(facts: &mut GitFacts, pathspecs: &[OsString]) -> Result<Vec<Vec<u8>>, String> {
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
        return Err(format!("Git could not read the paths the index holds: {detail}"));
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}
