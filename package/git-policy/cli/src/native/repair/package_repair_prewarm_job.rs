//! What:
//!  Fill the Cargo build directories a new worktree's record asks for,
//! inside the commit service,
//! after the add that queued it has returned.
//! Why:
//!  The clone of a source checkout's build directory is the heaviest step the package repair
//!  does,
//! and it only ever fills a directory the new worktree does not have yet,
//! so nothing about the worktree's correctness depends on it finishing inside the add
//! (`package_repair_prewarm.rs`).
//! The filling is three steps per workspace:
//! clone the source's build directory into a staging directory beside the destination,
//! clean the worktree's own crates out of the staging copy,
//! and publish the staging copy under the destination's name by a rename that never replaces.
//! Both directories the clean reads and writes are named here,
//! so the filling deletes nothing from the worktree itself:
//! its only effects there are the one rename that publishes a directory the worktree did not have
//! and the `Cargo.lock` the checked-out one goes back over,
//! because Cargo resolves the worktree's workspace for the clean.
//! That lock is never written by the clean:
//! it runs with `--locked`,
//! a lock such a clean refuses is resolved by Cargo first,
//! and the checked-out bytes go back only when the file still holds exactly what that resolution wrote,
//! so a write of the person's lands in their own file and stays there.
//! A destination that exists means a build got there first,
//! so the staging copy is dropped and the build keeps everything it made.
//! The staging directory,
//! the clean's target directory and the marker beside them are names of this job's own,
//! and the marker holding this job's own text is what proves a directory at one of them is its leftover:
//! anything at those names without that proof is somebody's work and is never removed,
//! so a workspace whose usual names are taken is filled under names of this run's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prewarmAwaiting(item); // for each entry: clone to the staging name, clean it, publish it unless the destination exists
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_cargo::{locked_clean_arguments, manual_command};
/// The lock file the filling's report names,
/// and the faces that read it and put the checked-out bytes back over a resolution's.
use crate::package_repair_cargo_lock::{LOCK_FILE, read_lock, restore_only_the_cleans_own};
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, find_program, run_tool_awaiting};
use crate::package_repair_prewarm::{BUILD_DIRECTORY_VARIABLE, PrewarmEntry, STAGING_SUFFIX, TARGET_DIRECTORY_VARIABLE, decode};
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_tree_clone::{Cloned, clone_tree_awaiting};
use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::work_item::WorkItem;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairPrewarmJob";

/// The program the filling resolves,
/// as the repair does.
const PROGRAM: &str = "cargo";

/// Where a worktree keeps its own link to the toolchain's Cargo,
/// relative to the worktree root,
/// as the toolchain repair resolves `mpm-shadow`.
const OWN_CARGO_LINK: &str = ".cache/toolchain-bin/cargo";

/// What:
///  What every workspace of one record needs before it copies anything.
/// Why:
///  The program lookup and the bounds are the same for every workspace of one record,
/// so they are read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fill = { cargo: string; limits: ToolLimits };
/// ```
pub(crate) struct Fill {
    /// The Cargo that cleans the staging copy.
    pub(crate) cargo: PathBuf,
    /// The bounds it runs under.
    pub(crate) limits: ToolLimits,
    /// The Git directory of the worktree whose lock the clean may displace,
    /// or nothing when no Git directory is known,
    /// in which case a lock the clean changed is left as it is found.
    pub(crate) kept_in: Option<PathBuf>,
}

/// What:
///  The program that cleans a staging copy:
/// the search path's `cargo`,
/// else the worktree's own link to the toolchain's.
/// Why:
///   The service is a user unit whose search path need not hold the toolchain the worktree's
/// launchers name,
/// and the worktree's own link is the one its commands use.
/// The link sits at the worktree's root,
/// so the caller passes the worktree root and not the workspace below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cargoFor = (worktreeRoot, environment) => findProgram('cargo', environment.path()) ?? (exists(join(worktreeRoot, OWN_CARGO_LINK)) ? join(worktreeRoot, OWN_CARGO_LINK) : undefined);
/// ```
fn cargo_for(root: &Path, environment: &RepairEnvironment) -> Option<PathBuf> {
    if let Some(found) = find_program(PROGRAM, environment.path()) {
        return Some(found);
    }
    let own: PathBuf = root.join(OWN_CARGO_LINK);
    return if own.is_file() { Some(own) } else { None };
}

/// What:
///  The staging directory a workspace's build directory is cloned into.
/// Why:
///   It sits beside the destination,
/// so publishing it is one rename on one filesystem,
/// and its name is the destination's,
/// so no workspace can fill another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagingOf = (destination) => `${destination}${STAGING_SUFFIX}`;
/// ```
pub fn staging_of(destination: &Path) -> PathBuf {
    let mut named: OsString = destination.as_os_str().to_os_string();
    named.push(STAGING_SUFFIX);
    return PathBuf::from(named);
}

