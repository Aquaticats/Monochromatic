//! What:
//!  The judging of stage claims and their temporaries,
//! and the removal of the stages that interrupted copies left behind.
//! Why:
//!  A claim is the record that a stage may exist.
//! A claim whose owner is dead and whose stage no journal adopted is reaped;
//! a
//!  stage a copy left is a private directory this wrapper made,
//! while a stage an add left is removed only when the record of
//!  the object its creation made proves that object still stands there and holds nothing the record cannot account for
//!  (`worktree_copy_stage_made_object.rs`,
//! `worktree_copy_stage_entries_the_record_accounts_for.rs`);
//! a stage that record does
//!  not prove is reported and never removed,
//! and the claim stays so that the next command reports it again.
//! What an interrupted `git worktree add` settles to lives beside this file,
//! and is named here so one import path reaches both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await judgeStageOwners(commonDir);
//! const notices = await reapJudgedStages(commonDir, verdicts);
//! ```

use crate::json_record::quote;
use crate::path_bytes_in_records::path_of_the_two_fields;
use crate::private_storage::sync_directory;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_recovery::{RecoveryDestinations, finishes_destination};
use crate::worktree_copy_journal::{Role, assert_private_path, journal_root, read_pending_journals, temporary_writer_token};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_journal_removal::{remove_file_if_present, remove_stage};
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::{STAGE_PREFIX, js_order};
use crate::worktree_copy_stage_claim::{COMPLETE_SUFFIX, CONTAINER_HEX_PREFIX, DESTINATION_SUFFIX, OWNER_LINE_PREFIX, claim_names};
use crate::worktree_copy_stage_made_object::remove_made;
use crate::worktree_copy_stage_reaping_proof_and_words::{Provenance, kept_stage_words, provenance_of, removed_copy_stage_words, removed_stage_words, still_there};
// The sweeps that remove files rather than stages live beside this file, so this one stays under the line limit.
pub use crate::worktree_copy_stage_claim_swept_files::remove_dead_temporaries;
pub(crate) use crate::worktree_copy_stage_claim_swept_files::remove_records_nothing_claims;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What an interrupted `git worktree add` settles to,
/// the proof its checkout is whole or gone,
/// and the names of the files that proof reads:
/// re-exported so a reader of the claims and a control of them reach them here.
/// The settlement lives in a file of its own (`worktree_copy_interrupted_add_settlement.rs`),
/// which is where this
///  branch's proof of the destination's pointer now is too.
pub use crate::worktree_copy_interrupted_add_settlement::*;

/// What:
///  The container and the owner a claim names.
/// Why:
///   `read_claim` acts on these values,
/// so the shape of a record is decided in one place.
struct ClaimFields<'a> {
    /// The container as text,
    /// when the record spells it on one line.
    text: Option<&'a str>,
    /// The container's bytes,
    /// when the record holds them.
    bytes: Option<&'a str>,
    /// The owner token,
    /// when the record names one.
    owner: Option<String>,
}

/// What:
///  Whether a line of a claim holds the container's bytes.
/// Why:
///   A reader must place the bytes line and count it,
/// so both questions are asked in one way of one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsBytes = (line: string) => line.startsWith('containerHex=')
/// ```
fn holds_bytes(line: &str) -> bool {
    return line.starts_with(CONTAINER_HEX_PREFIX);
}

