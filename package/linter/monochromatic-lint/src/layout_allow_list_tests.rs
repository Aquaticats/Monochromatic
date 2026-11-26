//! What:
//!  Controls of the allow-list reader.
//! Why:
//!  A layout rule's `allow` option names a file whose lines record the findings it tolerates today,
//!  and a line that names nothing must fail the run rather than pass for an entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a repeated line is an error', () => expect(() => parse(text)).toThrow());
//! ```

use super::{
    AllowKind,
    AllowList,
    parse_allow_list,
    read_allow_list,
};
use crate::{
    run_test_support::write,
    test_fs::Fixture,
};
use std::path::Path;

/// Read a list from text,
/// as the file it would be.
fn parsed(lines: &str) -> AllowList {
    return parse_allow_list(lines, Path::new("package/allow-list.txt"))
        .expect("the fixture list is valid");
}

/// The message of the error a bad list raises.
fn error_of(lines: &str) -> String {
    return parse_allow_list(lines, Path::new("package/allow-list.txt"))
        .expect_err("the fixture list is invalid");
}

/// Every shape of entry is read in file order,
/// with its line number,
/// and a comment and a blank line are skipped.
#[test]
fn every_shape_of_entry_is_read_in_file_order() {
    let list: AllowList = parsed(
        "# today's loops\n\
         uses src/native/landing -> src/native/foundation\n\
         \n\
         file-uses src/native/command command_options.rs -> command_options_long.rs\n\
         glob src/native/lib.rs invocation::*\n",
    );
    let entries = list.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0].text,
        "uses src/native/landing -> src/native/foundation"
    );
    assert_eq!(entries[0].line, 2);
    assert!(
        matches!(
            &entries[0].kind,
            AllowKind::FolderUse { from, to }
                if from == Path::new("src/native/landing") && to == Path::new("src/native/foundation")
        ),
        "{:?}",
        entries[0].kind
    );
    assert_eq!(
        entries[1].text,
        "file-uses src/native/command command_options.rs -> command_options_long.rs"
    );
    assert!(
        matches!(
            &entries[1].kind,
            AllowKind::FileUse { folder, from, to }
                if folder == Path::new("src/native/command")
                    && from == "command_options.rs"
                    && to == "command_options_long.rs"
        ),
        "{:?}",
        entries[1].kind
    );
    assert!(
        matches!(
            &entries[2].kind,
            AllowKind::Key(key) if key == "glob src/native/lib.rs invocation::*"
        ),
        "{:?}",
        entries[2].kind
    );
}

/// A rule asks the list about one use and about one key,
/// and an arrow points one way only.
#[test]
fn a_use_is_tolerated_one_way_and_a_key_by_its_exact_text() {
    let list: AllowList = parsed(
        "uses src/a -> src/b\n\
         file-uses src/c one.rs -> two.rs\n\
         glob src/d/lib.rs child::*\n",
    );
    assert!(list.covers_use(Path::new("src/a"), Path::new("src/b")));
    assert!(!list.covers_use(Path::new("src/b"), Path::new("src/a")));
    assert!(list.covers_file_use(Path::new("src/c"), "one.rs", "two.rs"));
    assert!(!list.covers_file_use(Path::new("src/c"), "two.rs", "one.rs"));
    assert!(!list.covers_file_use(Path::new("src/other"), "one.rs", "two.rs"));
    assert!(list.covers_key("glob src/d/lib.rs child::*"));
    assert!(!list.covers_key("glob src/d/lib.rs other::*"));
}

/// A use line needs two folders around the arrow.
#[test]
fn a_use_line_without_two_folders_is_an_error() {
    let error: String = error_of("uses src/a\n");
    assert!(error.contains("Line 1"), "{error}");
    assert!(error.contains("uses <folder> -> <folder>"), "{error}");
}

/// A file use line needs a folder and two file names around the arrow.
#[test]
fn a_file_use_line_without_a_folder_and_two_files_is_an_error() {
    for line in [
        "file-uses src/a one.rs\n",
        "file-uses src/a one.rs -> \n",
        "file-uses src/a one.rs two.rs\n",
    ] {
        let error: String = error_of(line);
        assert!(
            error.contains("file-uses <folder> <file> -> <file>"),
            "{error}"
        );
    }
}

/// A path is relative to the package folder,
/// so an absolute one and one that climbs out are errors.
#[test]
fn a_path_that_is_absolute_or_climbs_out_is_an_error() {
    for line in [
        "uses /home/someone/src/a -> src/b\n",
        "uses src/a -> ../../elsewhere/b\n",
        "file-uses ../other one.rs -> two.rs\n",
    ] {
        let error: String = error_of(line);
        assert!(error.contains("relative to the package folder"), "{error}");
    }
}

/// A list that names one thing twice is not a list of distinct findings.
#[test]
fn a_repeated_entry_is_an_error() {
    for lines in [
        "uses src/a -> src/b\nuses src/a -> src/b\n",
        "glob src/a/lib.rs child::*\nglob src/a/lib.rs child::*\n",
    ] {
        let error: String = error_of(lines);
        assert!(error.contains("repeats the entry"), "{error}");
    }
}

/// A list file that is not there is an error that names it.
#[test]
fn a_list_file_that_is_not_there_is_an_error() {
    let fixture: Fixture = Fixture::new();
    let missing = fixture.path.join("absent.txt");
    let error: String = read_allow_list(&missing).expect_err("an absent list is an error");
    assert!(error.contains("cannot be read"), "{error}");
    assert!(error.contains("absent.txt"), "{error}");
}

/// A list file that is there is read whole.
#[test]
fn a_list_file_that_is_there_is_read_whole() {
    let fixture: Fixture = Fixture::new();
    let written = write(
        &fixture.path,
        "allow-list.txt",
        "# one entry\nuses src/a -> src/b\n",
    );
    let list: AllowList = read_allow_list(&written).expect("the fixture list reads");
    assert_eq!(list.entries().len(), 1);
    assert_eq!(list.source(), written.as_path());
}

/// A path is written with `/` on every machine,
///  so an entry that holds a backslash is refused instead of naming a folder no key names.
#[test]
fn a_path_with_a_backslash_is_refused() {
    let error: String = error_of("uses src\\a -> src/b\n");
    assert!(error.contains("backslash"), "{error}");
    assert!(error.contains("relative to the package folder"), "{error}");
}

/// A `.` segment and a trailing separator name the folder a finding names,
///  so a line spelled either way tolerates the use,
///  and a line that repeats the use is refused however it is spelled.
#[test]
fn a_dot_segment_and_a_trailing_separator_name_the_folder_findings_show() {
    let list: AllowList = parsed("uses ./src/a/ -> src/b\n");
    assert!(list.covers_use(Path::new("src/a"), Path::new("src/b")));
    let error: String = error_of("uses src/a -> src/b\nuses ./src/a/ -> src/b\n");
    assert!(error.contains("repeats the entry"), "{error}");
}
