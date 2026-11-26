//! What:
//!  A store folder holding `.parent` and one slot record and nothing else,
//!  and the parent identity that store records beside the live folder.
//! Why:
//!  The store's listing and the recorded parent identity are the two facts
//!  whose checks judge a folder listing and an identity,
//!  so they are built here at the shape a healthy store has,
//!  and a control changes the one entry or the one field it judges.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = { entries: [{ name: '.parent' }, { name: 'one' }], parent: { recorded: '.parent' }, slots: [{ name: 'one', facts: { recorded: 'one' } }] };
//! ```

use super::identity_checks::{
    LiveIdentity,
    ParentIdentityFacts,
    RecordedIdentity,
};
use super::route_test_facts_names::ACCOUNT;
use super::store_listing::{
    ListedEntry,
    RecordFacts,
    SlotRecordFacts,
    StoreListingFacts,
};

/// A store folder holding `.parent` and one slot record,
/// and nothing else.
pub(crate) fn healthy_store() -> StoreListingFacts {
    return StoreListingFacts {
        entries: vec![
            ListedEntry {
                name: String::from(".parent"),
                modified_seconds_ago: Some(90),
            },
            ListedEntry {
                name: String::from("one"),
                modified_seconds_ago: Some(60),
            },
        ],
        parent: RecordFacts::Read {
            recorded: String::from(".parent"),
        },
        slots: vec![SlotRecordFacts {
            name: String::from("one"),
            facts: RecordFacts::Read {
                recorded: String::from("one"),
            },
        }],
    };
}

/// The parent identity a store records and the live folder answers.
pub(crate) fn healthy_parent_identity() -> ParentIdentityFacts {
    let identity: crate::worktree_clone_mount::CloneFolderIdentity = folder_identity(7);
    return ParentIdentityFacts {
        recorded: RecordedIdentity::Read(Box::new(identity.clone())),
        live: LiveIdentity::Held(Box::new(identity)),
    };
}

/// One folder identity,
/// with a device and inode a control can tell apart.
pub(crate) fn folder_identity(inode: u64) -> crate::worktree_clone_mount::CloneFolderIdentity {
    return crate::worktree_clone_mount::CloneFolderIdentity {
        device: 2049,
        inode,
        owner: ACCOUNT,
        mode: 0o700,
        access_acl: None,
        default_acl: None,
    };
}
