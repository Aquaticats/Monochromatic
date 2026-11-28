//! What:
//!  What recovery settles an interrupted `git worktree add` to:
//! the record it keeps,
//! the notice it gives,
//! and the proof that decides whether the checkout it left is whole or gone.
//! Why:
//!  A record that cannot be read and a checkout that is not proven whole keep the record,
//! so the next command reports the same obligation again;
//! only a destination that is gone,
//! or one whose index and empty worktree together prove the add finished,
//! lets the record go.
//! The proof reads Git's own pointer and index rather than running Git,
//! because recovery never runs a Git command and never waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { notice, keep } = await reportInterruptedAdd(record);
//! const whole = !exists(destination) || (exists(join(admin, 'index')) && !exists(join(admin, 'index.lock')) && (entryCount(index) === 0 || entries(destination).length > 1))
//! ```

use crate::json_record::quote;
use crate::worktree_add_binding::{
    Recorded,
    decode_record,
};
/// What proves the checkout whole or gone,
/// re-exported so a reader and a control of the proof keep this module's path.
pub(crate) use crate::worktree_copy_interrupted_add_settlement_checkout_proof::{
    INDEX,
    INDEX_LOCK,
    add_is_finished,
};
use crate::worktree_copy_stage_claim::COMPLETE_SUFFIX;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};
/// `Duration` is the age a record that names no Git is kept for.
use std::time::Duration;

/// What recovery decided about one add's record.
pub struct Settlement {
    /// What to tell the person,
    /// if anything.
    pub notice: Option<String>,
    /// Whether the record and the claim stay because the state is unknown or the add is not proven whole,
    /// so the
    ///  next command reports it again.
    pub keep: bool,
}

#[path = "worktree_copy_stage_claim_pointer_presence.rs"]
mod pointer_presence_of_the_destination;

use pointer_presence_of_the_destination::{
    POINTER,
    PointerPresence,
    pointer_presence,
};

/// The name the clone rung writes its index under while the paths Git must write are unwritten
/// (`worktree_clone_index::PendingIndex`).
const WAITING_INDEX: &str = "index.pending";

/// The lock a Git run pointed at the waiting index holds while it writes it.
const WAITING_INDEX_LOCK: &str = "index.pending.lock";

/// What:
///  How long a record that names no Git process may be kept before it settles.
/// Why:
///   The name is written a few calls after the spawn,
/// so a command killed in between leaves a record that names no process
///  while its Git may still be at work;
/// nothing proves that Git gone,
/// so the record is kept while it is younger than this,
/// and
///  a minute is far longer than that gap and far shorter than a record that nothing will settle should stay.
/// Measured on this host on 2026-11-09,
/// in a scope of four CPUs,
/// with 200 runs and the same call path the add
///  uses:
/// the name reached the record between 5 ms and 1.45 s after the spawn (median 24 ms,
/// p95 99 ms),
/// and a real `git worktree add --no-checkout` made its destination between 4.5 ms and 9.1 ms
///  after its own spawn,
/// so the whole window this bound covers read 1.45 s at its longest,
/// which a minute
///  exceeds about forty times over.
pub(crate) const UNNAMED_CHILD_GRACE: Duration = Duration::from_secs(60);

/// What:
///  What a record settles to when it names no Git process,
/// or the platform will not say whether the one it names runs.
/// Why:
///   An add whose Git is not proven gone may still make its destination,
/// so its record stays and nothing is said;
/// the age is
///  the only bound there is,
/// and past it nothing of that add can still be starting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const young = Math.max(0, Date.now() - stat(record).mtimeMs) < UNNAMED_CHILD_GRACE_MS;
/// return { notice: undefined, keep: young };
/// ```
fn unnamed_child_settlement(record: &Path) -> Settlement {
    let aged: Option<Duration> = modified_at(record).and_then(age_since);
    return Settlement {
        notice: None,
        keep: matches!(aged, Some(age) if age < UNNAMED_CHILD_GRACE),
    };
}

/// The time the watch of `record` last changed,
/// or nothing when it cannot be read.
fn modified_at(record: &Path) -> Option<std::time::SystemTime> {
    return std::fs::metadata(record).ok().and_then(modified_of);
}