/// What:
///  The target directory the clean is given,
/// so it deletes nothing from the worktree's own.
/// Why:
///  `cargo clean --package` removes a package's final artifacts from the target directory as well as
///  its intermediates from the build directory,
/// and the worktree's target directory may hold what the copy carried and what a build made.
/// Naming a directory of this job's own keeps the filling's deletions inside directories it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const targetOf = (staging) => `${staging}${TARGET_SUFFIX}`;
/// ```
fn target_of(staging: &Path) -> PathBuf {
    let mut named: OsString = staging.as_os_str().to_os_string();
    named.push(TARGET_SUFFIX);
    return PathBuf::from(named);
}

/// What the target directory of a clean is suffixed with,
/// beside the staging directory.
const TARGET_SUFFIX: &str = ".target";

/// What:
///  The staging name a filling of this workspace works in:
/// the usual name, then the token of the process filling it and the instant it asked.
/// Why:
///   A name only its maker can have is a folder only its maker can be in.
/// Two fillings of one workspace never meet, so nothing has to decide who may clean under a shared name, and nothing
/// has to ask a lock whether the writer of a marker still lives: the three facts a marker wrote and read at different
/// instants are gone (the review of `a7a6958f9d`, first and second findings; `AGENTS.md`, rule `NLK`).
/// The token names the maker by its process and the identity of its birth, as every record of a live process in this
/// package does (`process_identity.rs`), so a leftover is recognised by a maker that is gone.
/// The instant keeps two fillings of one process apart: one process fills many workspaces, and a workspace can be filled
/// twice in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagingOfThisRun = (usual) => `${usual}.${currentCopyOwner().token()}.${nanos().toString(16)}`;
/// ```
fn a_staging_name_of_this_runs_own(usual: &Path) -> Result<PathBuf, String> {
    let owner: crate::worktree_copy_owner::CopyOwner = match crate::worktree_copy_owner::current_copy_owner() {
        Ok(owner) => owner,
        Err(refusal) => return Err(format!("this process cannot name itself, so nothing beside {} was filled ({})", usual.display(), refusal.0)),
    };
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let named: OsString = format!("{}.{}.{:x}", usual.display(), owner.token(), since_the_epoch.as_nanos()).into();
    return Ok(PathBuf::from(named));
}

/// What:
///  The maker a name at the end of a string names,
/// when the name is one this release chose:
/// a token, then the instant the filling asked,
/// after whatever the name stands beside.
/// Why:
///   The token is what a leftover is judged by,
/// and a name that does not end in one (a person's directory, an older release's names, the usual name itself) is nobody's
/// to remove.
/// A token never holds a dot (`CopyOwner::token`), so the two last parts of a maker name are the token and the instant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makerAtTheEndOf = (name) => { const [token, instant] = name.split('.').slice(-2); return /^[0-9a-f]+$/.test(instant) ? CopyOwner.fromToken(token) : undefined };
/// ```
fn the_maker_at_the_end_of(name: &std::ffi::OsStr) -> Option<crate::worktree_copy_owner::CopyOwner> {
    use std::os::unix::ffi::OsStrExt;
    let bytes: &[u8] = name.as_bytes();
    // A target directory carries the staging name's maker and one more suffix.
    let named: &[u8] = bytes.strip_suffix(TARGET_SUFFIX.as_bytes()).unwrap_or(bytes);
    let at: usize = named.iter().rposition(|byte| return *byte == b'.')?;
    let (before, instant) = (named.get(..at)?, named.get(at + 1..)?);
    // The instant is the hexadecimal count a filling wrote: anything else ends no maker's name.
    if instant.is_empty() || !instant.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let token_at: usize = before.iter().rposition(|byte| return *byte == b'.')?;
    let token: &str = std::str::from_utf8(before.get(token_at + 1..)?).ok()?;
    return crate::worktree_copy_owner::CopyOwner::from_token(token);
}

/// What:
///  The maker a name of this workspace's usual names names,
/// and nothing for any other name.
/// Why:
///  The usual name is a prefix this workspace's names begin with, and a name that begins differently is another
///  workspace's, which this filling has no business in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makerOfThisWorkspacesName = (name, usualName) => name.startsWith(`${usualName}.`) ? makerAtTheEndOf(name) : undefined
/// ```
fn the_maker_of_this_workspaces_name(name: &std::ffi::OsStr, usual_name: &std::ffi::OsStr) -> Option<crate::worktree_copy_owner::CopyOwner> {
    use std::os::unix::ffi::OsStrExt;
    let mut prefix: Vec<u8> = usual_name.as_bytes().to_vec();
    prefix.push(b'.');
    if !name.as_bytes().starts_with(prefix.as_slice()) {
        return None;
    }
    return the_maker_at_the_end_of(name);
}

