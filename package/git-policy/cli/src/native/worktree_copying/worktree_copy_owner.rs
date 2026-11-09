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

use crate::pid_namespace::{ForeignStanding, current_pid_namespace, foreign_owner_standing};
use crate::process_identity::{current_birth_identity, process_birth_identity};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure_of};
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
    ///  When the platform refuses to report the process,
    ///  or when the owner is in another namespace and its item is older than the
    ///  patience.
    pub fn is_alive(&self, item: &Path) -> Result<bool, WorktreeCopyError> {
        return self.alive_by(foreign_owner_standing(self.pid_namespace.as_deref(), item));
    }

    /// `is_alive` from a standing that was read already,
    /// so one decision never reads the age twice.
    fn alive_by(&self, standing: Option<ForeignStanding>) -> Result<bool, WorktreeCopyError> {
        match standing {
            Some(ForeignStanding::Refused(refused)) => return Err(WorktreeCopyError(refused.describe(self.pid))),
            Some(ForeignStanding::Standing) => return Ok(true),
            // The item of another namespace that its owner removed is a finished add's: nothing of it runs
            //  (cross-vendor review of 2026-11-09, finding 3).
            Some(ForeignStanding::Finished) => return Ok(false),
            None => {}
        }
        // `match` on the platform's answer: a live process with the recorded birth is the owner; no process is a dead owner.
        match process_birth_identity(self.pid).map_err(raw_failure_of)? {
            Some(current) => return Ok(current == self.birth_identity),
            None => return Ok(false),
        }
    }
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
/// An owner of another namespace whose item
///  is older than the patience is neither:
/// the command stops (owner,
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
    let Some(owner) = token.and_then(CopyOwner::from_token) else {
        return Ok(true);
    };
    let standing = foreign_owner_standing(owner.pid_namespace.as_deref(), item);
    if let Some(ForeignStanding::Refused(refused)) = &standing {
        return Err(WorktreeCopyError(refused.describe(owner.pid)));
    }
    // `match` on the proof: only a clean answer of "not running" is a dead owner; what the platform will not say is not proof.
    match owner.alive_by(standing) {
        Ok(alive) => return Ok(!alive),
        Err(_unproven) => return Ok(false),
    }
}

#[cfg(test)]
#[path = "worktree_copy_owner_tests.rs"]
mod tests;
