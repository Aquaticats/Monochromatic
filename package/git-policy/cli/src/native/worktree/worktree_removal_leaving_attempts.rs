//! What:
//!  The attempts one leaving may cost:
//!  a slot taken by the one command that makes the attempt,
//!  published before the attempt with an exclusive create or a rename only one command can win,
//!  and marked once the attempt is made,
//!  so a slot a killed command took but never used is taken over by the next command.
//! Why:
//!  A budget holds only if every attempt is counted by the one who makes it,
//!  at the moment it is made,
//!  in a way two commands cannot both pass:
//!  a number read and then written back lets four commands decide on the same count and then all act
//!  (`Leavings` and the look that fills it),
//!  and a count written after the attempt lets a command killed between the two spend the budget on nothing,
//!  leaving a delete no command queues again.
//!  So a slot is a file, published atomically **before** the act it counts:
//!  - `reserved-<leaving>.<n>` holds the command that will make attempt `n`,
//!    published with an exclusive create so two commands can never take the same number;
//!  - `taking-<leaving>.<n>.<command>` is a slot taken over from a command that was proved gone,
//!    made by renaming the slot's own name with `renameat2` and `RENAME_NOREPLACE`,
//!    so exactly one command wins,
//!    and the name itself carries the taker's identity,
//!    which is what a later command reads to know whether the taker may still be running;
//!  - `made-<leaving>.<n>` is published once the attempt was made,
//!    and it is never removed by the command that made the attempt.
//!  A slot whose owner may still be running is another command's attempt and is left alone;
//!  a slot whose owner is proved gone is taken over and its attempt made,
//!  so a kill between the slot and the act costs the act's lateness and never the delete.
//!  A slot that names no command this program could have written is not a count at all:
//!  the caller is told what could not be read and queues the delete anyway
//!  (`Attempt::Unreadable`), because a count that cannot be read is not a count.
//!  Nothing here reads a number and writes it back:
//!  the number a command gets is the name it published, and the budget is the set of names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = takeAnAttempt({ commonDir, id, owner });   // { kind: 'took', n } | { kind: 'spent' } | { kind: 'unreadable', reason }
//! if (slot.kind !== 'took') return slot;
//! await makeTheAttempt();                                  // queue the delete, or ask the service
//! markTheAttemptMade(slot.slot);                           // after the act
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::service_work::MAX_TRIES;
use crate::worktree_removal_finishes_an_interrupted_destroy::owner_may_be_running;
use std::path::{Path, PathBuf};

/// The folder of the common Git directory that holds one file per attempt slot of each leaving.
pub(crate) const ATTEMPTS_FOLDER: &str = "monochromatic-git-policy-cli/leaving-attempts";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavingAttempts";

/// The number one leaving's attempts are numbered up to,
/// which is the budget the look promises a person reads of.
const THE_BUDGET: u32 = MAX_TRIES;

/// The words every line a person reads from this module starts with,
/// so a terminal shows which program said it.
const SAID: &str = "monochromatic-git-policy-cli:";

/// What:
///  The name of the reservation of attempt `number` of one leaving.
/// Why:
///   The name is what the exclusive create decides on,
///  so one file is one attempt and two commands cannot take the same number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(attemptsFolder, `reserved-${id}.${n}`);
/// ```
fn reserved_path(folder: &Path, id: &str, number: u32) -> PathBuf {
    return folder.join(format!("reserved-{id}.{number}"));
}

/// What:
///  The name that says attempt `number` of one leaving was made.
/// Why:
///   The budget counts attempts that were made,
///  and this is the only file that says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(attemptsFolder, `made-${id}.${n}`);
/// ```
fn made_path(folder: &Path, id: &str, number: u32) -> PathBuf {
    return folder.join(format!("made-{id}.{number}"));
}

/// What:
///  The name a slot takes when this command takes the attempt over,
///  which carries the taker's own identity.
/// Why:
///   The rename is the claim,
///  and it is published with `RENAME_NOREPLACE`,
///  so of any number of commands that find the same dead slot exactly one takes it.
///   The name carries the taker because the file it moves holds the dead command's line:
///  a later command that found that line would take a live command's attempt over,
///  so what a taker left has to say who the taker is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = `taking-${id}.${n}.${hex(owner)}`;
/// ```
fn taking_name(id: &str, number: u32, owner: &str) -> String {
    return format!("taking-{id}.{number}.{}", crate::hexadecimal_of_bytes::hexadecimal_of_bytes(owner.as_bytes()));
}

