//! What:
//!  The ignored-state copy an interrupted `git worktree add` still owes,
//! made by the next command that finds it.
//! Why:
//!  An add records the destination,
//! the source and the selection before real Git runs,
//! because the copy that fills the new worktree can be cut off at any instruction:
//! the stage is still being filled,
//! the stage is whole and no journal names it,
//! or the installation never ran.
//! Left alone, the worktree stays unfilled and the person must remove it and add it again,
//! losing their own work in it;
//! so the next command stages from the recorded source and installs under the journal,
//! when the destination is still the registration the add made and its checkout is whole.
//! Nothing is overwritten:
//! the installation accepts an exact entry and refuses a differing one,
//! so a worktree that gained anything since the add is named rather than written into.
//! A copy that cannot be made becomes the durable record of an unfilled worktree,
//! which every later command names,
//! rather than being staged again by each one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = await finishOwedWorktreeCopies({ commonDir, verdicts, context, stderr });
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{Binding, OwedCopy, OwedSelection, Recorded, bind_registration, decode_record, owed_copy_of, path_identity};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_interrupted_add_settlement::{add_is_finished, registration_directory};
use crate::worktree_copy_journal::{Role, assert_private_path, journal_root, read_pending_journals};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_journal_removal::remove_stage;
use crate::worktree_copy_registry::{WorktreeCopyObservation, read_registered_roots};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_source_still_being_filled::source_is_still_being_filled;
use crate::worktree_copy_stage_binding::stage_binding;
use crate::worktree_copy_stage_claim::{DESTINATION_SUFFIX, StageClaim, claim_names};
use crate::worktree_copy_stage_claim_reaping::{SCRATCH_SUFFIX, StageVerdicts, read_claim};
use crate::worktree_copy_synchronize::synchronize_created_worktree_noting;
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner, token_owner_is_dead};
use crate::worktree_copy_unfilled_record::remember_unfilled_with;
use crate::json_record::quote;
use crate::shell_quote::shell_quote;
use std::io::Write;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// What:
///  The selection an add's record names,
/// as the copy takes it.
/// Why:
///   The record writes the two listings a source can have,
/// and a reader takes only those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectionOf = (owed) => owed.selection === 'main-checkout' ? CopySelection.MainCheckout : CopySelection.Ignored;
/// ```
fn selection_of(owed: &OwedCopy) -> CopySelection {
    return match owed.selection {
        OwedSelection::Ignored => CopySelection::Ignored,
        OwedSelection::MainCheckout => CopySelection::MainCheckout,
    };
}

/// What:
///  Which of an add's ignored-state copies a selection is,
/// or nothing when the selection names paths a record cannot hold.
/// Why:
///   The record has room for the two listings a source can have,
/// so a command whose copy takes named paths records no
///  copy and its worktree is named instead of filled from the wrong set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = owedSelectionOf(selection); // 'ignored' | 'main-checkout' | undefined
/// ```
pub fn owed_selection_of(selection: &CopySelection) -> Option<OwedSelection> {
    return match selection {
        CopySelection::Ignored => Some(OwedSelection::Ignored),
        CopySelection::MainCheckout => Some(OwedSelection::MainCheckout),
        CopySelection::Paths(_) => None,
    };
}

/// What:
///  The line that says the copy an interrupted add owed was made,
/// and how much it copied.
/// Why:
///   The person reads one line naming the worktree,
/// the source and the count,
/// and knows nothing is left to do there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `finished the ignored-state copy an interrupted git worktree add owed ${destination}: copied ${copied} entries from ${source}.`
/// ```
fn finished_notice(destination: &Path, source: &Path, copied: usize) -> String {
    let entry_noun: &str = if copied == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: finished the ignored-state copy an interrupted `git worktree add` owed {}: copied {copied} ignored filesystem {entry_noun} from {}.",
        quote(destination.to_string_lossy().as_ref()),
        quote(source.to_string_lossy().as_ref())
    );
}

/// What:
///  The line that says the copy could not be made,
/// names the worktree,
/// what failed,
/// and the two commands that settle it.
/// Why:
///   The same words come again from the record of the unfilled worktree on every later command,
/// so they must say everything the person needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the ignored-state copy an interrupted git worktree add owed ${destination} could not be made (${reason}); ...`
/// ```
fn not_finished_notice(destination: &Path, reason: &str) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let commands: String = shell_quote(destination.as_os_str().as_bytes());
    return format!(
        "monochromatic-git-policy-cli: the ignored-state copy an interrupted `git worktree add` owed {shown} could not be made ({reason}), and nothing in that worktree was changed. `git worktree remove --force {commands}` removes that worktree, and running the same `git worktree add` line again copies the ignored state; this line comes again from every command in this repository until that worktree is removed or replaced."
    );
}

