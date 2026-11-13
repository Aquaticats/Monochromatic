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
        WrittenPaths::Tree { revision, guesses_a_remote_branch, .. } => {
            tree_paths(facts, revision.as_deref(), &[], *guesses_a_remote_branch)
        }
        WrittenPaths::MatchingTree { revision, pathspecs } => {
            // A form that names paths writes the tree of the word before them, and Git resolves that word as a
            //  revision alone: `checkout -f topic -- b.txt` answers `invalid reference: topic` for a remote-only name.
            tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false)
        }
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
                    PathspecTarget::Tree(revision) => tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false),
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
    // Git itself waits for a writer on a FIFO and for a mount that stopped answering
    //  (`timeout 10 git checkout --pathspec-from-file=<fifo>` was still waiting when the probe ended it,
    //  measured on this host with native Git 2.56.0, 2026-11-13),
    //  and the wrapper's own rule is that nothing waits forever, so the read runs on a thread this
    //  process stops waiting for at the read ceiling.
    let ceiling: std::time::Duration = crate::wait_ceilings::PATHSPEC_FILE_READ_CEILING;
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
    let opened: PathBuf = absolute.clone();
    let Some(read) = crate::work_awaited_within_a_ceiling::work_within(ceiling, move || {
        return std::fs::read(opened.as_path());
    }) else {
        return Err(format!(
            "the file `{}` that `--pathspec-from-file` names gave no bytes within {} seconds, so the guard cannot tell which paths this command writes over",
            absolute.display(),
            ceiling.as_secs()
        ));
    };
    let bytes: Vec<u8> = match read {
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
///  or every path of it when there are none,
///  read from the name the command itself turns into a tree.
///  `bool` says whether `checkout` and `switch` are free to guess the remote branch a short name
///  stands for.
/// Why:
///   `git ls-tree -r -z --name-only` prints each path of the tree,
///  and Git itself matches the pathspecs,
///  so a directory, a glob and a literal name reach exactly what the form reaches.
///  `-` is the previous branch only to `switch` and `checkout`,
///  so it is read as `@{-1}` before the tree is listed.
///  A name Git cannot turn into a tree is never read as "this form writes nothing":
///  `git ls-tree topic` fails for a remote-only branch while `git checkout -f topic` writes that
///  branch's tree,
///  so the name is resolved the way the command resolves it,
///  and a tree Git knows but cannot list is a refusal
///  (`name_git_may_still_reach` says how the two are told apart).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treePaths(facts, revision, pathspecs, guesses): Promise<Buffer[]>; // throws
/// ```
fn tree_paths(
    facts: &mut GitFacts,
    revision: Option<&OsStr>,
    pathspecs: &[OsString],
    guesses_a_remote_branch: bool,
) -> Result<Vec<Vec<u8>>, String> {
    // `-` is the previous branch only to `switch` and `checkout`, so it is read as the expression Git resolves
    //  wherever an object name is read; `@{-N}` needs no such reading.
    let named: OsString = match revision {
        Some(written) => match crate::command_discard::previous_branch_expression(written.as_encoded_bytes()) {
            Some(expression) => OsString::from(String::from_utf8_lossy(expression).into_owned()),
            None => written.to_os_string(),
        },
        None => OsString::from("HEAD"),
    };
    let Some(paths) = listed_paths_of(facts, named.as_os_str(), pathspecs)? else {
        return tree_of_a_name_git_may_still_reach(facts, named.as_os_str(), pathspecs, guesses_a_remote_branch);
    };
    return Ok(paths);
}

/// What:
///  The tree of a name `git ls-tree` did not list,
///  when Git may still reach one through it:
///  the local branch a `checkout` or a `switch` creates from a remote-tracking branch of that name.
/// Why:
///  `git checkout -f topic` with only `refs/remotes/origin/topic` writes that tree over the
///  worktree
///  (`Switched to a new branch 'topic'`, measured on this host with native Git 2.56.0,
///  2026-11-13),
///  so a guard that read the failed listing as "nothing is written" let the forced form overwrite
///  an untracked file.
///  The three answers this reading can give are all safe ones:
///  the tree's paths, so the entries in the way are found;
///  no path at all, only when Git itself refuses the name and no branch is there to guess; and
///  a refusal, when Git knows an object by the name but could not list its tree,
///  or when the guessed branch's tree could not be listed either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeOfANameGitMayStillReach(facts, 'topic', [], true): Promise<Buffer[]>; // throws
/// ```
fn tree_of_a_name_git_may_still_reach(
    facts: &mut GitFacts,
    named: &OsStr,
    pathspecs: &[OsString],
    guesses_a_remote_branch: bool,
) -> Result<Vec<Vec<u8>>, String> {
    if git_holds_an_object_named(facts, named)? {
        return Err(tree_reading_failure(named, "Git knows an object by that name and did not list its tree"));
    }
    // A form that names paths, a reset, and a restore never guess: Git refuses the name itself
    //  (`invalid reference`, `ambiguous argument`, `could not resolve`), so there is nothing to write.
    if !guesses_a_remote_branch || !pathspecs.is_empty() || !guessing_is_left_on_in_this_repository(facts)? {
        return Ok(Vec::<Vec<u8>>::new());
    }
    let Some(remote_branch) = remote_branch_a_checkout_would_create_from(facts, named)? else {
        return Ok(Vec::<Vec<u8>>::new());
    };
    return match listed_paths_of(facts, remote_branch.as_os_str(), &[])? {
        Some(paths) => Ok(paths),
        None => Err(tree_reading_failure(
            remote_branch.as_os_str(),
            "Git named that branch and did not list its tree",
        )),
    };
}

/// What:
///  The sentence a refusal prints when the guard could not read the tree a form writes,
///  naming the tree and what Git answered.
/// Why:
///  The guard cannot tell which of the person's files the command would write over,
///  so the command is not run;
///  the sentence says which name failed, that nothing ran, and that the option runs the command
///  as real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = treeReadingFailure('topic', 'Git knows an object by that name and did not list its tree');
/// ```
fn tree_reading_failure(named: &OsStr, what_git_did: &str) -> String {
    return format!(
        "the guard could not read the tree `{}` names, so it cannot tell which of your files this command would write over ({what_git_did})",
        named.to_string_lossy()
    );
}

/// What:
///  The paths `git ls-tree -r -z --name-only` answers for one name and its pathspecs,
///  or nothing when Git exited nonzero.
/// Why:
///  A listing that ran and failed says the name did not resolve to a tree Git can list;
///  a listing that could not run at all is a failure of the query, never a killed read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedPathsOf(facts, 'other', ['b.txt']): Promise<Buffer[] | undefined>;
/// ```
fn listed_paths_of(facts: &mut GitFacts, named: &OsStr, pathspecs: &[OsString]) -> Result<Option<Vec<Vec<u8>>>, String> {
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
        return Ok(None);
    }
    return Ok(Some(nul_separated_paths(output.stdout.as_slice())));
}

