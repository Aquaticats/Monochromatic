//! What:
//!  Controls of the folder's refusal words:
//!  each cause names the path and says that nothing was changed.
//! Why:
//!  A person reads one of these after an add that could not file its trigger,
//!  so the sentence has to say what stands and what to do
//!  (`AGENTS.md` rule `DGT`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refusal.message()).toContain('Nothing was read as an unanswered request');
//! ```

use super::RequestsRefusal;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::EntryRefusal;
use std::path::PathBuf;

/// Every refusal says what was not read and that the clone's request stands.
#[test]
fn every_refusal_says_nothing_was_read_as_a_request() {
    let path: PathBuf = PathBuf::from("/home/user/repo/.git/zfs-unanswered-requests");
    let named: Vec<RequestsRefusal> = vec![
        RequestsRefusal::Folder {
            path: path.clone(),
            reason: String::from("Permission denied (os error 13)"),
        },
        RequestsRefusal::File {
            path: path.clone(),
            reason: String::from("it is not a regular file"),
        },
        RequestsRefusal::Text {
            path: path.clone(),
            refusal: EntryRefusal::Fields {
                found: 2,
            },
        },
    ];
    for refusal in named {
        let message: String = refusal.message();
        assert!(
            message.contains("Nothing was read as an unanswered request"),
            "{message}"
        );
        assert!(
            message.contains(path.as_path().to_str().expect("the path is text")),
            "{message}"
        );
    }
}
