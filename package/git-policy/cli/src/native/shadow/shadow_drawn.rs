//! What:
//!  The finished shadows the resident service draws out of the pool ahead of the commits that ask for one:
//!  where they are kept,
//!  the draw from a pool member,
//!  the sweep of the trees nobody renewed,
//!  and the value a commit holds while it uses one.
//! Why:
//!  The owner chose the service to rename a member into place ahead of a commit and to hand the commit that path
//!  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
//!  "The design the owner's decision made").
//!  The reason is the one call a claim makes:
//!  330 measured claims put a median 0.08 ms and at most 5,606 ms in the `rename`,
//!  while the forty reads around it never passed 100 ms.
//!  A drawn tree is a member already renamed by the service and checked by the claim's own question,
//!  so a commit that is handed one writes no directory entry before it builds,
//!  and it still runs every check this record's evidence asks of a member.
//!  A tree in use is a fresh drawn tree and an abandoned one is a stale one,
//!  which is the rule a commit's scratch tree already has (`service_scratch_abandonment.rs`):
//!  the command that holds a tree renews it while it lives,
//!  so no lock is written and none can be left held.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const drawn = await draw({ context, template }); // the service's work, never a commit's
//! using held = HandedShadow.hold(tree);            // the commit's, renewed while it runs
//! sweepDrawn(commonDir, now);                       // what nobody renewed
//! ```

/// Private directory creation and tree removal.
use crate::private_storage::{create_private_directory_all, remove_tree};
/// Random names for the trees.
use crate::random_id::random_uuid;
/// The facts a creation is made from.
use crate::shadow_create::ShadowTemplate;
/// The entries a shadow of a worktree shares,
/// and whether the worktree keeps its own `info`.
use crate::shadow_links::keeps_its_own_info;
/// The pool a member is drawn out of.
use crate::shadow_pool::{members, pool_key, pool_root};
/// What a tree must be for this invocation,
/// which is the question a claim asks and a draw asks the same way.
use crate::shadow_pool_member::refusal;
/// One Git start.
use crate::transaction_git::{GitContext, GitFailure};
/// The renewal that keeps a tree a command holds from aging out,
/// and the age it is written against.
use crate::service_scratch_abandonment::SCRATCH_TREE_ABANDONED_AFTER;
use crate::service_scratch_renewal::ScratchRenewal;
/// What the folder beside the pool is called.
use crate::wrapper_names_old_and_new::SHADOW_DRAWN_FOLDER;
/// Paths,
/// the moment and the ages.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How many drawn trees one worktree keeps ready.
/// Measured:
///  the pool's own `POOL_LIMIT` is two because the busiest ten minutes of the wrapper's records held 43 commits,
///  and one drawn tree is what one commit at a time takes:
///  a second commit that asks while the first holds one is served by the pool,
///  which is what it does today,
///  so a stock of two would keep a tree for nothing and cost the pool a member.
pub const DRAWN_LIMIT: usize = 1;

/// The prefix of a drawn tree's name.
const DRAWN_PREFIX: &str = "drawn-";

/// How many pool members one draw looks at before it leaves the stock empty.
/// Two members is the pool's whole size,
/// and a refused one is removed as it is found,
/// so a draw that finds more than this is looking at a pool another fill is repairing.
const DRAW_ATTEMPTS: usize = 4;

/// What:
///  The folder of drawn trees for one worktree.
/// Why:
///   It is named by the same key rule the pool's folder is,
///  so a draw,
///  a sweep and a removal all find the trees of the worktree they are asked about,
///  and it sits beside the pool and never inside it
///  (`SHADOW_DRAWN_FOLDER`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// drawnRoot(template) // <common-dir>/monochromatic-git-policy-cli-shadow-drawn/<worktree>
/// ```
pub fn drawn_root(template: &ShadowTemplate) -> PathBuf {
    return drawn_root_for(template.common_dir.as_path(), template.git_dir.as_path());
}

