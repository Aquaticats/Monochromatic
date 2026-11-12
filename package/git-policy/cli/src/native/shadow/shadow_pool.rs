//! What:
//!  The pool of finished shadow repositories a commit can take instead of making one:
//! where it lives,
//!  one `rename` to claim a member,
//! the fill the resident service runs ahead,
//! and the sweeps of a fill that was killed and of a worktree Git no longer lists.
//! Why:
//!  The creation makes about 40 directory-entry creations per commit on a volume where those calls wait seconds,
//!  and `shadow links` alone held 11,383 ms of 22,657 ms of `buildMs` in one measured window
//! (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//! "The ranking").
//!  The owner chose this way off it
//! (`doc/handover/questions-for-the-owner.md`,
//! "May a commit take its private repository from a pool made ahead?":
//! "Pool made ahead (Recommended),
//! We don't need to build a shadow for every commit.
//! Be smarter.").
//!  A claim is one `rename`,
//! which fails for the second claimant,
//! so a member is reserved without a lock
//! (`AGENTS.md`,
//! `NLK`).
//!  An empty pool is not a failure:
//! the commit makes its creation exactly as it did before.
//!  A member this invocation refuses is the wrapper's own cache being repaired,
//! which the person can neither cause nor fix:
//! the refusal is kept in the debug log and named in the commit's timing record,
//!  as a build part,
//! and never written to the terminal
//! (`shadow_pool_member.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const member = await take(template, shadow); // true when the pool had one
//! await fill({ context, template });            // the service's work, never a commit's
//! ```

/// Private directory creation and tree removal.
use crate::private_storage::{create_private_directory_all, remove_tree};
/// Random member names.
use crate::random_id::random_uuid;
/// The facts a creation is made from,
/// and the creation without a commit's own parts.
use crate::shadow_create::{ShadowTemplate, create_shadow_template};
/// What a member must be for this invocation.
use crate::shadow_pool_member::refusal;
/// Whether a worktree's shadows can be taken from a pool at all.
use crate::shadow_links::keeps_its_own_info;
/// One Git start.
use crate::transaction_git::{GitContext, GitFailure};
/// What the pool's folder is called,
/// beside the repository scratch directory.
use crate::wrapper_names_old_and_new::SHADOW_POOL_FOLDER;
/// The build's named stretches,
/// which the fill names its own work under
/// and the claim names a refusal in.
use crate::commit_build_part_durations::{BuildPartClock, note_build_part};
/// Paths and time.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How many finished shadows are kept for one worktree.
/// Measured:
/// the busiest ten minutes of the wrapper's own records of 2026-11-10 hold 43 commits,
/// each claim takes one member,
/// and one fill at a time is as many as a landing makes,
/// so a pool of two covers the moment two commits of one worktree overlap while the fill after the last landing runs,
/// and a third member is one a busy repository would keep for nothing.
/// (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What each part of a commit's private repository is for,
/// and how many commits need it".)
pub const POOL_LIMIT: usize = 2;

/// How old a half-made member must be before a fill removes it.
/// Measured:
/// a fill is about 40 calls and took 2 to 10 ms in the traces of
/// `doc/planning/cli-git-commit-latency-under-many-agents.md`
/// ("The creation is,
/// counted"),
/// and the volume has held one call for 64 s,
/// so this covers a fill that waits and leaves nothing a commit could take.
pub const FILLING_STALE_AFTER: Duration = Duration::from_secs(900);

/// The prefix of a finished member's name.
const MEMBER_PREFIX: &str = "member-";

/// The pool folder of the main checkout,
/// whose own Git directory is the common directory the pool lives in.
const MAIN_KEY: &str = "main";

/// The prefix of a linked worktree's pool folder,
/// followed by the name of its own Git directory.
const WORKTREE_KEY_PREFIX: &str = "worktree-";

/// The prefix of a member that is still being made.
const FILLING_PREFIX: &str = "filling-";