/// What:
///  Remove the staging names and the clean's targets of this workspace whose maker is gone.
/// Why:
///   A filling that was killed leaves the name it made, and only a maker that is gone leaves it to be cleaned.
/// A name whose maker lives is that filling's and is left exactly as it is, and so is every name no filling of this
/// release chose: a person's directory, an older release's, and the usual name itself, which nothing of this release
/// makes and nothing removes ("never discard user data").
/// The question is the package's own liveness question (`worktree_copy_owner::token_owner_is_dead`), which answers
/// "not dead" for a maker the platform will not report on, so nothing is removed on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdir(parent)) if (makerOf(name)?.isDead()) { rm(`${name}`, {recursive:true, force:true}); rm(`${name}.target`, ...) }
/// ```
fn remove_the_leftovers_of_makers_that_are_gone(usual: &Path) {
    let (Some(parent), Some(usual_name)) = (usual.parent(), usual.file_name()) else { return };
    let Ok(entries) = std::fs::read_dir(parent) else { return };
    for entry in entries.flatten() {
        let name: OsString = entry.file_name();
        let Some(maker) = the_maker_of_this_workspaces_name(name.as_ref(), usual_name) else { continue };
        let path: PathBuf = entry.path();
        // Only a name a filling marked is the wrapper's to remove; a name that merely looks right is somebody's.
        if !a_filling_made_this(path.as_path()) {
            continue;
        }
        let dead: bool = matches!(crate::worktree_copy_owner::token_owner_is_dead(Some(maker.token().as_str()), path.as_path()), Ok(true));
        if !dead {
            continue;
        }
        if let Err(reason) = remove_if_any(path.as_path()) {
            debug(TAG, format!("{reason}; a staging copy a filling that is gone left stays for the next filling").as_str());
        }
        if let Err(reason) = remove_if_any(target_of(path.as_path()).as_path()) {
            debug(TAG, format!("{reason}; the clean's target of a filling that is gone stays for the next filling").as_str());
        }
    }
}

/// What:
///  The names one filling of one workspace works in:
/// a staging directory that is this filling's alone, and the target the clean deletes from beside it.
/// Why:
///   The staging copy is what the source's build state is cloned into and published from,
/// and the target directory is what the clean deletes from, so the clean never reaches a worktree's own.
/// Both carry the maker, so a name only this filling can have is a folder only this filling can be in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheNames = { staging: string; target: string };
/// ```
struct TheNames {
    /// The directory the source's build state is cloned into,
    /// and published from.
    staging: PathBuf,
    /// The target directory the clean deletes from.
    target: PathBuf,
}

/// What:
///  The names this filling works in,
/// with the names of fillings of this workspace that are gone given back first.
/// Why:
///   A name of this filling's own cannot collide with another filling's, so taking the names removes nothing that
/// anybody holds; what is removed is what fillings that ended left behind, and only when their maker is gone.
///
/// # Errors
///
/// A process that cannot name itself, which fills nothing.
fn names_for(destination: &Path) -> Result<TheNames, String> {
    let usual: PathBuf = staging_of(destination);
    remove_the_leftovers_of_makers_that_are_gone(usual.as_path());
    let staging: PathBuf = a_staging_name_of_this_runs_own(usual.as_path())?;
    return Ok(TheNames { staging: staging.clone(), target: target_of(staging.as_path()) });
}

/// What:
///  Give the one name back when the filling of one workspace ends,
/// whatever it came to.
/// Why:
///   The target directory is the clean's, and it is removed here rather than on the failing paths alone,
/// so a successful filling leaves no directory of its own behind for every build directory it fills.
/// The staging name is not removed here: a published copy is renamed away, and a staging copy that could not be finished
/// is removed by the step that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfAny(names.target);
/// ```
fn give_back(names: &TheNames) {
    if let Err(reason) = remove_if_any(names.target.as_path()) {
        // The filling itself is done and published, and this directory is this job's own: the next filling of the same
        //  workspace clears it.
        debug(TAG, format!("{reason}; the filling is done and the directory stays for the next one").as_str());
    }
}

/// What:
///  Give back the state the add moved aside,
/// whose record named it,
/// and nothing when that name is not one a filling of this job made.
/// Why:
///   The add staged it under a name carrying its maker and handed the name in the work item, so the record is the proof;
/// and once the worktree has its build directory there is nothing to publish it under, so it is the wrapper's to remove
/// ("a name only its maker can have is a folder only its maker can be in", and the record says which one this job may
/// give back).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBackTheStagedState = (staging) => { if (!makerAtTheEndOf(basename(staging))) return; removeIfAny(staging); removeIfAny(`${staging}.target`); };
/// ```
fn give_back_the_staged_state(staging: &Path) {
    if staging.file_name().and_then(the_maker_at_the_end_of).is_none() || !a_filling_made_this(staging) {
        debug(TAG, format!("{} is not a name this job staged, so it is left exactly as it is found", staging.display()).as_str());
        return;
    }
    if let Err(reason) = remove_if_any(staging) {
        debug(TAG, format!("{reason}; the staged state stays for the next filling").as_str());
    }
    if let Err(reason) = remove_if_any(target_of(staging).as_path()) {
        debug(TAG, format!("{reason}; the clean's target of the staged state stays for the next filling").as_str());
    }
}

/// The file a filling writes beside a staged name it made:
/// the one thing a removal may rest on, because a record cannot write it into a directory it did not stage.
pub(crate) const STAGED_MARKER_SUFFIX: &str = ".made";

/// The marker beside the staged name `staged`.
fn marker_of_the_staging(staged: &Path) -> PathBuf {
    let mut named: OsString = staged.as_os_str().to_os_string();
    named.push(STAGED_MARKER_SUFFIX);
    return PathBuf::from(named);
}

