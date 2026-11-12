//! What:
//!  The notice that names each saved entry,
//!  and the rejection of a save that could not be made.
//! Why:
//!  These are the only words a person whose file was moved aside has to find it again,
//!  and the only words that say why a command they ran did not run;
//!  each names its entry,
//!  the place,
//!  and what to do next,
//!  without an implementation term a person would have to look up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // overwrittenSavedNotice([{ path: 'b.txt', savedTo: '/r/.git/.../discarded/s/b.txt' }], 'other')
//! ```

use super::{NotSaved, overwritten_not_saved_message, overwritten_saved_notice};
use crate::conflict_sides::{FailureKind, SaveFailure, SavedCopy};
use std::path::PathBuf;

/// One saved file, at a place this control names.
fn saved(path: &str, place: &str) -> SavedCopy {
    return SavedCopy {
        path: String::from(path),
        saved_to: Some(PathBuf::from(place)),
    };
}

/// The notice names the entry,
/// the place it is,
/// and what the command writes over it.
#[test]
fn the_notice_names_the_file_and_its_copy() {
    let copies: [SavedCopy; 1] = [saved("b.txt", "/r/.git/monochromatic-git-policy-cli/discarded/s/b.txt")];
    let notice: String = overwritten_saved_notice(copies.as_slice(), "other");
    assert!(notice.contains("saved your copy of b.txt"), "{notice}");
    assert!(notice.contains("/r/.git/monochromatic-git-policy-cli/discarded/s/b.txt"), "{notice}");
    assert!(notice.contains("because other writes over it"), "{notice}");
    assert!(notice.ends_with('\n'), "{notice}");
}

/// An entry that was gone when the save ran says so instead of naming a place it is not.
#[test]
fn an_entry_with_nothing_to_copy_says_so() {
    let copies: [SavedCopy; 1] = [SavedCopy {
        path: String::from("b.txt"),
        saved_to: None,
    }];
    let notice: String = overwritten_saved_notice(copies.as_slice(), "other");
    assert!(notice.contains("b.txt was gone"), "{notice}");
    assert!(!notice.contains("saved your copy"), "{notice}");
}

/// The rejection names what stopped the save,
/// the entries at stake,
/// where the copies go,
/// and the one deliberate way through.
#[test]
fn the_rejection_names_what_stopped_it_and_the_way_forward() {
    let failure: SaveFailure = SaveFailure::io(String::from("b.txt"), String::from("No space left on device"));
    let requested: [String; 1] = [String::from("b.txt")];
    let message: String = overwritten_not_saved_message(NotSaved {
        failure: &failure,
        saved_root: std::path::Path::new("/r/.git/monochromatic-git-policy-cli/discarded"),
        requested: requested.as_slice(),
    });
    assert!(message.contains("would write over `b.txt`"), "{message}");
    assert!(message.contains("the system refused b.txt: No space left on device"), "{message}");
    assert!(message.contains("/r/.git/monochromatic-git-policy-cli/discarded"), "{message}");
    assert!(message.contains("nothing in your worktree changed"), "{message}");
    assert!(message.contains("--no-enforce-worktree"), "{message}");
}

/// A rejection names at most the first few entries and counts the rest,
/// so a worktree with thousands of them still prints a sentence a person can read.
#[test]
fn a_long_list_is_named_and_counted() {
    let failure: SaveFailure = SaveFailure {
        subject: String::from("one"),
        reason: String::from("a reason"),
        kind: FailureKind::Io,
    };
    let requested: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"].iter().map(|name| String::from(*name)).collect();
    let message: String = overwritten_not_saved_message(NotSaved {
        failure: &failure,
        saved_root: std::path::Path::new("/r/.git/monochromatic-git-policy-cli/discarded"),
        requested: requested.as_slice(),
    });
    assert!(message.contains("and 2 more"), "{message}");
    assert!(!message.contains("and 3 more"), "{message}");
}