/// The modification time of a metadata reading.
fn modified_of(held: std::fs::Metadata) -> Option<std::time::SystemTime> {
    return held.modified().ok();
}

/// How long ago `changed` was,
/// or nothing when the clock cannot say.
fn age_since(changed: std::time::SystemTime) -> Option<Duration> {
    return std::time::SystemTime::now().duration_since(changed).ok();
}

/// What:
///  Whether nothing stands at the destination any more.
/// Why:
///   A destination that was removed,
/// moved or pruned holds no worktree to tell the person about,
/// and a record whose
///  worktree is gone settles without a word,
/// unlike one whose checkout is whole and whose ignored state never arrived.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { lstatSync(destination); return false } catch (e) { return e.code === 'ENOENT' }
/// ```
fn destination_is_gone(destination: &Path) -> bool {
    return matches!(std::fs::symlink_metadata(destination), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound);
}

/// What:
///  Whether the add's own snapshot became the worktree at `destination`,
/// so the ignored state the record still names is already there.
/// Why:
///   A snapshot rung swaps its finished stage with the destination,
/// and the record of the object its creation made then names the
///  object standing at the destination.
/// That object carries the source's ignored state with the files,
/// so nothing is missing there
///  and no copy is owed:
/// saying either would send a person to rebuild a worktree that is whole.
/// The destination must still be one of
///  this repository's registrations,
/// because a worktree nobody registers is named to the person instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotIsInPlace = (record, destination) => theMadeObjectStandsAt(claimOf(record), destination) && registrationDirectory(destination) !== undefined;
/// ```
fn the_snapshot_is_in_place(
    record: &Path,
    destination: &Path,
) -> bool {
    let claim: PathBuf = record.with_extension("stage");
    if !crate::worktree_copy_stage_made_object::the_made_object_stands_at(
        claim.as_path(),
        destination,
    ) {
        return false;
    }
    return registration_directory(destination).is_some();
}

/// What:
///  The report of an add whose Git checkout finished and whose ignored-state copy was never marked complete.
/// Why:
///   The record of an add exists because a copy of the source's ignored state was owed,
/// and the proof that a checkout is
///  whole reads Git's own work only:
/// a kill between real Git's checkout and the copy's completion marker leaves a worktree
///  that looks ordinary and never received the ignored state,
/// so a scanner there refuses the repository as
///  one with no rules file.
/// The record goes once that is said:
/// nothing about the worktree can make the copy appear later,
/// and
///  saying it on every later command would name a worktree that needs no finishing,
/// only remaking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `an interrupted \`git worktree add\` left ${shown} registered and its ignored-state copy never finished; ...`
/// ```
fn unfinished_copy_notice(
    shown: &str,
    owed: bool,
) -> String {
    if owed {
        // The record names the source and the selection, so a command run in that worktree can still make the copy.
        return format!(
            "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered, and the copy of its source's ignored state never finished, so files that source ignores may be missing there; that copy is still owed, and the first command run in that worktree makes it. Nothing was changed here."
        );
    }
    return format!(
        "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered, and the copy of its source's ignored state never finished, so files that source ignores may be missing there; nothing was changed. `git worktree remove {shown}` removes the worktree and leaves one that holds changes alone, so save them first, and running the same `git worktree add` line again copies the ignored state."
    );
}

/// What:
///  The administrative directory the destination's `.git` file names,
/// when that directory exists.
/// Why:
///   A record written before the add was bound carries only the destination,
/// so the pointer Git wrote is what names the
///  registration and the index the proof reads;
/// a pointer that is not a `gitdir:` line,
/// or one whose target is
///  not there,
/// proves nothing and is no registration.
/// The pointer is Git's own file,
/// so it is read by the plain-file reader,
/// which waits for no
///  writer,
/// follows no link and reads no more than `GIT_FILE_LIMIT`:
/// the destination is a directory
///  anything may write,
/// and a name left there as a named pipe must end the proof at once rather than
///  hold a command on an open that waits for a writer that may never come
/// ("nothing should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = readPlainFile(join(destination, '.git'), GIT_FILE_LIMIT).toString(); return line.startsWith('gitdir: ') ? resolve(destination, line.slice(8).trim()) : undefined;
/// ```
pub(crate) fn registration_directory(destination: &Path) -> Option<PathBuf> {
    // Git's own reader, on bytes (`worktree_add_binding::pointer_target`): the rest of the file after the prefix, with only
    //  its trailing newlines removed, so a path holding a newline is whole and the remedy never names another repository.
    let pointer: Vec<u8> = crate::plain_file::read_plain_file(
        destination.join(POINTER).as_path(),
        crate::worktree_convert_inspection::GIT_FILE_LIMIT,
    )
    .ok()?;
    let resolved: PathBuf =
        crate::worktree_add_binding::pointer_target(destination, pointer.as_slice()).ok()?;
    return resolved.is_dir().then_some(resolved);
}