/// What a refused member is named as in the commit's timing record.
/// Why:
///  The record already names the creation a refusal forced,
/// and what happened before it is the one fact a reading of a slow commit needs:
///  a person sees nothing,
/// because a member of the wrapper's own cache is nothing they can act on,
///  and the name is a fixed one rather than a path or a reason,
///  which is what a build part may hold.
const REFUSED_PART: &str = "shadow pool refused";

/// How many members one claim looks at before it makes its own shadow.
/// Two members is the pool's whole size,
/// and a refused one is removed as it is found,
/// so a claim that finds more than this is looking at a pool another fill is repairing.
const CLAIM_ATTEMPTS: usize = 4;

/// What:
///  The folder of finished shadows for one worktree.
/// Why:
///  A member's config names the worktree root,
/// so a linked worktree's members are its own;
/// the service of a worktree is the one that fills them.
///  It sits in the Git common directory beside the repository scratch directory and never inside it:
/// a member is standing state the owner asked for
/// (`doc/handover/questions-for-the-owner.md`,
/// "The private repository every commit builds"),
/// not scratch,
/// so the check that a commit leaves no scratch behind keeps meaning what it says,
/// and no sweep,
/// no release of a commit's tree and no reclamation of a removed worktree treats a pooled shadow as a leftover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poolRoot(template) // <common-dir>/monochromatic-git-policy-cli-shadow-pool/<worktree>
/// ```
pub fn pool_root(template: &ShadowTemplate) -> PathBuf {
    return pool_root_for(template.common_dir.as_path(), template.git_dir.as_path());
}

/// What:
///  The folder of finished shadows for one worktree,
///  from the two directories that name it.
/// Why:
///   The service asks whether a fill is needed at all before it reads any fact that costs a Git start,
/// and the paths are all that answer needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poolRootFor(commonDir, gitDir)
/// ```
pub fn pool_root_for(common_dir: &Path, git_dir: &Path) -> PathBuf {
    return common_dir.join(SHADOW_POOL_FOLDER).join(pool_key(common_dir, git_dir));
}

/// What:
///  The folder name one worktree's pool has.
/// Why:
///  The sweep of a gone worktree's pool and the name a fill and a claim use must be one rule,
///  or a sweep would look at a folder nothing fills and leave the one a commit takes from.
///  The name is lossy text of the Git directory's own name,
///  so both sides compare the same text whatever the path's bytes are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poolKey(commonDir, gitDir) // 'main' or 'worktree-<name>'
/// ```
fn pool_key(common_dir: &Path, git_dir: &Path) -> String {
    return match git_dir == common_dir {
        true => String::from(MAIN_KEY),
        false => format!("{WORKTREE_KEY_PREFIX}{}", git_dir.file_name().unwrap_or_default().to_string_lossy()),
    };
}

/// What:
///  Takes one finished shadow into `destination`,
/// and answers whether the pool had one this invocation can use.
/// Why:
///  The `rename` is the claim:
/// two commits cannot rename one member,
/// so no lock is taken and no commit waits for another.
///  A member that is not what this invocation's creation would make is removed and the next one is tried,
/// and a pool with none left,
/// or one whose entries a concurrent sweep took,
/// leaves the commit to make its own creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// take(template, destination) // true when the destination is now a member
/// ```
pub fn take(template: &ShadowTemplate, destination: &Path) -> bool {
    // A reftable repository's shadow is Git's own tables, which no member of this pool holds.
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return false;
    }
    let root: PathBuf = pool_root(template);
    let Ok(entries) = std::fs::read_dir(root.as_path()) else { return false };
    let mut examined: usize = 0;
    for entry in entries.flatten() {
        if examined == CLAIM_ATTEMPTS {
            break;
        }
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(MEMBER_PREFIX) {
            continue;
        }
        examined += 1;
        // Another commit that claimed this member first leaves nothing here to rename.
        if std::fs::rename(entry.path(), destination).is_err() {
            continue;
        }
        match refusal(template, destination) {
            Ok(None) => return true,
            Ok(Some(reason)) => note_refusal(name.as_str(), reason.as_str()),
            Err(error) => note_refusal(name.as_str(), error.to_string().as_str()),
        }
        let _removed: std::io::Result<()> = remove_tree(destination);
    }
    return false;
}

