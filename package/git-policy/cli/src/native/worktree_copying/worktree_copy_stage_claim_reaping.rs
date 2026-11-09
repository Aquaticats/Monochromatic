//! What:
//!  The judging of stage claims and their temporaries,
//! and the removal of the stages that interrupted copies and interrupted
//!  `git worktree add` runs left behind.
//! Why:
//!  A claim is the record that a stage may exist.
//! A claim whose owner is dead and whose stage no journal adopted is reaped;
//! a
//!  claim of an add names a stage by path alone,
//! which proves nothing,
//! so that stage is reported and never removed,
//! and the claim stays so that the
//!  next command reports it again.
//! An add's record stays too while the worktree it names is not proven whole,
//! whether the add was bound or killed
//!  before the binding was written:
//! a line on one command's standard error is not a place to keep an obligation a person has
//!  to act on,
//! and the destination's own pointer names the registration that proves the checkout whole or not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await judgeStageOwners(commonDir);
//! const notices = await reapJudgedStages(commonDir, verdicts);
//! ```

use crate::json_record::quote;
use crate::path_bytes_in_records::path_of_the_two_fields;
use crate::private_storage::sync_directory;
use crate::worktree_add_binding::{Recorded, decode_record};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_journal::{Role, assert_private_path, journal_root, read_pending_journals, temporary_writer_token};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_journal_removal::{remove_file_if_present, remove_stage};
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_selection::{STAGE_PREFIX, js_order};
use crate::worktree_copy_stage_claim::{COMPLETE_SUFFIX, CONTAINER_HEX_PREFIX, DESTINATION_SUFFIX, OWNER_LINE_PREFIX, claim_names};
use std::collections::BTreeSet;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

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

/// The name of the lock Git takes to write an index,
/// in the administrative directory of a worktree.
pub(crate) const INDEX_LOCK: &str = "index.lock";

/// The name of the index itself,
/// in the administrative directory of a worktree.
pub(crate) const INDEX: &str = "index";

/// The name Git gives the pointer file that makes a directory a worktree.
const POINTER: &str = ".git";

/// What:
///  How an index file starts:
/// its magic,
/// its version and its entry count.
/// Why:
///   The count alone answers whether a checkout had anything to write,
/// and reading it needs no object
///  name length,
/// so recovery reads a dozen bytes instead of parsing every entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = readSync(index).subarray(0, 12); // 'DIRC' | version | count
/// ```
const INDEX_HEADER_BYTES: usize = 12;

/// The index's magic at the start of its header (`include/git/read-cache-ll.h`).
const INDEX_MAGIC: [u8; 4] = *b"DIRC";

/// Where the entry count sits in the index header.
const INDEX_COUNT_AT: usize = 8;



/// What:
///  How many entries an index file holds,
/// or nothing when the file does not start like an index.
/// Why:
///   The count is the header's,
/// so a file that cannot be read or does not begin with the index magic proves nothing,
/// and a count of zero says the tree the checkout wrote has nothing in it to write into the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const header = readSync(index).subarray(0, 12); return header.toString('latin1', 0, 4) === 'DIRC' ? header.readUInt32BE(8) : undefined;
/// ```
fn index_entry_count(index: &Path) -> Option<usize> {
    let mut header = [0_u8; INDEX_HEADER_BYTES];
    let mut file = std::fs::File::open(index).ok()?;
    std::io::Read::read_exact(&mut file, &mut header).ok()?;
    if header[..INDEX_MAGIC.len()] != INDEX_MAGIC {
        return None;
    }
    let count: [u8; 4] = header[INDEX_COUNT_AT..INDEX_HEADER_BYTES].try_into().ok()?;
    return Some(u32::from_be_bytes(count) as usize);
}