#[cfg(test)]
thread_local! {
    /// Whether the next marker write is refused, as storage refusing it does.
    pub(crate) static A_MARKER_WRITE_THE_STORAGE_REFUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What:
///  The words a filling writes beside a name it staged.
const STAGED_MARKER_TEXT: &[u8] = b"monochromatic-git-policy-cli staged the directory beside this file and may remove it.\n";

/// What:
///  Write the marker beside a staged name this filling made,
///  and never over one that is there.
fn note_that_a_filling_made(staged: &Path) {
    let marker: PathBuf = marker_of_the_staging(staged);
    // What: A control's own refusal of the marker write, taken out when it is read so it happens once.
    // Why:  The state this leaves behind is what a storage that denies the write leaves behind, and no host this test may
    //       use can be made to deny it on demand.
    #[cfg(test)]
    if A_MARKER_WRITE_THE_STORAGE_REFUSED.with(|asked: &std::cell::Cell<bool>| return asked.replace(false)) {
        debug(TAG, format!("{} was refused by a control, as storage denying the write refuses it", marker.display()).as_str());
        return;
    }
    if let Err(reason) = crate::private_storage::write_private_file_unsynced(marker.as_path(), STAGED_MARKER_TEXT) {
        debug(TAG, format!("{} could not be written, so {} is left for a later filling ({reason})", marker.display(), staged.display()).as_str());
    }
}

/// What:
///  Whether a filling wrote the marker beside `staged`,
///  which is what makes the directory there the wrapper's to remove.
/// Why:
///  A name is a string, and a record can spell it: a removal on a record's word needs something the wrapper itself wrote,
///  and a marker a record cannot have written into a directory it did not stage is that
///  (the review of `da9c767270`, finding 1).
fn a_filling_made_this(staged: &Path) -> bool {
    let limit: u64 = (STAGED_MARKER_TEXT.len() as u64).saturating_add(1);
    return matches!(crate::private_storage::read_regular_file(marker_of_the_staging(staged).as_path(), limit), Ok(text) if text == STAGED_MARKER_TEXT);
}

/// What:
///  Whether the state a prewarm record says the add staged for `to` is one this wrapper made there.
/// Why:
///   The work directory is written by other commands, and a record is their text:
/// a name that parses as a maker's name is no proof that the wrapper made what stands there.
/// What a filling may clean and publish is derived from what the wrapper computed for this destination:
/// `staging_of(to)`, beside `to`, then a maker and its instant (`a_staging_name_of_this_runs_own`).
/// A record that names anything else is refused, so a record can never point a removal at a folder of somebody
/// else's (the review of `7a6504e5b6`, finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagedStateIsOurs = (to, staged) => staged.dirname === stagingOf(to).dirname && basename(staged).startsWith(`${basename(stagingOf(to))}.`) && makerAtTheEndOf(staged) !== undefined
/// ```
fn staged_state_is_ours(to: &Path, staged: &Path) -> bool {
    let usual: PathBuf = staging_of(to);
    if staged.parent() != usual.parent() {
        return false;
    }
    let (Some(name), Some(ours)) = (staged.file_name(), usual.file_name()) else { return false };
    use std::os::unix::ffi::OsStrExt;
    if !name.as_bytes().starts_with(ours.as_bytes()) {
        return false;
    }
    if name.as_bytes().get(ours.as_bytes().len()) != Some(&b'.') {
        return false;
    }
    return the_maker_at_the_end_of(name).is_some();
}

/// What:
///  Remove a directory this job made when a step failed or a build got there first.
/// Why:
///   The names belong to this job alone,
/// and a caller removes only a name this filling made, or a name of this workspace whose maker is gone
/// (`remove_the_leftovers_of_makers_that_are_gone`),
/// so what they hold was made by this filling or by a filling of the same workspace that ended, and nothing a person or a
/// build made.
/// Every call passes one of those two kinds:
///  a leftover of a maker that is gone in `names_for`, in `fill_one` and in `stage_the_worktrees_own_state`,
///  the staging of the names this filling took in `fill_the_staging`,
///  and the target of those names in `give_back`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfAny(path); // absent is fine
/// ```
fn remove_if_any(path: &Path) -> Result<(), String> {
    return match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{} could not be removed ({error})", path.display())),
    };
}

/// What the clean is told about the worktree's lock,
/// so the report of a clean it stopped names the flag to look up.
/// Why:
///   The switch itself is one place,
/// in `package_repair_cargo::locked_clean_arguments`,
/// and the words a person reads are another,
/// so the report spells the same flag these words name.
const LOCK_FLAG: &str = "--locked";

/// What:
///  Why a tool ended as it did,
/// in the words a person reads.
/// Why:
///   A report names what stopped the clean,
/// and the tool's own last words name the manifest or lock it refused,
/// so a variant's name alone is not the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endReason = (end) => end.kind === 'failed' ? end.reason : end.kind === 'timedOut' ? 'it showed no sign of work for its quiet window and was stopped' : ceilingReason(Tool.cargo, end.ceiling);
/// ```
fn end_reason(end: &ToolEnd) -> String {
    return match end {
        ToolEnd::Succeeded => String::from("it succeeded"),
        ToolEnd::Failed(reason) => reason.clone(),
        ToolEnd::TimedOut => String::from("it showed no sign of work for its quiet window and was stopped"),
        ToolEnd::CeilingReached(ceiling) => crate::package_repair_report::ceiling_reason(crate::package_repair_report::Tool::Cargo, *ceiling),
    };
}

