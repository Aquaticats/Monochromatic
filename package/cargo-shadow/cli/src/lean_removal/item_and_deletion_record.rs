//! What:
//! The two records a removal keeps:
//! the item as it was judged,
//! and what a completed deletion did.
//! Why:
//! The claim compares against the recorded identity,
//! and the proof line reports the bytes and the modes the deletion changed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Item = { path: string; identity: Identity };
//! type Deleted = { bytes: number; modesAdded: string[]; remaining: string[] };
//! ```
/// Imports the device and inode an item was judged by.
use crate::lean_identity::Identity;
/// Imports the owned path type both records name a path with.
use std::path::PathBuf;
/// One thing to remove,
/// with the identity it had when it was classified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    /// Where it is.
    pub path: PathBuf,
    /// What it was when judged.
    pub identity: Identity,
}

/// What a completed deletion did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Deleted {
    /// The disk bytes of the files whose last link in this walk went.
    pub bytes: u64,
    /// The directories whose owner read,
    /// write and search permission this deletion added so it could remove what they held.
    pub modes_added: Vec<PathBuf>,
    /// One reason for each part of the entry that stayed.
    pub remaining: Vec<String>,
}
