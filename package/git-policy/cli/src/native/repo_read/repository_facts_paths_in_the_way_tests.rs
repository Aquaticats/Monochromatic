//! What:
//!  The order of the entries the worktree holds where a form writes,
//!  and why only the outermost of two is named.
//! Why:
//!  A directory that stands in the way holds every file inside it,
//!  and naming a file inside that directory as well describes a loss the message already covers;
//!  the answer is the same on every run,
//!  so a refusal and a record of it can be compared with the last one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // outermostEntries([Buffer.from('a/b'), Buffer.from('a'), Buffer.from('ab')]) === ['a', 'ab']
//! ```

use super::{entries_the_form_would_lose, outermost_entries, stands_where_the_form_writes};
use crate::command_test_support::{fixture, remove};

/// The paths as the reading sorts them.
fn written(paths: &[&[u8]]) -> Vec<Vec<u8>> {
    let mut sorted: Vec<Vec<u8>> = paths.iter().map(|path| return path.to_vec()).collect();
    sorted.sort();
    sorted.dedup();
    return sorted;
}

/// An entry stands where a written path stands,
/// below one,
/// or above one,
/// and a name that merely starts the same letters stands nowhere.
#[test]
fn a_written_path_holds_the_entries_at_below_and_above_it() {
    let asked: Vec<Vec<u8>> = written(&[b"nested/file.txt", b"a/b.txt", b"b.txt"]);
    assert!(stands_where_the_form_writes(asked.as_slice(), b"a/b.txt"), "at a written path");
    assert!(stands_where_the_form_writes(asked.as_slice(), b"nested/file.txt/deeper.txt"), "below one");
    assert!(stands_where_the_form_writes(asked.as_slice(), b"nested"), "above one");
    assert!(stands_where_the_form_writes(asked.as_slice(), b"a"), "an ancestor of one");
    assert!(!stands_where_the_form_writes(asked.as_slice(), b"nested2"), "a name that only starts alike");
    assert!(!stands_where_the_form_writes(asked.as_slice(), b"nest"), "a shorter name that is no ancestor");
    assert!(
        stands_where_the_form_writes(asked.as_slice(), b"a/b.txt/inside.txt"),
        "below a written path as well"
    );
}

/// A directory kept whole holds everything below it,
/// so an entry inside it is dropped in favour of the directory itself,
/// whose printed name keeps its separator so the message names the directory.
#[test]
fn a_collapsed_directory_is_kept_beside_an_unrelated_file() {
    let found: Vec<Vec<u8>> = vec![b"nested/file.txt".to_vec(), b"nested/".to_vec(), b"other.txt".to_vec()];
    assert_eq!(
        outermost_entries(found),
        vec![b"nested/".to_vec(), b"other.txt".to_vec()]
    );
}

/// A file beside a directory of the same name is another entry,
/// a file inside a directory that is already named is not,
/// and the same path twice is one entry.
#[test]
fn only_the_outermost_entry_of_each_path_is_kept() {
    let found: [&[u8]; 4] = [b"a/b".as_slice(), b"a".as_slice(), b"ab".as_slice(), b"a/b".as_slice()];
    let paths: Vec<Vec<u8>> = found.iter().map(|path| return path.to_vec()).collect();
    assert_eq!(outermost_entries(paths), vec![b"a".to_vec(), b"ab".to_vec()]);
}

/// An answer that is one path is that path,
/// and no answer is no path.
#[test]
fn one_entry_stays_one_entry() {
    assert_eq!(outermost_entries(vec![b"b.txt".to_vec()]), vec![b"b.txt".to_vec()]);
    assert_eq!(outermost_entries(Vec::<Vec<u8>>::new()), Vec::<Vec<u8>>::new());
}

/// A name that is not text is named as the bytes it is,
/// never dropped:
/// a file whose name cannot be read is still a file the command writes over.
#[test]
fn a_name_that_is_not_text_is_still_named() {
    let paths: Vec<Vec<u8>> = vec![vec![b'a', 0xff, b'b']];
    assert_eq!(outermost_entries(paths).len(), 1);
}

/// A file the form writes is named as it is,
/// a directory the listing collapsed is named by the first entry inside it the form replaces,
/// a directory holding nothing written inside it loses nothing and is not named,
/// and a directory at the path of a written file is replaced whole and named as it is.
#[test]
fn a_written_path_names_what_lies_inside_a_collapsed_directory() {
    let root = fixture("paths-in-the-way-collapsed");
    std::fs::create_dir_all(root.join("sub/nested")).expect("the collapsed directory");
    std::fs::write(root.join("sub/nested/b.txt"), b"the person's file\n").expect("the file inside it");
    std::fs::create_dir_all(root.join("empty")).expect("a directory with nothing written inside");
    std::fs::write(root.join("empty/notes.txt"), b"notes\n").expect("a file of its own");
    std::fs::write(root.join("plain.txt"), b"the person's file\n").expect("a file above a written path");
    std::fs::create_dir_all(root.join("b.txt")).expect("a directory where the form writes a file");
    std::fs::write(root.join("b.txt/inside.txt"), b"inside\n").expect("a file inside it");
    std::fs::create_dir_all(root.join("a/b.txt")).expect("a directory where a file must go");
    std::fs::write(root.join("a/b.txt/kept.txt"), b"kept\n").expect("a file inside it");
    let kept: Vec<Vec<u8>> = vec![
        b"plain.txt".to_vec(),
        b"sub".to_vec(),
        b"empty".to_vec(),
        b"b.txt".to_vec(),
        b"a".to_vec(),
    ];
    let asked: Vec<Vec<u8>> = vec![
        b"b.txt".to_vec(),
        b"plain.txt/inner.txt".to_vec(),
        b"sub/nested/b.txt".to_vec(),
        b"a/b.txt".to_vec(),
    ];
    assert_eq!(
        entries_the_form_would_lose(kept, asked.as_slice(), root.as_path()),
        vec![
            String::from("a/b.txt"),
            String::from("b.txt"),
            String::from("plain.txt"),
            String::from("sub/nested/b.txt")
        ]
    );
    // A written path the worktree does not hold is a file Git creates, so no bytes of the person's go.
    let nothing_held: Vec<Vec<u8>> = vec![b"sub".to_vec()];
    let created: Vec<Vec<u8>> = vec![b"sub/something-new.txt".to_vec()];
    assert!(entries_the_form_would_lose(nothing_held, created.as_slice(), root.as_path()).is_empty());
    // A file where the form needs a directory is named, inside a collapsed directory as well.
    std::fs::write(root.join("sub/nested/file.txt"), b"blocks the directory\n").expect("a file in the way");
    let blocking: Vec<Vec<u8>> = vec![b"sub".to_vec()];
    let needs_a_directory: Vec<Vec<u8>> = vec![b"sub/nested/file.txt/inner.txt".to_vec()];
    assert_eq!(
        entries_the_form_would_lose(blocking, needs_a_directory.as_slice(), root.as_path()),
        vec![String::from("sub/nested/file.txt")]
    );
    remove(root.as_path());
}
