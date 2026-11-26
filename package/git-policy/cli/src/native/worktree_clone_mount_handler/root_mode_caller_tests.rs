//! What:
//!  The controls of the sentence a root mode refuses a caller that is not root
//!  with:
//!  the words every root mode is reached through,
//!  the mode,
//!  the effective user,
//!  what was untouched,
//!  why root,
//!  and the way forward.
//! Why:
//!  The sentence is the whole answer such a caller gets,
//!  so a phrase that falls out of it leaves a person with a refusal that names
//!  no next step,
//!  and a sentence that opens with no words leaves root's journal with a
//!  refusal nothing ties to the modes
//!  (`root_mode_caller`;
//!  `package/git-policy/cli/SPEC.md`,
//!  "the modes' refusals and lines open with the words").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(sentence).toContain('effective user is 1000');
//! ```

use super::NotRootRefusal;
use crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name;

/// The sentence names the mode,
/// the effective user,
/// what was not touched,
/// why the mode runs as root,
/// and the way forward.
#[test]
fn the_sentence_names_the_mode_the_user_and_the_way_forward() {
    let refusal: NotRootRefusal<'_> = NotRootRefusal {
        mode: "start-probe",
        nothing: "read, mounted or recorded",
        because: "the record it writes decides which clones a boot admits",
        way: "run the mode as root",
        effective_user: 1000,
    };
    let sentence: String = refusal.sentence();
    assert!(
        sentence.starts_with(&format!("{}: ", root_modes_name())),
        "the sentence opens with the words every root-mode refusal opens with: {sentence}"
    );
    assert!(
        sentence.contains("the start-probe mode runs as root"),
        "{sentence}"
    );
    assert!(sentence.contains("effective user is 1000"), "{sentence}");
    assert!(
        sentence.contains("Nothing was read, mounted or recorded"),
        "{sentence}"
    );
    assert!(
        sentence.contains("because the record it writes decides which clones a boot admits"),
        "{sentence}"
    );
    assert!(
        sentence.contains("Ways forward: run the mode as root."),
        "{sentence}"
    );
}

/// A caller that is not root is told the number the kernel reported,
/// so the caller can tell a mode that refused them from one that refused the
/// machine.
#[test]
fn another_effective_user_is_named_as_it_was_read() {
    let refusal: NotRootRefusal<'_> = NotRootRefusal {
        mode: "adopt-worktree-parent",
        nothing: "read or recorded",
        because: "the store under root control decides",
        way: "run the mode as root",
        effective_user: 4242,
    };
    assert!(
        refusal.sentence().contains("effective user is 4242"),
        "{}",
        refusal.sentence()
    );
}
