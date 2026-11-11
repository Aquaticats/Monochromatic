//! What:
//!  The reading of both dry run answers,
//!  their quoting,
//!  the shapes of nested repository Git removes,
//!  and the answer a refused dry run gives.
//! Why:
//!  A path read one byte wrong is a repository deleted unsaved,
//!  and a name Git had to quote is exactly the path a person is least likely to retype by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(nestedRepositoriesTheCleanDeletes(['Would remove nested/'], [], root)).toEqual([Buffer.from('nested')]);
//! ```

use super::{
    deleted_directories_from_dry_run, dry_run_lines_from_answer, kept_repositories_from_dry_run,
    nested_repositories_the_clean_deletes,
};
use crate::command_test_support::{fixture, remove};
use crate::git_metadata::MetadataOutput;
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

/// A typed dry run names the paths it removes,
/// and the one sentence in its answer that names no path is left out.
#[test]
fn a_typed_dry_run_names_the_directories_it_removes() {
    let answer_lines: Vec<Vec<u8>> = lines(&[
        "Would remove nested/\n",
        "Would remove build/cache.txt\n",
        "Would refuse to remove current working directory\n",
    ]);
    assert_eq!(
        deleted_directories_from_dry_run(answer_lines.as_slice()),
        vec![line("nested"), line("build/cache.txt")]
    );
}

/// A dry run written with one force names each nested repository it keeps,
/// and every removal line beside them names no repository.
#[test]
fn a_one_force_dry_run_names_the_repositories_it_keeps() {
    let answer_lines: Vec<Vec<u8>> = lines(&[
        "Would skip repository cache/nested\n",
        "Would remove cache/build.txt\n",
        "Would skip repository vendor/lib\n",
    ]);
    assert_eq!(
        kept_repositories_from_dry_run(answer_lines.as_slice()),
        vec![line("cache/nested"), line("vendor/lib")]
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
        deleted_directories_from_dry_run(answer_lines.as_slice()),
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
    assert!(deleted_directories_from_dry_run(answer_lines.as_slice()).is_empty());
}

/// A repository that is itself the removed path is found through its own `.git` entry,
/// a directory holding no `.git` entry is not a repository,
/// and only the outermost of two repositories is returned.
#[test]
fn the_removed_paths_are_read_through_their_own_git_entry() {
    let root: PathBuf = fixture("clean-nested-shapes");
    std::fs::create_dir_all(root.join("nested/.git")).expect("a nested repository");
    std::fs::create_dir_all(root.join("cache/build")).expect("an ignored directory");
    std::fs::create_dir_all(root.join("vendor")).expect("a directory Git keeps");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove nested/\n", "Would remove cache/\n", "Would remove vendor/\n"]).as_slice(),
        lines(&[]).as_slice(),
        root.as_path(),
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("nested")]);
    remove(root.as_path());
}

/// A repository below a removed directory is exactly what the one-force answer names,
/// and the typed answer's parent directory is not a repository of its own.
#[test]
fn a_repository_below_a_removed_directory_comes_from_the_one_force_answer() {
    let root: PathBuf = fixture("clean-nested-below");
    std::fs::create_dir_all(root.join("cache/nested/.git")).expect("a nested repository");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove cache/\n"]).as_slice(),
        lines(&["Would skip repository cache/nested\n"]).as_slice(),
        root.as_path(),
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("cache/nested")]);
    remove(root.as_path());
}

/// What:
///  The outermost repository is returned once when both answers name the same tree.
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
        lines(&["Would remove cache/\n"]).as_slice(),
        lines(&["Would skip repository cache/inner\n"]).as_slice(),
        root.as_path(),
    )
    .expect("a plain answer");
    assert_eq!(found, vec![line("cache")]);
    remove(root.as_path());
}

/// A symbolic link is removed as the link alone,
///  so a repository it points at is not one this clean deletes.
#[test]
fn a_link_to_a_repository_is_not_a_repository_the_clean_deletes() {
    let root: PathBuf = fixture("clean-nested-link");
    std::fs::create_dir_all(root.join("elsewhere/.git")).expect("a repository beside the link");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("linked")).expect("a symbolic link");
    let found: Vec<Vec<u8>> = nested_repositories_the_clean_deletes(
        lines(&["Would remove linked/\n"]).as_slice(),
        lines(&[]).as_slice(),
        root.as_path(),
    )
    .expect("a plain answer");
    assert!(found.is_empty(), "{found:?}");
    remove(root.as_path());
}

/// A repository Git names outside the worktree is a failure,
///  never a directory this policy moves.
#[test]
fn a_repository_name_that_leaves_the_worktree_is_a_failure() {
    let root: PathBuf = fixture("clean-nested-outside");
    let failure: String = nested_repositories_the_clean_deletes(
        lines(&[]).as_slice(),
        lines(&["Would skip repository ../outside\n"]).as_slice(),
        root.as_path(),
    )
    .expect_err("a path above the worktree");
    assert!(failure.contains("../outside"), "{failure}");
    assert!(failure.contains("not a plain path"), "{failure}");
    remove(root.as_path());
}

/// A dry run Git refused is an error naming Git's own words,
///  because a refusal deletes nothing and must never read as "nothing to save".
#[test]
fn a_refused_dry_run_is_an_error() {
    let failed: MetadataOutput = answer(false, b"", b"fatal: clean.requireForce is true and -f not given\n");
    let failure: String = dry_run_lines_from_answer(&failed, "which repositories the clean would keep")
        .expect_err("a refused dry run");
    assert_eq!(
        failure,
        "Git could not read which repositories the clean would keep: fatal: clean.requireForce is true and -f not given"
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
