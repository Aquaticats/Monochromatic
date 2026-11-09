//! What:
//!  The name one command holds a notice's record under while it shows it,
//! who may take such a name over,
//! and how a
//!  notice goes back to its own name when it could not be shown.
//! Why:
//!  A record is shown once,
//! and two commands that read it before either showed it must not both print it;
//! the
//!  rename is the only claim the file system gives atomically,
//! so the winner is the command whose rename succeeded.
//!  A command killed while it holds a name leaves it there,
//! and a later command takes it over once the holder is provably
//!  gone,
//! which is why the name carries the holder's process number and the kernel's start time for it:
//! a number
//!  alone is reused.
//! A name whose holder cannot be judged,
//! because `/proc` cannot be read or the name cannot be parsed,
//! is
//!  taken over:
//! showing a notice twice costs a repeat,
//! while never showing it loses the person's only word about a
//!  commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = (record, token) => rename(record, `${record}.${token}.claim`);
//! const taken = (claim) => isAlive(ownerOf(claim)) ? undefined : rename(claim, `${recordOf(claim)}.${ourToken}.claim`);
//! ```

use crate::process_tree::member_of;
use std::path::{Path, PathBuf};

/// The end of the name a command holds a record under while it shows it.
pub const CLAIM_SUFFIX: &str = ".claim";

/// The end of a record's own name.
const RECORD_SUFFIX: &str = ".txt";

/// Separates the parts of a claim's owner,
/// and is a character neither a process number nor a start time contains.
const FIELD: char = '~';

/// What:
///  The names one command's claims are made from.
/// Why:
///   Two commands of the same process must not take each other's claims,
/// so the name carries a random part as
///  well as the process number and its start time;
/// and no two claims of one command share a name either,
/// because one
///  command takes a fresh name for every record it claims and every claim it takes over,
/// so a claim can never be
///  renamed over another claim of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${process.pid}~${startTimeOf(process.pid)}~${randomUUID().slice(0, 8)}` for each claim, with a counter
/// ```
#[derive(Debug)]
pub struct ClaimToken {
    /// The part that names this command:
    /// its process number,
    /// the kernel's start time for it,
    /// and a random part.
    command: String,
    /// How many claims this command has made,
    /// so every one of them has a name of its own.
    made: std::sync::atomic::AtomicU64,
}

impl ClaimToken {
    /// What:
    ///  The names this command's claims are made from.
    /// Why:
    ///   The process number and its start time prove later which process a claim belongs to,
    /// the random part keeps
    ///  two commands of one process apart,
    /// and the counter keeps two claims of one command apart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimToken(`${process.pid}~${started}~${randomUUID().slice(0, 8)}`, 0)
    /// ```
    pub fn of_this_command() -> ClaimToken {
        let number: u32 = std::process::id();
        let started: u64 = member_of(i32::try_from(number).unwrap_or(0)).map_or(0, started_at);
        let part: String = match crate::random_id::random_uuid() {
            Ok(id) => id.chars().take(8).collect(),
            // A command with no random source still claims under its number and start time alone.
            Err(_) => String::from("command"),
        };
        return ClaimToken { command: format!("{number}{FIELD}{started}{FIELD}{part}"), made: std::sync::atomic::AtomicU64::new(0) };
    }

    /// The name of the next claim of this command,
    /// for the record named `record`.
    fn name_for(&self, record: &str) -> String {
        let nth: u64 = self.made.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        return format!("{record}.{}{FIELD}{nth}{CLAIM_SUFFIX}", self.command);
    }
}

/// What:
///  Moves `from` to `to` only when `to` does not exist.
/// Why:
///   `rename` replaces its destination,
/// so a claim renamed over a newer record or over another claim would delete a
///  statement nobody has read;
/// `renameat2` with `RENAME_NOREPLACE` fails instead.
/// On a system without it the move is a hard link
///  followed by the removal of the old name,
/// which fails on an existing destination and leaves both names only if the process dies
///  between them,
/// which costs a repeat and loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(from, to) // renameat2(RENAME_NOREPLACE), or link(from, to) then unlink(from)
/// ```
fn move_without_replacing(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        return rustix::fs::renameat_with(rustix::fs::CWD, from, rustix::fs::CWD, to, rustix::fs::RenameFlags::NOREPLACE)
            .map_err(std::io::Error::from);
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::fs::hard_link(from, to)?;
        return std::fs::remove_file(from);
    }
}

