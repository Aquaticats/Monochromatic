//! What:
//!  The process that owns one item of a copy transaction (a journal,
//! a stage claim,
//! a temporary file),
//! written into the
//!  item's own name or text,
//! and the proof of whether that process still runs.
//! Why:
//!  Concurrent adds in one repository must never wait for one another,
//! so no lock stands between them.
//! An item has exactly
//!  one owner,
//! and any other command may touch an item only when its owner is proven dead:
//! the live copy of an add is never
//!  recovered or reaped,
//! a dead one is.
//! The owner is part of the item's name,
//! so taking over a dead owner's journal is one
//!  atomic rename (a single winner,
//! no lock file,
//! no stale lock to retire).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owner = currentCopyOwner(); // { pid, birthIdentity, pidNamespace }
//! const name = `${id}.${owner.token()}.json`;
//! if (!CopyOwner.fromToken(token).isAlive(path)) { /* the item may be taken over */ }
//! ```

use crate::pid_namespace::{ForeignStanding, UnreadableOwnerStanding, UnreadablePart, current_pid_namespace, foreign_owner_standing, unreadable_owner_standing};
use crate::process_identity::{birth_identity_value, current_birth_identity, process_birth_identity};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use std::path::Path;

/// What:
///  The process identity that owns an item:
/// the process id,
/// the identity of its birth,
/// and its process-id namespace.
/// Why:
///   A process id alone is reused;
/// the birth identity tells a reused id from the owner,
/// and the namespace tells an owner
///  this process cannot interpret from one that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyOwner = { pid: number; birthIdentity: string; pidNamespace?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyOwner {
    /// The owner's process id.
    pub pid: i64,
    /// The owner's birth identity as the platform reports it.
    pub birth_identity: String,
    /// The owner's process-id namespace,
    /// when the platform has one.
    pub pid_namespace: Option<String>,
}

/// The separator between the fields of an owner in a token;
/// it is never a character a field is written with unescaped.
const FIELD_SEPARATOR: char = '~';

/// The escape that introduces two hexadecimal digits for a byte a name cannot carry as it is.
const ESCAPE: char = '%';

/// Whether a byte stands in a token as it is:
/// letters,
/// digits and the few marks the identities use.
/// A dot is never one of them,
/// because a dot separates the parts of a journal's name.
fn is_plain(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'[' | b']');
}

/// A field of an owner as text a file name carries:
/// plain bytes as they are,
/// every other byte as `%` and two hexadecimal digits.
fn escape_field(field: &str) -> String {
    let mut written: String = String::with_capacity(field.len());
    for byte in field.bytes() {
        if is_plain(byte) {
            written.push(char::from(byte));
        } else {
            written.push(ESCAPE);
            written.push_str(crate::hexadecimal_of_bytes::hexadecimal_of_byte(byte).as_str());
        }
    }
    return written;
}

