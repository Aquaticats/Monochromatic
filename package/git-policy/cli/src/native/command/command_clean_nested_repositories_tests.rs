//! What:
//!  The reading of a typed dry run answer,
//!  its quoting,
//!  the shapes of nested repository Git removes,
//!  and the two failures a dry run can give.
//! Why:
//!  A path read one byte wrong is a repository deleted unsaved,
//!  a repository below a name Git printed is one only a look inside finds,
//!  and a name Git had to quote is exactly the path a person is least likely to retype by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(nestedRepositoriesTheCleanDeletes(['Would remove nested/'], root)).toEqual([Buffer.from('nested')]);
//! ```

use super::{
    dry_run_lines_from_answer, nested_repositories_the_clean_deletes, removed_paths_from_dry_run,
};
use crate::command_test_support::{fixture, remove};
use crate::git_metadata::MetadataOutput;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Lines as the answer's split leaves them:
///  what a dry run wrote,
///  without the line feed that ended it.
fn lines(values: &[&str]) -> Vec<Vec<u8>> {
    return values
        .iter()
        .map(|value| { return value.strip_suffix('\n').unwrap_or(value).as_bytes().to_vec(); })
        .collect();
}

/// One line as Git wrote it,
///  without a line feed,
///  which is what the answer's split leaves.
fn line(value: &str) -> Vec<u8> {
    return value.as_bytes().to_vec();
}

/// A one-line `git clean --dry-run` answer.
fn answer(success: bool, stdout: &[u8], stderr: &[u8]) -> MetadataOutput {
    return MetadataOutput {
        success,
        code: Some(if success { 0 } else { 128 }),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };
}

/// What:
///  A typed dry run names every path it removes,
///  with the trailing slash of a top-level directory and without the one of a directory it prints from inside
///  a directory that survives.
/// Why:
///  Both spellings are real Git answers (`remove_dirs` writes both),
///  so neither may be dropped before the caller decides which name is a directory;
///  the one sentence that names no path is left out.
#[test]
fn a_typed_dry_run_names_every_path_it_removes() {
    let answer_lines: Vec<Vec<u8>> = lines(&[
        "Would remove nested/\n",
        "Would remove build/cache.txt\n",
        "Would skip repository container/kept\n",
        "Would refuse to remove current working directory\n",
        "Would remove cache/plain\n",
    ]);
    assert_eq!(
        removed_paths_from_dry_run(answer_lines.as_slice()),
        vec![line("nested"), line("build/cache.txt"), line("cache/plain")]
    );
}

/// What:
///  Names Git had to quote,
///  decoded byte for byte.
/// Why:
///   `quote_c_style` escapes a byte it cannot print,
///  and reading the quoted text as the name would find no repository there.
///  The cases are the whole escape set:
/// every named escape,
/// an octal byte,
/// and a name that is only a quote.
#[test]
fn a_name_git_had_to_quote_is_decoded() {
    let answer_lines: Vec<Vec<u8>> = lines(&[
        "Would remove \"a\\nb/\"\n",
        "Would remove \"a\\tb/\"\n",
        "Would remove \"a\\\\b/\"\n",
        "Would remove \"a\\\"b/\"\n",
        "Would remove \"aé/\"\n",
        "Would remove \"a\\303\\251/\"\n",
        "Would remove \"\\\"\"\n",
        "Would remove \"a\\ab/\"\n",
        "Would remove \"a\\vb/\"\n",
        "Would remove \"a\\fb/\"\n",
        "Would remove \"a\\rb/\"\n",
    ]);
    assert_eq!(
        removed_paths_from_dry_run(answer_lines.as_slice()),
        vec![
            b"a\nb".to_vec(),
            b"a\tb".to_vec(),
            b"a\\b".to_vec(),
            b"a\"b".to_vec(),
            "aé".as_bytes().to_vec(),
            "aé".as_bytes().to_vec(),
            vec![b'"'],
            vec![b'a', 0x07, b'b'],
            vec![b'a', 0x0b, b'b'],
            vec![b'a', 0x0c, b'b'],
            vec![b'a', 0x0d, b'b'],
        ]
    );
}

/// A quoted form this reader cannot read names nothing,
///  and the only quoting Git writes with an escape outside its set is not believed.
#[test]
fn a_quoted_name_this_reader_cannot_read_names_nothing() {
    let answer_lines: Vec<Vec<u8>> = lines(&[
        "Would remove \"a\\zb/\"\n",
        "Would remove \"a\\09/\"\n",
        "Would remove \"a\\400/\"\n",
        "Would remove \"unterminated/\n",
    ]);
    assert!(removed_paths_from_dry_run(answer_lines.as_slice()).is_empty());
}