/// What:
///  Keeps a refused member where a person diagnosing a commit can find it:
///  in this commit's timing record and in the debug log,
/// and nowhere on the terminal.
/// Why:
///  The member was the wrapper's,
/// so no line about it is something the person did or can do,
/// and a timestamped warning in the middle of a commit's output only alarms them
/// (`AGENTS.md`,
/// the rule that a confusing behaviour we control is a defect).
///  The refusal moved that commit onto the creation,
/// which the record already names,
/// so the part says why the build paid for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteRefusal(name, reason); // build part + debug line, no warning
/// ```
fn note_refusal(name: &str, reason: &str) {
    note_build_part(REFUSED_PART, Duration::ZERO);
    crate::diagnostic_log::debug("shadowPool", format!("refused the pooled shadow {name}: {reason}").as_str());
}

/// What:
///  Makes one finished shadow ahead,
/// when the pool holds room for it.
/// Why:
///   The resident service runs this beside the landing of a commit it already answered,
/// so no commit pays for the creation of another.
///  An empty pool is not a failure:
/// a fill that cannot run leaves an empty pool,
/// and the next commit makes its own creation.
///  A worktree that keeps its own `info` directory is filled for nothing:
/// every member of it would be refused,
/// so a fill makes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fill({ context, template }) // the member's path, or undefined when none was made
/// ```
///
/// # Errors
///
/// `GitFailure` when the shadow could not be created;
/// the half-made member is removed first,
/// so the pool never holds one.
pub fn fill(context: &GitContext, template: &ShadowTemplate) -> Result<Option<PathBuf>, GitFailure> {
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return Ok(None);
    }
    match keeps_its_own_info(template.common_dir.as_path(), template.git_dir.as_path()) {
        Ok(false) => {}
        Ok(true) => {
            crate::diagnostic_log::debug(
                "shadowPool",
                format!("made no member for {}: the worktree keeps its own info directory, so its shadows are made where the commit is", template.common_dir.display()).as_str(),
            );
            return Ok(None);
        }
        Err(error) => return Err(GitFailure(format!("whether the worktree keeps its own info directory cannot be read: {error}"))),
    }
    let root: PathBuf = pool_root(template);
    if members(root.as_path()).len() >= POOL_LIMIT {
        return Ok(None);
    }
    if let Err(error) = create_private_directory_all(root.as_path()) {
        return Err(GitFailure(format!("the shadow pool {} cannot be made: {error}", root.display())));
    }
    sweep_abandoned(root.as_path(), SystemTime::now());
    let staging: PathBuf = root.join(format!("{FILLING_PREFIX}{}", random_uuid().unwrap_or_default()));
    // The creation names its own stretches; nothing reads them here, so they are taken and dropped rather than
    // left in the process's ledger for a record that this process never writes.
    let mut clock: BuildPartClock = BuildPartClock::begin("shadow fill");
    let made: Result<(), GitFailure> = create_shadow_template(context, template, staging.as_path(), &mut clock);
    let _parts = crate::commit_build_part_durations::drain_build_parts();
    if let Err(error) = made {
        let _removed: std::io::Result<()> = remove_tree(staging.as_path());
        return Err(error);
    }
    let member: PathBuf = root.join(format!("{MEMBER_PREFIX}{}", random_uuid().unwrap_or_default()));
    // One `rename` publishes the member whole, so no claim can take a tree that is still being made.
    if let Err(error) = std::fs::rename(staging.as_path(), member.as_path()) {
        let _removed: std::io::Result<()> = remove_tree(staging.as_path());
        return Err(GitFailure(format!("the finished shadow {} cannot be published: {error}", staging.display())));
    }
    crate::diagnostic_log::debug("shadowPool", format!("filled the shadow pool with {}", member.display()).as_str());
    return Ok(Some(member));
}