/// What:
///  The folder of drawn trees for one worktree,
///  from the two directories that name it.
/// Why:
///   The service's draw names the folder from a request,
///  and the sweep asks every folder under the root,
///  so the rule is one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// drawnRootFor(commonDir, gitDir)
/// ```
pub fn drawn_root_for(common_dir: &Path, git_dir: &Path) -> PathBuf {
    return common_dir.join(SHADOW_DRAWN_FOLDER).join(pool_key(common_dir, git_dir));
}

/// What:
///  The drawn trees one worktree's folder holds now.
/// Why:
///   A control counts them,
///  and the sweep needs the names before it asks their ages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// drawnMembers(root)
/// ```
pub fn drawn_members(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else { return Vec::new() };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(DRAWN_PREFIX) {
            found.push(entry.path());
        }
    }
    return found;
}

/// What:
///  Whether `tree` is a drawn shadow of the worktree whose two directories are given.
/// Why:
///   A request names the tree the service removes once its landing is over,
///  so the name must be one only the wrapper makes:
///  directly inside that worktree's drawn folder,
///  named as a drawn tree,
///  and with no part that could lead elsewhere.
///  A request is checked by this rule before it is queued,
///  and the removal is checked again because a request built in this process never passes the decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isADrawnTree(tree, commonDir, gitDir) // tree is one of the wrapper's drawn trees of that worktree
/// ```
pub fn is_a_drawn_tree(tree: &Path, common_dir: &Path, git_dir: &Path) -> bool {
    let root: PathBuf = drawn_root_for(common_dir, git_dir);
    let Some(name) = tree.file_name().and_then(std::ffi::OsStr::to_str) else { return false };
    let identity: &str = name.strip_prefix(DRAWN_PREFIX).unwrap_or("");
    return tree.parent() == Some(root.as_path())
        && !identity.is_empty()
        && identity.as_bytes().iter().all(crate::wrapper_names_old_and_new::is_identity_byte)
        && common_dir.is_absolute()
        && git_dir.is_absolute()
        && !tree.to_string_lossy().contains('\0');
}

/// What:
///  Draws one finished shadow out of the pool and answers where it now is,
///  or nothing when the pool held none this invocation could use.
/// Why:
///   The rename is the work the commit used to wait for,
///  and it happens here,
///  on the service's own thread and before any commit asks.
///  A member that is not what this invocation's creation would make is removed and the next one is tried,
///  exactly as a claim does,
///  so a drawn tree is a member a claim's own question already passed;
///  the commit asks the question again of the tree it is handed,
///  because the repository can change between the draw and the claim.
///  A reftable repository and a linked worktree that keeps its own `info` are drawn for by nothing:
///  every member of theirs would be refused,
///  so a draw makes none (`shadow_pool.rs`, `fill`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drawn = draw({ context, template }); // the tree's path, or undefined
/// ```
///
/// # Errors
///
/// `GitFailure` when the folder of drawn trees cannot be made,
/// or when a member cannot be renamed into it.
pub fn draw(_context: &GitContext, template: &ShadowTemplate) -> Result<Option<PathBuf>, GitFailure> {
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return Ok(None);
    }
    match keeps_its_own_info(template.common_dir.as_path(), template.git_dir.as_path()) {
        Ok(false) => {}
        Ok(true) => return Ok(None),
        Err(error) => return Err(GitFailure(format!("whether the worktree keeps its own info directory cannot be read: {error}"))),
    }
    let root: PathBuf = drawn_root(template);
    if let Err(error) = create_private_directory_all(root.as_path()) {
        return Err(GitFailure(format!("the folder of drawn shadows {} cannot be made: {error}", root.display())));
    }
    let mut examined: usize = 0;
    for member in members(pool_root(template).as_path()) {
        if examined == DRAW_ATTEMPTS {
            break;
        }
        examined += 1;
        let drawn: PathBuf = root.join(format!("{DRAWN_PREFIX}{}", random_uuid().unwrap_or_default()));
        // One `rename` is the whole of the draw: a second draw of the same member fails, and nothing reads a half-drawn tree.
        if std::fs::rename(member.as_path(), drawn.as_path()).is_err() {
            continue;
        }
        match refusal(template, drawn.as_path()) {
            Ok(None) => return Ok(Some(drawn)),
            Ok(Some(reason)) => note_refusal(drawn.as_path(), reason.as_str()),
            Err(error) => note_refusal(drawn.as_path(), error.to_string().as_str()),
        }
        let _removed: std::io::Result<()> = remove_tree(drawn.as_path());
    }
    return Ok(None);
}