/// A repository that is itself the removed path is found through its own `.git` entry,
///  a directory holding no `.git` entry is not a repository,
///  and only the outermost of two repositories is returned.
#[test]
fn the_removed_paths_are_read_through_their_own_git_entry() {
    let root: PathBuf = fixture("clean-nested-shapes");
    std::fs::create_dir_all(root.join("nested/.git")).expect("a nested repository");
    std::fs::create_dir_all(root.join("cache/build")).expect("an ignored directory");
    std::fs::create_dir_all(root.join("vendor")).expect("a directory Git keeps");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove nested/\n", "Would remove cache/\n", "Would remove vendor/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("nested")]);
    remove(root.as_path());
}

/// What:
///  A repository below a removed directory is found however deep it sits,
///  and the directory Git named is not a repository of its own.
/// Why:
///  Git names the directory it removes,
///  never the repository inside it,
///  and a repository below an untracked directory was deleted unsaved while the wrapper looked only at the
///  names Git printed.
#[test]
fn a_repository_below_a_removed_directory_is_found() {
    let root: PathBuf = fixture("clean-nested-below");
    std::fs::create_dir_all(root.join("container/nested/.git")).expect("a nested repository");
    std::fs::create_dir_all(root.join("container/deeper/again/nested/.git")).expect("a deeper repository");
    std::fs::create_dir_all(root.join("container/elsewhere")).expect("an ordinary directory");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove container/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect("a plain answer");
    assert_eq!(
        found,
        vec![line("container/nested"), line("container/deeper/again/nested")]
    );
    remove(root.as_path());
}

/// What:
///  The outermost repository is returned once when both a removed path and its inside name one.
/// Why:
///   Moving a repository takes everything inside it,
///  so a second move of a path below it would fail on a directory that is no longer there,
///  after the first move already changed the worktree.
#[test]
fn only_the_outermost_repository_is_returned() {
    let root: PathBuf = fixture("clean-nested-outermost");
    std::fs::create_dir_all(root.join("cache/.git")).expect("the outer repository");
    std::fs::create_dir_all(root.join("cache/inner/.git")).expect("the inner repository");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove cache/\n", "Would remove cache/inner/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("cache")]);
    remove(root.as_path());
}

/// A symbolic link is removed as the link alone,
///  so a repository it points at,
///  or one reached through it,
///  is not one this clean deletes.
#[test]
fn a_link_to_a_repository_is_not_a_repository_the_clean_deletes() {
    let root: PathBuf = fixture("clean-nested-link");
    std::fs::create_dir_all(root.join("elsewhere/.git")).expect("a repository beside the link");
    std::fs::create_dir_all(root.join("elsewhere/inner/.git")).expect("a repository inside it");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("linked")).expect("a symbolic link");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove linked/\n", "Would remove linked/inner/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect("a plain answer");
    assert!(found.is_empty(), "{found:?}");
    remove(root.as_path());
}

/// A removed name that leaves the worktree looks inside nothing,
///  and no directory of the worktree is moved for it.
#[test]
fn a_removed_name_that_leaves_the_worktree_names_nothing() {
    let root: PathBuf = fixture("clean-nested-outside");
    std::fs::create_dir_all(root.join("nested/.git")).expect("a nested repository");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove ../outside/\n", "Would remove /absolute/\n", "Would remove nested/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("nested")]);
    remove(root.as_path());
}

/// What:
///  A pathspec may name a directory above the one the clean runs in,
///  and a repository below that name is still one the clean deletes.
/// Why:
///  Git prints that path with the parent components that reach it from the command's directory,
///  so a reader that refuses a parent component drops a name Git really printed,
///  and the repository inside it is deleted unsaved.
#[test]
fn a_name_above_the_command_but_below_the_worktree_is_read() {
    let root: PathBuf = fixture("clean-nested-above");
    std::fs::create_dir_all(root.join("sub/kept")).expect("the command's directory");
    std::fs::create_dir_all(root.join("other/nested/.git")).expect("a repository beside the command");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove ../other/\n"]).as_slice(),
        root.as_path(),
        b"sub/",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("other/nested")]);
    remove(root.as_path());
}