/// What:
///  Removes the pool folder of every worktree Git no longer lists,
///  and answers which folders it removed.
/// Why:
///  A worktree's pool is named after that worktree's own Git directory,
///  and the members of a folder whose Git directory is gone are private repositories with their links into this one:
///  no commit can ever claim them.
///  A `git worktree prune`,
///  a registration deleted by hand and a worktree destroyed without Git all leave that folder,
///  and the sweep is what collects it,
///  so a pool left unused does not keep a copy of the repository's shape for ever.
///  The question is asked of the repository's own `worktrees` folder and never of Git,
///  so a command pays one `read_dir` that usually finds no pool folder at all.
///  A worktree Git is adding keeps its folder:
///  Git makes the registration before the checkout,
///  so a registration is the worktree's presence and not a leftover.
///  A folder the wrapper did not name stays:
///  only a folder named for a worktree can be proven to belong to one,
///  and the pool holds state of the person's own otherwise.
///  Nothing here can fail a command:
///  a folder that cannot be removed is named in the diagnostic log and left for the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = sweepGoneWorktrees(commonDir); // the pool folders of worktrees Git no longer lists
/// ```
pub fn sweep_gone_worktrees(common_dir: &Path) -> Vec<PathBuf> {
    let root: PathBuf = common_dir.join(SHADOW_POOL_FOLDER);
    let Ok(entries) = std::fs::read_dir(root.as_path()) else { return Vec::new() };
    // The keys of the worktrees Git lists, made with the same rule the folders were named by.
    let mut listed: Vec<String> = Vec::new();
    if let Ok(worktrees) = std::fs::read_dir(common_dir.join("worktrees")) {
        for entry in worktrees.flatten() {
            listed.push(pool_key(common_dir, &common_dir.join("worktrees").join(entry.file_name())));
        }
    }
    let mut removed: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(WORKTREE_KEY_PREFIX) || listed.contains(&name) {
            continue;
        }
        let folder: PathBuf = entry.path();
        match remove_tree(folder.as_path()) {
            Ok(()) => {
                crate::diagnostic_log::debug("shadowPool", format!("removed the pool of the worktree Git no longer lists: {name}").as_str());
                removed.push(folder);
            }
            Err(error) => {
                crate::diagnostic_log::warn("shadowPool", format!("the pool of the worktree Git no longer lists, {}, was not removed: {error}", folder.display()).as_str());
            }
        }
    }
    return removed;
}

/// What:
///  The finished shadows the pool holds now.
/// Why:
///   A fill makes nothing when the pool is full,
/// and a control counts what a claim could find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// members(root)
/// ```
pub fn members(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else { return Vec::new() };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(MEMBER_PREFIX) {
            found.push(entry.path());
        }
    }
    return found;
}

/// What:
///  Removes the half-made members a killed fill left,
///  as of the moment given.
/// Why:
///   A fill publishes its member with one rename,
/// so anything still named as a fill belongs to a process that stopped,
/// and nothing else ever claims it.
///  The moment is a parameter so a control can show the age that removes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepAbandoned(root, now)
/// ```
fn sweep_abandoned(root: &Path, now: SystemTime) {
    let Ok(entries) = std::fs::read_dir(root) else { return };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(FILLING_PREFIX) {
            continue;
        }
        let old: bool = match entry.metadata().and_then(modification_time) {
            Ok(modified) => now.duration_since(modified).is_ok_and(older_than_the_fill_age),
            Err(_) => false,
        };
        if old {
            let _removed: std::io::Result<()> = remove_tree(entry.path().as_path());
            crate::diagnostic_log::debug("shadowPool", format!("removed the abandoned fill {name}").as_str());
        }
    }
}

/// What:
///  When one entry of the pool was last written.
/// Why:
///  A fill that was killed leaves a folder whose age says whether a fill is still writing into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modificationTime = (metadata) => metadata.modified();
/// ```
fn modification_time(metadata: std::fs::Metadata) -> std::io::Result<SystemTime> {
    return metadata.modified();
}

/// What:
///  Whether a fill has been unfinished for longer than any fill takes.
/// Why:
///  `sweep_abandoned` removes what this answers yes for,
/// and names the age it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const olderThanTheFillAge = (age) => age > FILLING_STALE_AFTER;
/// ```
fn older_than_the_fill_age(age: Duration) -> bool {
    return age > FILLING_STALE_AFTER;
}

/// Pool controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_pool_tests.rs"]
mod tests;
