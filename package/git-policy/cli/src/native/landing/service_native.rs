//! What:
//!  The commits real Git builds for the commit service:
//! an editor,
//! a signature,
//! and every command line the service does
//!  not build itself.
//! Why:
//!  Git owns the editor,
//! the signer and the message semantics of whatever the service cannot embed,
//! so this module points
//!  real `git commit` at a thin scratch repository (its `HEAD` and branch at the base,
//! the policy-settled private index,
//!  the real configuration included,
//! every hook off) whose new objects go straight into the real object store.
//! Nothing is
//!  locked and nothing is registered:
//! the scratch repository sits inside the commit's own scratch directory,
//! is removed
//!  with it,
//! and the commit it makes is only an object until the service moves the branch to it by compare-and-swap
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Commits Git builds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { commit, parents } = await buildWithGit({ context, cwd, capture, scratch, commitIndex, arguments });
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::commit_args::{resolve_private_commit_args, split_commit_invocation, without_all_flag};
use crate::commit_preparation::{PreparationRequest, PreparedCommit, read_prepared_commit, run_native_commit};
/// The build phase's stretches,
/// which the creation and Git's build mark as they go.
use crate::commit_build_part_durations::{BuildPartClock, note_build_part};
use crate::commit_transaction::TransactionStop;
use crate::global_arguments::global_layout;
use crate::shadow_create::take_or_create_shadow;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use crate::transaction_journal::TransactionMode;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What the scratch repository's `git commit` made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitBuilt {
    /// The commit,
    /// already in the real object store.
    pub commit: String,
    /// Its parents in order.
    pub parents: Vec<String>,
    /// Its message as stored:
    /// what the person typed,
    /// after Git's clean-up.
    pub message: String,
    /// The finished shadow the service drew ahead of this commit,
    /// which Git built the commit in,
    /// and nothing when the shadow is the commit's own.
    /// The request names it so the service removes it once that request's landing is over.
    pub drawn: Option<PathBuf>,
}

/// What:
///  Where the first blank line of a stored commit starts,
/// nothing when there is none.
/// Why:
///  The header block of a stored commit ends at it and the message follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// raw.indexOf('\n\n')
/// ```
fn blank_line_at(raw: &[u8]) -> Option<usize> {
    for (at, window) in raw.windows(2).enumerate() {
        if window == b"\n\n" {
            return Some(at);
        }
    }
    return None;
}

/// What:
///  The parent a header line names,
/// nothing for any other line.
/// Why:
///  `parents_of` lists the parents of a stored commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentOfLine = (line: string) => line.startsWith('parent ') ? line.slice(7) : undefined;
/// ```
fn parent_of_line(line: &str) -> Option<String> {
    return line.strip_prefix("parent ").map(str::to_string);
}

/// The message after the header block of a stored commit.
pub fn message_of(raw: &[u8]) -> String {
    // The message starts after the first blank line; a stored commit without one has none.
    let start: usize = match blank_line_at(raw) {
        Some(at) => at + 2,
        None => raw.len(),
    };
    return String::from_utf8_lossy(&raw[start..]).into_owned();
}

/// Everything the build needs from the preparation that settled the private index.
pub struct BuildInput<'a> {
    /// How to run the real Git.
    pub context: &'a GitContext,
    /// The directory the command was given in.
    pub cwd: &'a Path,
    /// The invocation capture:
    /// the branch,
    /// the base,
    /// the object directory.
    pub capture: &'a InvocationCapture,
    /// The commit's own scratch directory,
    /// which holds the scratch repository and is removed with it.
    pub scratch: &'a Path,
    /// The private index the policies settled.
    pub commit_index: &'a Path,
    /// The arguments after the policies' fixed transforms,
    /// global options included.
    pub arguments: &'a [OsString],
    /// The positional pathspecs of the command line,
    /// which the private index already holds,
    /// each with its own bytes.
    pub pathspecs: &'a [OsString],
    /// Whether the line commits named paths or the index.
    pub mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub staged_all: bool,
    /// Whether the private index already holds a selection made with `--include`,
    /// `--patch` or `--interactive`,
    /// so that Git
    ///  is not given those options again.
    pub selected_privately: bool,
    /// The build phase's clock,
    /// which the shadow's creation and Git's build name their stretches under.
    pub clock: &'a mut BuildPartClock,
    /// Where the service of this worktree listens,
    /// which the commit asks for a finished shadow before it builds anything.
    pub service: &'a crate::service_server_socket_binding::Bind,
    /// Where the drawn shadow this commit was handed is put,
    /// so the value that renews and releases it outlives the build:
    /// a commit in an editor holds it for as long as the person leaves the editor open.
    pub handed: &'a mut Option<crate::shadow_drawn::HandedShadow>,
}

