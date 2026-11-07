//! What:
//!  The plain values the descriptor-based reader of the folder of saved copies hands around:
//! which directory was seen,
//!  what was met below it,
//! how deep a pass goes,
//! and what a failed walk had done.
//! Why:
//!  `discarded_store.rs` holds the reader and stays under the file-size limit;
//! these values need nothing from it,
//! and it
//!  names them again so every path that used them stays the same.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Identity = { device: number; inode: number; changedSeconds: number; changedNanos: number };
//! ```

/// Which directory a listing saw:
/// the pair that tells a directory from one that took its name later.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Identity {
    /// The device it lives on.
    pub device: u64,
    /// Its inode number.
    pub inode: u64,
    /// When it last changed,
    /// whole seconds.
    pub changed_seconds: i64,
    /// When it last changed,
    /// the nanoseconds past the second.
    pub changed_nanos: i64,
}

/// The files met below a directory,
/// as names relative to it,
/// and the sum of their sizes (links by their own size).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tally {
    /// The relative names,
    /// in the order they were met.
    pub names: Vec<String>,
    /// The sum of the sizes;
    /// wide,
    /// so no number of files can wrap it.
    pub bytes: u128,
}

/// The deepest tree a pass enters;
/// a deeper one is kept and reported,
/// because every level holds a descriptor open.
pub const MAX_DEPTH: usize = 128;

/// A walk that failed after meeting `tally` of what it was asked to count or remove.
#[derive(Debug)]
pub struct Interrupted {
    /// What was met (surveying) or removed (removing) before the failure.
    pub tally: Tally,
    /// What stopped the walk.
    pub error: std::io::Error,
}
