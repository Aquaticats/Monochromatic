//! What:
//!  Controls for the file a `--pathspec-from-file` option names.
//! Why:
//!  The value of that option is a file name the person typed,
//! so it is bytes:
//!  a spelling with replacement characters names a file nothing wrote,
//!  and the commit then fails where native Git succeeds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readFileSync(pathspecFileForWhere(scratch, cwd, source))).toEqual(Buffer.from('a.txt\n'));
//! ```

use super::the_pathspec_file;
use crate::command_commit::parse_commit_region;
use crate::path_bytes::{argument_bytes, os_from_path_bytes};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  The file a `--pathspec-from-file` value names,
///  as the path the preparation reads.
/// Why:
///   The value's bytes are the file name the person typed,
///  so a name that is not UTF-8 is opened as it is and never as its spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathspecFile(region, ['--pathspec-from-file=' + raw], scratch).toString()).toEqual(raw)
/// ```
#[test]
fn a_pathspec_file_whose_name_is_not_text_is_read_where_the_person_named_it() {
    let root: PathBuf = fixture("pathspec-file-name-bytes");
    let cwd: &Path = root.as_path();
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).expect("fixture scratch");
    let name: std::ffi::OsString = os_from_path_bytes(b"paths-\xff");
    std::fs::write(root.join(name.as_os_str()), b"a.txt\n").expect("fixture pathspec file");
    let mut option: std::ffi::OsString = std::ffi::OsString::from("--pathspec-from-file=");
    option.push(name.as_os_str());
    let arguments: Vec<std::ffi::OsString> = vec![option];
    let region = parse_commit_region(arguments.as_slice(), &[]).expect("a region Git accepts");
    let read: PathBuf = the_pathspec_file(&region, arguments.as_slice(), scratch.as_path(), cwd, None).expect("the file the option names").expect("a named file");
    assert_eq!(argument_bytes(read.as_os_str()), argument_bytes(root.join(name.as_os_str()).as_os_str()));
    assert_eq!(std::fs::read(read.as_path()).expect("the file the person named"), b"a.txt\n");
    remove(root.as_path());
}

/// What:
///  A pathspec file named `-` is the standard input read once,
///  written into the scratch directory.
/// Why:
///   A second read would see nothing,
/// so every preparation after the first reads the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathspecFile(region, ['--pathspec-from-file=-'], scratch, cwd, stdin).toString()).toEqual(scratch + '/pathspec.stdin')
/// ```
#[test]
fn standard_input_becomes_a_file_in_the_scratch_directory() {
    let root: PathBuf = fixture("pathspec-file-standard-input");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).expect("fixture scratch");
    let arguments: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("--pathspec-from-file=-")];
    let region = parse_commit_region(arguments.as_slice(), &[]).expect("a region Git accepts");
    let read: PathBuf = the_pathspec_file(&region, arguments.as_slice(), scratch.as_path(), root.as_path(), Some(b"a.txt\n"))
        .expect("the file the option names")
        .expect("a named file");
    assert_eq!(std::fs::read(read.as_path()).expect("the copy of the standard input"), b"a.txt\n");
    remove(root.as_path());
}

/// What:
///  A line that names no pathspec file reads none.
/// Why:
///  The preparation then stages the pathspecs of the command line,
/// and a file nothing named would be read instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathspecFile(region, ['--', 'a.txt'], scratch)).toBeUndefined()
/// ```
#[test]
fn a_line_without_the_option_names_no_file() {
    let root: PathBuf = fixture("pathspec-file-absent");
    let arguments: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("--"), std::ffi::OsString::from("a.txt")];
    let region = parse_commit_region(arguments.as_slice(), &[]).expect("a region Git accepts");
    assert!(the_pathspec_file(&region, arguments.as_slice(), root.as_path(), root.as_path(), None).expect("no file to read").is_none());
    remove(root.as_path());
}
