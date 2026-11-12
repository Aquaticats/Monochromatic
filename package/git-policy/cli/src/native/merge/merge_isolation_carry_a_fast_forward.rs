//! What:
//!  A merge that fast-forwards,
//!  carried here instead of handed to native Git:
//!  the person's own work at the paths the incoming commits
//!  write is captured and kept,
//!  the branch moves to the incoming commit itself,
//!  and their work is merged with what
//!  that commit wrote.
//! Why:
//!  A fast-forward writes exactly the paths the two commits differ in,
//!  so the work it can lose is the work at those paths,
//!  which is what a merge that creates a commit carries as well
//!  (`SPEC.md`,
//! "Carry edits forward";
//!  the owner's rules "Manual merge should never fail too."
//!  and "never discard user data").
//!  The branch moves to the incoming commit itself,
//!  because that is what a fast-forward is:
//! no merge commit is made,
//!  `--ff-only`'s
//!  promise that the branch only ever moves forward holds,
//!  and the tree the checkout is updated to is that commit's own.
//!  The flow is the same steps a merge commit takes,
//!  in the same order,
//!  and differs in what it never does:
//! there is no merge tree to compute,
//!  no message to word,
//!  no editor to
//!  open and no commit to create.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dirty = dirtyPaths(...); const captured = captureFor(context, root, overlap(dirty, written)); return apply(input, session, plan);
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::overlapping_paths;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The paths whose own work the merge would write over,
/// the edits an index mark hides,
/// and the capture that keeps them.
use crate::merge_isolation_dirty_paths::{capture_for, dirty_paths};
/// The marks that hide an edit from every status.
use crate::merge_isolation_marks_the_status_hides::paths_the_status_hides;
/// The named pipes,
/// sockets and devices this merge would have to remove,
/// and the line that stops it instead.
use crate::merge_isolation_a_special_file_the_merge_changes::{a_special_file_the_merge_changes, the_line as a_special_file_line};
/// The person's work as the capture holds it.
use crate::merge_isolation_carry_parts::Capture;
/// The writing half of the merge.
use crate::merge_isolation_apply::apply;
/// The session a merge asks its questions in,
/// and the values it passes between its steps.
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, IsolatedMergeInput, MergeConfig, Plan, TAG, refused};
use crate::merge_options::MergeShape;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Everything one carried fast-forward reads from the merge that decided to carry it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FastForwardToCarry = { session; shape; config; reference; gitDir; root; commits; written };
/// ```
pub struct FastForwardToCarry<'a> {
    /// The session the merge asks Git its questions in.
    pub session: &'a Session,
    /// The command line.
    pub shape: &'a MergeShape,
    /// The settings read for the merge.
    pub config: &'a MergeConfig,
    /// The commit or branch the person named,
    /// as its message and its lines name it.
    pub reference: &'a str,
    /// The worktree's Git directory.
    pub git_dir: &'a Path,
    /// The worktree top level.
    pub root: &'a Path,
    /// The commit the branch holds,
    /// and the incoming commit the branch moves to.
    pub commits: (&'a str, &'a str),
    /// The paths the incoming commits write,
    /// which are the paths this fast-forward changes.
    pub written: &'a [Vec<u8>],
}

/// What:
///  Complete a fast-forward over the person's own work,
///  or give up and let native Git run the command.
/// Why:
///   Every read that fails is native Git's answer,
///  as it is for a merge that creates a commit:
///  the command
///  is then forwarded,
///  and its own refusal or its own loss is what the person sees without the wrapper.
///  Nothing is
///  written before the capture exists and the record names it,
///  so a kill at any instant leaves the person's bytes where
///  recovery finds them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carryTheFastForward(input, toCarry): Promise<IsolatedMerge>; // throws Bail
/// ```
pub fn carry_the_fast_forward(input: &IsolatedMergeInput<'_>, to_carry: &FastForwardToCarry<'_>) -> Result<IsolatedMerge, Bail> {
    let session: &Session = to_carry.session;
    let (git_dir, root): (&Path, &Path) = (to_carry.git_dir, to_carry.root);
    let (head, target): (&str, &str) = to_carry.commits;
    let written: &[Vec<u8>] = to_carry.written;
    let mut dirty: Vec<Vec<u8>> = dirty_paths(session, root, written)?;
    dirty.extend(paths_the_status_hides(&session.context, root, written).map_err(Bail::from)?);
    // A fast-forward writes exactly the paths its two commits differ in, so the work it would write over is the work at
    //  those paths and nowhere else.
    let overlap: Vec<Vec<u8>> = overlapping_paths(dirty.as_slice(), written);
    // A named pipe, a socket or a device holds no bytes for the carry, and Git's own update would remove it without a
    //  copy; the merge stops before the first write and names what it kept, as it does for one that makes a commit.
    let special: Vec<Vec<u8>> = a_special_file_the_merge_changes(root, written);
    if !special.is_empty() {
        return Err(refused(1, a_special_file_line(to_carry.reference, special.as_slice())));
    }
    let carried: Option<Capture> = match capture_for(&session.context, root, overlap.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("native Git runs this fast-forward: its paths hold work this command cannot carry: {reason}").as_str());
            return Err(Bail::Forward(None));
        }
    };
    let plan: Plan<'_> = Plan {
        shape: to_carry.shape,
        config: to_carry.config,
        git_dir,
        root,
        old_oid: head,
        new_oid: target,
        carried: carried.as_ref(),
        fast_forward: true,
        written,
    };
    return apply(input, session, &plan);
}
