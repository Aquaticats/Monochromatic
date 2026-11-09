//! What:
//!  The file a typed message is kept in when its commit cannot be made.
//! Why:
//!  The text is the person's only copy after the commit ends,
//!  and a write that fails says so rather than losing it silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(keepTheTypedMessage(gitDir, id, 'typed')).toBe(join(gitDir, 'monochromatic-git-policy-cli-saved-message-0a1b2c3d'));
//! ```

/// The rule under control.
use super::{SAVED_MESSAGE_PREFIX, keep_the_typed_message};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A message is kept in a file named for the request,
/// a request with no message keeps nothing,
/// and a write that fails answers nothing rather than a path nobody can read.
#[test]
fn a_typed_message_is_kept_where_the_command_names_it() {
    let root: PathBuf = fixture("saved-message");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    let kept: Option<PathBuf> = keep_the_typed_message(git_dir.as_path(), "0a1b2c3d-4e5f", Some("typed: seed"));
    let named: PathBuf = git_dir.join(format!("{SAVED_MESSAGE_PREFIX}0a1b2c3d"));
    assert_eq!(kept, Some(named.clone()), "the path is the one the command names");
    assert_eq!(std::fs::read_to_string(&named).expect("the saved message"), "typed: seed");
    assert_eq!(keep_the_typed_message(git_dir.as_path(), "0a1b2c3d-4e5f", None), None, "nothing typed, nothing kept");
    // A directory in place of the file is a write that fails, and the failure is not a path.
    let sealed: PathBuf = root.join("sealed/.git");
    std::fs::create_dir_all(sealed.join(format!("{SAVED_MESSAGE_PREFIX}deadbeef"))).expect("a directory where the file goes");
    assert_eq!(keep_the_typed_message(sealed.as_path(), "deadbeef-0000", Some("typed")), None);
    remove(root.as_path());
}
