//! Controls for the questions about the file system that the look at an earlier commit service asks:
//! the part of a journal word's path the file system knows,
//! and the identity of each earlier journal that exists,
//! each answered in disposable folders of its own.

use super::*;
use crate::test_support::{fixture_with_prefix, remove};
use std::path::{Path, PathBuf};

/// What:
///  A word that reaches into a link is compared by the folder the link names,
/// and the names below the longest part that exists are kept as the word spells them.
/// Why:
///   An earlier helper shows its journal word before it makes that folder,
/// so the folder has no
///  identity to compare,
/// and the two spellings of one folder (a link such as `/home` for `/var/home`) must still meet
///  at the part the file system knows.
/// A name below that part is the same name in both spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(resolvedAsFarAsTheFileSystemKnows(`${link}/not-made-yet`)).toBe(`${real}/not-made-yet`)
/// ```
#[cfg(unix)]
#[test]
fn the_part_of_a_word_that_exists_is_resolved_and_the_names_below_it_are_kept_as_spelled() {
    let root: PathBuf = fixture_with_prefix("native-look-paths", "resolved-so-far");
    let real: PathBuf = root.join("state");
    std::fs::create_dir_all(real.as_path()).expect("the folder the link names");
    let link: PathBuf = root.join("a-link-to-the-state-folder");
    std::os::unix::fs::symlink(real.as_path(), link.as_path()).expect("the link");
    let spelled: PathBuf = link.join("cli-git/service/an-instance");
    assert!(!spelled.exists(), "the folder below the link is not made yet");
    assert_eq!(
        resolved_as_far_as_the_file_system_knows(spelled.as_path()).expect("the link resolves"),
        real.join("cli-git/service/an-instance"),
        "the part that exists is resolved and the names below it are kept as the word spells them"
    );
    assert_eq!(
        resolved_as_far_as_the_file_system_knows(real.join("an-instance").as_path()).expect("the state folder resolves"),
        real.join("an-instance"),
        "a word with no missing name is resolved whole"
    );
    remove(root.as_path());
}

/// What:
///  An identity is counted for every earlier journal that is there,
/// and a journal that is not there adds none;
/// a folder that cannot be looked at for another reason leaves the question open.
/// Why:
///   The look compares a folder a word names with these identities,
/// so an earlier journal that is not
///  there must not be counted as an identity of nothing,
/// and a permission that hides one must not read as
///  "no journal is there".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(journalIdentities([there, absent])).toHaveLength(1); expect(journalIdentities([hidden])).toBeUndefined()
/// ```
#[cfg(unix)]
#[test]
fn an_identity_is_counted_for_each_journal_that_is_there_and_an_unreadable_one_leaves_the_question_open() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let root: PathBuf = fixture_with_prefix("native-look-paths", "identities");
    let there: PathBuf = root.join("an-instance");
    std::fs::create_dir_all(there.as_path()).expect("an earlier journal");
    let absent: PathBuf = root.join("another-instance");
    let found = journal_identities(&[there.clone(), absent]).expect("two journals, one of them absent");
    let known = std::fs::metadata(there.as_path()).expect("the journal that is there");
    assert_eq!(found, vec![(known.dev(), known.ino())], "the identity of the journal that is there, and none for the absent one");
    // A folder whose parent cannot be looked into is not a folder that is not there.
    let hidden: PathBuf = root.join("hidden");
    std::fs::create_dir(hidden.as_path()).expect("a folder that will be hidden");
    let journal: PathBuf = hidden.join("an-instance");
    std::fs::create_dir(journal.as_path()).expect("a journal below it");
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o0)).expect("a folder this user may not look into");
    let unreadable: Option<Vec<(u64, u64)>> = journal_identities(&[journal]);
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the permission is given back");
    assert_eq!(unreadable, None, "a journal that cannot be looked at leaves the question open");
    remove(root.as_path());
}

/// The spelling of a path is what a command line carries,
/// with no loss of bytes a text conversion would replace.
#[cfg(unix)]
#[test]
fn a_paths_spelling_is_the_bytes_it_is_carried_as() {
    use std::os::unix::ffi::OsStrExt;
    let word: &Path = Path::new("/a/folder/with a space/and-ä");
    assert_eq!(path_word(word), word.as_os_str().as_bytes(), "the bytes of the path, not a lossy text form");
}
