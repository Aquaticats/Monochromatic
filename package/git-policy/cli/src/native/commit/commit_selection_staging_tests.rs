//! Controls for this module:
//! which pathspec elements subtract paths,
//! and which elements a selection names when it names a pathspec file.

use super::*;
use crate::transaction_journal::TransactionMode;
use std::path::PathBuf;

/// A repository directory under this control's fixture,
/// for a pathspec file to live in.
fn fixture_directory(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture(name);
    std::fs::create_dir(root.join("repo")).expect("repository directory");
    return root.join("repo");
}

/// The selection that names one pathspec file,
/// as the preparation hands it over.
fn selection_naming_file<'selection>(file: &'selection Path, nul: bool) -> IndexSelection<'selection> {
    return IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: &[],
        pathspec_file: Some(file),
        pathspec_file_nul: nul,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: "HEAD",
    };
}

/// The long form names the subtraction by its whole word,
/// which Git matches in lower case only,
/// and the short form by `!` and `^`.
#[test]
fn the_subtracting_magic_of_a_pathspec_is_read_as_git_reads_it() {
    for subtracting in [
        ":(exclude)a.txt",
        ":(exclude,top)*.txt",
        ":(literal,exclude)a.txt",
        ":!a.txt",
        ":^a.txt",
        ":!!a.txt",
        ":/!a.txt",
    ] {
        assert!(excludes_paths(OsStr::new(subtracting)), "{subtracting} subtracts paths");
    }
    for naming in [":(EXCLUDE)a.txt", ":(!)a.txt", ":(exclude-)a.txt", ":(-exclude)a.txt", ":(top)a.txt", ":(attr:foo)a.txt", ":foo", "a.txt", "x:!y.txt"] {
        assert!(!excludes_paths(OsStr::new(naming)), "{naming} names paths");
    }
}

/// The elements of a command line are the selection its pathspecs name.
#[test]
fn a_selection_without_a_pathspec_file_names_its_own_elements() {
    let pathspecs: Vec<OsString> = vec![OsString::from("a.txt"), OsString::from(":(exclude)b.txt")];
    let selection: IndexSelection<'_> = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: pathspecs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: "HEAD",
    };
    assert_eq!(selection_elements(&selection), Some(pathspecs));
}

/// A pathspec file is the same selection as the elements it holds,
/// whether its records end with a newline or with NUL.
#[test]
fn a_selection_that_names_a_pathspec_file_names_the_elements_of_that_file() {
    let directory: PathBuf = fixture_directory("selection-elements-file");
    let newline: PathBuf = directory.join("newline.txt");
    std::fs::write(&newline, "a.txt\n:(exclude)b.txt\n").expect("newline records");
    assert_eq!(
        selection_elements(&selection_naming_file(newline.as_path(), false)),
        Some(vec![OsString::from("a.txt"), OsString::from(":(exclude)b.txt")])
    );
    let nul: PathBuf = directory.join("nul.txt");
    std::fs::write(&nul, b"a b.txt\0c.txt\0").expect("NUL records");
    assert_eq!(
        selection_elements(&selection_naming_file(nul.as_path(), true)),
        Some(vec![OsString::from("a b.txt"), OsString::from("c.txt")])
    );
    crate::test_support::remove(directory.parent().expect("the fixture root"));
}

/// A file this reader cannot take names nothing,
/// so the staging hands the file to Git and Git reports it.
#[test]
fn a_pathspec_file_this_reader_cannot_take_names_nothing() {
    let directory: PathBuf = fixture_directory("selection-elements-unreadable");
    assert_eq!(selection_elements(&selection_naming_file(directory.join("missing.txt").as_path(), false)), None);
    assert_eq!(selection_elements(&selection_naming_file(directory.as_path(), false)), None);
    let empty_record: PathBuf = directory.join("empty-record.txt");
    std::fs::write(&empty_record, b"a.txt\n\nb.txt\n").expect("an empty record");
    assert_eq!(selection_elements(&selection_naming_file(empty_record.as_path(), false)), None);
    let too_large: PathBuf = directory.join("too-large.txt");
    let held: std::fs::File = std::fs::File::create(&too_large).expect("a sparse file");
    held.set_len(PATHSPEC_FILE_LIMIT + 1).expect("a file past the bound");
    assert_eq!(selection_elements(&selection_naming_file(too_large.as_path(), false)), None);
    crate::test_support::remove(directory.parent().expect("the fixture root"));
}