/// What:
///  The marker one command writes in the journal root before it stages a copy,
/// and removes when it is done with that destination.
/// Why:
///   Two commands that find the same dead add's claim would both stage into one name and one of them would remove the
/// other's live stage;
/// the marker is one exclusive create whose name carries its owner,
/// so exactly one command copies and a marker
///  whose owner died is taken over rather than left waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using taking = await takeTheCopy(root, claim); // undefined when another command holds it
/// ```
struct Taking {
    /// The marker file this command created.
    path: PathBuf,
}

impl Drop for Taking {
    /// Removes the marker however this command ends,
    /// so a copy that fails leaves no claim that another command holds it.
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
    }
}

/// The suffix of that marker.
const TAKING_SUFFIX: &str = ".taking";

/// What:
///  The marks one add's identifier has been given,
/// in name order.
/// Why:
///   A command that finds a dead owner's marker removes it and takes the copy;
/// one that finds a live owner's leaves the copy to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marks = takingMarksOf(root, id);
/// ```
fn taking_marks_of(root: &Path, identifier: &str) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    let prefix: String = format!("{identifier}.");
    let mut marks: Vec<PathBuf> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed.map_err(crate::worktree_copy_error::raw_failure_of)?.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix.as_str()) && name.ends_with(TAKING_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| { return crate::worktree_copy_selection::js_order(left.as_str(), right.as_str()); });
    for name in names {
        marks.push(root.join(name.as_str()));
    }
    return Ok(marks);
}

/// What:
///  Takes the copy of one add's destination,
/// or answers that another command holds it.
/// Why:
///   The take is one exclusive create whose name carries this command's owner,
/// so of two commands that find the same dead
///  claim exactly one copies;
/// a marker whose owner died is taken over,
/// because nothing of that command runs and
///  the worktree it would have filled is still unfilled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taking = await takeTheCopy(root, claim);
/// ```
fn take_the_copy(root: &Path, claim: &Path) -> Result<Option<Taking>, WorktreeCopyError> {
    let Some(identifier) = claim.file_stem().map(|stem| { return stem.to_string_lossy().into_owned(); }) else {
        return Ok(None);
    };
    let owner: CopyOwner = current_copy_owner()?;
    // A marker whose owner is proven dead holds nothing, so it is removed; one whose owner lives keeps the copy.
    let mut held: bool = false;
    for mark in taking_marks_of(root, identifier.as_str())? {
        let token: Option<String> = std::fs::read_to_string(mark.as_path()).ok().map(|text| { return text.trim().to_string(); });
        if !token_owner_is_dead(token.as_deref(), mark.as_path())? {
            held = true;
            break;
        }
        // The name carries the owner, and the file is a marker of this namespace alone.
        let _removed: std::io::Result<()> = std::fs::remove_file(mark.as_path());
    }
    if held {
        return Ok(None);
    }
    let path: PathBuf = root.join(format!("{identifier}.{}{TAKING_SUFFIX}", owner.token()));
    // `match` on the exclusive create: the one command that made the file copies, and the rest read it as another's, so no
    // second attempt is needed, not even when two commands removed the same dead marker.
    return match std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_path()) {
        Ok(mut written) => {
            use std::io::Write as _;
            written.write_all(owner.token().as_bytes()).map_err(crate::worktree_copy_error::raw_failure_of)?;
            written.sync_all().map_err(crate::worktree_copy_error::raw_failure_of)?;
            Ok(Some(Taking { path }))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(error) => Err(crate::worktree_copy_error::raw_failure(&error)),
    };
}

/// What:
///  One add's unfinished state,
/// as the claim and its two records say it.
/// Why:
///   Every check below reads the same fields,
/// so they are parsed once and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedAdd = { claim: string; container: string; record: string; destination: string; owed: OwedCopy };
/// ```
struct OwedAdd {
    /// The claim file,
    /// whose stage name the copy fills.
    claim: PathBuf,
    /// The stage container the claim names.
    container: PathBuf,
    /// The record of the destination written beside the claim.
    record: PathBuf,
    /// The worktree the add registered.
    destination: PathBuf,
    /// The identity the record holds for that worktree,
    /// when it was bound before the command died.
    destination_identity: Option<crate::fs_beneath::ObjectId>,
    /// The ignored-state copy the add owes.
    owed: OwedCopy,
}

