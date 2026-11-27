//! What:
//!  The worktree paths an explicit-path commit stages into its private index:
//!  which selected elements name one file,
//!  which name a directory or a pattern,
//!  the two `git add` starts that stage them,
//!  and the failure that is the caller's own pathspec naming nothing.
//! Why:
//!  Native Git commits the tracked paths and the already-staged additions a directory or a pattern matches,
//! and never a file nothing staged,
//!  while an exact path the person named is taken as the wrapper's own deliberate difference.
//!  A start whose pathspecs are the caller's own ends for two different reasons,
//! and only one of them is the caller's input (issue 6283),
//!  so the two are told apart here and the caller reports each in its own class.
//!  The file is split from `commit_index.rs`,
//! which initializes the index and stays inside its line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await git(['add', '--', ...exactFiles], { index: commitIndex }); await git(['add', '--update', '--', ...patterns], { index: commitIndex });
//! ```

/// The selection a commit names,
///  and the arguments one of its forms is staged with.
use crate::commit_index_selection::{
    IndexSelection,
    selection_arguments,
};
/// The pathspec a caller named that Git refuses for matching nothing,
///  and the words Git's answer is read with.
use crate::commit_pathspec_matching_nothing::{
    WrittenPathspecs,
    pathspec_matching_nothing,
};
/// The words one pathspec element is read as,
/// kept in a file of their own,
/// re-exported where every reader already looks for them.
pub(crate) use crate::commit_selection_staging_element_words::{
    excludes_paths,
    names_one_file,
    pattern_characters_are_literal,
};
use crate::commit_selection_staging_element_words::{
    names_a_path,
    subtracts_paths,
};
/// The failure class and the sentence a refusal keeps,
/// kept in a file of their own,
/// re-exported where every caller of it already looks.
pub use crate::commit_selection_staging_failure::SelectionStagingFailure;
use crate::commit_selection_staging_failure::explain_a_git_failure;
/// What one selection gives this staging step,
/// kept in a file of its own,
/// re-exported where every caller already looks for it.
pub(crate) use crate::commit_selection_staging_selection::{
    ITEMS_BYTES_LIMIT,
    PATHSPEC_FILE_LIMIT,
    Selected,
    selection_elements,
    selection_items,
};
/// The private index and object store a commit's preparation selects,
///  and the request builder of its Git starts.
use crate::private_index_request::{
    CaptureIndexes,
    caller_pathspec_request,
};
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    GitRequest,
    checked_from,
    run_git,
    run_git_checked,
};
/// `OsString` owns one whole argument.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Stages the named paths into the private index,
/// as Git's partial commit does:
/// what the worktree holds at them,
/// deletions
///  included.
/// Why:
///   Both ways of making the private index (reset to the base,
/// or the real index kept as it is) end here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--', ...exactFiles], { index: commitIndex }); await git(['add', '--update', '--', ...patterns], { index: commitIndex })
/// ```
pub(crate) fn add_selected_paths(
    context: &GitContext,
    cwd: &Path,
    worktree_root: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), SelectionStagingFailure> {
    match selection_items(selection) {
        Selected::Items(items) => {
            return stage_items(context, cwd, worktree_root, indexes, items.as_slice());
        }
        // An empty selection stages nothing, and the scratch commit then refuses as native Git refuses the command.
        Selected::Empty => return Ok(()),
        // A pathspec file too large to name on one command line keeps its own form, and only the tracked paths it matches
        // are staged, so no file nothing staged reaches the commit. Its start ends for the same two reasons the elements'
        // does, and Git's refusal of one of its pathspecs is the caller's input too (issue 6283).
        Selected::File => {
            let mut arguments: Vec<OsString> =
                vec![OsString::from("add"), OsString::from("--update")];
            arguments.extend(selection_arguments(selection));
            return stage_the_callers_pathspec_file(context, cwd, indexes, arguments.as_slice());
        }
        // A pathspec file this reader cannot take is Git's to read, and every path it names is staged as before.
        Selected::Unreadable => {
            let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--all")];
            arguments.extend(selection_arguments(selection));
            return private_git_with_modes(context, cwd, indexes, arguments.as_slice())
                .map_err(explain_a_git_failure);
        }
    }
}