/// What:
///  One attempt slot this command holds,
///  named so that only this command may remove it.
/// Why:
///   This command holds the slot until the act it counts is made or given back,
///  and both steps remove the one name it took,
///  never another command's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { folder: string; id: string; number: number; holding: string };
/// ```
pub struct Slot {
    /// The attempts folder the name stands in.
    folder: PathBuf,
    /// The leaving the attempt is of.
    id: String,
    /// The number of the attempt this slot counts.
    number: u32,
    /// The one name this command holds, which is the only name it may remove.
    holding: PathBuf,
    /// Whether that name is a taker's, which this command removes when the attempt is made,
    /// so a leaving whose slot was taken over keeps no file per takeover.
    taken_over: bool,
}


/// What:
///  What one call for an attempt came to.
/// Why:
///   Three endings need three steps from the caller:
///  an act that is counted,
///  the words a person reads when the budget is spent,
///  and a count that cannot be read,
///  which is no count at all and costs the act nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempt = { kind: 'took'; slot } | { kind: 'spent' } | { kind: 'held' } | { kind: 'unreadable'; reason };
/// ```
pub(crate) enum Attempt {
    /// An attempt this command may make, which it must mark or give back.
    Took(Slot),
    /// Every attempt this leaving may cost was made.
    Spent,
    /// Every number is taken, and at least one of them is another command's attempt that command may still make.
    Held,
    /// The slots cannot be read or written, with the reason a person reads.
    Unreadable(String),
}

/// What:
///  Whether one name in the attempts folder belongs to this leaving,
///  what kind of slot it names,
///  and the token a `taking-` name carries.
/// Why:
///   The folder holds the slots of every leaving of the repository,
///  so a name is read once into what it says and never guessed again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoundSlot = { kind: 'made' | 'reserved' | 'claimed' | 'taking'; number: number; path: string; token?: string };
/// ```
struct FoundSlot {
    /// Which of the four names this slot stands under.
    kind: SlotKind,
    /// The attempt number the name carries.
    number: u32,
    /// The file that holds the slot.
    path: PathBuf,
    /// The identity an earlier taker wrote into a `taking-` name, or nothing for the other names.
    token: Option<String>,
}

/// The four names a slot of one leaving may stand under.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    /// A name an earlier release's takeover wrote, whose file holds its owner's line.
    /// What reads it:
    /// this release, so that a slot an earlier release left is still counted and still taken over,
    /// and never written.
    Claimed,
    /// A name a command published for itself; its file holds its owner's line.
    Reserved,
    /// The name of an attempt that was made.
    Made,
    /// A name a taker renamed a dead slot to; the name holds the taker's identity.
    Taking,
}

/// What:
///  The number and the token a slot name carries after `prefix`,
///  or nothing when the name is not one of this leaving's.
/// Why:
///   One shape reads all four names,
///  and a name of another leaving,
///  or one a person wrote,
///  answers nothing and is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const after = name.startsWith(prefix) ? name.slice(prefix.length).split('.') : undefined;
/// ```
fn number_and_token(name: &str, prefix: &str) -> Option<(u32, Option<String>)> {
    let rest: &str = name.strip_prefix(prefix)?;
    let (number, token): (&str, Option<&str>) = match rest.split_once('.') {
        Some((number, token)) => (number, Some(token)),
        None => (rest, None),
    };
    return Some((number.parse::<u32>().ok()?, token.map(str::to_owned)));
}

