//! What:
//!  The records of an interrupted ignored-state copy that cannot be read:
//!  the command that finds one moves it aside into a folder of its own,
//!  says so in one line,
//!  and goes on.
//! Why:
//!  A copy publishes every record whole by one rename,
//!  so a record that does not read is what a crash leaves
//!  when the filesystem publishes a name before its bytes
//!  (`SPEC.md`,
//!  "The records an add writes for itself").
//! Every later command reads these records first,
//! and on this machine thirty or more agents work in linked worktrees at once,
//  so one unreadable record that stops a command stops every command of that repository,
//  in every worktree,
//  until a person removes the file.
//! A record is never deleted here:
//! it is the only trace of what the interrupted copy was doing.
//! What it named is found again from the records beside it where it can be,
//  and the words of the notice say what may still stand on disk where it cannot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = await settleUnreadableRecords({ commonDir, verdicts });
//! ```

use crate::json_record::quote;
use crate::random_id::random_uuid;
use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};
use crate::worktree_add_binding::decode_record;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use crate::worktree_copy_interrupted_add_settlement::settle_interrupted_add;
use crate::worktree_copy_journal::{JournalFile, Role, assert_private_path, ensure_journal_root, ensure_private_directory, journal_owner_is_dead, journal_root, read_listed_journals};
use crate::worktree_copy_journal_encoding_and_validation::{owner_token_of, parse_journal_reading};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::worktree_copy_stage_binding::{BindingFile, STAGE_BINDING_NAME, stage_binding_reading};
use crate::worktree_copy_stage_claim::{DESTINATION_SUFFIX, claim_names, rewrite_claim};
use crate::worktree_copy_stage_claim_reaping::{ClaimFile, StageVerdicts, claim_file_reading};
use crate::worktree_copy_stage_made_object::read_made;
use crate::worktree_copy_unfilled_record::{UnfilledFile, unfilled_file_reading, unfilled_names};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  The folder under the journal root that keeps the records this release cannot read.
/// Why:
///   A record is the only trace of what an interrupted copy was doing,
/// so it is moved rather than deleted,
/// and one name keeps every one of them out of the readers' way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNREADABLE_FOLDER = 'unreadable';
/// ```
pub const UNREADABLE_FOLDER: &str = "unreadable";

/// How many names one move aside tries before it gives up on a folder that keeps colliding.
/// Exempt from a citation under `LQM` (class: retry bound):
/// each try appends a fresh random identifier, and a folder holding a thousand records of one name is not a state any
/// run produces;
/// the bound keeps a broken random source from looping forever.
const MOVE_ASIDE_TRIES: usize = 16;

/// What:
///  The folder that keeps the records this release cannot read,
/// created when it is not there.
/// Why:
///   The folder is private to this account like every other item of a copy,
/// and creating it on the first move keeps the readers off the cost of one that was never needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureUnreadableRecordsFolder(commonDir);
/// ```
fn unreadable_folder(common_dir: &Path) -> Result<PathBuf, WorktreeCopyError> {
    let root: PathBuf = ensure_journal_root(common_dir)?;
    let folder: PathBuf = root.join(UNREADABLE_FOLDER);
    ensure_private_directory(folder.as_path())?;
    return Ok(folder);
}

