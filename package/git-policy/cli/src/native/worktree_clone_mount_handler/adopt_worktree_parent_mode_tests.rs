//! What:
//!  Controls of the adoption's refusals on a machine without a root-owned store:
//!  the not-root answer,
//!  the sentence each cause gives,
//!  and that a refusal records nothing.
//! Why:
//!  The mode writes one store under `/var/lib`,
//!  which a control must never touch,
//!  so what runs here is the part before any write:
//!  the two facts a control cannot set,
//!  the store's path and the process's effective user,
//!  are arguments of the driver,
//!  and the end-to-end arm runs in the throwaway guest
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 5,
//!  "Acceptance tests").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(adoptAt({ runningAsRoot: false, effectiveUser: 1000 })).toEqual({ refused: 'not-root' });
//! ```

use super::{AdoptFailure, adopt_at};
use crate::worktree_clone_mount::parent_adoption_decision::AdoptionRefusal;
use crate::worktree_clone_mount::slot_identity_store_refusals::StoreRefusal;
use crate::worktree_clone_mount_handler::handler_mode_arguments::AdoptRequest;
use std::path::Path;
use std::path::PathBuf;

/// An adoption of the checkout this repository is, identified as the test reads it.
fn request() -> AdoptRequest {
    return AdoptRequest { checkout: PathBuf::from("/home/user/Monochromatic"), device: 66306, inode: 1_234_567 };
}

/// A caller that is not root is refused before the store is opened,
/// and the sentence names the effective user.
#[test]
fn a_caller_that_is_not_root_is_refused_before_the_store_is_read() {
    let refusal: AdoptFailure = adopt_at(&request(), Path::new("/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities"), false, 1000)
        .expect_err("a caller that is not root records nothing");
    assert_eq!(refusal, AdoptFailure::NotRoot { effective_user: 1000 });
    let message: String = refusal.message();
    assert!(message.contains("effective user is 1000"), "{message}");
    assert!(message.contains("Nothing was read or recorded"), "{message}");
}

/// Every refusal names what an administrator fixes next,
/// and every one says the record is unchanged.
#[test]
fn every_refusal_names_its_cause_and_the_way_forward() {
    let cases: Vec<(AdoptFailure, &str)> = vec![
        (
            AdoptFailure::Store(StoreRefusal::Unreadable { path: PathBuf::from("/var/lib/x"), reason: String::from("the file is not there") }),
            "could not be read",
        ),
        (AdoptFailure::NoParentRecord { path: PathBuf::from("/var/lib/x") }, "holds no record"),
        (
            AdoptFailure::Decision(AdoptionRefusal::Identity { field: crate::worktree_clone_mount::CloneFolderField::Inode, identified: String::from("1"), held: String::from("2") }),
            "Nothing was recorded",
        ),
        (
            AdoptFailure::Write(StoreRefusal::Unreadable { path: PathBuf::from("/var/lib/x"), reason: String::from("the folder is gone") }),
            "unchanged",
        ),
    ];
    for (refusal, expected) in cases {
        let message: String = refusal.message();
        assert!(message.contains(expected), "{refusal:?} should say {expected}: {message}");
        assert!(message.contains("Ways forward"), "{refusal:?} should say what to do next: {message}");
    }
}