/// What:
///  What a person does next when the filling's clean stopped,
/// in the words they read.
/// Why:
///   The one failure this job causes by itself is a lock Cargo would have had to change,
/// and its remedy is the lock's own command or the clean the add would have run;
/// any other reason is the tool's,
/// and the tool's own words name it.
/// A person is told where their bytes went either way,
/// so a stopped filling never leaves them guessing whether the worktree was written.
fn what_to_do_next(entry: &PrewarmEntry, reason: &str) -> String {
    if !reason.contains(LOCK_FILE) {
        return String::from("The reason above names what to fix, and the next add fills the copy.");
    }
    return format!(
        "Update the lock in {} (for example `cargo update --offline --workspace` there) and the next add fills the copy, or clean the worktree's \
         own crates with `{}`, which may update the lock as your own clean would",
        entry.root.display(),
        manual_command(entry.root.as_path(), entry.members.as_slice()),
    );
}

/// What:
///  Clean the worktree's own crates out of the staging copy.
/// Why:
///   The source's own artifacts name the source's paths,
/// so they must not reach the worktree,
/// while the dependency builds hold nothing path-specific and are the reason the copy exists.
/// Both directories are this job's own,
/// which is what keeps the clean away from the workspace's live state,
/// and the clean itself is told to leave `Cargo.lock` alone,
/// which is what keeps the clean out of the worktree's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanTheCopy({ fill, entry, names }): ToolRun
/// ```
async fn clean_the_copy(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> ToolRun {
    let command: ToolCommand = ToolCommand {
        arguments: locked_clean_arguments(entry.members.as_slice()),
        directory: entry.root.clone(),
        set: vec![
            (OsString::from(BUILD_DIRECTORY_VARIABLE), names.staging.as_os_str().to_os_string()),
            (OsString::from(TARGET_DIRECTORY_VARIABLE), names.target.as_os_str().to_os_string()),
        ],
    };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// What:
///  Ask Cargo to resolve the worktree's workspace,
/// which is what writes the `Cargo.lock` the clean itself must not write.
/// Why:
///   `cargo clean` resolves the workspace,
/// and for a lock that no longer matches its manifest that rewrite is part of the clean;
///  `--locked` keeps it out of the clean,
/// so the resolution is asked for as a step of its own.
/// The bytes it writes are Cargo's own,
/// which is what lets the caller tell Cargo's rewrite from a person's newer write
/// (`package_repair_cargo_lock.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTool(cargo, { arguments: ['metadata', '--offline', '--format-version', '1'], directory: entry.root });
/// ```
async fn resolve_the_lock(fill: &Fill, entry: &PrewarmEntry) -> ToolRun {
    let arguments: Vec<OsString> = ["metadata", "--offline", "--format-version", "1"].iter().map(OsString::from).collect();
    let command: ToolCommand = ToolCommand { arguments, directory: entry.root.clone(), set: Vec::new() };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// What:
///  Publish the staging copy under the workspace's build directory name,
/// or say that something else is already there.
/// Why:
///   The rename never replaces,
/// so a build that started while the copy was being made keeps every artifact it wrote and the
/// staging copy is the only thing removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const publish = (staging, destination) => renameNoReplace(staging, destination);
/// ```
fn publish(staging: &Path, destination: &Path) -> Result<bool, String> {
    return match rename_noreplace(staging, destination) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) if rename_unsupported(&error) => Err(format!("{} cannot be published, because this filesystem cannot rename without replacing ({error})", destination.display())),
        Err(error) => Err(format!("{} could not be published ({error})", destination.display())),
    };
}

/// What a workspace's filling came to.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Filled {
    /// The copy was published under the workspace's build directory name.
    Published,
    /// The worktree already had a build directory,
    /// so nothing was published.
    Kept,
}

