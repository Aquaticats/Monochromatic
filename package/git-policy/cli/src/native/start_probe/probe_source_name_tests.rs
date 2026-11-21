//! What:
//!  Controls of the dataset-name rule:
//!  the names the kernel takes,
//!  each byte it refuses,
//!  the bound,
//!  and the `@ready` source the design fixes.
//! Why:
//!  A configuration the setup wrote must not make every boot's attach arm fail
//!  for a name the probe itself could have refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkDatasetName('pool/dataset_1-2.3:4')).toBe(undefined); expect(checkDatasetName('pool/ds@snap')).toEqual('snapshot-or-bookmark');
//! ```

use super::{DATASET_NAME_BOUND, NameRefusal, ProbeSource, READY_SNAPSHOT_SUFFIX, check_dataset_name};

/// The names the kernel takes pass,
/// each separator included.
#[test]
fn a_dataset_name_passes() {
    for name in ["pool", "zpcachyos/nosnap/git-worktree-probe", "pool/dataset_1-2.3:4", "POOL/UPPER"] {
        assert_eq!(check_dataset_name(name), Ok(()), "{name}");
    }
}

/// A name that carries a snapshot or a bookmark is refused,
/// because the probe adds its own snapshot.
#[test]
fn a_snapshot_or_bookmark_is_refused() {
    assert_eq!(check_dataset_name("pool/ds@snap"), Err(NameRefusal::SnapshotOrBookmark));
    assert_eq!(check_dataset_name("pool/ds#bookmark"), Err(NameRefusal::SnapshotOrBookmark));
}

/// An empty name,
/// an empty component and a byte a component does not hold are refused.
#[test]
fn malformed_names_are_refused() {
    assert_eq!(check_dataset_name(""), Err(NameRefusal::Empty));
    assert_eq!(check_dataset_name("pool//ds"), Err(NameRefusal::EmptyComponent));
    assert_eq!(check_dataset_name("pool/ds%"), Err(NameRefusal::BadCharacter { byte: '%' }));
    assert_eq!(check_dataset_name("pool/ds one"), Err(NameRefusal::BadCharacter { byte: ' ' }));
}

/// The bound judges the whole `dataset@ready` source,
/// so the longest dataset leaves room for the suffix and one byte more is refused.
#[test]
fn the_bound_is_the_kernels_source_bound() {
    let at_bound: String = "a".repeat(DATASET_NAME_BOUND - READY_SNAPSHOT_SUFFIX.len());
    assert_eq!(check_dataset_name(at_bound.as_str()), Ok(()));
    assert_eq!(format!("{at_bound}{READY_SNAPSHOT_SUFFIX}").len(), DATASET_NAME_BOUND);
    let past: String = "a".repeat(DATASET_NAME_BOUND - READY_SNAPSHOT_SUFFIX.len() + 1);
    assert_eq!(check_dataset_name(past.as_str()), Err(NameRefusal::TooLong));
}

/// The source is the configured dataset and the fixed snapshot.
#[test]
fn the_source_is_the_ready_snapshot() {
    let source: ProbeSource = ProbeSource { dataset: String::from("zpcachyos/nosnap/git-worktree-probe") };
    assert_eq!(source.snapshot_name(), "zpcachyos/nosnap/git-worktree-probe@ready");
}