/// What:
///  The add's unfinished state,
/// or nothing when the claim is not one an add made or its record names no copy.
/// Why:
///   A copy's own claim records no destination,
/// and a record written before the source was written names none:
/// both settle exactly as they did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedAdd = owedAddOf(claimPath, container);
/// ```
fn owed_add_of(claim: &Path, container: &Path) -> Option<OwedAdd> {
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    let text: String = std::fs::read_to_string(record.as_path()).ok()?;
    let (destination, destination_identity): (PathBuf, Option<crate::fs_beneath::ObjectId>) = match decode_record(text.as_str())? {
        Recorded::Planned(named) => (named, None),
        Recorded::Registered(binding) => (binding.destination, Some(binding.destination_id)),
    };
    let owed: OwedCopy = owed_copy_of(text.as_str())?;
    return Some(OwedAdd { claim: claim.to_path_buf(), container: container.to_path_buf(), record, destination, destination_identity, owed });
}

/// What:
///  The registration of a destination,
/// when its checkout is whole and it is still a worktree of this repository.
/// Why:
///   The copy fills a worktree;
/// a destination that is gone,
/// whose checkout is not whole,
/// or whose pointer no longer names a registration here has nothing this pass may write into,
/// and the
///  settlement names it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = wholeRegistration(commonDir, destination); // undefined when nothing may be filled
/// ```
fn whole_registration(common_dir: &Path, destination: &Path) -> Option<Binding> {
    let bound: Binding = bind_registration(common_dir, destination).ok()?.binding;
    // `match` on the pointer: the registration it names now is the one whose index the proof reads, and a destination whose
    // pointer cannot be read is proven against the registration this command bound.
    let admin: PathBuf = match registration_directory(destination) {
        Some(found) => found,
        None => bound.admin.clone(),
    };
    if !add_is_finished(destination, admin.as_path()) {
        return None;
    }
    return Some(bound);
}

/// What:
///  Whether the source a record names is a worktree of this repository now.
/// Why:
///   Staging reads the source's ignored files,
/// so a source that was removed,
/// moved or replaced is another worktree and never one this copy may take state from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = roots.some((root) => canonical(root) === canonical(source));
/// ```
fn source_is_registered(roots: &[PathBuf], source: &Path) -> bool {
    let wanted: PathBuf = match std::fs::canonicalize(source) {
        Ok(canonical) => canonical,
        Err(_gone) => return false,
    };
    // Git writes the roots as it sees them, so both sides are compared in their canonical form: a symbolic link above a
    // worktree must not hide a source the repository still registers.
    return roots.iter().any(|root| { return std::fs::canonicalize(root).is_ok_and(|found| { return found.as_path() == wanted.as_path(); }); });
}

/// What:
///  Removes the stage this copy itself left,
/// or says that nothing proves it is the copy's.
/// Why:
///   A stage killed while it was being filled holds part of the source's ignored state,
/// and the copy cannot fill that name again without removing it first;
/// only a container that records the registration this copy was made for proves the stage is the
///  copy's own, so a directory another process put at that name is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheOwnStage(container, destination); // false when the container is not the copy's own
/// ```
fn clear_the_own_stage(container: &Path, destination: &Path) -> Result<bool, WorktreeCopyError> {
    let present: bool = match std::fs::symlink_metadata(container) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    if !present {
        return Ok(true);
    }
    // A copy writes its binding into the container before its first payload entry, so a container that holds nothing but
    // the parts of that record is a copy killed while recording it; nothing of the source's state is there to lose, and
    // only the claim's random identifier names that container.
    let own: bool = match stage_binding(container)? {
        Some(binding) => binding.destination == destination,
        None => holds_only_a_binding(container),
    };
    if !own {
        return Ok(false);
    }
    assert_private_path(container, Role::PrivateStage)?;
    remove_stage(container)?;
    return Ok(true);
}

/// What:
///  Whether a container holds nothing but the stage binding a copy writes before its first payload entry,
/// or a part of the write that publishes it.
/// Why:
///   The write of that record publishes it by a rename,
/// so a kill leaves either the record's own temporary or the
///  file that temporary was being written in,
/// and no binding;
/// such a container holds no ignored file of the source,
/// while one that holds a payload or a
///  snapshot holds data and is never removed.
/// Only the record's own names count,
/// so an ignored file of the source that merely begins with them keeps the
///  container,
/// and a container holding no entry at all proves nothing either:
/// another program can make an empty directory at that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = readdir(container);
/// const bare = named.length > 0 && named.every((name) => namesTheBindingWrite(name));
/// ```
fn holds_only_a_binding(container: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(container) else { return false };
    let mut named: bool = false;
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !names_the_binding_write(name.as_str()) {
            return false;
        }
        named = true;
    }
    return named;
}