/// What:
///  Says in the diagnostic log why a drawn member was refused,
///  and nothing on the terminal.
/// Why:
///   The tree was the wrapper's own cache,
///  so a person can neither cause nor fix the refusal,
///  and the commit that would have taken it takes the pool or the creation instead
///  (`shadow_pool.rs`,
///  `note_refusal`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteRefusal(path, reason); // one debug line
/// ```
fn note_refusal(drawn: &Path, reason: &str) {
    crate::diagnostic_log::debug("shadowDrawn", format!("refused the drawn shadow {}: {reason}", drawn.display()).as_str());
}

/// What:
///  Removes every drawn tree of every worktree that nothing has renewed,
///  and a worktree's folder once nothing is in it.
/// Why:
///   A tree nothing renewed is a tree whose command stopped,
///  by an answer,
///  by a kill or by a panic,
///  and the sweep is what collects it,
///  so a repository that is not committed in is left with nothing of one.
///  The age is the scratch tree's own (`SCRATCH_TREE_ABANDONED_AFTER`),
///  because a drawn tree is held exactly as long as the tree a commit would have made in its scratch:
///  a commit that runs for two hours is the bound both rules are written against.
///  Nothing else reaches a drawn tree:
///  no sweep of the repository scratch directory,
///  and no pool removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepDrawn(commonDir, now) // the trees it removed
/// ```
pub fn sweep_drawn(common_dir: &Path, now: SystemTime) -> Vec<PathBuf> {
    let root: PathBuf = common_dir.join(SHADOW_DRAWN_FOLDER);
    let Ok(entries) = std::fs::read_dir(root.as_path()) else { return Vec::new() };
    let mut removed: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let folder: PathBuf = entry.path();
        removed.extend(sweep_drawn_folder(folder.as_path(), now));
        // An empty folder is the wrapper's own and goes with its last tree, so a worktree that is gone leaves nothing.
        let _gone: std::io::Result<()> = std::fs::remove_dir(folder.as_path());
    }
    return removed;
}

/// What:
///  Removes the drawn trees of one worktree that nothing has renewed.
/// Why:
///   A removal takes the worktree it is given and leaves a tree a live commit holds,
///  which is a fresh one:
///  the commit's own Git is reading that tree while the removal runs,
///  and it must still be there when the commit finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepDrawnOf(commonDir, gitDir, now) // the trees it removed
/// ```
pub fn sweep_drawn_of(common_dir: &Path, git_dir: &Path, now: SystemTime) -> Vec<PathBuf> {
    return sweep_drawn_folder(drawn_root_for(common_dir, git_dir).as_path(), now);
}

/// What:
///  Removes the drawn trees of one folder that nothing has renewed,
///  and answers which it removed.
/// Why:
///  The three callers above and below differ only in which folders they ask about,
///  and the age rule is one rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepDrawnFolder(folder, now) // the trees it removed
/// ```
fn sweep_drawn_folder(folder: &Path, now: SystemTime) -> Vec<PathBuf> {
    let mut removed: Vec<PathBuf> = Vec::new();
    for drawn in drawn_members(folder) {
        if !nothing_holds(drawn.as_path(), now) {
            continue;
        }
        match remove_tree(drawn.as_path()) {
            Ok(()) => {
                crate::diagnostic_log::debug("shadowDrawn", format!("removed the drawn shadow nothing renewed: {}", drawn.display()).as_str());
                removed.push(drawn);
            }
            Err(error) => {
                crate::diagnostic_log::warn("shadowDrawn", format!("the drawn shadow {} was not removed: {error}", drawn.display()).as_str());
            }
        }
    }
    return removed;
}