/// The parents named by a stored commit's header block.
pub fn parents_of(raw: &[u8]) -> Vec<String> {
    let header_end: usize = blank_line_at(raw).unwrap_or(raw.len());
    return String::from_utf8_lossy(&raw[..header_end])
        .lines()
        .filter_map(parent_of_line)
        .collect();
}

/// What:
///  The kept global options and the commit options of the arguments,
/// with the pathspecs and the selection options the
///  private index already holds removed.
/// Why:
///   Real `git commit` must commit the private index as it stands,
/// so it is given neither the paths nor `-a`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateInvocation({ arguments, pathspecs, mode, stagedAll })
/// ```
pub fn private_invocation(
    arguments: &[OsString],
    pathspecs: &[OsString],
    mode: TransactionMode,
    staged_all: bool,
    selected_privately: bool,
) -> (Vec<OsString>, Vec<OsString>) {
    let private_args: Vec<OsString> = resolve_private_commit_args(arguments, pathspecs, mode, selected_privately);
    let index: usize = global_layout(private_args.as_slice()).prefix_len;
    let (globals, commit_args): (Vec<OsString>, Vec<OsString>) = split_commit_invocation(private_args.as_slice(), index);
    return if staged_all { (globals, without_all_flag(commit_args.as_slice())) } else { (globals, commit_args) };
}