/// What:
///  The directory the relative pathspecs of this command are read from.
/// Why:
///   Git reads a relative pathspec below the top of the worktree the command works on,
/// from the directory it runs in when that directory is inside that worktree,
///  and from the worktree's own top when it is outside it.
///  Which worktree that is has owners the wrapper does not repeat:
/// a command line names one with `--git-dir` and `--work-tree`,
///  a shell names one as `GIT_DIR` and `GIT_WORK_TREE`,
///  a configuration names one as `core.worktree`,
///  and a linked worktree is found from its own `.git` file,
/// so the top Git answered for the invocation is the one asked here
/// (`capture_invocation.rs`,
/// `repository_root`,
/// which `rev-parse --show-toplevel` gives).
///  A reader that derived it from the command line's own `--work-tree` instead found none of the other shapes,
///  took the invocation directory for the base,
/// found no file at an exact name,
///  took the element for a pattern,
/// and left the commit to a `git add --update` that refuses a path nothing tracked;
///  a directory selection misread as an exact path staged every untracked file below it.
///  An invocation directory that cannot be resolved answers with itself,
/// which is where Git reads the pathspec when the worktree it names is the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathspecBase = (cwd, worktreeRoot) => realpath(cwd)?.startsWith(realpath(worktreeRoot)) ? cwd : worktreeRoot;
/// ```
pub(crate) fn pathspec_base(
    cwd: &Path,
    worktree_root: &Path,
) -> PathBuf {
    let Some(root) = std::fs::canonicalize(worktree_root).ok() else {
        return cwd.to_path_buf();
    };
    let here: PathBuf = match std::fs::canonicalize(cwd) {
        Ok(resolved) => resolved,
        // A directory that cannot be resolved keeps the name it was given, and a pathspec is then read from it.
        Err(_unresolved) => cwd.to_path_buf(),
    };
    if here.starts_with(root.as_path()) {
        return cwd.to_path_buf();
    }
    return root;
}

/// What:
///  Stages the selection's elements into the private index:
/// an element that names one file as it is,
/// and a directory,
/// a pattern or pathspec magic as the tracked paths it matches.
/// Why:
///  Native Git commits the tracked and already-staged paths a directory or a pattern matches,
/// and never a file nothing staged,
///  while a `git add --all` over the same pathspec staged every untracked file it matched:
/// a commit of `.`,
///  `*` or a directory then published scratch files nobody staged.
///  An exact path the worktree holds is the one shape step 3 of `SPEC.md` "Explicit-path commit" takes from the worktree:
///  a path an ignore rule matches is still refused by Git with the hint the remedy explains,
/// and a file nothing staged is committed where native Git refuses the pathspec (the wrapper's deliberate difference).
///  The elements that subtract paths (`:(exclude)`,
/// `:!`,
/// `:^`) apply to every element of the selection,
/// the exact file names included,
/// because that is how native Git reads one list:
///  `a.txt ':(exclude)a.txt'` takes no path,
/// and one `git add` per element would stage `a.txt` anyway.
///  A list of nothing but subtracting elements reads as every path,
/// which is native Git's reading when the caller wrote no other element,
/// and is not this start's to take when the caller named exact files instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--', ...exactFiles, ...exclusions]); await git(['add', '--update', '--', ...patterns]);
/// ```
fn stage_items(
    context: &GitContext,
    cwd: &Path,
    worktree_root: &Path,
    indexes: &CaptureIndexes,
    items: &[OsString],
) -> Result<(), SelectionStagingFailure> {
    let literal: bool = pattern_characters_are_literal(context);
    let base: PathBuf = pathspec_base(cwd, worktree_root);
    let mut named_files: Vec<&OsString> = Vec::new();
    let mut patterns: Vec<&OsString> = Vec::new();
    for item in items {
        if names_one_file(base.as_path(), item.as_os_str(), literal) {
            named_files.push(item);
        } else {
            patterns.push(item);
        }
    }
    let exclusions: Vec<&OsString> = patterns.iter().copied().filter(subtracts_paths).collect();
    // True while the patterns still name something: a directory, a pattern or additive magic. Only these keep the
    //  subtracting elements from reading as every path.
    let names_a_pattern: bool = patterns.iter().any(names_a_path);
    if !named_files.is_empty() {
        let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--")];
        for name in &named_files {
            arguments.push((*name).clone());
        }
        for exclusion in &exclusions {
            arguments.push((*exclusion).clone());
        }
        private_git_with_modes(context, cwd, indexes, arguments.as_slice())
            .map_err(explain_a_git_failure)?;
    }
    // A directory or a pattern stages the tracked paths it matches (`--update`), never a file nothing staged. The start is
    //  skipped when the only elements left would read as every path and the caller named the exact files instead: their
    //  start above already took them, minus the same exclusions.
    if !patterns.is_empty() && (names_a_pattern || named_files.is_empty()) {
        stage_patterns(context, cwd, indexes, patterns.as_slice())?;
    }
    return Ok(());
}

