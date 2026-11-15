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
    // What: The capture is taken again when it cannot be taken, a bounded number of times, and the build starts only when it
    //       was taken.
    // Why:  A kept copy that went, or a private index that went, leaves this attempt unable to say what the person staged;
    //       the capture reads the private index again on the next try, and a copy that was merely taken away by a cleaner is
    //       restored by that read. A capture that still cannot be taken after the tries stops the build: a commit whose tree
    //       nobody can name is a commit the person did not write, and building it is the loss this guard exists to prevent
    //       (`SPEC.md`, "Merges and commits never fail";
    //       the bound's reason is `THE_CAPTURE_IS_TAKEN_AGAIN_AT_MOST`).
    let mut asked_for: AskedForCommit;
    let mut kept_index: Vec<u8> = Vec::new();
    let mut capture_taken: usize = 0;
    loop {
        capture_taken += 1;
        // What: The private index is read into memory and the copy is written from those bytes.
        // Why:  `git write-tree` given an index file that is not there answers with the empty tree and no error,
        //       so a copy that a cleaner took between the writing and the reading would be a guard that says the commit
        //       asked for nothing (the seventh review's first finding). The bytes in hand are what the copy was made of,
        //       and the read after the tree confirms the copy is still those bytes.
        //       The bytes stay in memory for a rebuild, so a copy that goes later cannot stop the commit being built again.
        let kept: std::io::Result<Vec<u8>> = std::fs::read(input.commit_index).and_then(|bytes: Vec<u8>| {
            return std::fs::write(captured_index.as_path(), bytes.as_slice()).map(|()| { return bytes; });
        });
        // What: A control may hold the commit at this instant, with the copy of the index kept and its tree not yet read
        //       (`transaction_phase.rs`, `Phase::CommitCaptureTaken`).
        // Why:  A copy that goes between the keeping and the read is a commit that cannot say what the person staged,
        //       and no other instant of a build lets a control place that loss.
        //       Nothing arms this marker in a run.
        crate::service_pruned::reach_service_phase(crate::transaction_phase::Phase::CommitCaptureTaken).map_err(|error| {
            return TransactionStop::Message(format!("the capture's test-only phase marker could not be reached: {error}"));
        })?;
        asked_for = the_commit_that_was_asked_for(input, kept.as_deref());
        // What: A control may hold the commit at this instant, with what it asked for known and no shadow yet taken
        //       (`transaction_phase.rs`, `Phase::CommitCaptured`).
        // Why:  A scratch that goes here has not been read by any Git start of the build, and no other instant of a build
        //       lets a control place that loss.
        //       Nothing arms this marker in a run.
        crate::service_pruned::reach_service_phase(crate::transaction_phase::Phase::CommitCaptured).map_err(|error| {
            return TransactionStop::Message(format!("the capture's test-only phase marker could not be reached: {error}"));
        })?;
        let Some(reason) = asked_for.could_not_tell() else {
            kept_index = match kept {
                Ok(bytes) => bytes,
                // A capture that was taken has its bytes; nothing else reaches this branch.
                Err(error) => {
                    return Err(TransactionStop::Message(format!(
                        "the private index {} this commit reads was read and then lost ({error}), so nothing was committed and nothing landed",
                        input.commit_index.display()
                    )));
                }
            };
            break;
        };
        if capture_taken >= THE_CAPTURE_IS_TAKEN_AGAIN_AT_MOST {
            return Err(TransactionStop::Message(format!(
                "what this commit was asked for cannot be told after {capture_taken} tries ({reason}), so nothing was committed and nothing landed"
            )));
        }
        crate::diagnostic_log::debug(
            "serviceNative",
            format!("the capture of this commit's tree could not be taken on try {capture_taken} ({reason}), so it is taken again").as_str(),
        );
    }
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
            // What: The index this command was given is put back exactly as it was read.
            // Why:  Git rewrites the private index as it prepares a commit, and the tree this command asked for is the one the
            //       bytes this command read name; writing them again says what the person staged even when the copy on the
            //       volume went with the first attempt's tree (`SPEC.md`, "Merges and commits never fail").
            if let Err(error) = std::fs::write(input.commit_index, kept_index.as_slice()) {
                return Err(TransactionStop::Message(format!(
                    "the private index {} cannot be put back as it was read ({error}), so nothing was committed and nothing landed",
                    input.commit_index.display()
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
///  What a value this command asked for came to when it was taken from the captures.
/// Why:
///  Three answers, and two of them are not each other:
///  a value the captures name is one Git's commit must hold,
///  a value the command's own form leaves out is one nothing is held against,
///  and a value the captures could not take is a question that has not been answered at all.
///  A two-answer type (`Option`, say) lets the third be read as the second by any caller that asked "is it there",
///  and a guard that could not be computed would then count as a guard that passed:
///  that is the finding this type exists to make unwritable
///  (`landing/service_native.rs`,
///  `build_with_git`;
///  the review of 2026-11-15).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Asked<T> = { known: T } | { leftOut: string } | { couldNotTell: string };
/// ```
#[derive(Debug)]
pub enum Asked<T> {
    /// The captures name it,
    ///  and Git's commit must hold exactly it.
    Known(T),
    /// The command's own form leaves it out,
    ///  and the form is named,
    ///  so a reader knows which silence this is.
    LeftOut(&'static str),
    /// The captures could not say,
    ///  and the words of why.
    CouldNotTell(String),
}

/// What:
///  Which of the three answers a value came to,
///  each reader naming the one it can meet.
/// Why:
///  Only `build_with_git` may meet `CouldNotTell`,
///  and it meets it before anything is built:
///  every other reader of an `Asked` is a reader of a value that was taken,
///  and `holds` answers `CouldNotTell` with a difference rather than with a pass,
///  so no path exists on which a question nobody answered counts as answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.known(); value.leftOutBecause(); value.couldNotTell()
/// ```
impl<T> Asked<T> {
    /// The value the captures name,
    /// or nothing when they name none.
    fn known(&self) -> Option<&T> {
        return match self {
            Asked::Known(found) => Some(found),
            Asked::LeftOut(_) | Asked::CouldNotTell(_) => None,
        };
    }

    /// The form that leaves this value out,
    /// or nothing when some answer was taken.
    fn left_out_because(&self) -> Option<&'static str> {
        return match self {
            Asked::LeftOut(form) => Some(form),
            Asked::Known(_) | Asked::CouldNotTell(_) => None,
        };
    }

    /// Why the captures could not say,
    /// or nothing when they did.
    fn could_not_tell(&self) -> Option<&str> {
        return match self {
            Asked::CouldNotTell(reason) => Some(reason.as_str()),
            Asked::Known(_) | Asked::LeftOut(_) => None,
        };
    }
}

/// What:
///  The commit this command asked for:
///  the parents it must have, and the tree it must hold, taken before Git runs.
/// Why:
///  It is the whole of the question a landable commit answers,
///  and it is answered from the captures and not from the scratch tree,
///  so no file that goes under the scratch tree can change the answer after the captures were taken.
///  Each of the two is one `Asked`,
///  because an amend's parents and a reword fixup's tree are left out by the command's own form,
///  while a capture that could not be taken is a different thing entirely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AskedForCommit = { parents: Asked<string[]>; tree: Asked<string> };
/// ```
#[derive(Debug)]
pub struct AskedForCommit {
    /// The parents Git's commit must name,
    /// in order:
    ///  known, or left out by an amend, or not to be had.
    parents: Asked<Vec<String>>,
    /// The tree Git's commit must hold:
    ///  known, or left out by a reword fixup, or not to be had.
    tree: Asked<String>,
}

impl AskedForCommit {
    /// What:
    ///  Why the captures could not say what this command asked for,
    ///  or nothing when they could.
    /// Why:
    ///  It is the one question `build_with_git` asks before it builds,
    ///  and either value can answer it:
    ///  a commit is held against a parent list and a tree, and one of them missing is the whole of what a build cannot do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// asked.couldNotTell() // the words, or undefined
    /// ```
    fn could_not_tell(&self) -> Option<&str> {
        return self.parents.could_not_tell().or_else(|| { return self.tree.could_not_tell(); });
    }

    /// What:
    ///  Whether the commit Git made is the one this command asked for,
    ///  or what differs.
    /// Why:
    ///  The two differences a person would meet are named in the words a person reads:
    ///  a parent that is not the one the command began with (a conclusion that lost its other side),
    ///  and a tree that is not the one the index held (a commit that would delete what the person kept).
    ///  A value the captures could not take is a difference too:
    ///  no commit can be held against a question nobody answered,
    ///  so it is refused here rather than skipped,
    ///  even though `build_with_git` never builds when its captures say so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// asked.holds(prepared) // ok, or the words of what differs
    /// ```
    fn holds(&self, prepared: &PreparedCommit) -> Result<(), String> {
        match &self.parents {
            Asked::Known(asked) => {
                let made: Vec<String> = parents_of(prepared.raw.as_slice());
                if &made != asked {
                    return Err(format!("its parents are {} where the command began with {}", made.join(", "), asked.join(", ")));
                }
            }
            // The command's form settles no parents of its own, and this is the one silence that is a pass.
            Asked::LeftOut(_form) => {}
            Asked::CouldNotTell(reason) => {
                return Err(format!("the parents this command asked for could not be told ({reason})"));
            }
        }
        match &self.tree {
            Asked::Known(asked) => {
                if &prepared.tree_oid != asked {
                    return Err(format!("its tree is {} where the index this command was given holds {asked}", prepared.tree_oid));
                }
            }
            // A reword fixup's commit holds the tree of the branch it rewords, so no tree of its own was asked.
            Asked::LeftOut(_form) => {}
            Asked::CouldNotTell(reason) => {
                return Err(format!("the tree this command asked for could not be told ({reason})"));
            }
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
fn expected_parents_of(input: &BuildInput<'_>) -> Asked<Vec<String>> {
    let amends: bool = input.arguments.iter().any(|argument| { return argument == "--amend"; });
    if amends {
        return Asked::LeftOut("an amend's parents are the amended commit's, and this command has not read it");
    }
    let mut parents: Vec<String> = Vec::new();
    if let crate::transaction_journal::Base::Commit(base) = &input.capture.base {
        parents.push(base.clone());
    }
    let merge_head: PathBuf = input.capture.layout.git_dir.join("MERGE_HEAD");
    // What: A `MERGE_HEAD` that cannot be read is a parent list nobody can name, and never a merge's absence.
    // Why:  A merge whose real `MERGE_HEAD` went unread would be expected as a one-parent commit and accepted as one,
    //       which is the commit the sixth review found a merge's shadow could become (`conclusion_cleanup.rs`).
    match std::fs::read_to_string(merge_head.as_path()) {
        Ok(text) => {
            for line in text.lines() {
                let name: &str = line.trim();
                if !name.is_empty() {
                    parents.push(String::from(name));
                }
            }
            return Asked::Known(parents);
        }
        // No merge state is the ordinary commit: the base is the whole of its parents.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Asked::Known(parents),
        Err(error) => {
            return Asked::CouldNotTell(format!("the merge state at {} cannot be read ({error})", merge_head.display()));
        }
    }
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
fn the_commit_that_was_asked_for(input: &BuildInput<'_>, kept: Result<&[u8], &std::io::Error>) -> AskedForCommit {
    let parents: Asked<Vec<String>> = expected_parents_of(input);
    // What: A command whose commit carries the tree it started from asks no tree of its own.
    // Why:  A reword fixup's commit holds the tree of the branch it rewords, whatever the index stages, so holding the
    //       index's tree against it would refuse the commit the person asked for; its parents still stand.
    let carries_its_base_tree: bool = input.arguments.iter().any(|argument| {
        let text: String = argument.to_string_lossy().to_string();
        return text.starts_with("--fixup=reword:") || text.starts_with("--fixup=amend:");
    });
    if carries_its_base_tree {
        return AskedForCommit {
            parents,
            tree: Asked::LeftOut("a reword fixup's commit holds the tree of the branch it rewords"),
        };
    }
    // What: A private index that could not be kept is a tree nobody can name, and never an empty one.
    // Why:  Git reads the private index as an empty index when it is gone, so a commit built without this answer is the empty
    //       tree and deletes every tracked file from history while the worktree keeps them.
    let bytes: &[u8] = match kept {
        Ok(bytes) => bytes,
        Err(error) => {
            return AskedForCommit {
                parents,
                tree: Asked::CouldNotTell(format!(
                    "the private index {} this commit reads cannot be kept ({error})",
                    input.commit_index.display()
                )),
            };
        }
    };
    return AskedForCommit { parents, tree: the_tree_the_captured_index_names(input, bytes) };
}

/// What:
///  The tree the kept copy of the index names,
///  or the words of why it cannot be read.
/// Why:
///  The tree is one `git write-tree` on the copy,
///  which writes no new object when the commit is the one asked for (Git has just written that tree),
///  and names exactly what the person staged when it is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = writeTree(capturedIndex) // known | couldNotTell
/// ```
fn the_tree_the_captured_index_names(input: &BuildInput<'_>, kept: &[u8]) -> Asked<String> {
    let captured: PathBuf = input.scratch.join(CAPTURED_INDEX);
    // What: The tree is read by the one function that reads a private index's tree, out of the kept copy.
    // Why:  It gives Git the index and the store the build itself gives it, so the tree this command is held against is the
    //       tree the build would write.
    let tree: String = match crate::commit_index::write_private_tree(input.context, input.cwd, captured.as_path(), input.capture.layout.object_directory.as_path()) {
        Ok(tree) => tree,
        Err(error) => {
            return Asked::CouldNotTell(format!(
                "the tree the kept copy of the index {} names cannot be read ({error})",
                captured.display()
            ));
        }
    };
    // What: An empty tree is an answer only when the kept index holds no entry.
    // Why:  Git answers a question about an index that is not there with the empty tree and no error at all,
    //       so a copy a cleaner took before this question would be read as "the command asked for nothing":
    //       the commit would delete every tracked file from history while the worktree kept them
    //       (the seventh review's first finding). The count of entries is in the index's own header,
    //       which needs no Git to read, and an index that holds one entry can never name the empty tree.
    if the_empty_tree_of(input) == Some(tree.as_str()) && !the_index_holds_no_entry(kept) {
        return Asked::CouldNotTell(format!(
            "the kept copy of the index {} answered with the empty tree where the index this command read holds entries",
            captured.display()
        ));
    }
    return Asked::Known(tree);
}

/// The empty tree's name for the repository's object format,
/// or nothing when this build has no name for it.
/// Why:
///   A name this table does not know is no reason to guess:
///  an empty tree that cannot be recognised is accepted as any other tree,
///  and the caller's other answer (the index's byte count) still stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const emptyTreeOf = (input) => emptyTreeName(input.capture.objectFormat)
/// ```
fn the_empty_tree_of(input: &BuildInput<'_>) -> Option<&'static str> {
    return crate::shadow::empty_tree::empty_tree_name(input.capture.layout.object_format.as_str());
}

/// What:
///  Whether an index file's own header says it holds no entry.
/// Why:
///   Git's index begins with `DIRC`, a version and the number of entries,
///  and every version writes that count in the eight bytes after the signature;
///  a file too short to hold them is no index Git wrote,
///  so it is treated as one whose entries cannot be counted.
///  Nothing here parses an entry:
///  the question is only whether there is one at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsNoEntry = (bytes) => bytes.length < 12 || readUInt32BE(bytes, 8) === 0
/// ```
fn the_index_holds_no_entry(bytes: &[u8]) -> bool {
    const SIGNATURE_AND_COUNT: usize = 12;
    if bytes.len() < SIGNATURE_AND_COUNT {
        return true;
    }
    let count: [u8; 4] = [bytes[8], bytes[9], bytes[10], bytes[11]];
    return u32::from_be_bytes(count) == 0;
}

/// How many times one command takes the capture of what it was asked for, and why.
/// Why this figure:
///  A capture is taken again only when the copy of the private index or the tree it names could not be read,
///  which is one cleaner or one sweep meeting the same attempt:
///  three tries are three such meetings, and a fourth would mean the scratch tree is being emptied as fast as this command
///  fills it, where the honest answer is to say what could not be told and land nothing.
pub const THE_CAPTURE_IS_TAKEN_AGAIN_AT_MOST: usize = 3;

/// The name a commit's own timing record carries when the shadow it built in went and it built again:
///  a fixed name and never a path,
///  because a build part holds a name a person can read and nothing of a machine's own.
pub const SHADOW_WENT_PART: &str = "shadow went";

/// Controls for building with Git stay out of the release executable.
#[cfg(test)]
#[path = "service_native_tests.rs"]
mod tests;