/// What:
///  Every slot name of this leaving the folder holds,
///  or the reason the folder cannot be listed.
/// Why:
///   The set of names is the count this module decides on,
///  so it is read once per call rather than asked name by name,
///  and a folder that cannot be listed is a count that cannot be read,
///  which the caller must hear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slots = readdir(folder).flatMap((name) => slotOf(name, id));
/// ```
fn the_slots_of(folder: &Path, id: &str) -> Result<Vec<FoundSlot>, String> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(folder) {
        Ok(listing) => listing,
        // The folder is gone, which is a leaving that has none of its attempts taken or made.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the folder {} cannot be listed ({error})", folder.display())),
    };
    let mut found: Vec<FoundSlot> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        // A name that is not text is no name this program writes, so it is no slot of this leaving's.
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue };
        let kinds: [(SlotKind, String); 4] = [
            (SlotKind::Made, format!("made-{id}.")),
            (SlotKind::Reserved, format!("reserved-{id}.")),
            (SlotKind::Claimed, format!("claimed-{id}.")),
            (SlotKind::Taking, format!("taking-{id}.")),
        ];
        for (kind, prefix) in kinds {
            let Some((number, token)) = number_and_token(name.as_str(), prefix.as_str()) else { continue };
            found.push(FoundSlot { kind, number, path, token });
            break;
        }
    }
    return Ok(found);
}

/// What:
///  The identity a `taking-` name carries,
///  or the reason a name under that shape is not one this program wrote.
/// Why:
///   A taker's identity is what tells a later command whether the attempt is still being made,
///  so a `taking-` name whose token is not the hexadecimal form of an identity names nothing this program may read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = textFromHex(slot.token) ?? throw new Error(`${slot.path} names no command`);
/// ```
fn the_taker_of(slot: &FoundSlot) -> Result<String, String> {
    let decoded: Option<String> = slot.token.as_deref().and_then(owner_of_hexadecimal);
    return match decoded {
        Some(owner) => Ok(owner),
        None => Err(format!("the attempt slot {} names no command that took it", slot.path.display())),
    };
}

/// What:
///  The bytes the hexadecimal digits `digits` spell,
///  or nothing when they spell no bytes or are no text.
/// Why:
///   A taker's identity in a name is hexadecimal so that any identity a platform prints fits a file name,
///  and this is the one place that spelling is read back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = digits.length % 2 === 0 ? Buffer.from(digits, 'hex').toString('utf8') : undefined;
/// ```
fn owner_of_hexadecimal(digits: &str) -> Option<String> {
    if digits.len() % 2 != 0 {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(digits.len() / 2);
    for pair in digits.as_bytes().chunks(2) {
        let text: &str = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(text, 16).ok()?);
    }
    return String::from_utf8(bytes).ok();
}

/// What:
///  The owner line a slot's own file holds,
///  or the reason it names no command that took it.
/// Why:
///   A reservation and a name an earlier release's takeover wrote hold the owner inside the file,
///  and a file whose line is empty,
///  is not text,
///  or carries no process number names no command at all:
///  it is a count that cannot be read,
///  and the caller must hear which file it could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = theOwnerInFile(slot.path); // `<pid>:<identity>`, or a reason
/// ```
fn the_owner_in_the_file(path: &Path) -> Result<String, String> {
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| format!("the attempt slot {} cannot be read ({error})", path.display()))?;
    let text: &str = std::str::from_utf8(bytes.as_slice()).map_err(|_not_text| format!("the attempt slot {} does not name the command that took it", path.display()))?;
    let spelled: &str = text.trim_end();
    let Some((process, _identity)) = spelled.split_once(':') else {
        return Err(format!("the attempt slot {} does not name the command that took it", path.display()));
    };
    if process.parse::<i64>().is_err() {
        return Err(format!("the attempt slot {} does not name the command that took it", path.display()));
    }
    return Ok(spelled.to_owned());
}

/// What:
///  The identity of the command that took a slot,
///  whichever of the four names it stands under,
///  or the reason the slot cannot be read.
/// Why:
///   A live command's slot is its own attempt,
///  and a dead command's is the next command's to make,
///  so this is the answer every takeover begins with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = slot.kind === 'taking' ? takerFromName(slot) : ownerInFile(slot.path);
/// ```
fn the_owner_of(slot: &FoundSlot) -> Result<String, String> {
    return match slot.kind {
        SlotKind::Taking => the_taker_of(slot),
        SlotKind::Reserved | SlotKind::Claimed => the_owner_in_the_file(slot.path.as_path()),
        SlotKind::Made => Err(format!("the attempt slot {} is a made attempt with no owner", slot.path.display())),
    };
}

