//! What:
//!  The pool of finished shadow repositories a commit can take instead of making one:
//! where it lives,
//!  one `rename` to claim a member,
//! the fill the resident service runs ahead,
//! and the sweep of a fill that was killed.
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
/// The facts a creation is made from, and the creation without a commit's own parts.
use crate::shadow_create::{ShadowTemplate, create_shadow_template};
/// What a member must be for this invocation.
use crate::shadow_pool_member::refusal;
/// The repository scratch root the pool lives in.
use crate::wrapper_names_old_and_new::REPOSITORY_SCRATCH;
/// One Git start.
use crate::transaction_git::{GitContext, GitFailure};
/// The build's named stretches, which the fill names its own work under.
use crate::commit_build_part_durations::BuildPartClock;
/// Paths and time.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The folder of finished shadows, inside the repository scratch root,
/// beside the shadow of a transaction.
pub const SHADOW_POOL_NAME: &str = "shadow-pool";

/// How many finished shadows are kept for one worktree.
/// Measured:
/// a fill survives about 40 commits a minute here at the busiest,
/// so two cover a burst that begins before the first fill ends,
/// and the third member a longer pool would hold is one no run of this host claimed between two fills.
pub const POOL_LIMIT: usize = 2;

/// How old a half-made member must be before a fill removes it.
/// Measured:
/// a fill is about 40 calls and took 2 to 10 ms in the traces of
/// `doc/planning/cli-git-commit-latency-under-many-agents.md`
/// ("The creation is, counted"),
/// and the volume has held one call for 64 s,
/// so this covers a fill that waits and leaves nothing a commit could take.
pub const FILLING_STALE_AFTER: Duration = Duration::from_secs(900);

/// The prefix of a finished member's name.
const MEMBER_PREFIX: &str = "member-";

/// The prefix of a member that is still being made.
const FILLING_PREFIX: &str = "filling-";

/// How many members one claim looks at before it makes its own shadow.
/// Two members is the pool's whole size, and a refused one is removed as it is found,
/// so a claim that finds more than this is looking at a pool another fill is repairing.
const CLAIM_ATTEMPTS: usize = 4;

/// What:
///  The folder of finished shadows for one worktree.
/// Why:
///  A member's config names the worktree root,
/// so a linked worktree's members are its own;
/// the service of a worktree is the one that fills them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poolRoot(template) // <common-dir>/<scratch>/shadow-pool/<worktree>
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
    let key: String = match git_dir == common_dir {
        true => String::from("main"),
        false => format!("worktree-{}", git_dir.file_name().unwrap_or_default().to_string_lossy()),
    };
    return common_dir
        .join(REPOSITORY_SCRATCH.current)
        .join(SHADOW_POOL_NAME)
        .join(key);
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
            Ok(Some(reason)) => {
                crate::diagnostic_log::warn("shadowPool", format!("refused the pooled shadow {name}: {reason}").as_str());
            }
            Err(error) => {
                crate::diagnostic_log::warn("shadowPool", format!("refused the pooled shadow {name}: {error}").as_str());
            }
        }
        let _removed: std::io::Result<()> = remove_tree(destination);
    }
    return false;
}

/// What:
///  Makes one finished shadow ahead, when the pool holds room for it.
/// Why:
///   The resident service runs this beside the landing of a commit it already answered,
/// so no commit pays for the creation of another.
///  Nothing here can fail a commit:
/// a fill that cannot run leaves an empty pool, and the next commit makes its own creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fill({ context, template }) // the member's path, or undefined when none was made
/// ```
///
/// # Errors
///
/// `GitFailure` when the shadow could not be created;
/// the half-made member is removed first, so the pool never holds one.
pub fn fill(context: &GitContext, template: &ShadowTemplate) -> Result<Option<PathBuf>, GitFailure> {
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return Ok(None);
    }
    let root: PathBuf = pool_root(template);
    if members(root.as_path()).len() >= POOL_LIMIT {
        return Ok(None);
    }
    create_private_directory_all(root.as_path())
        .map_err(|error| GitFailure(format!("the shadow pool {} cannot be made: {error}", root.display())))?;
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
        let old: bool = match entry.metadata().and_then(|metadata| metadata.modified()) {
            Ok(modified) => now.duration_since(modified).is_ok_and(|age| age > FILLING_STALE_AFTER),
            Err(_) => false,
        };
        if old {
            let _removed: std::io::Result<()> = remove_tree(entry.path().as_path());
            crate::diagnostic_log::debug("shadowPool", format!("removed the abandoned fill {name}").as_str());
        }
    }
}

/// Pool controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_pool_tests.rs"]
mod tests;