/// What:
///  The lines of a claim as this release and the one before it write them:
/// the container as text,
/// its bytes on the line after it,
/// then the owner.
/// Why:
///   The container is removed recursively,
/// so a line of bytes is believed only in the two places a writer puts one,
/// and a claim with any other line is refused:
/// the earlier release wrote a container path verbatim,
/// so a path holding a line break put lines of its own in the record,
/// and one of them could name any private directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimFields = (text: string) => { const lines = text.split('\n'); const textLine = lines[0]?.startsWith('containerHex=') ? undefined : lines[0]; ... }
/// ```
fn claim_fields(text: &str) -> Result<ClaimFields<'_>, String> {
    let lines: Vec<&str> = text.lines().collect();
    // The first line is the container's bytes or the container's text; its bytes then sit where this release puts them.
    let named_text: Option<&str> = match lines.first() {
        Some(line) if holds_bytes(line) => None,
        first => first.copied(),
    };
    let named_bytes: Option<&str> = match lines.get(usize::from(named_text.is_some())) {
        Some(line) => line.strip_prefix(CONTAINER_HEX_PREFIX),
        None => None,
    };
    let read: usize = usize::from(named_text.is_some()) + usize::from(named_bytes.is_some());
    // A line of container bytes past those is a path that ended early, not a line a writer put there.
    if lines.iter().skip(read).copied().any(holds_bytes) {
        return Err(String::from("its line of container bytes is not where this release writes one"));
    }
    // The owner, when the record names one, is the last line.
    let mut following = lines.iter().skip(read);
    let Some(first) = following.next() else {
        return Ok(ClaimFields { text: named_text, bytes: named_bytes, owner: None });
    };
    let Some(token) = first.strip_prefix(OWNER_LINE_PREFIX) else {
        return Err(String::from("a line this release does not write follows its container"));
    };
    if following.next().is_some() {
        return Err(String::from("more lines follow its owner"));
    }
    return Ok(ClaimFields { text: named_text, bytes: named_bytes, owner: Some(token.to_string()) });
}