/// What:
///  Rename one dead slot's name to the name this command takes it over into,
///  refusing a name that already stands,
///  and answer whether this command won the rename.
/// Why:
///   The rename is the whole of the claim:
///  the name it moves away cannot be moved by a second command,
///  so of any number of commands that find the same dead slot exactly one acts,
///  and a loser is told so by the same call that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const won = renameNoReplace(below(folder), slotName, below(folder), takingName); // false when another command won
/// ```
fn take_over_by_rename(folder: &Path, from: &Path, to: &Path) -> Result<bool, String> {
    let (Some(from_name), Some(to_name)) = (from.file_name(), to.file_name()) else {
        return Err(format!("{} or {} names no file", from.display(), to.display()));
    };
    let held: std::fs::File = crate::fs_beneath::open_root(folder).map_err(|error| format!("the folder {} cannot be opened ({error})", folder.display()))?;
    return match crate::fs_beneath::rename_at(&held, from_name, &held, to_name, false) {
        Ok(()) => Ok(true),
        // The name is gone, so another command took the slot over between the look and this rename.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        // A name that already stands is another taker's, which is the same answer as a lost race.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(format!("the attempt slot {} cannot be taken over ({error})", from.display())),
    };
}

/// What:
///  Whether some name other than `ours` says attempt `number` of this leaving is taken or made.
/// Why:
///   A command that looked before another took the number publishes its own name afterwards,
///  and without this second look both would act on one number:
///  the name that appeared is either the mark of a made attempt or a taker's name,
///  and either one means this command's look is out of date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = readdir(folder).find((slot) => slot.number === n && slot.path !== ours && slot.kind !== 'claimed');
/// ```
fn another_command_says_taken(slots: &[FoundSlot], number: u32, ours: &Path) -> Option<SlotKind> {
    return slots
        .iter()
        .find(|slot| { return slot.number == number && slot.path != ours && matches!(slot.kind, SlotKind::Made | SlotKind::Taking) })
        .map(|slot| { return slot.kind; });
}