/// What:
///  Runs real `git commit` against the scratch repository and reads the commit it made.
/// Why:
///   Git's own output,
/// editor and signer reach the person's terminal as they would from a plain `git commit`;
/// a nonzero
///  status of Git is the command's status and nothing lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await buildWithGit(input)
/// ```
///
///  A shadow can go at any of the three moments the build reads or writes it:
///  before Git is asked to build in it,
///  while Git builds in it,
///  and after Git has built in it and before the commit it made is read back.
///  The first two are answered before Git's work is trusted again,
///  and the third after it: Git's commit object is in the real repository but nothing points at it,
///  because the ref that named it went with the tree, so the commit is made again.
///  A shadow that has gone before Git is asked to build in it is one Git is not asked to build in at all:
///  the commit builds again in another tree, so no line of Git's about a tree that went is the person's to read.
///  A shadow that goes while Git builds in it is the wrapper's own cache and never the person's failure:
///  a tree no sweep explains can still vanish (a person removes it, a drive fills),
///  so a build whose tree is gone when it failed is made once more in another tree,
///  and the commit says in its record and its log what it did and lands
///  (`shadow_create.rs`, `take_or_create_shadow`);
///  a build whose tree is still there failed for the repository's or the volume's reason and is the commit's to report.
///  One more build and no more: the second attempt's own ceilings bound it, and a third would be a commit that could not end.
///
/// # Errors
///
/// `TransactionStop::NativeExit` with Git's status when Git declined or was aborted (an empty message,
/// an editor that
///  failed,
/// a signer that failed);
/// `TransactionStop::Message` when Git could not be started.
pub fn build_with_git(input: &mut BuildInput<'_>) -> Result<GitBuilt, TransactionStop> {
    let own_shadow: PathBuf = input.scratch.join("shadow");
    // Named paths while another session's merge, cherry-pick or revert is unfinished: real Git refuses that as a partial commit,
    //  and a commit never fails for another session's unfinished work, so the scratch repository holds none of that state and
    //  builds an ordinary commit of the paths; the worktree's state stays for its owner to conclude on the new head.
    let git_dir: &Path = input.capture.layout.git_dir.as_path();
    let mut has_conclusion_state: bool = false;
    for name in ["MERGE_HEAD", "CHERRY_PICK_HEAD"] {
        if git_dir.join(name).exists() {
            has_conclusion_state = true;
            break;
        }
    }
    let carry_conclusion: bool = !(input.mode == TransactionMode::ExplicitPath && has_conclusion_state);
    // The facts the shadow was made from, which the recovery asks about a tree before it trusts or leaves it.
    let template: crate::shadow_create::ShadowTemplate = crate::shadow_create::ShadowTemplate::of(input.context, &input.capture.layout)?;
    // The wrapper adds no option of its own to the person's command line: Git prints what it prints, and `--quiet` reaches it only
    //  when the person typed it.
    let (globals, commit_args): (Vec<OsString>, Vec<OsString>) =
        private_invocation(input.arguments, input.pathspecs, input.mode, input.staged_all, input.selected_privately);
    // What: The two inputs this command was given, taken before anything can remove them, so what Git makes can be held
    //       against them.
    // Why:  A file under the scratch tree can go at any instant, whoever takes it: a sweep, a cleaner of temporary files, a
    //       full drive, a kill. A list of files to look at before Git starts can only be longer, never enough, because Git
    //       reads them after the look. What cannot be evaded is holding the commit Git made against the index and the
    //       conclusion state the command began with, and that is asked here, before the request is built and before
    //       anything of the real repository's merge state is touched.
    //       The index is kept as a copy because Git reads the private one and updates it; the copy is the tree this command
    //       asked to commit, whoever removes the private one afterwards.
    let captured_index: PathBuf = input.scratch.join(CAPTURED_INDEX);
    if std::fs::copy(input.commit_index, captured_index.as_path()).is_err() {
        return Err(TransactionStop::Message(format!(
            "the private index {} this commit reads went before Git was asked to build in it, so nothing was committed and nothing landed",
            input.commit_index.display()
        )));
    }
    let asked_for: AskedForCommit = the_commit_that_was_asked_for(input, &expected_parents_of(input)?)?;
    let mut attempt: usize = 0;
    loop {
        attempt += 1;
        // What: A second build starts from a tree of its own and from the index this command was given.
        // Why:  The first attempt's tree has the ref store and the conclusion copies it wrote,
        //       and writing them again into the same tree fails on the wrapper's own files ("File exists");
        //       a tree of its own is what the commit's scratch name gives every attempt of a command,
        //       and the code that takes or makes one is the same code the first attempt ran.
        if attempt > 1 {
            let _gone: crate::scratch_entry_removal::Removal = crate::scratch_entry_removal::remove_within(own_shadow.as_path(), crate::scratch_entry_removal::SCRATCH_REMOVAL_CEILING);
            // The conclusion copies the first attempt made are made again from the real repository's state, which this
            //  command's captures leave as it was.
            let copies: PathBuf = input.scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY);
            let _cleared: crate::scratch_entry_removal::Removal = crate::scratch_entry_removal::remove_within(copies.as_path(), crate::scratch_entry_removal::SCRATCH_REMOVAL_CEILING);
            // What: The index this command was given is put back exactly as the captures hold it.
            // Why:  Git rewrites the private index as it prepares a commit, and the tree this command asked for is the one the
            //       kept copy names; a copy that went with the first attempt's tree cannot say what the person staged, so the
            //       commit is not built again rather than built on what Git left (`SPEC.md`, "Merges and commits never fail").
            if std::fs::copy(captured_index.as_path(), input.commit_index).is_err() {
                return Err(TransactionStop::Message(format!(
                    "the copy of the private index {} went before the commit could be built again, so nothing was committed and nothing landed",
                    captured_index.display()
                )));
            }
        }
        // The creation names each of its own stretches, so the shadow is named here only as the surface they run on.
        let tree: crate::shadow_create::ShadowTree = take_or_create_shadow(
            input.context,
            input.cwd,
            input.capture,
            own_shadow.as_path(),
            input.scratch,
            carry_conclusion,
            input.clock,
            input.service,
        )?;
        let shadow: PathBuf = tree.path.clone();
        let drawn: Option<PathBuf> = tree.handed.as_ref().map(|handed| { return handed.path().to_path_buf(); });
        *input.handed = tree.handed;
        // What: A control may hold the commit at this instant, with its shadow built and Git not yet run in it
        //       (`transaction_phase.rs`, `Phase::ShadowBuilt`).
        // Why:  A tree that goes now is the one loss only a second build answers, and no other instant lets a control place it.
        //       Nothing arms this marker in a run.
        crate::service_pruned::reach_service_phase(crate::transaction_phase::Phase::ShadowBuilt).map_err(|error| {
            return TransactionStop::Message(format!("the shadow build's test-only phase marker could not be reached: {error}"));
        })?;
        let request: PreparationRequest<'_> = PreparationRequest {
            capture: input.capture,
            shadow: shadow.as_path(),
            commit_index: input.commit_index,
            scratch: input.scratch,
            global_args: globals.as_slice(),
            commit_args: commit_args.as_slice(),
            object_directory: Some(input.capture.layout.object_directory.as_path()),
        };
        input.clock.next("client build");
        if !crate::shadow_create::the_shadow_holds_what_a_build_needs(shadow.as_path(), &template, &input.capture.base) {
            // The tree went between this commit's claim on it and the build, so Git is not started in it: a build that cannot
            //  succeed is one whose words would only alarm the person about the wrapper's own cache.
            note_build_part(SHADOW_WENT_PART, Duration::ZERO, Duration::ZERO);
            crate::diagnostic_log::debug(
                "serviceNative",
                format!("the shadow {} went before Git was asked to build in it, so this commit builds in a new tree", shadow.display()).as_str(),
            );
            if attempt == 1 {
                *input.handed = None;
                continue;
            }
            return Err(TransactionStop::Message(format!("the shadow {} went before Git was asked to build in it", shadow.display())));
        }
        let status: i32 = match run_native_commit(input.context, input.cwd, &request) {
            Ok(code) => code,
            Err(error) => return Err(TransactionStop::Message(error.to_string())),
        };
        if status != 0 {
            // What: A build that failed in a tree that is gone is made once more in another tree, and the commit lands.
            // Why:  The tree is the wrapper's own cache, and a commit may not end with a nonzero status for a tree that went
            //       under it for a cause no sweep explains (`SPEC.md`, "Merges and commits never fail"). A second build in a
            //       fresh tree is the commit's own work again and its own ceilings bound it; a third would be a commit that
            //       could not end, so this is one more and no more.
            if attempt == 1 && !crate::shadow_create::the_shadow_holds_what_a_build_needs(shadow.as_path(), &template, &input.capture.base) {
                note_build_part(SHADOW_WENT_PART, Duration::ZERO, Duration::ZERO);
                crate::diagnostic_log::debug(
                    "serviceNative",
                    format!("the shadow {} went while Git built in it, so this commit builds in a new tree", shadow.display()).as_str(),
                );
                // The value that held the tree goes with this attempt, and the next attempt takes or makes another.
                *input.handed = None;
                continue;
            }
            return Err(TransactionStop::NativeExit(status));
        }
        input.clock.next("commit read back");
        // What: A control may hold the commit at this instant, with Git's commit in the shadow and the commit not yet read out
        //       of it (`transaction_phase.rs`, `Phase::ShadowReadable`).
        // Why:  A tree that goes now takes the ref that names the commit Git made, so the read-back can only answer with its
        //       failure; no other instant lets a control place that loss. Nothing arms this marker in a run.
        crate::service_pruned::reach_service_phase(crate::transaction_phase::Phase::ShadowReadable).map_err(|error| {
            return TransactionStop::Message(format!("the commit read-back's test-only phase marker could not be reached: {error}"));
        })?;
        let prepared: PreparedCommit = match read_prepared_commit(input.context, shadow.as_path()) {
            Ok(prepared) => prepared,
            Err(error) => {
                // What: A read-back that failed in a tree that is gone is made once more in another tree, and the commit lands.
                // Why:  Git may have made its commit and this read is the only place that would name it, because the tree it was
                //       written in held the ref that pointed at it. The object it wrote is in the real repository and nothing
                //       points at it, so a collection takes it later; the commit that lands is made again, which means the same
                //       command line again, an editor and a signer asked a second time among it.
                //       A read-back whose tree is still there failed for the repository's or the volume's reason and is the
                //       commit's to report (`SPEC.md`, "Merges and commits never fail").
                if attempt == 1 && !crate::shadow_create::the_shadow_holds_what_a_build_needs(shadow.as_path(), &template, &input.capture.base) {
                    note_build_part(SHADOW_WENT_PART, Duration::ZERO, Duration::ZERO);
                    crate::diagnostic_log::debug(
                        "serviceNative",
                        format!("the shadow {} went before the commit it built could be read back ({error}), so this commit builds in a new tree", shadow.display()).as_str(),
                    );
                    *input.handed = None;
                    continue;
                }
                return Err(error.into());
            }
        };
        // What: The commit Git made is held against the index and the conclusion state this command began with, before the
        //       request is built and before anything of the real repository's merge state is touched.
        // Why:  A commit of the wrong parents or the wrong tree is a commit the person did not write, and a merge that lands
        //       as one parent also takes the real merge state with it when its conclusion is cleaned
        //       (`conclusion_cleanup.rs`). Nothing has landed here: the request is not built, the branch has not moved and the
        //       real conclusion state is untouched, so a difference is answered with one more build from the captures, and a
        //       second difference is said plainly with what differed (`SPEC.md`, "Merges and commits never fail").
        if let Err(differed) = asked_for.holds(&prepared) {
            note_build_part(THE_COMMIT_DIFFERED_PART, Duration::ZERO, Duration::ZERO);
            crate::diagnostic_log::debug("serviceNative", format!("the commit Git built is not the one this command asked for: {differed}").as_str());
            if attempt == 1 {
                *input.handed = None;
                continue;
            }
            return Err(TransactionStop::Message(format!("the commit Git built is not the one this command asked for: {differed}, so nothing landed and nothing in the repository changed")));
        }
        return Ok(GitBuilt { parents: parents_of(prepared.raw.as_slice()), message: message_of(prepared.raw.as_slice()), commit: prepared.oid, drawn });
    }
}