/// What:
///  Stages the patterns the caller named into the private index,
///  and reports Git's refusal of one that matches nothing as the caller's input error.
/// Why:
///   Git stays the only decider of whether a pathspec matches,
/// so nothing here predicts a match:
/// a directory,
/// a pattern,
/// a magic selector and an excluded one reach Git exactly as before.
///  These are the caller's own elements that named no file this reader could take,
/// so a path that exists is never classified here;
///  a path that was there and is gone between the check and Git stays an engine failure,
/// because that race is not a spelling (issue 6283).
///  Every other failure of the start keeps the wording and the class it had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = runGit(callerPathspecRequest(['add', '--update', '--', ...patterns])); const refused = outcome.ok ? null : pathspecMatchingNothing(outcome.stderr, patterns); if (refused) throw new InputRefusal(refused); checkedFrom(outcome);
/// ```
fn stage_patterns(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    patterns: &[&OsString],
) -> Result<(), SelectionStagingFailure> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("add"),
        OsString::from("--update"),
        OsString::from("--"),
    ];
    for pattern in patterns {
        arguments.push((*pattern).clone());
    }
    let request: GitRequest = caller_pathspec_request(
        context,
        cwd,
        arguments.as_slice(),
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    let outcome: std::io::Result<GitOutput> = run_git(context, &request);
    if let Ok(output) = outcome.as_ref()
        && !output.succeeded()
        && let Some(refused) = pathspec_matching_nothing(
            output.stderr.as_slice(),
            WrittenPathspecs::Elements(patterns),
        )
    {
        return Err(SelectionStagingFailure::PathspecMatchesNothing(refused));
    }
    checked_from(outcome, request.arguments.as_slice()).map_err(explain_a_git_failure)?;
    return Ok(());
}

/// What:
///  Stages the pathspec file the caller named,
/// as the file Git reads,
///  and reports Git's refusal of a pathspec in it that matches nothing as the caller's input error.
/// Why:
///   A file past the bound on one command line is passed to Git as a file,
/// so the tokens are never spelled here and every one of them is the caller's:
/// the reading accepts whatever pathspec Git names (issue 6283).
///  Every other failure of the start keeps the wording and the class it had,
/// and the ignored-refusal sentence still applies to a pathspec Git reads from that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = runGit(callerPathspecRequest(['add', '--update', '--pathspec-from-file=...'])); const refused = outcome.ok ? null : pathspecMatchingNothing(outcome.stderr, FromTheCallersFile); if (refused) throw new InputRefusal(refused); checkedFrom(outcome);
/// ```
fn stage_the_callers_pathspec_file(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    arguments: &[OsString],
) -> Result<(), SelectionStagingFailure> {
    let request: GitRequest = caller_pathspec_request(
        context,
        cwd,
        arguments,
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    let outcome: std::io::Result<GitOutput> = run_git(context, &request);
    if let Ok(output) = outcome.as_ref()
        && !output.succeeded()
        && let Some(refused) = pathspec_matching_nothing(
            output.stderr.as_slice(),
            WrittenPathspecs::FromTheCallersFile,
        )
    {
        return Err(SelectionStagingFailure::PathspecMatchesNothing(refused));
    }
    checked_from(outcome, request.arguments.as_slice()).map_err(explain_a_git_failure)?;
    return Ok(());
}

/// What:
///  `private_git` with the caller's own pathspec-mode options reaching Git.
/// Why:
///   The staging pathspecs are the caller's,
/// so the mode the caller wrote as a global option must reach them exactly as the mode they wrote as a variable does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGit(callerPathspecRequest(context, cwd, arguments, commitIndex, objectDirectory))
/// ```
fn private_git_with_modes(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    arguments: &[OsString],
) -> Result<(), GitFailure> {
    let request: GitRequest = caller_pathspec_request(
        context,
        cwd,
        arguments,
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    run_git_checked(context, &request)?;
    return Ok(());
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_selection_staging_tests.rs"]
mod tests;