/// What:
///  Take one attempt slot of this leaving for this command,
///  answer that the budget is spent,
///  or answer what could not be read.
/// Why:
///   The slot is published before the act it counts,
///  so a command killed between the two leaves a slot the next command takes over and makes the attempt for,
///  and a command killed after the act leaves a slot that is already spent.
///  A free number is published exclusively,
///  a slot whose command is gone is taken over by a rename only one command can win,
///  and a made number is never taken again because the mark of it is never removed.
///  A slot that cannot be read is no count:
///  the caller must hear which name it was and queue the delete without counting it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = takeAnAttempt({ commonDir, id, owner }); // Took | Spent | Unreadable
/// ```
pub(crate) fn take_an_attempt(common_dir: &Path, id: &str, owner: &str) -> Attempt {
    let folder: PathBuf = common_dir.join(ATTEMPTS_FOLDER);
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        return Attempt::Unreadable(format!("the folder {} cannot be made ({error})", folder.display()));
    }
    let slots: Vec<FoundSlot> = match the_slots_of(folder.as_path(), id) {
        Ok(found) => found,
        Err(reason) => return Attempt::Unreadable(reason),
    };
    // What: Whether some number of this leaving is another command's attempt that command may still make.
    // Why:  The words that say the attempts ran out are about attempts that were made, and a number another command is
    //       making right now is not one of those.
    let mut another_command_may_act: bool = false;
    for number in 1..=THE_BUDGET {
        if slots.iter().any(|slot| { return slot.kind == SlotKind::Made && slot.number == number }) {
            continue;
        }
        let holding: Vec<&FoundSlot> = slots.iter().filter(|slot| { return slot.kind != SlotKind::Made && slot.number == number }).collect();
        if !holding.is_empty() {
            // What: One name of this number that this command may take over, or nothing when each is another command's.
            // Why:  A slot whose owner may still be running is another command's attempt and is never touched.
            let mut taken_over: Option<PathBuf> = None;
            for slot in holding {
                let slot_owner: String = match the_owner_of(slot) {
                    Ok(spelled) => spelled,
                    Err(reason) => return Attempt::Unreadable(reason),
                };
                if owner_may_be_running(Some(&slot_owner)) {
                    another_command_may_act = true;
                    continue;
                }
                let to: PathBuf = folder.join(taking_name(id, number, owner));
                match take_over_by_rename(folder.as_path(), slot.path.as_path(), to.as_path()) {
                    Ok(true) => {
                        taken_over = Some(to);
                        break;
                    }
                    // Another command took this slot over between the look and this rename, so it is making the attempt.
                    Ok(false) => {
                        another_command_may_act = true;
                        continue;
                    }
                    Err(reason) => return Attempt::Unreadable(reason),
                }
            }
            let Some(holding_name) = taken_over else { continue };
            // What: The second look after the takeover, so an attempt another command made meanwhile is never made twice.
            // Why:  A made mark that appeared after the look above is the one answer that "made" is never removed for.
            let after: Vec<FoundSlot> = match the_slots_of(folder.as_path(), id) {
                Ok(found) => found,
                Err(reason) => {
                    let _given_back = std::fs::remove_file(&holding_name);
                    return Attempt::Unreadable(reason);
                }
            };
            match another_command_says_taken(after.as_slice(), number, holding_name.as_path()) {
                Some(SlotKind::Made) => {
                    let _given_back = std::fs::remove_file(&holding_name);
                    continue;
                }
                Some(_other) => {
                    let _given_back = std::fs::remove_file(&holding_name);
                    another_command_may_act = true;
                    continue;
                }
                None => {}
            }
            debug(TAG, format!("the attempt {number} of {id} was left by a command that is gone, so it is taken over").as_str());
            return Attempt::Took(Slot { folder, id: id.to_owned(), number, holding: holding_name, taken_over: true });
        }
        let reserved: PathBuf = reserved_path(folder.as_path(), id, number);
        match write_private_file_unsynced(reserved.as_path(), owner.as_bytes()) {
            Ok(()) => {
                // What: The second look after the publish, which is what keeps a made number from being taken again.
                // Why:  A command whose look predates another command's made mark publishes its own name once that
                //       command's reservation is gone, so the mark is read again here, after this command's own name stands.
                let after: Vec<FoundSlot> = match the_slots_of(folder.as_path(), id) {
                    Ok(found) => found,
                    Err(reason) => {
                        let _given_back = std::fs::remove_file(&reserved);
                        return Attempt::Unreadable(reason);
                    }
                };
                match another_command_says_taken(after.as_slice(), number, reserved.as_path()) {
                    Some(SlotKind::Made) => {
                        let _given_back = std::fs::remove_file(&reserved);
                        continue;
                    }
                    Some(_other) => {
                        let _given_back = std::fs::remove_file(&reserved);
                        another_command_may_act = true;
                        continue;
                    }
                    None => {}
                }
                return Attempt::Took(Slot { folder, id: id.to_owned(), number, holding: reserved, taken_over: false });
            }
            // Another command took this number between the look and the publish, so the next one is asked for.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                another_command_may_act = true;
            }
            Err(error) => {
                return Attempt::Unreadable(format!("the attempt {number} of {id} cannot be taken in {} ({error})", folder.display()));
            }
        }
    }
    if another_command_may_act {
        return Attempt::Held;
    }
    return Attempt::Spent;
}

/// What:
///  Mark the attempt this command took as made,
///  and drop the one name it holds when that name is a taker's.
/// Why:
///   An attempt is counted by the act it counts,
///  so this is called after the delete is queued or the service is asked.
///   The mark of a made attempt is never removed,
///  and neither is the reservation a command published,
///  because a command that looked before the mark and publishes afterwards finds the name still standing
///  and asks for another number instead of acting twice on one.
///   A taker's name is removed because only this command holds it,
///  and a leaving whose slot was taken over would otherwise keep one file per takeover for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markTheAttemptMade(slot); // made-<id>.<n>, and drop claimed-/taking- names of this command
/// ```
pub(crate) fn mark_the_attempt_made(slot: &Slot) {
    let made: PathBuf = made_path(slot.folder.as_path(), slot.id.as_str(), slot.number);
    match write_private_file_unsynced(made.as_path(), b"") {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            warn(TAG, format!("the made attempt {} of {} is not recorded ({error})", slot.number, slot.id).as_str());
        }
    }
    // What: Only a taker's name is dropped here.
    // Why:  A reservation this command published is the name that stops a command which looked before the mark from
    //       taking the number again, and a name an earlier release wrote belongs to the attempt that command took.
    if slot.taken_over {
        let _dropped_taking = std::fs::remove_file(slot.holding.as_path());
    }
}

