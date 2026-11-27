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
use std::{
    io::Read,
    path::Path,
};

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
///  so an absolute one is an error,
///  while one that climbs out with `..` names a folder the walk of a `#[path]` module reaches.
#[test]
fn an_absolute_path_is_an_error_and_a_climbing_one_names_the_folder() {
    for line in [
        "uses /home/someone/src/a -> src/b\n",
        "uses src/a -> /home/someone/src/b\n",
    ] {
        let error: String = error_of(line);
        assert!(error.contains("relative to the package folder"), "{error}");
    }
    // What: a folder outside the package is the walk from it, which is how every key writes that folder.
    // Why: a module a `#[path]` names elsewhere is a folder of the same tree, so its lines must read back.
    let list: AllowList =
        parsed("uses ../outside -> src/g/c0\nfile-uses ../outside one.rs -> two.rs\n");
    assert!(list.covers_use(Path::new("../outside"), Path::new("src/g/c0")));
    assert!(list.covers_file_use(Path::new("../outside"), "one.rs", "two.rs"));
}

/// A folder name may hold a space,
///  so a `uses` line is read from the two sides of the arrow it holds.
///  The four words are read when the line sets them off with one whitespace character each,
///  and every other line is read from its arrow with no space taken off either side.
#[test]
fn a_use_line_reads_the_two_sides_of_its_arrow() {
    let list: AllowList = parsed(
        "uses src/a b -> src/g/c0\n\
         uses src/x -> src/a b\n\
         uses src/c\u{a0}d -> src/x\n\
         uses\tsrc/x\t->\tsrc/g/c1\n",
    );
    assert!(list.covers_use(Path::new("src/a b"), Path::new("src/g/c0")));
    assert!(list.covers_use(Path::new("src/x"), Path::new("src/a b")));
    assert!(list.covers_use(Path::new("src/c\u{a0}d"), Path::new("src/x")));
    // The control: a line written with tabs is still the four words it spells.
    assert!(list.covers_use(Path::new("src/x"), Path::new("src/g/c1")));
    // What: two folders whose names start with a space, which the rule prints one space after `uses` and after the arrow.
    // Why: reading that one space away as if it were part of the word would name the folder beside each of them.
    let spelled: AllowList = parsed("uses  a -> src/b\nuses src/x ->  a\n");
    assert!(spelled.covers_use(Path::new(" a"), Path::new("src/b")));
    assert!(spelled.covers_use(Path::new("src/x"), Path::new(" a")));
    assert!(!spelled.covers_use(Path::new("a"), Path::new("src/b")));
    assert!(!spelled.covers_use(Path::new("src/x"), Path::new("a")));
}

/// A `uses` line whose arrow the reader cannot place is an error,
///  because a folder name holds no arrow of its own.
#[test]
fn a_use_line_with_two_arrows_is_an_error() {
    let error: String = error_of("uses src/a -> b -> src/c\n");
    assert!(error.contains("uses <folder> -> <folder>"), "{error}");
    assert!(error.contains("no folder name holds"), "{error}");
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

/// What:
///  A reader that yields a stated number of bytes and records how many it yielded.
/// Why:
///  A list read must stop at the cap itself, and a reader that could give more shows where it stopped;
///  `std::io::repeat` would simply run for ever, so a test that lost the stop would hang instead of failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Counted { left: number; served = 0; read(buffer: Uint8Array): number { /* one byte at a time */ } }
/// ```
struct Counted {
    /// Bytes this reader will still yield.
    left: u64,
    /// Bytes it has yielded.
    served: u64,
}

/// Serve up to the caller's buffer from what is left,
///  filling it with one byte value the list reader never reads as a line.
impl Read for Counted {
    /// Fill part of `buffer` and count what was taken from `left`.
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> std::io::Result<usize> {
        if self.left == 0 || buffer.is_empty() {
            return Ok(0);
        }
        // What: how many bytes this call may yield, as the machine's buffer length type.
        // Why: the buffer is `usize` long and the remaining count is `u64`, and the smaller one wins.
        let count: usize = usize::try_from(self.left.min(buffer.len() as u64))
            .expect("a buffer length fits the platform's word");
        for byte in &mut buffer[..count] {
            *byte = b'x';
        }
        self.left -= count as u64;
        self.served += count as u64;
        return Ok(count);
    }
}

/// A list as long as the one the densest loop measured here needs is read,
///  so the bound the module documents is the size that loop asked a round to write.
#[test]
fn a_list_as_long_as_the_dense_loop_needs_is_read() {
    // What: the bytes the densest loop a group's own file closes needed, measured on 2026-11-26
    // (`layout_allow_list::MAX_BYTES` records the same measurement).
    // Why: a bound of one mebibyte refused that very list, and every line of it is one use the rule asked to be recorded.
    const RECORDED_DENSE_LOOP_BYTES: usize = 1_681_290;
    let mut text: String = String::new();
    let mut lines: usize = 0;
    while text.len() < RECORDED_DENSE_LOOP_BYTES {
        text.push_str(format!("uses src/f{lines} -> src/g{lines}/c{lines}\n").as_str());
        lines += 1;
    }
    let fixture: Fixture = Fixture::new();
    let written = write(&fixture.path, "allow-list.txt", text.as_str());
    let list: AllowList = read_allow_list(&written).expect("the dense loop's list reads");
    assert_eq!(list.entries().len(), lines);
}

/// A reader that yields more than the cap is refused whatever its file's measured length was,
///  and the read itself stops one byte past the cap rather than reading the reader whole.
#[test]
fn a_reader_longer_than_the_cap_is_refused() {
    let path: &Path = Path::new("package/allow-list.txt");
    // What: an endless stream of one byte, cut one byte past the cap.
    // Why: it stands in for a file that grew after its length was measured, and it is over the cap without being on disk.
    let at_cap: std::io::Take<std::io::Repeat> = std::io::repeat(b'x').take(super::MAX_BYTES + 1);
    let error: String = super::read_at_most(at_cap, path).expect_err("one byte over the cap");
    assert!(error.contains("is longer than"), "{error}");
    assert!(error.contains("allow-list.txt"), "{error}");
    // The control: a reader under the cap is read whole, so the check above refuses only what is over.
    let under: &[u8] = b"uses src/a -> src/b\n";
    assert_eq!(
        super::read_at_most(under, path).expect("under the cap"),
        under
    );
}

/// The read stops at the cap even when the reader could give two bytes more,
///  so the bound the module documents is the one that ends the read.
#[test]
fn the_read_stops_one_byte_past_the_cap() {
    let path: &Path = Path::new("package/allow-list.txt");
    // What: a reader with two bytes more than the cap, kept so the test can ask what was taken from it.
    // Why: a reader cut at the cap beforehand would refuse for the wrong reason and pass with the stop removed.
    let mut reader: Counted = Counted {
        left: super::MAX_BYTES + 2,
        served: 0,
    };
    let error: String = super::read_at_most(&mut reader, path).expect_err("two bytes over the cap");
    assert!(error.contains("is longer than"), "{error}");
    // One byte past the cap is enough to know the file grew, and the second byte is never asked for.
    assert_eq!(reader.served, super::MAX_BYTES + 1);
    assert_eq!(reader.left, 1);
    // The control: a reader under the cap is read to its end, so the stop above cuts only what is over.
    let mut short: Counted = Counted {
        left: 4,
        served: 0,
    };
    assert_eq!(
        super::read_at_most(&mut short, path).expect("under the cap"),
        b"xxxx"
    );
    assert_eq!(short.served, 4);
}