/// The copy of the private index a build is judged against,
/// under the commit's own scratch tree and beside the index Git reads.
pub const CAPTURED_INDEX: &str = "captured.captured-index";

/// The name a commit's own timing record carries when Git's commit was not the one the command asked for and it built again:
///  a fixed name and never a path,
///  because a build part holds a name a person can read and nothing of a machine's own.
pub const THE_COMMIT_DIFFERED_PART: &str = "commit differed";

/// What:
///  The commit this command asked for:
///  the parents it must have, and the tree it must hold, taken before Git runs.
/// Why:
///  It is the whole of the question a landable commit answers,
///  and it is answered from the captures and not from the scratch tree,
///  so no file that goes under the scratch tree can change the answer after the captures were taken.
///  A parent list that cannot be told (an amend's,
///  whose parents are the amended commit's)
///  is left out of the question and the tree still stands for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AskedForCommit = { parents?: string[]; tree?: string };
/// ```
#[derive(Debug)]
pub struct AskedForCommit {
    /// The parents Git's commit must name,
    /// in order,
    /// or nothing when the command's form does not settle them.
    parents: Option<Vec<String>>,
    /// The tree Git's commit must hold,
    /// or nothing when the command's form settles no tree of its own,
    /// or when Git could not name the one the index held.
    tree: Option<String>,
}