/// What:
///  Whether the checkout at the destination is proven whole or gone,
/// which is when nothing is left for a person
///  to finish.
/// Why:
///   Recovery never runs Git and never waits,
/// so the proof is what the two directories say:
/// the destination
///  is gone,
/// its administrative directory holds an index,
/// and no `index.lock` stands in the way of the next Git
///  write there.
/// Then the index decides what a whole checkout holds beside Git's pointer:
/// an index with entries belongs to
///  a tree with files,
/// so a destination holding only its `.git` file is the shape a kill between real Git and
///  the checkout leaves;
/// an index with no entries belongs to a tree with nothing to write,
/// so a destination holding only its
///  `.git` file is whole there,
/// which is what a worktree at a commit with an empty tree looks like after
///  every step finished.
/// A lock with no index beside it is the shape a kill inside the index install leaves.
/// What the proof cannot see:
/// a process filling the destination beside an index that was written before it started
///  (another program's `git read-tree` without `-u`,
/// then a copy),
/// and the contents of the files it counts.
/// Git's
///  own `read-tree -u` and `reset --hard` hold `index.lock` for as long as they write,
/// so the lock
///  refuses those while they run,
/// and one such command writes the index last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// !exists(destination) || (exists(join(admin, 'index')) && !exists(join(admin, 'index.lock')) && (entryCount(index) === 0 || entries(destination).length > 1))
/// ```
fn add_is_finished(destination: &Path, admin: &Path) -> bool {
    // `match` on the lookup: a destination that is gone is nothing to finish, while one that cannot be examined is kept.
    match std::fs::symlink_metadata(destination) {
        Ok(_present) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(_refused) => return false,
    }
    if !admin.join(INDEX).exists() || admin.join(INDEX_LOCK).exists() {
        return false;
    }
    // An index with no entries is a tree with nothing to check out, so Git's pointer alone is the whole worktree.
    if index_entry_count(admin.join(INDEX).as_path()) == Some(0) {
        return true;
    }
    // More than the `.git` file Git wrote: a checkout put something beside it.
    let held_listing = match std::fs::read_dir(destination) {
        Ok(listing) => listing,
        Err(_unreadable) => return false,
    };
    return held_listing.count() > 1;
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = readFileSync(join(destination, '.git'), 'utf8'); return line.startsWith('gitdir: ') ? resolve(destination, line.slice(8).trim()) : undefined;
/// ```
pub(crate) fn registration_directory(destination: &Path) -> Option<PathBuf> {
    // Git's own reader, on bytes (`worktree_add_binding::pointer_target`): the rest of the file after the prefix, with only
    //  its trailing newlines removed, so a path holding a newline is whole and the remedy never names another repository.
    let pointer: Vec<u8> = std::fs::read(destination.join(POINTER)).ok()?;
    let resolved: PathBuf = crate::worktree_add_binding::pointer_target(destination, pointer.as_slice()).ok()?;
    return resolved.is_dir().then_some(resolved);
}

/// What:
///  What finishes or removes an interrupted add,
/// with the stale lock named first when one stands in the way.
/// Why:
///   `git -C <destination> reset --hard` cannot start while that lock exists,
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
pub(crate) fn finish_instructions(destination: &Path, admin: &Path) -> String {
    // Both paths are read by the person's shell, so they are quoted the way a shell reads a word back, not the way JSON
    //  spells a string: `$bucket` and a backquote inside double quotes would be expanded by the shell that runs `rm`.
    let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
    let finish: String = format!("`git -C {shown} reset --hard` completes its checkout, and `git worktree remove --force {shown}` removes it.");
    let lock: PathBuf = admin.join(INDEX_LOCK);
    if !lock.exists() {
        return finish;
    }
    let lock_shown: String = crate::shell_quote::shell_quote(lock.as_os_str().as_bytes());
    return format!("Its Git directory holds the lock {lock_shown} that a Git killed inside the index install left, and every Git write to the worktree fails while it stands: once no Git command is running there, `rm {lock_shown}` removes it, and then {finish}");
}

/// What:
///  Report an add that was interrupted before it was marked complete,
/// and change nothing.
/// Why:
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
    let marker: std::io::Result<bool> = match std::fs::symlink_metadata(record.with_extension(&COMPLETE_SUFFIX[1..])) {
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
pub fn settle_marker(record: &Path, marker: std::io::Result<bool>) -> Option<Settlement> {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    return match marker {
        Ok(true) => Some(Settlement { notice: None, keep: false }),
        Ok(false) => None,
        Err(error) => Some(Settlement {
            notice: Some(format!("monochromatic-git-policy-cli: whether the `git worktree add` of {shown_record} completed cannot be checked ({error}); nothing was changed and the record stays.")),
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
pub fn settle_text(record: &Path, read: std::io::Result<String>) -> Settlement {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    let text: String = match read {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Settlement { notice: None, keep: false },
        Err(error) => {
            return Settlement {
                notice: Some(format!("monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` cannot be read ({error}); nothing was changed and the record stays.")),
                keep: true,
            };
        }
    };
    return match decode_record(text.as_str()) {
        None => Settlement {
            notice: Some(format!("monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` is unreadable; nothing was changed.")),
            keep: false,
        },
        Some(Recorded::Planned(destination)) => {
            // The path is printed inside commands the person runs, so it is quoted for the shell.
            // The path is printed inside commands the person runs, so it is quoted for the shell.
            let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
            // Nothing at the path, or nothing Git's pointer there: the add made nothing that is left to finish.
            if !destination.join(POINTER).exists() {
                return Settlement { notice: None, keep: false };
            }
            // A kill between real Git's registration and the wrapper's record of the binding leaves the pointer without the
            //  plan, so the pointer is what names the administrative directory the proof reads.
            let registered: Option<PathBuf> = registration_directory(destination.as_path());
            if let Some(admin) = registered.as_ref()
                && add_is_finished(destination.as_path(), admin.as_path())
            {
                return Settlement { notice: None, keep: false };
            }
            let notice: String = match registered.as_ref() {
                Some(admin) => format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. {}",
                    finish_instructions(destination.as_path(), admin)
                ),
                None => format!("monochromatic-git-policy-cli: an interrupted `git worktree add` may have left {shown}; nothing proves it is the add's, so it was not changed. `git worktree list` shows whether it is registered, and `git worktree remove --force {shown}` removes it."),
            };
            // The record stays while something stands at the recorded path that is not proven whole, so every later command
            //  that runs the recovery says the same instead of leaving the obligation in one command's output.
            Settlement { notice: Some(notice), keep: true }
        }
        Some(Recorded::Registered(binding)) => {
            let shown: String = crate::shell_quote::shell_quote(binding.destination.as_os_str().as_bytes());
            // A worktree that is whole or gone settles silently; everything else keeps its record so the next command says
            //  the same again, because nothing else names the worktree, the stage or what finishes them.
            if add_is_finished(binding.destination.as_path(), binding.admin.as_path()) {
                return Settlement { notice: None, keep: false };
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

/// What:
///  Removes the temporaries that dead writers left in the journal root.
/// Why:
///   A write cut off before its rename leaves a file nothing else names.
/// The writer is in the file's name,
/// so the sweep
///  removes only what a dead process left and can never delete a temporary a live writer is about to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(root)) if (name.endsWith('.tmp') && ownerIsDead(writerOf(name))) await rm(join(root, name), { force: true });
/// ```
pub fn remove_dead_temporaries(root: &Path, verdicts: &StageVerdicts) -> Result<(), WorktreeCopyError> {
    for name in &verdicts.dead_temporaries {
        remove_file_if_present(root.join(name).as_path())?;
    }
    return Ok(());
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
        if name.ends_with(TEMPORARY_SUFFIX) {
            temporaries.push((name, entry.path()));
        }
    }
    temporaries.sort_by(by_temporary_name);
    for (name, path) in temporaries {
        // The name is `<item>.<writer>.<unique>.tmp`; a name of another shape has no writer and counts as a dead one's.
        if token_owner_is_dead(temporary_writer_token(name.as_str()), path.as_path())? {
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
/// A stage an add left is never removed,
/// and its claim stays while it stands,
/// so every command that runs the
///  recovery names it again instead of leaving the name in one command's output;
/// the claim goes when the person
///  has removed the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reapAbandonedStages(commonDir);
/// ```
pub fn reap_abandoned_stages(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    return reap_judged_stages(common_dir, &judge_stage_owners(common_dir)?);
}

/// `reap_abandoned_stages` on verdicts that were made before the command changed anything;
/// a claim or a temporary that the verdicts
///  do not name as dead is left whole.
pub fn reap_judged_stages(common_dir: &Path, verdicts: &StageVerdicts) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let names: Vec<String> = claim_names(root.as_path())?;
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
            let present: bool = match std::fs::symlink_metadata(container.as_path()) {
                Ok(_) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(raw_failure(&error)),
            };
            let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
            // The claim of a worktree add names a stage by its path alone, which proves nothing about what stands there now
            // (a snapshot, a directory another process put there, or nothing): such a stage is reported and never removed.
            // A record that cannot be looked up counts as one: when in doubt the stage is kept.
            let of_an_add: bool = !matches!(std::fs::symlink_metadata(record.as_path()), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound);
            let mut stage_stays: bool = false;
            if present && !journaled.contains(&container) && of_an_add {
                notices.push(format!(
                    "monochromatic-git-policy-cli: the stage {} was left by an interrupted `git worktree add` and was not changed; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).",
                    quote(container.to_string_lossy().as_ref())
                ));
                stage_stays = true;
            } else if present && !journaled.contains(&container) {
                assert_private_path(container.as_path(), Role::PrivateStage)?;
                remove_stage(container.as_path())?;
                notices.push(format!(
                    "monochromatic-git-policy-cli: removed the stage {} that an interrupted ignored-state copy left behind.",
                    quote(container.to_string_lossy().as_ref())
                ));
            }
            let settled: Settlement = settle_interrupted_add(record.as_path());
            if let Some(notice) = settled.notice {
                notices.push(notice);
            }
            // A stage of an add is never removed, so its claim is what names it: the record, the marker and the claim stay while
            //  the stage or the incomplete worktree is still there, and the next command reports them again.
            if settled.keep || stage_stays {
                continue;
            }
            remove_file_if_present(record.as_path())?;
            remove_file_if_present(claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
            remove_file_if_present(claim.as_path())?;
        }
    }
    remove_dead_temporaries(root.as_path(), verdicts)?;
    absent_is_fine(sync_directory(root.as_path()))?;
    return Ok(notices);
}