/// What:
///  Fill one workspace's build directory:
/// take the names this job fills under,
/// clone the source's,
/// clean the copy,
/// publish it,
/// and give the names back.
/// Why:
///   Each workspace's state is in a directory of its own,
/// so one workspace's failure is reported and the next one still runs.
/// The names are taken before anything is written,
/// so nothing this job did not make is removed,
/// and given back whatever the filling came to,
/// so a successful one leaves no directory behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillOne({ fill, entry }): Promise<Filled>;
/// ```
async fn fill_one(fill: &Fill, entry: &PrewarmEntry) -> Result<Filled, String> {
    // What:
    //  A record that names staged state the wrapper did not make for this destination is refused,
    //  before either branch reads, cleans, publishes or removes it.
    // Why:
    //  Nothing may be removed or published on the strength of a name a record asserts:
    //  the path must be the one this wrapper computed for this destination (the review of `7a6504e5b6`, finding 1).
    if let Some(staged) = entry.staged_in.as_ref()
        && (!staged_state_is_ours(entry.to.as_path(), staged.as_path()) || !a_filling_made_this(staged.as_path()))
    {
        return Err(format!(
            "the prewarm record names {} as the state the add staged, and this wrapper stages nothing there for {}; nothing was read, published or removed",
            staged.display(),
            entry.to.display()
        ));
    }
    // A destination that exists is a build that got there first: the worktree keeps everything it made. A filling of this
    //  workspace that was killed after it published left its own name and an empty staging copy beside that directory, and
    //  a name whose maker is gone is nobody's: this filling gives those names back.
    if entry.to.exists() {
        remove_the_leftovers_of_makers_that_are_gone(staging_of(entry.to.as_path()).as_path());
        // The state the add moved aside has no destination to be published under any more, so it is given back here:
        //  the record that carries its name is the proof it is this job's, and nothing will read it again.
        if let Some(staged) = entry.staged_in.as_ref() {
            give_back_the_staged_state(staged.as_path());
        }
        return Ok(Filled::Kept);
    }
    make_the_destinations_own_directory(entry)?;
    let names: TheNames = match entry.staged_in.as_ref() {
        // The add moved the worktree's own copied state aside before it answered, so there is nothing to clone: the state to
        //  clean is already there.
        Some(staging) => match staged_names(staging)? {
            Some(names) => names,
            None => return Ok(Filled::Kept),
        },
        None => names_for(entry.to.as_path())?,
    };
    let filled: Result<Filled, String> = fill_the_staging(fill, entry, &names).await;
    give_back(&names);
    return filled;
}

/// What:
///  The names a filling of the worktree's own staged state works in,
/// or nothing when the add staged none.
/// Why:
///   The add moves the copied build state to a name of its own making in one rename before the add answers,
/// so a build that starts at once finds no build directory and compiles everything itself.
/// This run cleans that state out of the way it always does,
/// and publishes it back only where the worktree still has no build directory of its own.
/// The name carries the maker the add recorded, and a name that carries none was not staged by this wrapper: it is
/// somebody's work and is left exactly as it is found.
///
/// # Errors
///
/// A staging name no filling of this job made.
fn staged_names(staging: &Path) -> Result<Option<TheNames>, String> {
    if !staging.exists() {
        return Ok(None);
    }
    let Some(_maker) = staging.file_name().and_then(the_maker_at_the_end_of) else {
        return Err(format!("{} holds what no filling of this job staged, so it is left exactly as it is found", staging.display()));
    };
    let target: PathBuf = target_of(staging);
    return Ok(Some(TheNames { staging: staging.to_path_buf(), target }));
}

/// What:
///  Move the worktree's own copied build state into the filling's staging name,
/// under this job's marker,
/// in one rename;
/// or say that there was nothing to move.
/// Why:
///   Cargo treats the copied artifacts of the worktree's own crates as fresh,
/// so a build that started before a clean would reuse them at the wrong paths.
/// The add cannot wait for the clean (`REPAIR_TOOL_CEILING` is eighteen minutes and the quiet window is six hundred
///  seconds), and it must not leave that state where a build can trust it,
/// so it moves the state aside in one rename before it answers:
/// a build that starts at once finds no build directory and compiles everything itself,
/// which is correct and only slower,
/// and this job cleans the state and publishes it back where nothing has rebuilt yet.
/// The marker is written first,
/// so a kill between the two leaves a name this job proves is its own.
///
/// # Errors
///
/// A staging name another file or directory holds,
/// and a state that cannot be moved.
pub(crate) fn stage_the_worktrees_own_state(destination: &Path, git_dir: Option<&Path>) -> Result<Staged, String> {
    if !destination.exists() {
        return Ok(Staged::Nothing);
    }
    // The staging names sit beside the build directory they take, so the rename stays inside the worktree's own subvolume:
    // btrfs refuses to rename a directory out of a subvolume, and a worktree of this repository is one.
    let usual: PathBuf = staging_of(destination);
    // What a filling of this workspace that is gone left behind goes first, so the names this add makes are its own.
    remove_the_leftovers_of_makers_that_are_gone(usual.as_path());
    let staging: PathBuf = a_staging_name_of_this_runs_own(usual.as_path())?;
    if staging.exists() {
        return Err(format!(
            "{} already holds something this wrapper did not stage, so the copied build state is left where it is; the worktree's first build would reuse it, so run `cargo clean --offline --package <member>` in {} instead",
            staging.display(),
            destination.display()
        ));
    }
    // The staging name and the target directory its clean uses stand inside the worktree, so the repository's own exclude file
    // names them and no `git status` or `git add -A` of the person's sees them.
    if let Some(directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(directory)
    {
        return Err(format!("{reason}, so the copied build state stays where it is; run `cargo clean --offline --package <member>` in the worktree instead"));
    }
    return match std::fs::rename(destination, staging.as_path()) {
        Ok(()) => {
            // What: The name this add staged under is marked as the wrapper's own.
            // Why:  The filling and a later look remove what stands there only on this mark (the review of `da9c767270`).
            note_that_a_filling_made(staging.as_path());
            Ok(Staged::Staged(staging))
        }
        Err(error) => Err(format!(
            "the copied build state at {} could not be moved aside ({error}), so it stays where it is; run `cargo clean --offline --package <member>` in that worktree instead",
            destination.display()
        )),
    };
}

/// The patterns that keep this job's staging names out of the person's `git status` and `git add -A`.
///
/// They are written into the repository's own exclude file (`<git-dir>/info/exclude`), which applies to every worktree of it,
/// because the names stand beside the build directory they take, inside the worktree.
pub(crate) const EXCLUDED_STAGING_PATTERNS: [&str; 4] = [
    // The names of this release: a staging name, its clean's target, and the same names carrying a filling's maker.
    "*.monochromatic-prewarm.*/",
    // The names of earlier releases: the usual name and its target, and the marker a marker-based release wrote.
    "*.monochromatic-prewarm/",
    "*.monochromatic-prewarm.target/",
    "*.monochromatic-prewarm.owner",
];

/// What:
///  Name this job's staging patterns in the repository's exclude file,
/// once.
/// Why:
///   The staging name sits inside the worktree beside the build directory it takes,
/// because btrfs refuses a rename of a directory out of a worktree that is a subvolume,
/// so without this the person's `git status` would show it and a `git add -A` would commit a copy of `target/`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludeTheStagingNames = (gitDir) => { const path = join(gitDir, 'info/exclude'); if (!read(path, '').includes(PATTERNS[0])) appendFileSync(path, PATTERNS.join('\n') + '\n'); };
/// ```
pub(crate) fn exclude_the_staging_names(git_dir: &Path) -> Result<(), String> {
    let info: PathBuf = git_dir.join("info");
    let exclude: PathBuf = info.join("exclude");
    let already: String = std::fs::read_to_string(exclude.as_path()).unwrap_or_default();
    if already.contains(EXCLUDED_STAGING_PATTERNS[0]) {
        return Ok(());
    }
    if let Err(error) = std::fs::create_dir_all(info.as_path()) {
        return Err(format!("{} could not be made ({error})", info.display()));
    }
    let named_before: usize = already.len();
    let mut text: String = already;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for pattern in EXCLUDED_STAGING_PATTERNS {
        text.push_str(pattern);
        text.push('\n');
    }
    // The file is the person's own and Git made it when the repository was, so this appends to it and never rewrites it:
    // a kill in the middle leaves their lines whole and at most adds this job's patterns twice.
    let opened = std::fs::OpenOptions::new().create(true).append(true).open(exclude.as_path());
    let mut file = match opened {
        Ok(file) => file,
        Err(error) => return Err(format!("{} could not be opened ({error})", exclude.display())),
    };
    return match std::io::Write::write_all(&mut file, &text.as_bytes()[named_before..]) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be appended to ({error})", exclude.display())),
    };
}

