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
///  The name of one command's claims,
/// made when that command first needs one.
/// Why:
///   Two commands of the same process must not take each other's claims,
/// so the name carries a random part as
///  well as the process number and its start time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${process.pid}~${startTimeOf(process.pid)}~${randomUUID().slice(0, 8)}`
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimToken(String);

impl ClaimToken {
    /// What:
    ///  A name only this command makes.
    /// Why:
    ///   The process number and its start time prove later which process a claim belongs to,
    /// and the random part keeps
    ///  two claims of the same process apart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimToken(`${process.pid}~${started}~${randomUUID().slice(0, 8)}`)
    /// ```
    pub fn of_this_command() -> ClaimToken {
        let number: u32 = std::process::id();
        let started: u64 = member_of(i32::try_from(number).unwrap_or(0)).map_or(0, |member| { return member.started; });
        let made: String = match crate::random_id::random_uuid() {
            Ok(id) => id.chars().take(8).collect(),
            // A command with no random source still claims under its number and start time alone.
            Err(_) => String::from("command"),
        };
        return ClaimToken(format!("{number}{FIELD}{started}{FIELD}{made}"));
    }

    /// The name of one claim of this command,
    /// for the record named `record`.
    fn name_for(&self, record: &str) -> String {
        return format!("{record}.{}{CLAIM_SUFFIX}", self.0);
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
    if std::fs::rename(path, claimed.as_path()).is_err() {
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
    return member_of(pid).is_some_and(|member| { return member.started == started; });
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
    if std::fs::rename(path, claimed.as_path()).is_err() {
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
    if std::fs::rename(claim, record).is_err() {
        // The record's own name is taken by a newer statement, so this claim stays under its holder's name and is taken
        // over once this command is gone.
        crate::diagnostic_log::debug("monochromatic-git-policy-cli", "a notice whose words could not be written stayed under its claim, where a later command finds it once this one is gone");
    }
}
