//! Controls for the commit command's pure helpers.

use super::*;

/// The directory a `-C` chain reaches.
#[test]
fn the_effective_directory_follows_every_chdir_option() {
    let start: &Path = Path::new("/work");
    assert_eq!(effective_directory(start, &[]), PathBuf::from("/work"));
    assert_eq!(
        effective_directory(start, &[OsString::from("-C"), OsString::from("sub")]),
        PathBuf::from("/work/sub")
    );
    assert_eq!(
        effective_directory(
            start,
            &[
                OsString::from("-c"),
                OsString::from("user.name=x"),
                OsString::from("-C"),
                OsString::from("a"),
                OsString::from("-C"),
                OsString::from("b"),
            ]
        ),
        PathBuf::from("/work/a/b")
    );
    // An absolute path replaces what came before it.
    assert_eq!(
        effective_directory(start, &[OsString::from("-C"), OsString::from("/elsewhere")]),
        PathBuf::from("/elsewhere")
    );
    // A `-C` with no value changes nothing.
    assert_eq!(effective_directory(start, &[OsString::from("-C")]), PathBuf::from("/work"));
}