/// What:
///  Takes the record at `path` under this command's name,
/// or answers nothing when another command took it first.
/// Why:
///   The rename is atomic:
/// only the command whose rename succeeded holds the record,
/// and it is the one that shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await rename(path, path + '.' + token + '.claim'); return claimPath } catch { return undefined }
/// ```
pub fn claim(path: &Path, token: &ClaimToken) -> Option<PathBuf> {
    let name: String = path.file_name()?.to_string_lossy().into_owned();
    let claimed: PathBuf = path.with_file_name(token.name_for(name.as_str()));
    if move_without_replacing(path, claimed.as_path()).is_err() {
        return None;
    }
    return Some(claimed);
}

/// What:
///  The record name a claim's name was made from,
/// or nothing when `name` is not a claim's.
/// Why:
///   The reading sorts notices by the record they came from,
/// and a claim is recognised by its ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = name.lastIndexOf('.txt.'); return name.endsWith('.claim') && at > 0 ? name.slice(0, at + 4) : undefined
/// ```
pub fn record_of_claim(name: &str) -> Option<&str> {
    let owned: &str = name.strip_suffix(CLAIM_SUFFIX)?;
    let at: usize = owned.rfind(RECORD_SUFFIX)?;
    return Some(&owned[..at + RECORD_SUFFIX.len()]);
}

/// What:
///  Whether the command that claims `name` is still the process the name was made for.
/// Why:
///   A claim left by a killed command is the person's only remaining word about the commit,
/// and a claim of a living
///  command belongs to that command alone.
/// The process number and the kernel's start time for it are compared,
/// and a name whose owner cannot be
///  judged is gone as far as this command is concerned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pid, started] = ownerOf(name); return startedAt(pid) === started
/// ```
pub fn claimed_by_a_live_command(name: &str) -> bool {
    let Some((number, started)) = owner_of(name) else {
        return false;
    };
    let Ok(pid) = i32::try_from(number) else {
        return false;
    };
    return match member_of(pid) {
        Some(member) => member.started == started,
        None => false,
    };
}

/// The kernel's start time for a process,
/// as the process table reports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (member) => member.started
/// ```
fn started_at(member: crate::process_tree::TreeMember) -> u64 {
    return member.started;
}

/// The process number and the kernel's start time a claim's name carries,
/// or nothing when it carries none this release can read.
fn owner_of(name: &str) -> Option<(u32, u64)> {
    let owned: &str = name.strip_suffix(CLAIM_SUFFIX)?;
    let after: &str = owned.rsplit_once(&format!("{RECORD_SUFFIX}."))?.1;
    let (number, rest) = after.split_once(FIELD)?;
    let (started, _made) = rest.split_once(FIELD)?;
    return Some((number.parse().ok()?, started.parse().ok()?));
}

/// What:
///  Takes a claim whose command is gone under this command's name,
/// or answers nothing when another command took it first.
/// Why:
///   Whoever wins the second rename shows the words nobody has read,
/// and a claim taken over this way keeps the
///  record it was made from in its own name,
/// so a later reader still knows which record it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await rename(claim, claim.slice(0, claim.lastIndexOf('.')) + '.' + token + '.claim'); return taken } catch { return undefined }
/// ```
pub fn take_over(path: &Path, token: &ClaimToken) -> Option<PathBuf> {
    let name: String = path.file_name()?.to_string_lossy().into_owned();
    let record: &str = record_of_claim(name.as_str())?;
    let claimed: PathBuf = path.with_file_name(token.name_for(record));
    if move_without_replacing(path, claimed.as_path()).is_err() {
        return None;
    }
    return Some(claimed);
}

/// What:
///  Gives a record this command claimed back its own name,
/// after the command could not show it.
/// Why:
///   A record whose words could not be written must be found by the next command,
/// which may be of this same process;
/// a name left to the holder's liveness would wait for this command to die.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await rename(claim, record) } catch { /* the record's name is taken, so the claim stays */ }
/// ```
pub fn give_back(claim: &Path, record: &Path) {
    if let Err(error) = move_without_replacing(claim, record) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a notice whose words could not be written could not go back to its own name: {error}").as_str());
        }
        // The record's own name holds a newer statement, so this claim stays under its holder's name and is taken over
        // once this command is gone: that costs a repeat of an older statement and loses nothing.
    }
}
