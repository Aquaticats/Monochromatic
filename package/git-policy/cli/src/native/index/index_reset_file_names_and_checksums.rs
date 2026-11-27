//! What:
//!  The bounds,
//!  the checksums and the names the landing's index update reads before it
//!  touches the index file.
//! Why:
//!  An index entry's name is the hash of the blob header and the bytes,
//!  so the update computes the same checksum the index file carries,
//!  derives the names a path-limited reset rewrites from the commit's own
//!  trees,
//!  and declines on a file bigger than the bound the in-process query layer
//!  already puts on the same file rather than on a new measurement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checksum = (format, bytes) => format === 'sha1' ? sha1(bytes) : sha256(bytes);
//! const names = namesOf({ database, layout, newTree, paths }); // Uint8Array[]
//! ```

/// The names the update resets,
/// and the trees its derived names are read from.
use crate::index_reset_tree::{
    changed_paths,
    root_tree,
};
/// Where the repository is,
/// and the object hash its names are spelled in.
use crate::repo_read_layout::{
    ObjectFormat,
    RepoLayout,
};
/// The most bytes of index file this reads,
/// which is the bound the in-process query layer already puts on it.
use crate::repo_read_object::name_length;
/// The objects one commit's trees are read from.
use crate::repo_read_odb::ObjectDatabase;
/// The index paths a caller hands the landing.
use crate::service_land_follow::IndexPaths;
/// The one object name that means "no commit before this one".
use crate::service_protocol::is_zero_object;
/// The SHA-1 and SHA-256 the checksum and an object name are computed with.
use crate::subvolume_sha::{
    sha1,
    sha256,
};
/// The most bytes of index file this reads:
/// the bound the in-process query layer already puts on the same file (`repo_read_query.rs`,
/// `Context::index`),
/// not a new measurement.
pub(super) const INDEX_LIMIT: u64 = 1 << 30;

/// The hook Git runs after it writes an index.
pub(super) const POST_INDEX_HOOK: &str = "post-index-change";

/// The checksum of the format's object hash over `bytes`.
pub(super) fn checksum(
    format: ObjectFormat,
    bytes: &[u8],
) -> Vec<u8> {
    return if name_length(format) == 20 {
        sha1_bytes(bytes)
    } else {
        sha256_bytes(bytes)
    };
}

/// The SHA-1 of `bytes` as a byte vector.
pub(super) fn sha1_bytes(bytes: &[u8]) -> Vec<u8> {
    return sha1(bytes).to_vec();
}

/// The SHA-256 of `bytes` as a byte vector.
pub(super) fn sha256_bytes(bytes: &[u8]) -> Vec<u8> {
    return sha256(bytes).to_vec();
}

/// The names (top-relative,
/// as bytes) the update resets.
pub(super) fn names_of(
    database: &ObjectDatabase,
    layout: &RepoLayout,
    new_tree: &[u8],
    paths: IndexPaths<'_>,
) -> Option<Vec<Vec<u8>>> {
    return match paths {
        IndexPaths::Listed(listed) => {
            let mut names: Vec<Vec<u8>> = Vec::new();
            for path in listed {
                names.push(path.as_bytes().to_vec());
            }
            Some(names)
        }
        IndexPaths::Named(named) => Some(named.to_vec()),
        IndexPaths::Derived {
            old,
        } => {
            let old_tree: Option<Vec<u8>> = if is_zero_object(old) {
                None
            } else {
                Some(root_tree(database, old, layout.object_format)?)
            };
            changed_paths(
                database,
                old_tree.as_deref(),
                new_tree,
                layout.object_format,
            )
        }
    };
}
