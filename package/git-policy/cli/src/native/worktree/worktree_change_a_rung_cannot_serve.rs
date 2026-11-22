//! What:
//!  One path the target tree of an add changes,
//!  and why a cheap rung cannot serve it:
//! a path the conversion fence must leave for Git to write,
//!  or a link whose copy no index entry can vouch for.
//! Why:
//!   Both rungs ask one question of the fence before they claim anything or copy anything,
//! and the person's sentence names the path and says what the rung does with it;
//! one type is what both rungs answer and what that sentence is made from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ChangeNotServed = { path: string; why: 'written' | 'link' };
//! ```

/// What:
///  One path the target tree changes that a rung cannot serve,
///  and why it cannot.
/// Why:
///   The skip names the path in the person's sentence and says what the rung does with it,
/// and two different sets of the fence answer it:
/// the snapshot rung's written paths,
///  and a clone rung's written paths together with its links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChangeNotServed = { path: string; why: 'written' | 'link' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeNotServed {
    /// The path,
    /// as the source's index names it.
    pub path: Vec<u8>,
    /// Why this rung cannot serve it.
    pub why: WhyNotServed,
}

/// What:
///  Why a rung cannot serve a changed path.
/// Why:
///   A path the fence hands to Git is written by Git's checkout,
/// whose bytes are the conversion of the blob;
/// a link is made again by the copies,
///  and no index entry can describe a link the copies made,
///  so `git read-tree -m -u` refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyNotServed = 'written' | 'link';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhyNotServed {
    /// The fence hands the path to Git,
    /// whose checkout writes other bytes than a copy of the source's file holds.
    TheFenceWritesIt,
    /// The copies make a link again,
    /// and no index entry can vouch for a link the copies made.
    ALinkIsCopied,
}
