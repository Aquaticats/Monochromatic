//! What:
//!  One step of a listing:
//! the name the kernel wrote,
//! the cursor to ask after,
//!  the statistics it filled,
//! and the property list it put in the reply.
//! Why:
//!  Both list requests write the next name into `zc_name` in place and move
//!  `zc_cookie`,
//! so a caller cannot rebuild either from its own input;
//! keeping them
//!  together means a caller never reads a half-updated cursor.
//!  The statistics are the identity the design compares against,
//! and the property
//!  list carries each property's value and source.

/// The statistics both list requests fill.
use crate::layout::DmuObjsetStats;
/// The decoded property list.
use crate::nvlist::NvList;

/// One dataset or snapshot a listing returned.
pub struct ListedObject {
    /// The full name the kernel wrote into `zc_name`.
    pub name: String,
    /// The cursor to pass to the next request,
    /// which is the kernel's own.
    pub cursor: u64,
    /// The statistics of the object,
    /// identity included.
    pub stats: DmuObjsetStats,
    /// The properties and their sources.
    pub properties: NvList,
}

/// What an identity read reports:
/// the statistics the kernel filled and the
/// property values with their sources.
pub struct DatasetIdentity {
    /// The dataset's statistics:
    /// type,
    /// identity,
    /// origin and flags.
    pub stats: DmuObjsetStats,
    /// The properties and their sources.
    pub properties: NvList,
}