/// What:
///  Moves one record that cannot be read into the folder beside the records,
/// under `moved_name`,
/// and returns where it went,
/// or nothing when the record was settled by another command first.
/// Why:
///   The record is the only trace of what the interrupted copy was doing,
/// so it is renamed, never deleted,
/// and never over another record:
/// a name that is taken gets a fresh random suffix rather than replacing what an earlier command kept.
/// The rename is the whole move,
/// so two commands that find the same record settle it once and the other finds it gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await moveAside({ commonDir, record, movedName }); // undefined when the record vanished
/// ```
pub fn move_aside(common_dir: &Path, record: &Path, moved_name: &OsStr) -> Result<Option<PathBuf>, WorktreeCopyError> {
    let folder: PathBuf = unreadable_folder(common_dir)?;
    let mut target: PathBuf = folder.join(moved_name);
    for attempt in 0..MOVE_ASIDE_TRIES {
        // `match` on the one rename: a vanished record is another command's work, and a taken name keeps both.
        match rename_noreplace(record, target.as_path()) {
            Ok(()) => return Ok(Some(target)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists || (rename_unsupported(&error) && target.exists()) => {
                target = folder.join(suffixed(moved_name, attempt));
            }
            // A filesystem that cannot rename without replacing is served by one checked rename: the folder is private,
            // so no other command puts a name there between the check and the rename.
            Err(error) if rename_unsupported(&error) => {
                std::fs::rename(record, target.as_path()).map_err(|refusal| {
                    return failure_with_cause(
                        format!(
                            "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved to {}.",
                            quote(record.to_string_lossy().as_ref()),
                            quote(target.to_string_lossy().as_ref())
                        )
                        .as_str(),
                        Some(&refusal),
                    );
                })?;
                return Ok(Some(target));
            }
            Err(error) => {
                return Err(failure_with_cause(
                    format!(
                        "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved to {}.",
                        quote(record.to_string_lossy().as_ref()),
                        quote(target.to_string_lossy().as_ref())
                    )
                    .as_str(),
                    Some(&error),
                ));
            }
        }
    }
    return Err(failure_with_cause(
        format!(
            "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved aside: every name it was given in {} is taken.",
            quote(record.to_string_lossy().as_ref()),
            quote(folder.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    ));
}

/// What:
///  The name `attempt` gives a moved record:
/// its own name the first time,
/// and then its own name with a fresh identifier.
/// Why:
///   A moved record keeps the name that says what it was,
/// and a second record of the same name keeps its own bytes rather than replacing the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const suffixed = (name, attempt) => attempt === 0 ? name : `${name}.${randomUUID()}`;
/// ```
fn suffixed(moved_name: &OsStr, attempt: usize) -> PathBuf {
    if attempt == 0 {
        return PathBuf::from(moved_name);
    }
    let unique: String = random_uuid().map_or_else(|_unmade| { return format!("{attempt}"); }, |made| { return made; });
    let mut name: OsString = moved_name.to_owned();
    name.push(format!(".{unique}"));
    return PathBuf::from(name);
}

/// What:
///  The line that names one record that cannot be read,
/// where it was moved,
/// and what may still stand on disk because of it.
/// Why:
///   The person reads what the wrapper could not read,
/// that nothing was deleted,
/// and what is left where,
/// so the leftovers are settled rather than searched for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${kind} ${quote(record)} could not be read (${reason}), so it was moved to ${quote(moved)}; ${left}.`
/// ```
fn settled_notice(kind: &str, record: &Path, moved: &Path, reason: &str, left: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the {kind} {} could not be read ({reason}), so it was moved to {} and nothing was deleted. {left}",
        quote(record.to_string_lossy().as_ref()),
        quote(moved.to_string_lossy().as_ref())
    );
}

/// What:
///  What a journal that cannot be read may leave on disk.
/// Why:
///   The journal is the only record that names the stage it filled,
/// so a stage folder can stand beside the worktree with nothing left to find it;
/// and the worktree itself is named by the record of an unfilled copy where the copy got that far.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The ignored-state copy it belonged to was interrupted and nothing in any worktree was changed; the private stage folder it named (whose name begins ${STAGE_PREFIX} beside the worktree it was filling) may still stand with the ignored state it holds, nothing finds it again, and \`rm -r\` removes it once nothing needs it. A worktree the copy never finished is named again by the record of an unfilled copy where that record stands.`
/// ```
fn what_a_lost_journal_leaves() -> String {
    return format!(
        "The ignored-state copy it belonged to was interrupted, and nothing in any worktree was changed: the private stage folder it named (a folder whose name begins {STAGE_PREFIX} beside the worktree it was filling) may still stand with the ignored state it holds, nothing of the wrapper's finds it again, and `rm -r` removes it once nothing needs it. A worktree the copy never finished is named again by the record of an unfilled copy where that record stands."
    );
}

/// What:
///  What a claim that cannot be read may leave on disk when its stage folder cannot be recovered.
/// Why:
///   Nothing beside the claim names the folder any more,
/// and the destination record beside it is settled on its own,
/// so both are said rather than left to be guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The stage folder it named (a private folder whose name begins ${STAGE_PREFIX} beside the worktree the add registered) may still stand with the ignored state it holds and nothing finds it again, and the record of the destination beside the claim is named and settled by the commands that read it.`
/// ```
fn what_a_lost_claim_leaves() -> String {
    return format!(
        "The stage folder it named (a private folder whose name begins {STAGE_PREFIX} beside the worktree that add registered) may still stand with the ignored state it holds, and nothing of the wrapper's finds it again; the record of the destination beside the claim is named and settled on its own by the commands that read it."
    );
}

/// What:
///  The stage container a claim that cannot be read still names through the records beside it.
/// Why:
///   A claim is the record that names the stage folder,
/// and two records beside it name the same folder:
/// the record of the object that folder's creation made holds its path,
/// and the destination record names the worktree whose parent holds it.
/// Either one is enough to settle the folder and to let an add's records be settled,
/// and a claim with neither leaves a folder nothing finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = recoveredContainer(claim); // undefined when nothing beside it names the folder
/// ```
fn recovered_container(claim: &Path) -> Option<PathBuf> {
    let identifier: String = claim.file_stem()?.to_string_lossy().into_owned();
    let expected: String = format!("{STAGE_PREFIX}{identifier}");
    // The record of the object the stage's creation made holds the folder's own path.
    if let Some(made) = read_made(claim)
        && made.container.file_name() == Some(OsStr::new(expected.as_str()))
    {
        return Some(made.container);
    }
    // The destination record names the worktree the add registered, and the folder sits beside it.
    let destination: PathBuf = destination_record_of(claim)?;
    return Some(destination.parent()?.join(expected));
}

/// What:
///  The destination one add's record names,
/// when the record beside a claim reads.
/// Why:
///   The stage folder of an add sits in the destination's parent,
/// so the destination is what finds the folder when the record of its object is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destination = decodeRecord(read(claim.replace(/\.stage$/, '.add')))?.destination;
/// ```
fn destination_record_of(record: &Path) -> Option<PathBuf> {
    let text: String = std::fs::read_to_string(record.with_extension(&DESTINATION_SUFFIX[1..])).ok()?;
    return match decode_record(text.as_str())? {
        crate::worktree_add_binding::Recorded::Planned(destination) => Some(destination),
        crate::worktree_add_binding::Recorded::Registered(binding) => Some(binding.destination),
    };
}

/// What:
///  Settles every record of this repository that cannot be read,
/// moving each aside with one line,
/// after judging the writer that left it.
/// Why:
///   The command that meets an unreadable record is the only one that can end the state,
/// and it may only act on records whose writer is gone:
/// a live owner's record is its running add or copy.
/// Every move is one rename,
/// so of several commands that find one record exactly one settles it and the others find it gone.
/// The pass changes no worktree:
/// it settles the wrapper's own records and names what it cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await settleUnreadableRecords({ commonDir, verdicts });
/// ```
pub fn settle_unreadable_records(common_dir: &Path, verdicts: &mut StageVerdicts) -> Result<Vec<String>, WorktreeCopyError> {
    let mut notices: Vec<String> = Vec::new();
    settle_unreadable_journals(common_dir, &mut notices)?;
    settle_unreadable_claims(common_dir, verdicts, &mut notices)?;
    settle_unreadable_unfilled_records(common_dir, &mut notices)?;
    return Ok(notices);
}

/// What:
///  Moves every journal that cannot be read aside,
/// where its writer is gone,
/// and settles the registration record of its stage first.
/// Why:
///   The journal's name carries its owner,
/// so the writer is judged before anything moves;
/// and a stage binding this release cannot read is settled while the journal still names the stage,
/// so the copy an earlier command would have finished proves its destination from that destination alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const listed of await readListedJournals(commonDir)) if (listed.kind === 'unreadable' && tokenOwnerIsDead(ownerOf(listed.path), listed.path)) { await settleBinding(listed); await moveAside(listed); }
/// ```
fn settle_unreadable_journals(common_dir: &Path, notices: &mut Vec<String>) -> Result<(), WorktreeCopyError> {
    for listed in read_listed_journals(common_dir)? {
        match listed {
            JournalFile::Pending(pending) => {
                // A copy whose writer is gone is one a later command finishes; the registration it was made for is
                // settled first, so that finishing it never stops for a record nothing can read.
                if journal_owner_is_dead(&pending)? {
                    if let Some(notice) = settle_unreadable_binding(common_dir, pending.journal.stage_container.as_path())? {
                        notices.push(notice);
                    }
                }
            }
            JournalFile::Unreadable { path, reason } => {
                // A journal with no owner in its name has none to wait for, which is what an interrupted add leaves.
                if !token_owner_is_dead(owner_token_of(path.as_path()).as_deref(), path.as_path())? {
                    continue;
                }
                let Some(name) = path.file_name() else { continue };
                let Some(moved) = move_aside(common_dir, path.as_path(), name)? else { continue };
                notices.push(settled_notice("worktree-copy journal", path.as_path(), moved.as_path(), reason.as_str(), what_a_lost_journal_leaves().as_str()));
            }
        }
    }
    return Ok(());
}

/// What:
///  Moves every stage claim that cannot be read aside,
/// writes it again where its stage folder can be recovered,
/// and settles the registration record inside that folder.
/// Why:
///   A claim names the stage folder an add may still owe a copy from,
/// so the folder is found again from the records beside it where they name it;
/// writing the claim again leaves the sweeps that settle the add and make its copy with something to read,
/// and a claim that names no owner has none to wait for.
/// A claim whose folder nothing names is moved aside with the words that say what may stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await claimNames(root)) { const read = await claimFileReading(join(root, name)); if (read.kind !== 'unreadable') continue; ... }
/// ```
fn settle_unreadable_claims(common_dir: &Path, verdicts: &mut StageVerdicts, notices: &mut Vec<String>) -> Result<(), WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    for name in claim_names(root.as_path())? {
        let claim: PathBuf = root.join(name.as_str());
        let Some(ClaimFile::Unreadable { reason }) = claim_file_reading_tolerant(claim.as_path())? else { continue };
        let container: Option<PathBuf> = recovered_container(claim.as_path());
        // The registration record of the folder is the copy's own, and its writer is the claim's: settling it here keeps
        // the copy the sweeps make from stopping at a record nothing can read.
        if let Some(found) = container.as_ref()
            && let Some(notice) = settle_unreadable_binding(common_dir, found.as_path())?
        {
            notices.push(notice);
        }
        let Some(moved) = move_aside(common_dir, claim.as_path(), OsStr::new(name.as_str()))? else { continue };
        match container {
            Some(found) => {
                // `match` on the write: a claim written again leaves the sweeps something to settle; one that could not be
                // written is named with what stands instead.
                match rewrite_claim(claim.as_path(), found.as_path()) {
                    Ok(()) => {
                        // The claim names no owner, which is what an interrupted add is, so this pass settles it too.
                        verdicts.record_dead_claim(name.as_str());
                        notices.push(settled_notice(
                            "worktree-copy stage claim",
                            claim.as_path(),
                            moved.as_path(),
                            reason.as_str(),
                            format!(
                                "It was written again beside the claim name from the record of the object its stage's creation made or from the destination record beside it, naming the stage folder {}, so the sweeps that settle that folder and make the ignored-state copy the add still owes find it.",
                                quote(found.to_string_lossy().as_ref())
                            )
                            .as_str(),
                        ));
                    }
                    Err(refusal) => notices.push(settled_notice(
                        "worktree-copy stage claim",
                        claim.as_path(),
                        moved.as_path(),
                        reason.as_str(),
                        format!(
                            "It could not be written again ({}), so the stage folder {} stays and nothing of the wrapper's settles it.",
                            refusal.0,
                            quote(found.to_string_lossy().as_ref())
                        )
                        .as_str(),
                    )),
                }
            }
            None => notices.push(settled_notice("worktree-copy stage claim", claim.as_path(), moved.as_path(), reason.as_str(), what_a_lost_claim_leaves().as_str())),
        }
    }
    return Ok(());
}

/// What:
///  `claim_file_reading` with a claim that another command settled in between read as gone.
/// Why:
///   The pass acts on what a listing found,
/// and a claim taken by the add that owns it is nobody's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await claimFileReading(claim); } catch (error) { if (error.code === 'ENOENT') return undefined; throw error; }
/// ```
fn claim_file_reading_tolerant(claim: &Path) -> Result<Option<ClaimFile>, WorktreeCopyError> {
    // `match` on the metadata first: the strict reader refuses a path that is not a private file, and a vanish is not that.
    return match std::fs::symlink_metadata(claim) {
        Ok(_) => claim_file_reading(claim),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Settles the registration record inside one private stage container when it cannot be read.
/// Why:
///   The record says which registration the copy was made for,
/// and a copy that cannot read it proves its destination from that destination's own registration,
/// which is what a stage an earlier release made does.
/// The record is moved into the journal root's folder for records that cannot be read,
/// under a name that keeps the stage container it belonged to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = await settleUnreadableBinding(commonDir, container);
/// ```
fn settle_unreadable_binding(common_dir: &Path, container: &Path) -> Result<Option<String>, WorktreeCopyError> {
    // `match` on the container: a stage another command removed holds no record of its own to settle.
    match std::fs::symlink_metadata(container) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_not_a_directory) => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(container, Role::PrivateStage)?;
    let Some(BindingFile::Unreadable { reason }) = stage_binding_reading(container)? else { return Ok(None) };
    let record: PathBuf = container.join(STAGE_BINDING_NAME);
    let Some(container_name) = container.file_name() else { return Ok(None) };
    let mut moved_name: OsString = container_name.to_owned();
    moved_name.push(format!(".{STAGE_BINDING_NAME}"));
    let Some(moved) = move_aside(common_dir, record.as_path(), moved_name.as_os_str())? else { return Ok(None) };
    return Ok(Some(settled_notice(
        "registration recorded for a private stage",
        record.as_path(),
        moved.as_path(),
        reason.as_str(),
        "The copy that stage belongs to proves its destination from that destination's own registration, as a stage an earlier release made does, and nothing in any worktree was changed by this.",
    )));
}

/// What:
///  Moves every record of a worktree whose copy gave up aside,
/// where nothing waits for the copy that wrote it,
/// and names what may stand.
/// Why:
///   The record is what tells every command that a worktree holds less than its source ignores;
/// when it cannot be read, the worktree it named is not named by it any more,
/// and the note says so once.
/// The writer lives only in the record's own bytes,
/// so a record that cannot be read names none, and nothing waits for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await unfilledNames(root)) { const read = await unfilledFileReading(join(root, name)); if (read.kind !== 'unreadable') continue; ... }
/// ```
fn settle_unreadable_unfilled_records(common_dir: &Path, notices: &mut Vec<String>) -> Result<(), WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    for name in unfilled_names(root.as_path())? {
        let path: PathBuf = root.join(name.as_str());
        let Some(UnfilledFile::Unreadable { reason }) = unfilled_file_reading(path.as_path())? else { continue };
        // The worktree is still named by the records of the copy it belongs to, where they stand and read.
        let destination: Option<PathBuf> = destination_of_the_copy(&root, path.as_path());
        let Some(moved) = move_aside(common_dir, path.as_path(), OsStr::new(name.as_str()))? else { continue };
        let left: String = match destination.as_deref() {
            Some(found) => format!(
                "The worktree that copy was filling, {}, may hold less than its source's ignored state, and nothing names it as one that does; `git worktree list` shows the worktrees of this repository, and the record of an unfilled copy is written again only by a copy that runs there.",
                quote(found.to_string_lossy().as_ref())
            ),
            None => String::from(
                "The worktree that copy was filling may hold less than its source's ignored state, and no record of this repository names it any more; `git worktree list` shows the worktrees of this repository.",
            ),
        };
        notices.push(settled_notice("record of a worktree whose ignored-state copy did not finish", path.as_path(), moved.as_path(), reason.as_str(), left.as_str()));
    }
    return Ok(());
}

/// What:
///  The worktree a record of an unfinished copy was written about,
/// as the records of the same copy name it.
/// Why:
///   The record's name shares its identifier with the journal the copy wrote, or with the claim an add wrote,
/// so those records still name the worktree even when the record itself does not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destination = journalOf({ root, record })?.destinationRoot ?? destinationRecordOf(claimBeside({ root, record }));
/// ```
fn destination_of_the_copy(root: &Path, record: &Path) -> Option<PathBuf> {
    let identifier: Option<&str> = record.file_name()?.to_str()?.strip_suffix(".unfilled");
    let identifier: String = identifier?.to_string();
    let journal: PathBuf = root.join(format!("{identifier}.json"));
    if let Ok(bytes) = std::fs::read(journal.as_path())
        && let Ok(text) = String::from_utf8(bytes)
        && let Ok(found) = parse_journal_reading(text.as_str(), journal.as_path(), root)
    {
        return Some(found.destination_root);
    }
    return destination_record_of(root.join(format!("{identifier}.stage")).as_path());
}

/// What:
///  The name of the file one record is moved under,
/// which is the record's own name in every case but a stage's registration record.
/// Why:
///   A moved record keeps the name that says what it was,
/// so the person who opens the folder reads the record the notice named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const movedName = (record) => basename(record);
/// ```
pub fn moved_name_of(record: &Path) -> OsString {
    return record.file_name().map_or_else(OsString::new, OsStr::to_os_string);
}

/// What:
///  The path one moved record would be moved to,
/// which no caller needs but a control that checks the folder.
/// Why:
///   The folder's name and the record's own name are what a person looks for,
/// and a control names the same pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(journalRoot(commonDir), UNREADABLE_FOLDER, movedName)
/// ```
pub fn moved_record_path(common_dir: &Path, moved_name: &OsStr) -> PathBuf {
    return journal_root(common_dir).join(UNREADABLE_FOLDER).join(moved_name);
}

/// What:
///  Whether the folder for records that cannot be read holds anything.
/// Why:
///   A control checks what became of a record this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(journalRoot(commonDir)/UNREADABLE_FOLDER).length > 0
/// ```
pub fn unreadable_records_stand(common_dir: &Path) -> bool {
    return match std::fs::read_dir(journal_root(common_dir).join(UNREADABLE_FOLDER).as_path()) {
        Ok(mut listing) => listing.next().is_some(),
        Err(_absent) => false,
    };
}

/// What:
///  The names of the records the folder keeps,
/// in name order.
/// Why:
///   A control reads which record was moved where without depending on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(folder)).sort();
/// ```
pub fn unreadable_record_names(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let folder: PathBuf = journal_root(common_dir).join(UNREADABLE_FOLDER);
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        names.push(listed.map_err(|error| { return raw_failure(&error); })?.file_name().to_string_lossy().into_owned());
    }
    names.sort_by(crate::worktree_copy_selection::by_js_order);
    return Ok(names);
}