/// What a staging step came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Staged {
    /// The worktree's own copied state now sits at this staging path, under this job's marker.
    Staged(PathBuf),
    /// The worktree held no build state of its own to move.
    Nothing,
}

/// What:
///  Make the directory the destination and its two staging names live in.
/// Why:
///   The names are taken before the clone,
/// and the marker that claims them sits beside the destination,
/// so that directory must exist before the marker is written.
/// Cargo keys an external build directory by the workspace's path,
/// and the name it keys it by is two levels below the root a person's configuration names
/// (`build-dir = "~/build/{workspace-path-hash}"` resolves to `~/build/8d/0e503e8832e433`),
/// so neither level exists before the first build of a new worktree,
/// and Cargo makes them itself before a build.
/// The clone makes the same directory before it publishes anything,
/// so making it here changes nothing the clone would leave.
///
/// # Errors
///
/// A directory that cannot be made,
/// which fills nothing and says why.
fn make_the_destinations_own_directory(entry: &PrewarmEntry) -> Result<(), String> {
    let Some(parent) = entry.to.parent() else { return Ok(()) };
    return match std::fs::create_dir_all(parent) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be made, so nothing of this workspace was filled ({error})", parent.display())),
    };
}

/// What:
///  The three steps of one workspace's filling:
/// clone the source's build directory into the staging copy,
/// clean the worktree's own crates out of the copy,
/// and publish it under the worktree's name.
/// Why:
///   A copy that could not be finished is this job's own,
/// and nothing else reads it,
/// so each step that fails removes it and says which step it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillTheStaging({ fill, entry, names }): Promise<Filled>;
/// ```
async fn fill_the_staging(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> Result<Filled, String> {
    if entry.staged_in.is_none() {
        let Some(from) = entry.from.as_ref() else {
            return Err(String::from("a filling that clones was handed no source build directory, so nothing was cleaned"));
        };
        let made: Result<Cloned, String> = clone_tree_awaiting(from.as_path(), names.staging.as_path(), 1).await;
        if let Err(reason) = made {
            let _removed = remove_if_any(names.staging.as_path());
            return Err(format!("the build state of {} could not be cloned ({reason})", from.display()));
        }
    }
    // A build that started during the clone owns the destination now, and this job touches nothing of it.
    if entry.to.exists() {
        let _removed = remove_if_any(names.staging.as_path());
        return Ok(Filled::Kept);
    }
    // Cargo resolves the worktree's workspace for this clean even though it deletes only from directories of this job's
    //  own, and that rewrites a `Cargo.lock` that no longer matches its manifest or creates one that is missing. This job
    //  runs after the add returned, when the person may be working in the worktree, so the clean itself must never write
    //  that lock (`--locked`): a lock it refuses is resolved by Cargo first, and the checked-out lock goes back only when
    //  the file still holds exactly the bytes that resolution wrote (`package_repair_cargo_lock.rs`).
    let lock_before: Option<Vec<u8>> = read_lock(entry.root.as_path());
    let mut run: ToolRun = clean_the_copy(fill, entry, names).await;
    if !matches!(run.end, ToolEnd::Succeeded) {
        let resolved: ToolRun = resolve_the_lock(fill, entry).await;
        if matches!(resolved.end, ToolEnd::Succeeded) {
            let the_cleans_own: Option<Vec<u8>> = read_lock(entry.root.as_path());
            run = clean_the_copy(fill, entry, names).await;
            if let Some(written) = the_cleans_own.as_deref() {
                match restore_only_the_cleans_own(entry.root.as_path(), lock_before.as_deref(), fill.kept_in.as_deref(), written) {
                    None => {}
                    Some(said) => debug(TAG, format!("{}: {said}", entry.root.display()).as_str()),
                }
            }
        }
    }
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_if_any(names.staging.as_path());
        let reason: String = end_reason(&run.end);
        return Err(format!(
            "the copy of {} could not be cleaned ({reason}), so nothing was published and everything staged for it was removed. The clean itself \
             never writes the worktree's {LOCK_FILE} (`{LOCK_FLAG}`): a lock it refuses is resolved by Cargo first, and the checked-out bytes go \
             back over bytes only that resolution wrote, so a write of yours stays yours. {}",
            entry.from.as_deref().unwrap_or(names.staging.as_path()).display(),
            what_to_do_next(entry, reason.as_str()),
        ));
    }
    return match publish(names.staging.as_path(), entry.to.as_path()) {
        Ok(true) => Ok(Filled::Published),
        Ok(false) => {
            let _removed = remove_if_any(names.staging.as_path());
            Ok(Filled::Kept)
        }
        Err(reason) => {
            let _removed = remove_if_any(names.staging.as_path());
            Err(reason)
        }
    };
}