/// What:
///  Whether one entry of a stage container is the binding record,
/// the temporary that publishes it,
/// or the file that temporary is written in.
/// Why:
///   Those three names are the only ones a kill between the container's creation and its first payload entry leaves,
/// so anything else is the source's ignored state or a directory another program put there,
/// and it keeps the container.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = (name) => name === RECORD || /^\.?RECORD\..*\.(tmp|partial)$/.test(name);
/// ```
fn names_the_binding_write(name: &str) -> bool {
    // The file a write is made in is hidden by a leading dot (`private_storage::staging_beside`), the temporary it is
    // published as is not.
    let bare: &str = name.strip_prefix('.').unwrap_or(name);
    let Some(rest): Option<&str> = bare.strip_prefix(crate::worktree_copy_stage_binding::STAGE_BINDING_NAME) else {
        return false;
    };
    if rest.is_empty() {
        return true;
    }
    return rest.starts_with('.') && (rest.ends_with(TEMPORARY_SUFFIX) || rest.ends_with(SCRATCH_SUFFIX));
}

/// What:
///  Makes the copy one interrupted add owed,
/// or reports why it was not made.
/// Why:
///   The copy is the same staging,
/// journaling and installation a creating command makes,
/// so its never-overwrite rule,
/// its records and its resumption are the proven ones;
/// this pass adds the
///  decision that the copy is still owed and that its destination is unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = finishOneOwedCopy({ commonDir, add, journaled, context, stderr });
/// ```
fn finish_one_owed_copy(
    common_dir: &Path,
    add: &OwedAdd,
    journaled: bool,
    context: &GitContext,
    stderr: &mut dyn Write,
) -> Result<Option<String>, WorktreeCopyError> {
    // A journal names the stage: the installation's own recovery resumes it, and this pass must not touch the stage.
    if journaled {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("{} is journaled", add.destination.display()).as_str());
        return Ok(None);
    }
    let Some(binding) = whole_registration(common_dir, add.destination.as_path()) else {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("{} is not a whole registration", add.destination.display()).as_str());
        return Ok(None);
    };
    // A worktree removed and made again at the same path is another object than the add's, so nothing is copied into the
    // one that stands there now; a record written before the add was bound names no identity and is filled as before.
    if let Some(recorded) = add.destination_identity
        && path_identity(add.destination.as_path()) != Some(recorded)
    {
        return Ok(None);
    }
    // The listing runs where the source is, so a command's own working directory never decides what the repository
    // registers.
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        common_dir: common_dir.to_path_buf(),
        effective_cwd: add.owed.source.clone(),
        source_root: Some(add.owed.source.clone()),
        selection: selection_of(&add.owed),
    };
    let roots: Vec<PathBuf> = match read_registered_roots(context, &observation) {
        Ok(found) => found,
        // A listing that cannot be read proves nothing about the source, so the records stay and the next command asks again.
        Err(_unreadable) => {
            crate::diagnostic_log::debug("worktreeCopyOwed", "the registered roots could not be read");
            return Ok(None);
        }
    };
    if !source_is_registered(roots.as_slice(), add.owed.source.as_path()) {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the source {} is not registered", add.owed.source.display()).as_str());
        let reason: String = format!(
            "the worktree it copies from, {}, is no longer a worktree of this repository",
            quote(add.owed.source.to_string_lossy().as_ref())
        );
        return settle_unfilled(add, reason.as_str());
    }
    // A source that is itself being filled holds part of its ignored state; the record stays and a later command asks again.
    if source_is_still_being_filled(common_dir, add.owed.source.as_path())?.is_some() {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the source {} is still being filled", add.owed.source.display()).as_str());
        return Ok(None);
    }
    // One command copies into one destination: a marker whose owner lives keeps this one out, and one whose owner died is
    // taken over.
    let taking: Option<Taking> = take_the_copy(&journal_root(common_dir), add.claim.as_path())?;
    if taking.is_none() {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("another command holds the copy of {}", add.destination.display()).as_str());
        return Ok(None);
    }
    // A stage nothing proves is the copy's own is named by the settlement, which keeps it and its claim.
    if !clear_the_own_stage(add.container.as_path(), add.destination.as_path())? {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the stage {} is not provably this copy's own", add.container.display()).as_str());
        return Ok(None);
    }
    let Some(claim) = StageClaim::adopted(add.claim.as_path(), add.container.as_path(), Some(add.record.as_path())) else {
        return Ok(None);
    };
    return match synchronize_created_worktree_noting(
        context,
        common_dir,
        add.owed.source.as_path(),
        add.destination.as_path(),
        roots.as_slice(),
        &selection_of(&add.owed),
        Some(claim),
        Some(&binding),
        stderr,
    ) {
        Ok(copied) => Ok(Some(finished_notice(add.destination.as_path(), add.owed.source.as_path(), copied))),
        Err(failure) => settle_unfilled(add, failure.0.as_str()),
    };
}

