//! What:
//!  The shapes the landing board answers with (an early answer,
//! a group admission,
//! a naming,
//! a decision) and the one check on the names it keeps:
//! which identities may be a decision's.
//! Why:
//!  `service_busy.rs` holds the board itself;
//! these need no board,
//! so they are a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type EarlyAnswer, GroupAdmission, Naming, Decision; function isPlain(id)
//! ```


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
    /// The requests this service will not land because it took no place of its own in its journal,
    /// so it cannot be sure that no other service would land one it promises never to land.
    pub refused: Vec<String>,
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
    /// and its decision carries this token.
    Lost(String),
}

/// Whether `id` can be a decision's key.
pub(crate) fn is_plain(id: &str) -> bool {
    return !id.is_empty() && id.len() <= LONGEST_ID && id.chars().all(is_plain_character);
}

/// The longest request identity the record keeps a decision for.
/// Why:
///   The identity is the key of a decision,
/// and a request of any other shape is not one a client makes;
/// a longer one costs a decision's memory and nothing else.
pub(crate) const LONGEST_ID: usize = 80;

/// What:
///  Whether a character may appear in an identity the record keeps a decision for:
/// an ASCII letter or digit,
/// or a hyphen.
/// Why:
///  `is_plain` checks every character of an identity with it,
/// and a request of any other shape is not one a client makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainCharacter = (character: string) => /^[A-Za-z0-9-]$/.test(character);
/// ```
pub(crate) fn is_plain_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '-';
}