/// What:
///  Settles what records of an add remain when no claim names them,
/// and removes the markers of adds whose claim is gone.
/// Why:
///   A record of an add is read through the claim beside it,
/// so a claim a command moved aside leaves the record unread;
/// the worktree it names must still be named,
/// which is what its own settlement does,
/// and the record is kept while something stands that it describes.
/// A marker whose claim is gone says nothing about anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await settleRecordsWithoutAClaim(root);
/// ```
pub fn settle_records_without_a_claim(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut records: Vec<PathBuf> = Vec::new();
    let mut markers: Vec<PathBuf> = Vec::new();
    for listed in listing {
        let path: PathBuf = listed.map_err(|error| { return raw_failure(&error); })?.path();
        let name: String = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        // A claim's own files end in `.stage`; everything a claim names keeps that name and changes only its suffix.
        let identifier: &str = match name.strip_suffix(DESTINATION_SUFFIX) {
            Some(stem) => stem,
            None => match name.strip_suffix(crate::worktree_copy_stage_claim::COMPLETE_SUFFIX) {
                Some(_stem) => {
                    markers.push(path);
                    continue;
                }
                None => continue,
            },
        };
        if !root.join(format!("{identifier}.stage")).exists() {
            records.push(path);
        }
    }
    markers.sort_by(|left, right| { return crate::worktree_copy_selection::js_order(&left.to_string_lossy(), &right.to_string_lossy()); });
    records.sort_by(|left, right| { return crate::worktree_copy_selection::js_order(&left.to_string_lossy(), &right.to_string_lossy()); });
    let mut notices: Vec<String> = Vec::new();
    for record in records {
        let settled = settle_interrupted_add(record.as_path());
        if let Some(notice) = settled.notice {
            notices.push(notice);
        }
        if settled.keep {
            continue;
        }
        remove_file_if_present(record.as_path())?;
        remove_file_if_present(record.with_extension(&crate::worktree_copy_stage_claim::COMPLETE_SUFFIX[1..]).as_path())?;
    }
    for marker in markers {
        // The marker says an add finished; with no record beside it there is no add it could be about.
        remove_file_if_present(marker.as_path())?;
    }
    return Ok(notices);
}

/// Unreadable-record controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_unreadable_records_tests.rs"]
mod tests;