/// What:
///  Remove the clone rung's waiting index and the lock a killed Git left beside it,
/// when they stand in the administrative directory `admin`.
/// Why:
///   A rung killed between writing its index under its own name and giving it the worktree's name leaves the file,
/// and the finish command the report prints writes the worktree's own `index` and knows nothing of it,
/// so a person who lists the directory would find a file of the add's that no step will ever take.
/// The names are this wrapper's own:
/// the rung publishes the waiting index as its last step,
/// so once the worktree's `index` stands no rung of ours is between the two names,
/// and a name another program left there is the only thing this could touch,
/// which is the same price the recovery already pays for the record itself.
/// The ZFS clone route's fill calls it too,
///  before it writes its own waiting
/// index again,
///  because `write_index_under_a_name` refuses a name that stands
/// (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
/// F42 and T6b).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheWaitingIndex(admin); // index.pending and index.pending.lock, when they stand
/// ```
pub(crate) fn clear_the_waiting_index(admin: &Path) {
    for name in [WAITING_INDEX, WAITING_INDEX_LOCK] {
        // A name that is not there is the ordinary case: a served rung published its index, and one killed earlier left
        //  nothing of its own.
        let _removed = std::fs::remove_file(admin.join(name));
    }
}

/// What:
///  What finishes or removes an interrupted add,
/// with the stale lock named first when one stands in the way.
/// Why:
///   `git -C <destination> reset --no-enforce-worktree --hard --no-recurse-submodules` cannot start while that lock exists,
/// and Git refuses it with
/// `fatal: Unable to create '.../index.lock': File exists`;
/// so the report names the file and the condition for removing it;
///  the worktree's own `git status` still works,
/// so nothing else would tell a person why every write to it fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = join(admin, 'index.lock')
/// return exists(lock) ? lockedSentence(lock, finish) : finish
/// ```
pub(crate) fn finish_instructions(
    destination: &Path,
    admin: &Path,
) -> String {
    // Both paths are read by the person's shell, so they are quoted the way a shell reads a word back, not the way JSON
    //  spells a string: `$bucket` and a backquote inside double quotes would be expanded by the shell that runs `rm`.
    let lock: PathBuf = admin.join(INDEX_LOCK);
    // The commands name the worktree guard's hatch: a checkout that never ran reads as uncommitted changes, so the guard
    //  refuses a hard reset in that worktree unless it is told.
    let finish: String = crate::worktree_finish_commands::finish_or_remove_sentence(destination);
    if !lock.exists() {
        return finish;
    }
    let lock_shown: String = crate::shell_quote::shell_quote(lock.as_os_str().as_bytes());
    return format!(
        "Its Git directory holds the lock {lock_shown} that a Git killed inside the index install left, and every Git write to the worktree fails while it stands: once no Git command is running there, `rm {lock_shown}` removes it, and then {finish}"
    );
}

/// What:
///  Whether the add a claim's record names recorded that it finished,
/// by the completion marker the claim and its records share a name with.
/// Why:
///   The marker is written once the ignored state has arrived,
/// from a copy that ran or from a snapshot
///  that carried it,
/// and it is the add's own record that nothing is left to finish.
/// A release removes the record of the
///  object the stage's creation made before the claim's own records,
/// so a command killed between those
///  removals leaves a finished add whose object record is gone;
/// the marker is what still says it finished,
///  and a claim that carries one owes no copy and names no missing file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finished = (claim) => existsSync(claim.replace(/\.stage$/, '.done'));
/// ```
pub(crate) fn the_add_finished(claim: &Path) -> bool {
    return std::fs::symlink_metadata(claim.with_extension(&COMPLETE_SUFFIX[1..])).is_ok();
}

