//! Controls for the words a write step prints when it keeps a copy of the person's file beside
//! its path:
//! the two commands that put the copy back and drop it,
//! and both paths in them.

use super::*;

/// What:
///  Both paths of a kept copy's commands are one shell word each,
/// so a person who copies the line runs it on the files it names.
/// Why:
///  A name with a space printed bare reached `mv` as several arguments,
/// which moved files the line never named or
///  refused the line altogether.
#[test]
fn the_commands_about_a_kept_copy_quote_both_paths() {
    let commands: String = commands_that_end_the_copy(Path::new("/repo/.cli-git-carry-1-2abc-gone with space.txt"), b"gone with space.txt");
    assert_eq!(
        commands,
        "`mv -- '/repo/.cli-git-carry-1-2abc-gone with space.txt' 'gone with space.txt'` puts the copy back over what is there, \
         `rm -- '/repo/.cli-git-carry-1-2abc-gone with space.txt'` drops the copy"
    );
}

/// A name a shell reads as it stands is printed as it stands.
#[test]
fn a_plain_name_is_printed_without_quotes() {
    let commands: String = commands_that_end_the_copy(Path::new("/repo/.cli-git-carry-1-2abc-a.txt"), b"a.txt");
    assert!(commands.contains("`mv -- /repo/.cli-git-carry-1-2abc-a.txt a.txt`"), "{commands}");
    assert!(commands.contains("`rm -- /repo/.cli-git-carry-1-2abc-a.txt`"), "{commands}");
}