/// What:
///  Fill every build directory one prewarm record names,
/// awaited by the service's round.
/// Why:
///   The add no longer does this work (`package_repair_prewarm.rs`);
/// the record says which workspaces,
/// which source checkout,
/// and which members must not be carried,
/// so no metadata read is repeated here.
/// A record whose payload cannot be read is a failure that says why,
/// and one workspace that fails does not stop the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prewarmAwaiting(item): Promise<Verdict>;
/// ```
pub async fn prewarm_awaiting(item: &WorkItem) -> Verdict {
    return prewarm_awaiting_with(item, &RepairEnvironment::from_process()).await;
}

/// What:
///  `prewarm_awaiting` with the environment handed over instead of read from the process.
/// Why:
///   The program lookup and the two bounds come from the process,
/// which a control cannot change,
/// so a control that proves where the worktree's own Cargo is found hands its own environment in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prewarmAwaitingWith(item, environment);
/// ```
async fn prewarm_awaiting_with(item: &WorkItem, environment: &RepairEnvironment) -> Verdict {
    let entries: Vec<PrewarmEntry> = match decode(item.parent.as_str()) {
        Ok(decoded) => decoded,
        Err(reason) => return Verdict::Failed(reason),
    };
    if entries.is_empty() {
        return Verdict::Skipped(String::from("the record names no workspaces"));
    }
    // The worktree's own link to the toolchain's Cargo sits at the worktree's root, whatever workspace the first entry
    //  names, so the record's repository is what the lookup starts from.
    let Some(cargo) = cargo_for(item.repository.as_path(), environment) else {
        return Verdict::Failed(format!("no `{PROGRAM}` could be found to clean the copies with"));
    };
    let fill: Fill = Fill { cargo, limits: environment.limits(), kept_in: Some(item.git_dir.clone()) };
    let mut published: usize = 0;
    let mut kept: usize = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in &entries {
        match fill_one(&fill, entry).await {
            Ok(Filled::Published) => published += 1,
            Ok(Filled::Kept) => kept += 1,
            Err(reason) => failures.push(reason),
        }
    }
    debug(TAG, format!("{}: {published} published, {kept} kept, {} failed", item.repository.display(), failures.len()).as_str());
    return match failures.first() {
        None => Verdict::Done,
        Some(first_failure) => Verdict::Failed(first_failure.clone()),
    };
}

/// What:
///  `fill_one` for a control,
/// which may not be a task of the runtime.
/// Why:
///   The filling awaits a tree clone and a Cargo child,
/// so a control enters the process's runtime once and waits there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillOneInAControl = (fill, entry) => runOnRuntime(fillOne(fill, entry));
/// ```
#[cfg(test)]
pub(crate) fn fill_one_in_a_control(fill: &Fill, entry: &PrewarmEntry) -> Result<Filled, String> {
    return match crate::lazy_runtime::run_on_runtime(fill_one(fill, entry)) {
        Ok(filled) => filled,
        Err(refusal) => Err(format!("the filling could not run ({refusal})")),
    };
}

/// The filling's controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_prewarm_job_tests.rs"]
mod tests;