/// The field an escaped text names,
/// or nothing when an escape is cut short,
/// is not hexadecimal,
/// or the bytes are not UTF-8.
fn unescape_field(text: &str) -> Option<String> {
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len());
    let mut remaining = text.bytes();
    while let Some(byte) = remaining.next() {
        if char::from(byte) == ESCAPE {
            let high: u8 = remaining.next()?;
            let low: u8 = remaining.next()?;
            let pair: [u8; 2] = [high, low];
            bytes.push(u8::from_str_radix(std::str::from_utf8(&pair).ok()?, 16).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    return String::from_utf8(bytes).ok();
}

/// What:
///  The owner of the running process.
/// Why:
///   Every item this process creates names it,
/// so a later command can tell a live copy from an abandoned one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentCopyOwner(): CopyOwner;
/// ```
///
/// # Errors
///  When the platform cannot report the running process's birth identity.
pub fn current_copy_owner() -> Result<CopyOwner, WorktreeCopyError> {
    let birth: String = match current_birth_identity() {
        Ok(found) => found,
        Err(error) => return Err(failure_with_cause("monochromatic-git-policy-cli: the identity of this process is unavailable, so a copy cannot name its owner.", Some(&error))),
    };
    return Ok(CopyOwner {
        pid: i64::from(std::process::id()),
        birth_identity: birth,
        pid_namespace: current_pid_namespace(),
    });
}

/// What:
///  The identity of `pid` as an owner a record can name,
/// or nothing when the platform will not report it.
/// Why:
///   A record of a worktree add names the Git process the add started,
/// so a later command can tell an add whose
///  Git still runs from one whose Git is gone:
/// the command still holds the child unreaped,
///  so its number cannot mean another process here,
/// and the birth identity is what a later reader compares after the
///  child is gone and its number is free again.
/// A process the platform will not report leaves the record unnamed,
/// and
///  the settlement's own bound covers that case rather than keeping the record forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = CopyOwner.ofProcess(pid); // undefined when /proc does not answer
/// ```
pub fn copy_owner_of_process(pid: u32) -> Option<CopyOwner> {
    let id: i64 = i64::from(pid);
    let birth: String = process_birth_identity(id).ok().flatten()?;
    return Some(CopyOwner { pid: id, birth_identity: birth, pid_namespace: current_pid_namespace() });
}

impl CopyOwner {
    /// The owner as text that holds no dot,
    /// slash,
    /// space or other character a file name's parts cannot carry,
    /// in about forty
    /// characters for a Linux process.
    pub fn token(&self) -> String {
        let namespace: String = escape_field(self.pid_namespace.as_deref().unwrap_or(""));
        return format!("{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{namespace}", self.pid, escape_field(self.birth_identity.as_str()));
    }

    /// The owner a token names,
    /// or nothing when the text is no token (a name an older release wrote,
    /// or a stranger's).
    pub fn from_token(token: &str) -> Option<CopyOwner> {
        let mut fields = token.split(FIELD_SEPARATOR);
        let pid: i64 = fields.next()?.parse::<i64>().ok()?;
        if pid < 1 {
            return None;
        }
        let birth: String = unescape_field(fields.next()?)?;
        if birth.is_empty() {
            return None;
        }
        let namespace: String = unescape_field(fields.next()?)?;
        if fields.next().is_some() {
            return None;
        }
        return Some(CopyOwner {
            pid,
            birth_identity: birth,
            pid_namespace: if namespace.is_empty() { None } else { Some(namespace) },
        });
    }

    /// The error to stop with when this owner is in another namespace and its item is older than the patience:
    /// neither alive nor
    ///  dead (owner,
    /// 2026-11-04,
    /// "Refuse and name it (Recommended)").
    pub fn expired_foreign_refusal(&self, item: &Path) -> Option<WorktreeCopyError> {
        return match foreign_owner_standing(self.pid_namespace.as_deref(), item) {
            Some(ForeignStanding::Refused(refused)) => Some(WorktreeCopyError(refused.describe(self.pid))),
            Some(ForeignStanding::Standing | ForeignStanding::Finished) | None => None,
        };
    }

    /// What:
    ///  Whether the owner still runs.
    /// Why:
    ///   An owner in a PID namespace this process cannot see is judged by how recently its item changed (`item`'s age),
    ///  because its process id means nothing here;
    /// any other owner is alive when a process with its id and its birth exists.
    /// The
    ///  age is read once for the decision.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// owner.isAlive(itemPath);
    /// ```
    ///
    /// # Errors
    ///  When the owner is in another namespace and its item is older than the patience,
    ///  or when a part of its record is no value this reader can read and the item is older than the patience.
    ///  A process the platform will not report on is not an error: it is a holder that cannot be told, and the item
    ///  stays (`Ok(true)` inside the patience, the words past it).
    pub fn is_alive(&self, item: &Path) -> Result<bool, WorktreeCopyError> {
        // What: the answer that keeps the item is "may run", and the one that takes it is "ended"; neither is a proof of
        //       the other, so the third answer carries the words a command stops with.
        // Why:  `token_owner_is_dead` and every recovery sweep read this one decision, so the three answers are made here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (owner.standing(itemPath)) { case 'may-run': return true; case 'ended': return false; case 'refused': throw refused; }
        // ```
        return match self.alive_by(item, foreign_owner_standing(self.pid_namespace.as_deref(), item))? {
            OwnerStanding::MayRun => Ok(true),
            OwnerStanding::Ended => Ok(false),
            OwnerStanding::CannotBeTold(refusal) => Err(refusal),
        };
    }

    /// `is_alive` from a standing that was read already,
    /// so one decision never reads the age twice.
    fn alive_by(&self, item: &Path, standing: Option<ForeignStanding>) -> Result<OwnerStanding, WorktreeCopyError> {
        match standing {
            Some(ForeignStanding::Refused(refused)) => return Err(WorktreeCopyError(refused.describe(self.pid))),
            Some(ForeignStanding::Standing) => return Ok(OwnerStanding::MayRun),
            // The item of another namespace that its owner removed is a finished add's: nothing of it runs
            //  (cross-vendor review of 2026-11-09, finding 3).
            Some(ForeignStanding::Finished) => return Ok(OwnerStanding::Ended),
            None => {}
        }
        // `match` on the platform's answer: a live process with the recorded birth is the owner; no process is a dead owner.
        return match process_birth_identity(self.pid) {
            Ok(Some(current)) => match (birth_identity_value(self.birth_identity.as_str()), birth_identity_value(current.as_str())) {
                // What: the two births name one value or two, and a value is what names one process.
                // Why:  The wrapper writes one spelling of its birth, so a record in another one came from a writer that is
                //       not this release (an older release, another tool, a damaged file). Compared as text, that spelling
                //       read as another process, and another process is a proved-dead owner: what a running command holds
                //       went with it.
                (Some(recorded), Some(live)) => Ok(if recorded == live { OwnerStanding::MayRun } else { OwnerStanding::Ended }),
                // What: a birth that names no value is neither a live owner's nor a dead one's, and the item's own age is
                //       the only bound there is.
                // Why:  Inside the patience the record stands for an owner that may run, past it the command stops with the
                //       item named, and neither answer takes an item of a command that runs nor keeps one for ever.
                _ => Ok(self.cannot_be_told_by(item, UnreadablePart::Birth(self.birth_identity.clone()))),
            },
            Ok(None) => Ok(OwnerStanding::Ended),
            // What: a process the platform will not report on is a holder that cannot be told, like a birth that names no
            //       value: the item stays and nothing fails.
            // Why:  Every `git worktree` command runs this recovery first, so an error here failed commands that were never
            //       asked about the record: a `git worktree add` stopped and made no worktree where the item proves
            //       nothing, and the error named no record and no command a person could run. The item's own age is the
            //       bound, so inside the patience nothing is said and past it the words name the item.
            Err(unreadable) => Ok(self.cannot_be_told_by(item, UnreadablePart::Process(unreadable.to_string()))),
        };
    }

    /// What:
    ///  What the item says when a part of the record is no value this reader can read.
    /// Why:
    ///   The reading of the item's age and the words are the ones every reader of such a record gives
    ///  (`unreadable_owner_standing`), so a copy cannot answer one way and a transaction another.
    fn cannot_be_told_by(&self, item: &Path, part: UnreadablePart) -> OwnerStanding {
        return match unreadable_owner_standing(item, part, self.pid) {
            UnreadableOwnerStanding::MayRun => OwnerStanding::MayRun,
            UnreadableOwnerStanding::Ended => OwnerStanding::Ended,
            UnreadableOwnerStanding::Refused(words) => OwnerStanding::CannotBeTold(WorktreeCopyError(words)),
        };
    }
}

/// What:
///  What a copy owner's record says about the owner.
/// Why:
///   A reader of these records answers one of three things,
///  and the third is not a yes or a no:
///  the owner runs,
///  it ended,
///  or the record names no value that would tell the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerStanding = 'may-run' | 'ended' | { refused: WorktreeCopyError };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerStanding {
    /// The owner runs, or nothing proves it ended.
    MayRun,
    /// The owner ended, or the number names a process of another birth.
    Ended,
    /// Neither: the item is named in the error a command stops with.
    CannotBeTold(WorktreeCopyError),
}

/// What:
///  Whether the owner a token names is proven dead;
/// an absent or unreadable token names nobody,
/// so nobody can be alive there.
/// Why:
///   Items written before owners existed,
/// and strangers' files,
/// must not stay in the journal root forever;
/// and an owner the
///  platform will not report on is not proven dead,
/// so its item is kept rather than taken.
/// A record whose birth is no value this reader can read is neither a live owner's nor a dead one's:
///  inside the patience it counts as one that may run,
///  and past it the command stops with the item named
///  (owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"),
/// from one
///  reading of the item's age,
/// so that a refusal is never swallowed as "not dead".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerIsDead(token: string | undefined, itemPath: string): boolean; // throws when the owner is of another namespace and the item is older than the patience
/// ```
pub fn token_owner_is_dead(token: Option<&str>, item: &Path) -> Result<bool, WorktreeCopyError> {
    return match token_owner_standing(token, item)? {
        OwnerStanding::MayRun => Ok(false),
        OwnerStanding::Ended => Ok(true),
        // The refusal is never swallowed here: a caller that only asks "is it dead" learns that it is neither.
        OwnerStanding::CannotBeTold(refusal) => Err(refusal),
    };
}

/// What:
///  What the owner a token names is, with the third answer kept apart from a refusal of another PID namespace.
/// Why:
///   The two answers that are neither "may run" nor "ended" are not read the same way:
///  a record of another PID namespace past the patience stops the command
///  (owner, 2026-11-04, "Refuse and name it (Recommended)"), which is the error `token_owner_standing` returns,
///  while a record whose holder can be shown neither alive nor gone is kept and the words naming it go to the person,
///  and the command goes on
///  (owner, 2026-11-16, "A: stays, command says how (Recommended)").
///  A caller that would go on beside the second must not go on beside the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerStanding(token: string | undefined, itemPath: string): 'may-run' | 'ended' | { cannotBeTold: string }; // throws for a foreign refusal
/// ```
pub fn token_owner_standing(token: Option<&str>, item: &Path) -> Result<OwnerStanding, WorktreeCopyError> {
    let Some(owner) = token.and_then(CopyOwner::from_token) else {
        return Ok(OwnerStanding::Ended);
    };
    return owner.alive_by(item, foreign_owner_standing(owner.pid_namespace.as_deref(), item));
}

#[cfg(test)]
#[path = "worktree_copy_owner_tests.rs"]
mod tests;