/// What:
///  The refusal of a claim whose lines are not a record this release writes.
/// Why:
///   Recovery never acts on a record it cannot trust:
/// the claim stays for the person to check,
/// and nothing the record names is removed or reported as removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`stage claim ${claim} is not a record this release writes (${reason})`)
/// ```
fn claim_refusal(claim: &Path, reason: &str) -> WorktreeCopyError {
    return failure_with_cause(
        format!(
            "monochromatic-git-policy-cli: worktree-copy stage claim {} is not a record this release writes ({reason}); nothing it names was changed, so remove the file after checking it.",
            quote(claim.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    );
}

/// What:
///  One claim file as read:
/// the stage container it names,
/// refused unless it is a monochromatic-git-policy-cli stage container name,
/// and the owner token.
/// Why:
///   The container is deleted recursively,
/// so a claim that names anything else is never acted on;
/// the owner decides whether
///  the claim may be reaped at all.
/// A claim that vanished while it was read was removed by its owner or by another command.
/// A claim whose lines are not a record this release writes is refused and kept,
/// because the earlier release wrote a container path verbatim,
/// and a path holding a line break then spells lines of the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { text, bytes, owner } = claimFields(await readFile(claimPath, 'utf8')); const container = bytes ?? text; assertStageContainerName(container);
/// ```
pub fn read_claim(claim: &Path) -> Result<Option<(PathBuf, Option<String>)>, WorktreeCopyError> {
    // `match` on the lookup: a claim already gone is nobody's fault, and nothing is left to reap.
    match std::fs::symlink_metadata(claim) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(claim, Role::JournalFile)?;
    // `match` on the read: the claim can vanish between the lookup and the read.
    let text: String = match std::fs::read_to_string(claim) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read worktree-copy stage claim {}.",
                quote(claim.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let fields: ClaimFields<'_> = match claim_fields(text.as_str()) {
        Ok(fields) => fields,
        Err(reason) => return Err(claim_refusal(claim, reason.as_str())),
    };
    let container: PathBuf = match path_of_the_two_fields(fields.bytes, fields.text) {
        Ok(Some(path)) => path,
        Ok(None) => PathBuf::new(),
        Err(reason) => {
            return Err(failure_with_cause(
                format!(
                    "monochromatic-git-policy-cli: worktree-copy stage claim {} holds a container in bytes that is not a path ({reason}); remove the file after checking it.",
                    quote(claim.to_string_lossy().as_ref())
                )
                .as_str(),
                None,
            ));
        }
    };
    // Both lines of a claim this release writes name one path; a line of text that says otherwise is a shorter path
    // that ended where the line break was, and the bytes it stood beside are not the record's own.
    if let (Some(named), Some(_)) = (fields.text, fields.bytes)
        && container.as_path() != Path::new(named)
    {
        return Err(claim_refusal(claim, "its line of text and its line of container bytes name different paths"));
    }
    let stage_named: bool = match container.file_name() {
        Some(name) => name.to_string_lossy().starts_with(STAGE_PREFIX),
        None => false,
    };
    let named: bool = container.is_absolute() && stage_named;
    if !named {
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: worktree-copy stage claim {} does not name a stage container; remove the file after checking it.",
                quote(claim.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(Some((container, fields.owner)));
}


/// Which owners were found dead,
/// decided before anything is removed.
#[derive(Debug, Default)]
pub struct StageVerdicts {
    /// The names of the claims whose owner is dead.
    dead_claims: BTreeSet<String>,
    /// The names of the temporaries whose writer is dead.
    dead_temporaries: BTreeSet<String>,
}

impl StageVerdicts {
    /// What:
    ///  Whether the claim of this name has a dead owner.
    /// Why:
    ///   The pass that finishes an owed copy and the sweep that settles the claim act on one judgement,
    /// taken before
    ///  anything is changed,
    /// so neither acts on a claim whose owner still runs or cannot be judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.namesDeadClaim(name)
    /// ```
    pub fn names_dead_claim(&self, name: &str) -> bool {
        return self.dead_claims.contains(name);
    }

    /// What:
    ///  The names of the temporaries whose writer is dead.
    /// Why:
    ///   The sweep that removes them lives beside this file,
    /// and a dead writer's names are the one thing it needs from here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.deadTemporaryNames() // the names of a dead writer's temporaries
    /// ```
    pub(crate) fn dead_temporary_names(&self) -> &BTreeSet<String> {
        return &self.dead_temporaries;
    }
}

/// The suffix of the file one write is made in before its own temporary takes its name:
/// a write cut off inside it leaves this rather than a temporary the sweep already knows.
pub(crate) const SCRATCH_SUFFIX: &str = ".partial";

/// What:
///  The writer's owner token in the name of the file one write was making,
/// or nothing for a name of another shape.
/// Why:
///   A write makes `<temporary>.<random>.partial` first,
/// so the temporary's own name,
/// with the writer in it,
/// is what the sweep judges before removing it;
/// a name of another shape names no writer and counts as a dead one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = scratchWriterToken(name); // stem before the last dot, parsed as a temporary name
/// ```
fn scratch_writer_token(name: &str) -> Option<String> {
    let stem: &str = name.strip_suffix(SCRATCH_SUFFIX)?;
    let (temporary, _unique) = stem.rsplit_once('.')?;
    return temporary_writer_token(temporary).map(str::to_string);
}

/// What:
///  Order two listed temporaries by name the way JavaScript's default sort does.
/// Why:
///   Which temporary is judged first,
/// and so which refusal is reported first,
/// must not depend on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaries.sort(([left], [right]) => compareJs(left, right))
/// ```
fn by_temporary_name(left: &(String, PathBuf), right: &(String, PathBuf)) -> std::cmp::Ordering {
    return js_order(&left.0, &right.0);
}

/// What:
///  Judges the owner of every stage claim and every temporary in the journal root,
/// and changes nothing.
/// Why:
///   A command that refuses an item of another namespace older than the patience must refuse before it changes anything,
/// so every
///  owner is judged first,
/// once (sol-1 round 3,
/// P1 C),
/// and the sweep acts on the verdicts without judging again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdicts = await judgeStageOwners(commonDir); // throws on an expired foreign item, before any removal
/// ```
pub fn judge_stage_owners(common_dir: &Path) -> Result<StageVerdicts, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let mut verdicts: StageVerdicts = StageVerdicts::default();
    for name in claim_names(root.as_path())? {
        let claim: PathBuf = root.join(name.as_str());
        let Some((_container, owner)) = read_claim(claim.as_path())? else {
            continue;
        };
        // A live owner's claim is its running add or copy; a claim whose owner cannot be judged is kept, too.
        if token_owner_is_dead(owner.as_deref(), claim.as_path())? {
            verdicts.dead_claims.insert(name);
        }
    }
    // `match` treats an absent root as having nothing to remove.
    let listing = match std::fs::read_dir(root.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(verdicts),
        Err(error) => return Err(raw_failure(&error)),
    };
    // In name order, so which temporary is judged first (and which refusal is reported first) does not depend on the directory's order.
    let mut temporaries: Vec<(String, PathBuf)> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(TEMPORARY_SUFFIX) || name.ends_with(SCRATCH_SUFFIX) {
            temporaries.push((name, entry.path()));
        }
    }
    temporaries.sort_by(by_temporary_name);
    for (name, path) in temporaries {
        // The name is `<item>.<writer>.<unique>.tmp`; a name of another shape has no writer and counts as a dead one's.
        let writer: Option<String> = match scratch_writer_token(name.as_str()) {
            Some(token) => Some(token),
            None => temporary_writer_token(name.as_str()).map(str::to_string),
        };
        if token_owner_is_dead(writer.as_deref(), path.as_path())? {
            verdicts.dead_temporaries.insert(name);
        }
    }
    return Ok(verdicts);
}

/// What:
///  Accepts a finished action,
/// or one that found its directory absent,
/// and reports every other failure.
/// Why:
///   A root that was never created holds nothing to make durable,
/// while any other refusal means the durable state is
///  unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await action(); } catch (e) { if (e.code !== 'ENOENT') throw e; }
/// ```
pub fn absent_is_fine(outcome: std::io::Result<()>) -> Result<(), WorktreeCopyError> {
    // `match` on the failure kind: only absence is accepted.
    match outcome {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes every stage container that a dead owner's claim names and no journal does,
/// then those claims,
/// and says what it
///  removed.
/// Why:
///   A claim whose owner still runs belongs to an add or a copy between claiming and journaling,
/// so it is left whole;
///  a claimed container no journal names,
/// whose owner is dead,
/// was abandoned by a process that died.
/// A stage a copy left is removed whenever it is a private directory;
/// a stage an add left is removed only when the
///  record of the object its creation made proves that object stands there and holds nothing the record cannot account
///  for (`worktree_copy_stage_made_object.rs`),
/// and is otherwise reported and kept,
/// so nothing a person added there
///  after the kill is lost and the claim goes on naming the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reapAbandonedStages(commonDir);
/// ```
pub fn reap_abandoned_stages(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    return reap_judged_stages(common_dir, &judge_stage_owners(common_dir)?, &RecoveryDestinations::EveryDestination);
}

/// `reap_abandoned_stages` on verdicts that were made before the command changed anything;
/// a claim or a temporary that the verdicts
///  do not name as dead is left whole.
pub fn reap_judged_stages(
    common_dir: &Path,
    verdicts: &StageVerdicts,
    destinations: &RecoveryDestinations<'_>,
) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let names: Vec<String> = claim_names(root.as_path())?;
    let mut changed: bool = false;
    let mut notices: Vec<String> = Vec::new();
    if !names.is_empty() {
        let mut journaled: BTreeSet<PathBuf> = BTreeSet::new();
        for pending in read_pending_journals(common_dir)? {
            journaled.insert(pending.journal.stage_container);
        }
        for name in names {
            let claim: PathBuf = root.join(name.as_str());
            let Some((container, _owner)) = read_claim(claim.as_path())? else {
                continue;
            };
            // A live owner's claim is its running add or copy; a claim whose owner cannot be judged is kept, too.
            if !verdicts.dead_claims.contains(name.as_str()) {
                continue;
            }
            let present: bool = still_there(container.as_path())?;
            let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
            // A record that cannot be looked up counts as one: when in doubt the stage is kept.
            let of_an_add: bool = !matches!(std::fs::symlink_metadata(record.as_path()), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound);
            let mut stage_stays: bool = false;
            if present && !journaled.contains(&container) {
                if of_an_add {
                    // Both proofs, or nothing is removed: the record of the object the creation made names the object
                    // standing there, and that object holds nothing the record cannot account for.
                    match provenance_of(claim.as_path(), container.as_path())? {
                        Provenance::Proved => {
                            remove_stage(container.as_path())?;
                            changed = true;
                            notices.push(removed_stage_words(container.as_path()));
                        }
                        Provenance::Gone => {}
                        Provenance::Unproved(reason) => {
                            notices.push(kept_stage_words(container.as_path(), reason.as_str()));
                            stage_stays = true;
                        }
                    }
                } else {
                    // A copy's stage is a private directory this wrapper made; one another command removed in between is
                    // nobody's failure.
                    match assert_private_path(container.as_path(), Role::PrivateStage) {
                        Ok(()) => {
                            remove_stage(container.as_path())?;
                            changed = true;
                            notices.push(removed_copy_stage_words(container.as_path()));
                        }
                        Err(_) if !still_there(container.as_path())? => {}
                        Err(error) => return Err(error),
                    }
                }
            }
            // A claim that still owes a copy into another worktree is that worktree's own work: the first command run
            // there makes the copy, and the pending check has already named the worktree. Nothing here settles it, so the
            // obligation outlives this command. The stage above is a partial copy nothing may resume, so its removal stays
            // open to every command: a worktree that is gone has no command left to clean up its own stage.
            if copy_owed_to_another_worktree(common_dir, claim.as_path(), record.as_path(), destinations) {
                continue;
            }
            let settled: Settlement = settle_interrupted_add(record.as_path());
            if let Some(notice) = settled.notice {
                notices.push(notice);
            }
            // While the stage stands its claim is what names it, so the record, the marker and the claim stay and the next
            //  command reports them again; a settled worktree keeps its records for the same reason.
            if settled.keep || stage_stays {
                continue;
            }
            remove_made(claim.as_path())?;
            remove_file_if_present(record.as_path())?;
            remove_file_if_present(claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
            remove_file_if_present(claim.as_path())?;
            changed = true;
        }
    }
    changed |= remove_dead_temporaries(root.as_path(), verdicts)?;
    changed |= remove_records_nothing_claims(root.as_path())?;
    // The flush makes the removals durable, and nothing else: an add that found other adds' live claims removed nothing and
    // paid a filesystem commit for it (measured 2026-11-11 as 4.6 to 13.9 seconds of the `recover` step of five
    // concurrent adds). A removal a power cut takes back only means the next command settles the same records again.
    if changed {
        absent_is_fine(sync_directory(root.as_path()))?;
    }
    return Ok(notices);
}


/// What:
///  Whether an interrupted add's record still owes a copy into a worktree other than the one the command runs
/// in,
/// with a checkout whole enough for the installation.
/// Why:
///  Such a copy is that worktree's own work and has a future there,
/// so this command must neither settle the obligation nor drop
///  the claim that names it;
/// the pending check writes one line naming the worktree
///  instead,
/// and the first command run there makes the copy or names why it cannot.
/// A destination that is not a worktree of this repository,
/// or one whose
///  checkout is not whole,
/// has no such future and settles as it always did,
/// which is what names a half-made
///  worktree to the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedElsewhere = (record, destinations) => { const text = read(record); const destination = destinationOfRecord(text); return destination !== undefined && !finishesDestinations(destinations, destination) && owedCopyOf(text) !== undefined && addIsFinished(destination); };
/// ```
fn copy_owed_to_another_worktree(
    common_dir: &Path,
    claim: &Path,
    record: &Path,
    destinations: &RecoveryDestinations<'_>,
) -> bool {
    let Ok(text) = std::fs::read_to_string(record) else {
        return false;
    };
    let destination: PathBuf = match crate::worktree_add_binding::decode_record(text.as_str()) {
        Some(crate::worktree_add_binding::Recorded::Planned(named)) => named,
        Some(crate::worktree_add_binding::Recorded::Registered(binding)) => binding.destination,
        None => return false,
    };
    if finishes_destination(destinations, destination.as_path()) {
        return false;
    }
    // A record written before the add named its source and selection owes no copy, so nothing may be filled from it.
    if crate::worktree_add_binding::owed_copy_of(text.as_str()).is_none() {
        return false;
    }
    // A destination holding the object the add's own snapshot became received the ignored state with it, so no copy is
    // owed anywhere: the records settle here, and no command says the worktree is missing files it holds.
    if crate::worktree_copy_stage_made_object::the_made_object_stands_at(claim, destination.as_path()) {
        return false;
    }
    // The copy has a future only where the registration and the checkout are whole under this repository.
    if crate::worktree_copy_recovery::destination_registration(common_dir, destination.as_path())
        != crate::worktree_copy_recovery::Registration::Registered
    {
        return false;
    }
    let Some(admin) = registration_directory(destination.as_path()) else {
        return false;
    };
    return add_is_finished(destination.as_path(), admin.as_path());
}
