//! What:
//!  The controls of the live read layer's two parsers:
//!  a listed name that is not the dataset's,
//!  and a GUID the kernel reported as zero.
//! Why:
//!  A listing that answered another dataset's name or a GUID no live object has is a read this
//!  route must refuse rather than record,
//!  and these are the two places the live layer can see it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery",
//! step 1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(nameBelow('a@own', 'b', '@')).toBeUndefined();
//! ```

use super::{guid_of, name_below};

#[test]
fn a_listed_name_is_read_below_its_own_dataset_and_every_other_is_refused() {
    assert_eq!(name_below("tank/wt@own", "tank/wt", '@').expect("a snapshot of the dataset reads"), "own");
    assert_eq!(name_below("tank/wt#from-own", "tank/wt", '#').expect("a bookmark of the dataset reads"), "from-own");
    assert!(name_below("tank/wt", "tank/wt", '@').is_err(), "a name with no separator is not that object");
    assert!(name_below("tank/other@own", "tank/wt", '@').is_err(), "another dataset's object is refused");
    assert!(name_below("tank/wt@own", "tank/wt", '#').is_err(), "a snapshot is not a bookmark");
}

#[test]
fn a_zero_guid_is_refused_and_a_canonical_one_reads() {
    assert!(guid_of(0, "tank/wt@own").is_err(), "no live object answers GUID 0");
    assert_eq!(guid_of(7_828_944_690_444_299_989, "tank/wt").expect("a canonical GUID reads").file_name(), "7828944690444299989");
}
