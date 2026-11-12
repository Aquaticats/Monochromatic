//! What:
//!  The order of the entries the worktree holds where a form writes,
//!  and why only the outermost of two is saved.
//! Why:
//!  A directory that stands in the way holds every file inside it,
//!  and saving a file inside a directory that was moved as well names a place the file is not in;
//!  the answer is the same on every run,
//!  so a record of a save can be compared with the last one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // outermostEntries([Buffer.from('a/b'), Buffer.from('a'), Buffer.from('ab')]) === ['a', 'ab']
//! ```

use super::outermost_entries;

/// A file beside a directory of the same name is another entry,
/// a file inside a directory that is already saved is not,
/// and the same path twice is one entry.
#[test]
fn only_the_outermost_entry_of_each_path_is_kept() {
    let found: [&[u8]; 4] = [b"a/b".as_slice(), b"a".as_slice(), b"ab".as_slice(), b"a/b".as_slice()];
    let paths: Vec<Vec<u8>> = found.iter().map(|path| path.to_vec()).collect();
    assert_eq!(outermost_entries(paths), vec![String::from("a"), String::from("ab")]);
}

/// An answer that is one path is that path,
/// and no answer is no path.
#[test]
fn one_entry_stays_one_entry() {
    assert_eq!(outermost_entries(vec![b"b.txt".to_vec()]), vec![String::from("b.txt")]);
    assert_eq!(outermost_entries(Vec::<Vec<u8>>::new()), Vec::<String>::new());
}

/// A name that is not text is named as the bytes it is,
/// never dropped:
/// a file whose name cannot be read is still a file the command writes over.
#[test]
fn a_name_that_is_not_text_is_still_named() {
    let paths: Vec<Vec<u8>> = vec![vec![b'a', 0xff, b'b']];
    assert_eq!(outermost_entries(paths).len(), 1);
}