/// What:
///  Give back the slot a command took for an act that did not happen,
///  so the attempt it would have made is still the person's to have.
/// Why:
///   A delete that cannot be queued and an ask that cannot be made cost nothing,
///  and a slot spent by an act that failed would shorten the budget by one for no attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releaseTheAttempt(slot); // drop the one name this command holds
/// ```
pub(crate) fn release_the_attempt(slot: &Slot) {
    let _given_back = std::fs::remove_file(slot.holding.as_path());
}

/// What:
///  Mark every ask this command took a slot for,
///  or give them all back when the wake they were for reached no service,
///  answering the line the person reads when it did not.
/// Why:
///   A wake that reached nobody is not an attempt:
///  a slot spent by it would end in the words that say the delete was given up,
///  while no service ever tried it.
///   The person reads the truth either way:
///  no attempt counted, no given-up words, and the delete still queued for the next command to ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = settleTheAsks(asks, await wakeTheService(context, commonDir)); // undefined when the wake was made
/// ```
pub(crate) fn settle_the_asks(asks: &[Slot], wake: Result<(), String>) -> Option<String> {
    return match wake {
        Ok(()) => {
            for slot in asks {
                mark_the_attempt_made(slot);
            }
            None
        }
        Err(why) => {
            for slot in asks {
                release_the_attempt(slot);
            }
            debug(TAG, format!("the commit service was not reached ({why}), so no ask is counted as an attempt").as_str());
            Some(format!(
                "{SAID} the delete of a removed worktree's subvolume is queued, and the commit service was not reached ({why}), so it stays for the next command of this repository to ask for; nothing here counts that as an attempt."
            ))
        }
    };
}

/// What:
///  How many of the attempts one leaving may cost are taken,
///  whether they were made or a command that died still holds them.
/// Why:
///   The number is the budget a person reads of,
///  and it is counted from the names commands can publish, never from a number one of them wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = attemptsTaken(commonDir, id); // one per number any of the four names carries
/// ```
pub(crate) fn attempts_taken(common_dir: &Path, id: &str) -> u32 {
    let folder: PathBuf = common_dir.join(ATTEMPTS_FOLDER);
    let slots: Vec<FoundSlot> = match the_slots_of(folder.as_path(), id) {
        Ok(found) => found,
        Err(_unreadable) => return 0,
    };
    let mut taken: u32 = 0;
    for number in 1..=THE_BUDGET {
        if slots.iter().any(|slot| { return slot.number == number }) {
            taken += 1;
        }
    }
    return taken;
}

/// What:
///  Drop every attempt slot of one leaving,
///  once nothing of this program's stands where it named.
/// Why:
///  The slots are the budget of one leaving,
///  and the next leaving of the same place must start with all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheAttempts(commonDir, id);
/// ```
pub(crate) fn forget_the_attempts(common_dir: &Path, id: &str) {
    let folder: PathBuf = common_dir.join(ATTEMPTS_FOLDER);
    if let Ok(listing) = std::fs::read_dir(folder.as_path()) {
        let prefixes: [String; 4] = [format!("made-{id}."), format!("reserved-{id}."), format!("claimed-{id}."), format!("taking-{id}.")];
        for entry in listing.flatten() {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue };
            if prefixes.iter().any(|prefix| { return name.starts_with(prefix.as_str()) }) {
                let _removed = std::fs::remove_file(entry.path());
            }
        }
    }
    let _kept = std::fs::remove_dir(folder.as_path());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leaving_attempts_tests.rs"]
mod tests;