/// What:
///  Whether a drawn tree has gone longer than any commit holds one without being renewed.
/// Why:
///   This is the whole of the question a tree in use is told from an abandoned one by:
///  a command renews what it holds,
///  and a command that stopped renews nothing,
///  so the answer needs no lock,
///  no process and no record.
///  A tree whose age cannot be told,
///  or one dated in the future,
///  is never old enough:
///  a wrong clock must not delete a live tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingHolds = (tree, now) => age(tree, now) > TWO_HOURS;
/// ```
fn nothing_holds(drawn: &Path, now: SystemTime) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(drawn) else { return false };
    let Ok(modified) = metadata.modified() else { return false };
    let Ok(age) = now.duration_since(modified) else { return false };
    return age > nothing_holds_after();
}

/// What:
///  How long a drawn tree may stand un renewed.
/// Why:
///  It is the scratch tree's own age,
///  named once so the sweep and its controls read the same rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingHoldsAfter = () => SCRATCH_TREE_ABANDONED_AFTER;
/// ```
fn nothing_holds_after() -> Duration {
    return SCRATCH_TREE_ABANDONED_AFTER;
}

/// What:
///  A drawn shadow one commit is handed,
///  renewed while that commit lives and released when it ends.
/// Why:
///   A tree's lifetime is the service's only while the commit that asked for it is alive,
///  and a killed client cannot be asked:
///  the renewal is what says a command still runs,
///  with no lock and no record of a process (`service_scratch_renewal.rs`).
///  A commit that is answered as not landed, or whose request the service never took,
///  removes the tree itself in the background,
///  as it removes a scratch tree the service handed back,
///  and one whose request landed leaves the removal to the service,
///  which removes it beside the loop once that landing is over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = HandedShadow.hold(tree); held.heldByTheService(landed);
/// ```
#[derive(Debug)]
pub struct HandedShadow {
    /// Where the tree is.
    path: PathBuf,
    /// The renewal that keeps it from aging out while this command lives,
    /// and that stops when this value goes:
    /// nothing reads it,
    /// because the thread it starts ends with it.
    _renewal: ScratchRenewal,
    /// Whether the service removes it,
    /// so this process must leave it alone.
    the_service_removes_it: bool,
}

impl HandedShadow {
    /// What:
    ///  Takes hold of a tree the service handed out,
    ///  and starts renewing it.
    /// Why:
    ///   The renewal ends with this value however the command ends,
    ///  and a process killed at any instruction stops renewing at that moment,
    ///  which is the whole of what the sweep needs to tell a live tree from an abandoned one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// HandedShadow.hold(tree)
    /// ```
    pub fn hold(path: PathBuf) -> Self {
        return Self::hold_renewed_every(path, crate::scratch_location::SCRATCH_REFRESH_EVERY);
    }

    /// What:
    ///  `hold`, with the pace the tree is renewed at given,
    ///  so a control can watch one renewal happen without waiting five minutes for it.
    /// Why:
    ///  The pace a commit uses is `SCRATCH_REFRESH_EVERY`,
    ///  the same one its scratch tree is renewed at,
    ///  and a control that could only use that pace could not show a renewal at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// HandedShadow.holdRenewedEvery(tree, 20)
    /// ```
    pub fn hold_renewed_every(path: PathBuf, every: Duration) -> Self {
        let renewal: ScratchRenewal = ScratchRenewal::start(path.as_path(), every);
        return Self { path, _renewal: renewal, the_service_removes_it: false };
    }

    /// Where the tree is.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Records that the service will remove the tree,
    ///  so this command must not.
    /// Why:
    ///   The service removes the trees a request named once that request's landing is over,
    ///  and a removal on both sides at once would report a directory that is already gone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// held.theServiceRemovesIt()
    /// ```
    pub fn the_service_removes_it(&mut self) {
        self.the_service_removes_it = true;
    }
}

impl Drop for HandedShadow {
    /// Removes the tree the service did not take,
    /// beside the command and within the removal's own ceiling.
    fn drop(&mut self) {
        if self.the_service_removes_it {
            return;
        }
        crate::service_scratch_release_removal::start(self.path.clone());
    }
}

/// The draw, the sweep and the value a commit holds are controlled in the test module.
#[cfg(test)]
#[path = "shadow_drawn_tests.rs"]
mod tests;