impl AskedForCommit {
    /// What:
    ///  Whether the commit Git made is the one this command asked for,
    ///  or what differs.
    /// Why:
    ///  The two differences a person would meet are named in the words a person reads:
    ///  a parent that is not the one the command began with (a conclusion that lost its other side),
    ///  and a tree that is not the one the index held (a commit that would delete what the person kept).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// asked.holds(prepared) // ok, or the words of what differs
    /// ```
    fn holds(&self, prepared: &PreparedCommit) -> Result<(), String> {
        if let Some(asked) = self.parents.as_ref() {
            let made: Vec<String> = parents_of(prepared.raw.as_slice());
            if &made != asked {
                return Err(format!("its parents are {} where the command began with {}", made.join(", "), asked.join(", ")));
            }
        }
        if let Some(asked) = self.tree.as_ref()
            && &prepared.tree_oid != asked
        {
            return Err(format!("its tree is {} where the index this command was given holds {asked}", prepared.tree_oid));
        }
        return Ok(());
    }
}

/// What:
///  The parents the command asked for,
///  taken from the real repository before Git runs in the shadow.
/// Why:
///  A commit's first parent is the captured base,
///  and a merge's other parents are the commits its real `MERGE_HEAD` names, in order.
///  A cherry-pick's or a revert's commit has the base alone,
///  so its own head file adds nothing.
///  An amend's parents are the amended commit's and are not knowable here without reading that commit,
///  so the command's form leaves them out and its tree is what stands for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parents = amends ? undefined : [base, ...mergeHeadLines(gitDir)];
/// ```
fn expected_parents_of(input: &BuildInput<'_>) -> Result<Option<Vec<String>>, TransactionStop> {
    let amends: bool = input.arguments.iter().any(|argument| { return argument == "--amend"; });
    if amends {
        return Ok(None);
    }
    let mut parents: Vec<String> = Vec::new();
    if let crate::transaction_journal::Base::Commit(base) = &input.capture.base {
        parents.push(base.clone());
    }
    let merge_head: PathBuf = input.capture.layout.git_dir.join("MERGE_HEAD");
    if let Ok(text) = std::fs::read_to_string(merge_head.as_path()) {
        for line in text.lines() {
            let name: &str = line.trim();
            if !name.is_empty() {
                parents.push(String::from(name));
            }
        }
    }
    return Ok(Some(parents));
}

