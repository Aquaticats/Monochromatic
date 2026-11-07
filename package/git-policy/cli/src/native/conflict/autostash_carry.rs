//! What:
//!  Carrying back what Git's apply of a merge's autostash left in the commit's scratch repository when the apply conflicted:
//!  the stash entry with its `autostash` reflog line,
//! and the unmerged index entries of the files it conflicted in.
//! Why:
//!  The commit that concludes a merge is built by real Git in a scratch repository,
//! and Git's `apply_autostash` runs inside it
//!  (`builtin/commit.c` line 1982).
//! A stash that applies cleanly leaves nothing behind;
//! one that conflicts is stored by `git stash
//!  store` in the scratch repository's own `refs/stash`,
//! and its conflict stages stay in the commit's private index.
//! Both were
//!  thrown away with the scratch directory,
//! so Git's promise ("Your changes are safe in the stash") was false in the real
//!  repository.
//! The objects need no copying:
//! the scratch repository writes into the real object store.
//! The stash entry is stored
//!  in the real repository with Git's own words,
//! and the stages are put into the index the landing installs,
//! under the lock it holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await scratchStash(cleanup) === copiedAutostash(cleanup)) { await storeStash(repository, oid); stages = await unmergedStages(cleanup); }
//! ```

/// The fail-closed recovery failure,
/// for the listing that cannot be read.
use crate::recovery_files::read_optional;
/// Scratch repository commands.
use crate::shadow_git::shadow_request;
/// The record of the commit's scratch directory.
use crate::service_protocol::Cleanup;
/// Running real Git.
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// Diagnostics for the log.
use crate::diagnostic_log::{debug, warn};
/// Borrowed filesystem paths.
use std::path::{Path, PathBuf};

/// The file of the conclusion state a merge's autostash is recorded in.
const AUTOSTASH_FILE: &str = "MERGE_AUTOSTASH";

/// The words Git's own `apply_autostash` gives the stash entry it stores (`sequencer.c`,
/// `git stash store -m autostash`).
const STASH_MESSAGE: &str = "autostash";

/// What:
///  The stash a conflicted apply left in the scratch repository,
/// when the commit consumed a merge's autostash.
/// Why:
///   The copy of `MERGE_AUTOSTASH` made when the commit was prepared names the stash;
/// the scratch repository's `refs/stash` names
///  it only when Git's apply failed and stored it.
/// Both naming the same object is the one case worth carrying.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function conflictedAutostash(git, cleanup): Promise<string | undefined>;
/// ```
fn conflicted_autostash(git: &GitContext, cleanup: &Cleanup) -> Option<String> {
    let copy: Vec<u8> = read_optional(cleanup.scratch.join("conclusion").join(AUTOSTASH_FILE).as_path()).ok().flatten()?;
    let recorded: String = String::from_utf8_lossy(copy.as_slice()).trim().to_owned();
    let shadow: PathBuf = cleanup.scratch.join("shadow");
    let stored = run_git(git, &shadow_request(shadow.as_path(), shadow.as_path(), &["rev-parse", "--verify", "--quiet", "refs/stash"])).ok()?;
    if !stored.succeeded() {
        return None;
    }
    let kept: String = String::from_utf8_lossy(stored.stdout.as_slice()).trim().to_owned();
    if recorded.is_empty() || kept != recorded {
        return None;
    }
    return Some(recorded);
}

/// Whether the text of `git stash list --format=%H` has a line equal to `stash`.
/// The first match settles it,
/// as `any` did.
fn lists_stash(listing: &str, stash: &str) -> bool {
    for line in listing.lines() {
        if line == stash {
            return true;
        }
    }
    return false;
}

/// Whether one byte is the NUL byte that ends each entry of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the space that separates the mode from the object name in an entry.
/// `&u8` borrows the byte.
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// Whether one byte is the tab that separates the stage columns from the path in an entry.
/// `&u8` borrows the byte.
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  Store the conflicted autostash of the commit in the real repository's stash list,
/// once.
/// Why:
///   The landing's cleanup may run again after a restart;
/// a stash already in the list is not stored a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carryStash(git, repository, cleanup): Promise<boolean>;
/// ```
pub fn carry_stash(git: &GitContext, repository: &Path, cleanup: &Cleanup) -> bool {
    let Some(stash) = conflicted_autostash(git, cleanup) else { return true };
    let listed = run_git(git, &GitRequest::new(repository, &["stash", "list", "--format=%H"]));
    if let Ok(output) = &listed
        && output.succeeded()
        && lists_stash(String::from_utf8_lossy(output.stdout.as_slice()).as_ref(), stash.as_str())
    {
        return true;
    }
    return match run_git(git, &GitRequest::new(repository, &["stash", "store", "-m", STASH_MESSAGE, stash.as_str()])) {
        Ok(output) if output.succeeded() => {
            debug("autostashCarry", format!("the stash {stash} the apply conflicted on was stored in the repository").as_str());
            true
        }
        other => {
            warn("autostashCarry", format!("the stash {stash} could not be stored in the repository: {other:?}").as_str());
            false
        }
    };
}

/// What:
///  Put the conflict stages the apply left in the commit's private index into `built`,
/// the index the landing is about to install.
/// Why:
///   The files the stash conflicted in must be unmerged in the real index as they are after Git's own apply.
/// A path is removed
///  first,
/// because a stage entry does not replace a merged one;
/// the stages come as `git ls-files --unmerged` prints them,
/// which
///  is what `update-index --index-info` reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carryStages(git, repository, cleanup, built): Promise<boolean>;
/// ```
pub fn carry_stages(git: &GitContext, repository: &Path, cleanup: &Cleanup, built: &Path) -> bool {
    if conflicted_autostash(git, cleanup).is_none() || !built.exists() {
        return true;
    }
    let mut listing: GitRequest = GitRequest::new(repository, &["ls-files", "--unmerged", "-z"]);
    listing.index = Some(cleanup.scratch.join("commit.index"));
    let Ok(stages) = run_git(git, &listing) else { return false };
    if !stages.succeeded() {
        return false;
    }
    if stages.stdout.is_empty() {
        return true;
    }
    // Each entry is `mode oid stage TAB path`, ended by a NUL; the first entry's object name gives the hash length.
    let mut entries: Vec<&[u8]> = Vec::new();
    for entry in stages.stdout.split(is_nul_byte) {
        if !entry.is_empty() {
            entries.push(entry);
        }
    }
    let length: usize = match entries[0].split(is_space_byte).nth(1) {
        Some(object_name) => object_name.len(),
        None => 0,
    };
    let mut paths: Vec<&[u8]> = Vec::new();
    for entry in &entries {
        if let Some(path) = entry.split(is_tab_byte).nth(1) {
            paths.push(path);
        }
    }
    paths.dedup();
    let mut input: Vec<u8> = Vec::new();
    for path in &paths {
        input.extend_from_slice(format!("0 {}\t", "0".repeat(length)).as_bytes());
        input.extend_from_slice(path);
        input.push(0);
    }
    for entry in &entries {
        input.extend_from_slice(entry);
        input.push(0);
    }
    let mut placing: GitRequest = GitRequest::new(repository, &["update-index", "-z", "--index-info"]);
    placing.index = Some(built.to_path_buf());
    placing.input = Some(input);
    return match run_git(git, &placing) {
        Ok(output) if output.succeeded() => true,
        other => {
            warn("autostashCarry", format!("the conflict stages of the stash could not be put into the index: {other:?}").as_str());
            false
        }
    };
}