/// What:
///  Report an add that was interrupted before it was marked complete,
/// and change nothing of the person's.
/// Why:
///   The only thing it removes is the clone rung's own waiting index,
/// which names nothing a person or a later step reads (`clear_the_waiting_index`).
///   Recovery never deletes or unregisters a worktree:
/// a path,
/// a registration or a name can be changed by another process
///  between any check and any act,
/// so nothing the wrapper recorded earlier is a right to remove something later.
/// The report
///  names the worktree and the two commands that finish or remove it,
/// as Git's own interrupted add leaves it.
/// A record that
///  cannot be read,
/// and a marker that cannot be looked up,
/// are not "incomplete" and not "complete":
/// the record stays and the
///  report says that the state is unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { notice, keep } = await reportInterruptedAdd(record);
/// ```
pub fn settle_interrupted_add(record: &Path) -> Settlement {
    let marker: std::io::Result<bool> =
        match std::fs::symlink_metadata(record.with_extension(&COMPLETE_SUFFIX[1..])) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        };
    // The record is read only when the marker is absent, as the lazy read of `settle_from` did.
    if let Some(settled) = settle_marker(record, marker) {
        return settled;
    }
    return settle_text(record, std::fs::read_to_string(record));
}

/// What:
///  The settlement the completion marker alone decides,
/// or nothing when the record must be read.
/// Why:
///   A marker means the add finished,
/// so its record is dropped;
/// a marker that cannot be checked keeps the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (marker === true) return { keep: false }; if (marker instanceof Error) return { keep: true, notice }; return undefined;
/// ```
pub fn settle_marker(
    record: &Path,
    marker: std::io::Result<bool>,
) -> Option<Settlement> {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    return match marker {
        Ok(true) => Some(Settlement {
            notice: None,
            keep: false,
        }),
        Ok(false) => None,
        Err(error) => Some(Settlement {
            notice: Some(format!(
                "monochromatic-git-policy-cli: whether the `git worktree add` of {shown_record} completed cannot be checked ({error}); nothing was changed and the record stays."
            )),
            keep: true,
        }),
    };
}

