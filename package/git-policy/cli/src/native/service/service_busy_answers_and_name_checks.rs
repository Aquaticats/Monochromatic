//! What:
//!  The shapes the landing board answers with (an early answer,
//! a group admission,
//! a naming,
//! a decision) and the checks and
//!  readers of the names it keeps:
//! which identities may name a file,
//! the withdrawal token of a decision,
//! which entries are
//!  confirmations.
//! Why:
//!  `service_busy.rs` holds the board itself;
//! these need no board,
//! so they are a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type EarlyAnswer, GroupAdmission, Naming, Decision; function isPlain(id), withdrawalToken(content)
//! ```

use std::path::PathBuf;
use std::time::SystemTime;

/// What the decision file says when a withdrawal took the request:
/// this word,
/// a colon and the token of the withdrawer that made it.
pub(crate) const WITHDRAWN: &str = "withdrawn";

/// The extension of the file that confirms a withdrawal,
/// named `<id>.<token>.cancelled` with the token the decision carries:
/// the
///  withdrawer that won the decision made it after it asked the journal a second time and found no batch naming the request.
/// A
///  decision without its confirmation is provisional:
/// nobody but its creator may rely on it.
/// The token ties the confirmation to
///  that one decision,
/// so a confirmation that outlives its decision (the retention removes the pair in two steps) confirms nothing
///  of the decision that replaces it.
pub(crate) const CANCELLED_EXTENSION: &str = "cancelled";

/// The longest request identity the record keeps a decision for;
/// a longer one,
/// or one with a character outside letters,
/// digits and
///  hyphens,
/// gets no early answer,
/// because its name could not be made into a file name that is only a name.
pub(crate) const LONGEST_ID: usize = 80;

/// How a withdrawal asked of a connection thread is answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EarlyAnswer {
    /// The landing thread is not stuck,
    /// or the request is the landing thread's,
    /// or nothing could be established:
    /// the withdrawal goes
    ///  to the landing thread as before.
    Queued,
    /// The request had landed:
    /// the commit is the answer.
    Landed(String),
    /// The request has not begun to land and never will.
    Withdrawn(String),
}

/// What the landing thread learns about the requests of a group before it lands them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GroupAdmission {
    /// The requests a withdrawal took:
    /// dropped,
    /// and answered to nobody,
    /// because their clients were told already.
    pub dropped: Vec<String>,
    /// The requests whose admission could not be established:
    /// answered with a failure and not landed.
    pub unavailable: Vec<String>,
}

/// What the journal says about a request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Naming {
    /// A finished batch names the request,
    /// with the commit it became.
    Landed(String),
    /// A batch still open names it:
    /// its commit may be on the branch.
    Open,
    /// No batch names it.
    No,
    /// The journal could not be read whole,
    /// so nothing is known.
    Unknown,
}

/// What a try at the decision came to.
pub(crate) enum Decision {
    /// This side made the decision.
    Won,
    /// The other side made it first,
    /// and wrote this.
    Lost(String),
}

/// Whether `id` can be a file name that is only a name.
pub(crate) fn is_plain(id: &str) -> bool {
    return !id.is_empty() && id.len() <= LONGEST_ID && id.chars().all(is_plain_character);
}

/// What:
///  Whether a character may appear in an identity that names a file:
/// an ASCII letter or digit,
/// or a hyphen.
/// Why:
///  `is_plain` checks every character of an identity with it,
/// so no identity can name a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainCharacter = (character: string) => /^[A-Za-z0-9-]$/.test(character);
/// ```
pub(crate) fn is_plain_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '-';
}

/// What:
///  Whether a directory entry is the confirmation of a withdrawal:
/// a file with the cancelled extension.
/// Why:
///  The count and the pruning of confirmed withdrawals both filter the directory listing with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isConfirmation = (entry: Dirent) => path.extname(entry.name) === '.' + CANCELLED_EXTENSION;
/// ```
pub(crate) fn is_confirmation(entry: &std::fs::DirEntry) -> bool {
    return entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some(CANCELLED_EXTENSION);
}

/// What:
///  The modification time and the path of a directory entry,
/// nothing when either cannot be read.
/// Why:
///  `prune_withdrawals` sorts the confirmations by age,
/// oldest first,
/// and entries that cannot be measured are left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modifiedAndPath = (entry: Dirent) => { try { return [statSync(p).mtime, p]; } catch { return undefined; } };
/// ```
pub(crate) fn modified_and_path(entry: std::fs::DirEntry) -> Option<(SystemTime, PathBuf)> {
    return Some((entry.metadata().ok()?.modified().ok()?, entry.path()));
}

/// The token a `withdrawn` decision carries,
/// when `content` is one.
pub(crate) fn withdrawal_token(content: &str) -> Option<&str> {
    // The word, a colon, and a token that is not empty; anything else is no withdrawal.
    let rest: &str = content.strip_prefix(WITHDRAWN)?;
    let token: &str = rest.strip_prefix(':')?;
    if token.is_empty() {
        return None;
    }
    return Some(token);
}
