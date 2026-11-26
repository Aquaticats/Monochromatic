//! What:
//!  The control of the refusal's sentence:
//!  it names the file,
//!  the rule and the field,
//!  and promises that nothing changed.
//! Why:
//!  The project decided a record that cannot be read is refused and named,
//!  and the sentence is what a person acts on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refusal.message(path)).toContain('Nothing was destroyed');
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::{
    CleanupRefusal,
    CleanupRefusalCause,
};
use std::path::Path;

#[test]
fn the_sentence_names_the_file_the_rule_and_the_field() {
    let refusal: CleanupRefusal = CleanupRefusal::at(
        CleanupRefusalCause::OutOfOrder,
        "state",
        String::from("the grammar writes it earlier"),
    );
    let text: String = refusal.message(Path::new(
        "/repo/.git/monochromatic-git-policy-cli/origin-cleanups/git-worktree-clone-origin-00",
    ));
    assert!(text.contains(
        "/repo/.git/monochromatic-git-policy-cli/origin-cleanups/git-worktree-clone-origin-00"
    ));
    assert!(text.contains("out-of-order"));
    assert!(text.contains("`state`"));
    assert!(text.contains("Nothing was destroyed"));
    assert!(text.contains("never rewrites a record it cannot read"));
}