/// What:
///  Whether Git holds an object the name resolves to,
///  which tells a name Git does not know from a tree it could not read.
/// Why:
///  `git ls-tree` exits nonzero for both,
///  and the two mean opposite things for a guard:
///  a name Git does not know is a form Git refuses itself, so nothing is written,
///  while an object Git knows and cannot list is a read that failed,
///  and what such a command writes is unknown.
///  `rev-parse --verify` resolves the name without reading the object,
///  so it answers for a tree whose bytes are missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const known = (facts, 'topic') => facts.query(['rev-parse', '--verify', '--quiet', '--end-of-options', 'topic']).success;
/// ```
fn git_holds_an_object_named(facts: &mut GitFacts, named: &OsStr) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "rev-parse", "--verify", "--quiet", "--end-of-options"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(named.to_os_string());
    let output: MetadataOutput = facts.query(arguments, "whether Git knows this revision")?;
    return Ok(output.success);
}

/// What:
///  Whether the repository leaves `checkout`'s guess of a remote branch on.
/// Why:
///  `checkout.guess=false` makes Git refuse the short name
///  (`error: pathspec 'topic' did not match any file(s) known to git`),
///  so the guard must not resolve a branch Git never touches:
///  a refusal over a name Git rejects is a command refused that Git would not have run.
///  An unset key, or an empty value, leaves the guess on;
///  a value Git does not read as a boolean is a configuration error of its own,
///  so the guard leaves the command to Git (which refuses with its own sentence about the value).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const on = (facts) => { const v = facts.query(['config', '--get', 'checkout.guess']); return !v.success || v.stdout.length === 0 || /^(true|yes|on|1)$/i.test(v.stdout.trim()); };
/// ```
fn guessing_is_left_on_in_this_repository(facts: &mut GitFacts) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "config", "--get", "checkout.guess"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the repository's answer on guessing a remote branch")?;
    if !output.success {
        // `config --get` exits 1 for a key that is not set, and Git then guesses.
        //  Any other nonzero is a configuration Git objects to, and Git then refuses the command
        //  itself, so the guard resolves nothing either.
        return Ok(output.code == Some(1));
    }
    let value: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_ascii_lowercase();
    if value.is_empty() {
        return Ok(true);
    }
    return Ok(matches!(value.as_str(), "true" | "yes" | "on" | "1"));
}