/// What:
///  The index and the conclusion state this command was given,
///  with the tree Git's commit must hold read out of the kept copy of the index.
/// Why:
///  The tree is one `git write-tree` on the copy,
///  which writes no new object when the commit is the one asked for (Git has just written that tree),
///  and names exactly what the person staged when it is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = { parents, tree: writeTree(capturedIndex) };
/// ```
fn the_commit_that_was_asked_for(input: &BuildInput<'_>, parents: &Option<Vec<String>>) -> Result<AskedForCommit, TransactionStop> {
    // What: A command whose commit carries the tree it started from asks no tree of its own.
    // Why:  A reword fixup's commit holds the tree of the branch it rewords, whatever the index stages, so holding the
    //       index's tree against it would refuse the commit the person asked for; its parents still stand.
    let carries_its_base_tree: bool = input.arguments.iter().any(|argument| {
        let text: String = argument.to_string_lossy().to_string();
        return text.starts_with("--fixup=reword:") || text.starts_with("--fixup=amend:");
    });
    if carries_its_base_tree {
        return Ok(AskedForCommit { parents: parents.clone(), tree: None });
    }
    let captured: PathBuf = input.scratch.join(CAPTURED_INDEX);
    let mut request: GitRequest = GitRequest::new(input.cwd, &["write-tree"]);
    request.without_prefix = true;
    // The index this command was given, and the store its trees are written to, exactly as the build reads them.
    request.index = Some(captured.clone());
    request.object_directory = Some(input.capture.layout.object_directory.clone());
    let tree: Option<String> = match run_git_checked(input.context, &request) {
        Ok(output) => Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string()),
        Err(error) => {
            crate::diagnostic_log::debug("serviceNative", format!("the tree this commit asked for cannot be read from {}: {error}", captured.display()).as_str());
            None
        }
    };
    return Ok(AskedForCommit { parents: parents.clone(), tree });
}

/// The name a commit's own timing record carries when the shadow it built in went and it built again:
///  a fixed name and never a path,
///  because a build part holds a name a person can read and nothing of a machine's own.
pub const SHADOW_WENT_PART: &str = "shadow went";

/// Controls for building with Git stay out of the release executable.
#[cfg(test)]
#[path = "service_native_tests.rs"]
mod tests;