/// What:
///  The settlement of an interrupted add from what its record said,
/// or the failure to read it.
/// Why:
///   A record that is gone,
/// unreadable,
/// planned or registered each ends differently,
/// and none deletes what is not proven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settlement = settleText(record, await readFile(record).catch((error) => error));
/// ```
pub fn settle_text(
    record: &Path,
    read: std::io::Result<String>,
) -> Settlement {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    let text: String = match read {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Settlement {
                notice: None,
                keep: false,
            };
        }
        Err(error) => {
            return Settlement {
                notice: Some(format!(
                    "monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` cannot be read ({error}); nothing was changed and the record stays."
                )),
                keep: true,
            };
        }
    };
    return match decode_record(text.as_str()) {
        None => Settlement {
            notice: Some(format!(
                "monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` is unreadable; nothing was changed."
            )),
            keep: false,
        },
        Some(Recorded::Planned(destination)) => {
            // The path is printed inside commands the person runs, so it is quoted for the shell.
            let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
            // Nothing at the path, or nothing Git's pointer there: the add made nothing that is left to finish. A lookup that
            //  fails otherwise is no answer: a directory the person or another process holds closed for a moment must not read
            //  as "no pointer" and take the record of a worktree that is still registered. And a destination that is absent is
            //  no answer either while the Git this add started may still be making it, which the record says: a named Git that
            //  is proven running keeps the record, a named Git that is gone settles it, and a record too young to have named
            //  one yet keeps it until the grace has passed.
            match pointer_presence(destination.as_path()) {
                PointerPresence::Unknown(why) => {
                    return Settlement {
                        notice: Some(format!(
                            "monochromatic-git-policy-cli: whether an interrupted `git worktree add` left {shown} cannot be examined ({why}); nothing was changed and the record stays."
                        )),
                        keep: true,
                    };
                }
                PointerPresence::Absent => {
                    let Some(child) = crate::worktree_add_binding::running_child_of(text.as_str())
                    else {
                        return unnamed_child_settlement(record);
                    };
                    return match child.is_alive(record) {
                        Ok(true) => Settlement {
                            notice: None,
                            keep: true,
                        },
                        Ok(false) => Settlement {
                            notice: None,
                            keep: false,
                        },
                        Err(_unproven) => unnamed_child_settlement(record),
                    };
                }
                PointerPresence::Present => {}
            }
            // A snapshot that was swapped in is the whole worktree: its ignored state arrived with it, so nothing is owed and
            // nothing is missing, and the records settle without a word (the kill point right after the exchange).
            if the_snapshot_is_in_place(record, destination.as_path()) {
                return Settlement {
                    notice: None,
                    keep: false,
                };
            }
            // A kill between real Git's registration and the wrapper's record of the binding leaves the pointer without the
            //  plan, so the pointer is what names the administrative directory the proof reads.
            let registered: Option<PathBuf> = registration_directory(destination.as_path());
            if destination_is_gone(destination.as_path()) {
                return Settlement {
                    notice: None,
                    keep: false,
                };
            }
            // The rung's waiting index is the add's own bookkeeping, so it goes whether the checkout is proven whole or
            //  not: the finish command the notice prints writes the worktree's own `index` and nothing takes these names
            //  afterwards, so a person who lists the directory would find files of the add that no step will use.
            if let Some(admin) = registered.as_ref() {
                clear_the_waiting_index(admin.as_path());
            }
            let owes_a_copy: bool =
                crate::worktree_add_binding::owed_copy_of(text.as_str()).is_some();
            if let Some(admin) = registered.as_ref()
                && add_is_finished(destination.as_path(), admin.as_path())
            {
                // A record that names the copy still owed is what a command run in that worktree reads to make it, so
                // it stays whole until the copy is made or the worktree is gone; a record that names no copy can never
                // be finished by any command, and it is said once.
                return Settlement {
                    notice: Some(unfinished_copy_notice(shown.as_str(), owes_a_copy)),
                    keep: owes_a_copy,
                };
            }
            let notice: String = match registered.as_ref() {
                Some(admin) => format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. {}",
                    finish_instructions(destination.as_path(), admin)
                ),
                None => format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` may have left {shown}; nothing proves it is the add's, so it was not changed. `git worktree list` shows whether it is registered, and `git worktree remove --force {shown}` removes it."
                ),
            };
            // The record stays while something stands at the recorded path that is not proven whole, so every later command
            //  that runs the recovery says the same instead of leaving the obligation in one command's output.
            Settlement {
                notice: Some(notice),
                keep: true,
            }
        }
        Some(Recorded::Registered(binding)) => {
            let shown: String =
                crate::shell_quote::shell_quote(binding.destination.as_os_str().as_bytes());
            // A destination that is gone holds nothing to finish, and one whose Git checkout is whole is named for the copy
            //  that recorded no completion; everything else keeps its record so the next command says the same again,
            //  because nothing else names the worktree, the stage or what finishes them.
            if destination_is_gone(binding.destination.as_path()) {
                return Settlement {
                    notice: None,
                    keep: false,
                };
            }
            // The same proof as above, on a record that was bound before the command died.
            if the_snapshot_is_in_place(record, binding.destination.as_path()) {
                return Settlement {
                    notice: None,
                    keep: false,
                };
            }
            // As above: the rung's waiting index names nothing a person needs once the add is reported.
            clear_the_waiting_index(binding.admin.as_path());
            let owes_a_copy: bool =
                crate::worktree_add_binding::owed_copy_of(text.as_str()).is_some();
            if add_is_finished(binding.destination.as_path(), binding.admin.as_path()) {
                // As above: a record that names the copy still owed stays, because a command in that worktree can make it.
                return Settlement {
                    notice: Some(unfinished_copy_notice(shown.as_str(), owes_a_copy)),
                    keep: owes_a_copy,
                };
            }
            Settlement {
                notice: Some(format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. {}",
                    finish_instructions(binding.destination.as_path(), binding.admin.as_path())
                )),
                keep: true,
            }
        }
    };
}