/// What:
///  A name that steps above the worktree names nothing,
///  and a name below it is still read.
/// Why:
///  Git removes nothing outside the worktree,
///  so a name that leaves it is no directory this policy may move,
///  while one beside it is.
#[test]
fn a_name_that_steps_above_the_worktree_names_nothing() {
    let root: PathBuf = fixture("clean-nested-above-root");
    std::fs::create_dir_all(root.join("sub/nested/.git")).expect("a nested repository");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove ../../outside/\n", "Would remove nested/\n"]).as_slice(),
        root.as_path(),
        b"sub/",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("sub/nested")]);
    remove(root.as_path());
}

/// What:
///  A clean started in a subdirectory prints its paths below the command's own directory,
///  and a repository below one of them is still found.
/// Why:
///  Git prints a removed path relative to the directory the command runs in
///  (`relative_path` in `builtin/clean.c`'s print),
///  with a ledot when the path is named from inside a directory it refuses to remove itself,
///  so reading that name from the worktree root names a directory Git never looked at,
///  and rejecting the ledot drops the only name that holds the repository.
#[test]
fn the_paths_a_subdirectory_clean_prints_are_read_below_the_command() {
    let root: PathBuf = fixture("clean-nested-subdirectory");
    std::fs::create_dir_all(root.join("sub/container/nested/.git")).expect("a nested repository");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would refuse to remove current working directory\n", "Would remove ./container\n"]).as_slice(),
        root.as_path(),
        b"sub/",
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("sub/container/nested")]);
    remove(root.as_path());
}

/// What:
///  A directory the wrapper cannot look inside stops the clean,
///  and the sentence names that directory.
/// Why:
///   What it holds was not seen,
///  so a repository inside it may be about to be deleted unsaved,
///  and reporting no repository there would be a guess.
#[test]
fn a_directory_the_wrapper_cannot_look_inside_stops_the_clean() {
    let root: PathBuf = fixture("clean-nested-unreadable");
    std::fs::create_dir_all(root.join("cache/held")).expect("a directory Git would remove");
    std::fs::write(root.join("cache/held/data.txt"), b"the person work\n").expect("a file");
    std::fs::set_permissions(root.join("cache"), std::fs::Permissions::from_mode(0o300)).expect("lock the cache");
    let failure: String = nested_repositories_the_clean_deletes(
        lines(&["Would remove cache/\n"]).as_slice(),
        root.as_path(),
        b"",
    )
    .expect_err("a directory that cannot be read");
    std::fs::set_permissions(root.join("cache"), std::fs::Permissions::from_mode(0o700)).expect("free the cache");
    assert!(failure.contains("cache"), "{failure}");
    assert!(failure.contains("not judged"), "{failure}");
    remove(root.as_path());
}

/// A dry run Git refused is an error naming Git's own words,
///  because a refusal deletes nothing and must never read as "nothing to save".
#[test]
fn a_refused_dry_run_is_an_error() {
    let failed: MetadataOutput = answer(false, b"", b"fatal: clean.requireForce is true and -f not given\n");
    let failure: String = dry_run_lines_from_answer(&failed, "what the clean would delete").expect_err("a refused dry run");
    assert_eq!(
        failure,
        "Git could not read what the clean would delete: fatal: clean.requireForce is true and -f not given"
    );
}

/// What:
///  A dry run that ended without an exit status is told apart from Git's own refusal.
/// Why:
///   Git's refusal deletes nothing and is Git's own answer to the person,
///  while a read that never answered says nothing at all,
///  and the two words differ so the caller can stop the command for the second.
#[test]
fn a_dry_run_that_never_answered_is_not_a_refusal() {
    let killed: MetadataOutput = MetadataOutput {
        success: false,
        code: None,
        stdout: Vec::<u8>::new(),
        stderr: Vec::<u8>::new(),
    };
    let failure: String =
        dry_run_lines_from_answer(&killed, "what the clean would delete").expect_err("a read that never answered");
    assert_eq!(
        failure,
        "Git never answered what the clean would delete: the read ended without an exit status, so the clean was not judged"
    );
}

/// A dry run that would do nothing answers with no line at all,
///  and a line of whitespace alone names nothing either.
#[test]
fn an_empty_answer_names_nothing() {
    let empty: MetadataOutput = answer(true, b"\n  \n", b"");
    let read: Vec<Vec<u8>> = dry_run_lines_from_answer(&empty, "what the clean would delete").expect("a successful answer");
    assert!(read.is_empty(), "{read:?}");
}