/// What:
///  Turns an add whose copy cannot be made into the durable record of an unfilled worktree,
/// and settles its claim.
/// Why:
///   Every later command names a worktree whose record says files are missing,
/// and none of them stages the source again trying what just failed;
/// the person settles it by removing the worktree or replacing it.
/// The record's own line is written by this same pass,
/// so the failure is said once there;
/// only a destination whose identity cannot be read,
/// and which therefore gets no record,
/// is named by this function's own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = await settleUnfilled(add, reason); // undefined when the record says it
/// ```
fn settle_unfilled(add: &OwedAdd, reason: &str) -> Result<Option<String>, WorktreeCopyError> {
    let identity: Option<crate::fs_beneath::ObjectId> = path_identity(add.destination.as_path());
    if let Some(settle) = StageClaim::adopted(add.claim.as_path(), add.container.as_path(), Some(add.record.as_path())) {
        settle.settle()?;
    }
    let Some(found) = identity else {
        return Ok(Some(not_finished_notice(add.destination.as_path(), reason)));
    };
    remember_unfilled_with(
        add.claim.as_path(),
        add.destination.as_path(),
        found,
        add.owed.source.as_path(),
        None,
        reason,
    )?;
    return Ok(None);
}

/// What:
///  Removes the taking markers of commands that no longer run.
/// Why:
///   A marker holds the copy of one destination back for as long as its owner lives,
/// and a command killed after making one
///  leaves it behind;
/// removing it loses nothing,
/// because the marker says only which command was copying,
/// and its
///  name carries the owner that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeDeadTakingMarks(root);
/// ```
fn remove_dead_taking_marks(root: &Path) -> Result<(), WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    for listed in listing {
        let path: PathBuf = listed.map_err(crate::worktree_copy_error::raw_failure_of)?.path();
        let name: String = path.file_name().map(|found| { return found.to_string_lossy().into_owned(); }).unwrap_or_default();
        if !name.ends_with(TAKING_SUFFIX) {
            continue;
        }
        // The name is `<identifier>.<owner>.taking`, and the owner field is the one before the suffix.
        let token: Option<String> = name.strip_suffix(TAKING_SUFFIX).and_then(|stem| { return stem.rsplit_once('.').map(|(_item, owner)| { return owner.to_string(); }); });
        if token_owner_is_dead(token.as_deref(), path.as_path())? {
            let _removed: std::io::Result<()> = std::fs::remove_file(path.as_path());
        }
    }
    return Ok(());
}

/// What:
///  Makes every copy an interrupted add still owes,
/// on the dead owners the verdicts named.
/// Why:
///   A live owner's add is running and its copy with it,
/// so only a dead owner's records are finished here;
/// the judgement is the sweep's own,
/// taken before anything is changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await finishOwedWorktreeCopies({ commonDir, verdicts, context, stderr });
/// ```
pub fn finish_owed_worktree_copies(
    common_dir: &Path,
    verdicts: &StageVerdicts,
    context: &GitContext,
    stderr: &mut dyn Write,
) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    remove_dead_taking_marks(root.as_path())?;
    let names: Vec<String> = claim_names(root.as_path())?;
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut journaled: Vec<PathBuf> = Vec::new();
    for pending in read_pending_journals(common_dir)? {
        journaled.push(pending.journal.stage_container);
    }
    let mut notices: Vec<String> = Vec::new();
    for name in names {
        if !verdicts.names_dead_claim(name.as_str()) {
            continue;
        }
        let claim: PathBuf = root.join(name.as_str());
        let Some((container, _owner)) = read_claim(claim.as_path())? else {
            continue;
        };
        let Some(add) = owed_add_of(claim.as_path(), container.as_path()) else {
            continue;
        };
        let journaled_here: bool = journaled.iter().any(|named| { return named.as_path() == add.container.as_path(); });
        if let Some(notice) = finish_one_owed_copy(common_dir, &add, journaled_here, context, stderr)? {
            notices.push(notice);
        }
    }
    return Ok(notices);
}

/// Owed-copy controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_owed_copy_tests.rs"]
mod tests;