/// What:
///  The remote-tracking ref a `checkout` or a `switch` would create the local branch from,
///  worked out the way Git works it out
///  (`unique_tracking_name` in `branch.c`).
/// Why:
///  One remote-tracking branch of that name is the branch Git creates;
///  several are an ambiguity Git refuses
///  (`hint: Branch name 'topic' appears in multiple remotes`, measured on this host with native Git
///  2.56.0, 2026-11-13)
///  unless `checkout.defaultRemote` names one of them,
///  and then that one is Git's answer.
///  The remotes are read from Git rather than split from the refname, because a remote's own name
///  may hold a slash:
///  with the remote `a/b`, `refs/remotes/a/b/topic` is the branch `topic` Git guesses from
///  (`Switched to a new branch 'topic'`, measured on this host).
///  A listing that failed is a failure of the query, never "no branch to guess".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remoteBranchACheckoutWouldCreateFrom(facts, 'topic'): Promise<string | undefined>;
/// ```
fn remote_branch_a_checkout_would_create_from(facts: &mut GitFacts, named: &OsStr) -> Result<Option<OsString>, String> {
    let remotes: Vec<Vec<u8>> = remote_names(facts)?;
    if remotes.is_empty() {
        return Ok(None);
    }
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "for-each-ref", "--format=%(refname)", "refs/remotes/"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remote-tracking branches this repository holds")?;
    if !output.success {
        return Err(String::from(
            "Git could not list the remote-tracking branches, so the guard cannot tell what this command would write over",
        ));
    }
    let mut candidates: Vec<(Vec<u8>, Vec<u8>)> = Vec::<(Vec<u8>, Vec<u8>)>::new();
    for line in output.stdout.split(|byte| return *byte == b'\n') {
        let Some(short) = line.strip_prefix(b"refs/remotes/") else {
            continue;
        };
        for remote in &remotes {
            let mut wanted: Vec<u8> = remote.clone();
            wanted.push(b'/');
            wanted.extend(named.as_encoded_bytes());
            if short == wanted.as_slice() {
                candidates.push((remote.clone(), line.to_vec()));
            }
        }
    }
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() == 1 {
        return Ok(candidates.first().map(|(_, full)| return os_string_of(full.as_slice())));
    }
    let Some(default_remote) = configured_remote_to_guess_from(facts)? else {
        // Git refuses the ambiguous name itself, so the form writes nothing.
        return Ok(None);
    };
    return Ok(candidates
        .iter()
        .find(|(remote, _)| return remote.as_slice() == default_remote.as_encoded_bytes())
        .map(|(_, full)| return os_string_of(full.as_slice())));
}

/// What:
///  Every remote this repository has, as Git lists them.
/// Why:
///  A remote-tracking refname is `<remote>/<branch>`,
///  and a remote's own name may hold a slash,
///  so the split between the two is only knowable from the list of remotes.
///  A listing that failed is a failure, never "no remote".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remotes = (facts) => facts.query(['remote']).stdout.toString().split('\n').filter(Boolean);
/// ```
fn remote_names(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "remote"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remotes this repository has")?;
    if !output.success {
        return Err(String::from(
            "Git could not list the remotes, so the guard cannot tell what this command would write over",
        ));
    }
    let mut names: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in output.stdout.split(|byte| return *byte == b'\n') {
        if !line.is_empty() {
            names.push(line.to_vec());
        }
    }
    return Ok(names);
}

/// What:
///  The `checkout.defaultRemote` this repository sets,
///  or nothing when it sets none.
/// Why:
///  It decides which remote Git guesses from when several carry the branch,
///  and a value that does not name one of them leaves Git's own ambiguity refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remote = (facts) => { const v = facts.query(['config', '--get', 'checkout.defaultRemote']); return v.success ? v.stdout.trim() : undefined; };
/// ```
fn configured_remote_to_guess_from(facts: &mut GitFacts) -> Result<Option<OsString>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "config", "--get", "checkout.defaultRemote"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remote this repository guesses a branch from")?;
    if !output.success {
        return Ok(None);
    }
    let trimmed: &[u8] = output.stdout.strip_suffix(b"\n").unwrap_or(output.stdout.as_slice());
    if trimmed.is_empty() {
        return Ok(None);
    }
    return Ok(Some(os_string_of(trimmed)));
}

/// What:
///  One name as the operating system takes it,
///  from the bytes Git printed.
/// Why:
///  A refname is bytes;
///  converting through `String` would replace a byte that is not UTF-8 and name another ref.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (bytes) => bytes; // a Buffer is already the name
/// ```
fn os_string_of(bytes: &[u8]) -> OsString {
    return path_from_bytes(bytes);
}

/// The names this reading takes from Git's answers,
/// and the paths they resolve to,
/// beside real-Git controls of each way a word becomes a tree.
#[cfg(test)]
#[path = "repository_facts_written_paths_tests.rs"]
mod tests;

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
        for path in tree_paths(facts, Some(revision.as_os_str()), &[], false)? {
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
