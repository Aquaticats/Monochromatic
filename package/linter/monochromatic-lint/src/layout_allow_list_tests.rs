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
    counted_work,
    dead_entry_findings,
    parse_allow_list,
    read_allow_list,
    tolerated,
};
use crate::{
    layout_allow_entries::name_holds_an_arrow_word,
    layout_finding::LayoutFinding,
    run_test_support::write,
    test_fs::Fixture,
};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{
        Path,
        PathBuf,
    },
};

/// Read a list from text,
/// as the file it would be,
/// with every listed path read back against the package folder it belongs to.
fn parsed(lines: &str) -> AllowList {
    return parse_allow_list(
        lines,
        Path::new("package/allow-list.txt"),
        Path::new("package"),
    )
    .expect("the fixture list is valid");
}

/// The message of the error a bad list raises.
fn error_of(lines: &str) -> String {
    return parse_allow_list(
        lines,
        Path::new("package/allow-list.txt"),
        Path::new("package"),
    )
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

/// A name before the arrow that ends in whitespace is a misspelling that names no folder,
///  because no folder name of a tree the rules accept can end in whitespace:
///  a name whose last character the reader trims is refused before any check.
///  One whitespace character sets each word off, and any second one beside a name belongs to that name,
///  on both sides of the arrow and after the word `uses`,
///  so the three leftovers of a column-aligned line all read alike.
#[test]
fn a_run_of_whitespace_beside_a_name_is_part_of_that_name() {
    let list: AllowList = parsed(
        "uses src/a  -> src/b\n\
         uses src/c\t -> src/d\n\
         uses src/e ->  src/f\n\
         uses  src/g -> src/h\n",
    );
    assert!(list.covers_use(Path::new("src/a "), Path::new("src/b")));
    assert!(list.covers_use(Path::new("src/c\t"), Path::new("src/d")));
    assert!(list.covers_use(Path::new("src/e"), Path::new(" src/f")));
    assert!(list.covers_use(Path::new(" src/g"), Path::new("src/h")));
    // The control: the same uses set off with one space each are the uses they spell.
    let plain: AllowList = parsed("uses src/a -> src/b\n");
    assert!(plain.covers_use(Path::new("src/a"), Path::new("src/b")));
    assert!(!plain.covers_use(Path::new("src/a "), Path::new("src/b")));
}

/// The six spellings of one gap the alpha.19 read built, and the two-character ones' spare character.
/// Why:
///  A tab and a no-break space align a list as a space does, and a doubled gap is one separator character
///  and one character of the name it stands beside, so each spelling's reading follows from the gap alone.
const GAPS: [&str; 6] = [" ", "  ", "\t", " \t", "\t ", "\u{a0}"];

/// What:
///  The whitespace one gap leaves beside the name after it,
///  which is everything after the character that sets the word off.
/// Why:
///  A gap on the left of a name spends its first character as the separator, so what is left belongs to the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftOver = (gap) => gap.slice(1);
/// ```
fn left_of_a_name(gap: &str) -> String {
    return gap.chars().skip(1).collect::<String>();
}

/// What:
///  The whitespace one gap leaves beside the name before it,
///  which is everything before the character that sets the arrow off.
/// Why:
///  A gap on the right of a name spends its last character as the separator, so what is left belongs to the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rightOfName = (gap) => gap.slice(0, -1);
/// ```
fn right_of_a_name(gap: &str) -> String {
    let mut kept: String = gap.chars().collect::<String>();
    kept.pop();
    return kept;
}

/// A `uses` line is read whatever whitespace character sets its words off,
///  and the two sides of the arrow are read alike:
///  every gap of one whitespace character reads the names it spells,
///  and any second character beside a name belongs to that name,
///  on either side of the arrow and after the word `uses`.
///  The spellings are the ones the alpha.19 read built:
///  every combination of one space, two spaces, a tab, a space then a tab, a tab then a space
///  and a no-break space in the line's three gaps, and four padded ends.
#[test]
fn every_spelling_of_a_use_line_reads_the_names_it_sets_off() {
    let mut spellings: usize = 0;
    for after_the_word in GAPS {
        for before_the_arrow in GAPS {
            for after_the_arrow in GAPS {
                let line: String =
                    format!("uses{after_the_word}src/a{before_the_arrow}->{after_the_arrow}src/b");
                let list: AllowList = parsed(format!("{line}\n").as_str());
                // What: the folder the line names before its arrow, and the one it names after it.
                // Why: one whitespace character per gap is the separator, and every other character there is the name's own.
                let from: String = format!(
                    "{}src/a{}",
                    left_of_a_name(after_the_word),
                    right_of_a_name(before_the_arrow)
                );
                let to: String = format!("{}src/b", left_of_a_name(after_the_arrow));
                assert!(
                    list.covers_use(Path::new(from.as_str()), Path::new(to.as_str())),
                    "`{line}` must read the use `{from}` -> `{to}`"
                );
                spellings += 1;
            }
        }
    }
    // What: the four padded ends, which the reader trims before it reads a line at all.
    // Why: padding is the plainest way to line a list up, and it must keep reading as the use it spells.
    for line in [
        "  uses src/a -> src/b",
        "uses src/a -> src/b  ",
        "\tuses src/a -> src/b",
        "uses src/a -> src/b\t",
    ] {
        let list: AllowList = parsed(format!("{line}\n").as_str());
        assert!(
            list.covers_use(Path::new("src/a"), Path::new("src/b")),
            "`{line}` must read the use `src/a` -> `src/b`"
        );
        spellings += 1;
    }
    // What: the count the alpha.19 read built, named so a spelling lost from the table above is a failure.
    // Why: every one of them is a line a person can type, and the read's own table is the whole of its evidence.
    const SPELLINGS_THE_READ_BUILT: usize = 220;
    assert_eq!(spellings, SPELLINGS_THE_READ_BUILT);
}

/// A line the reader cannot place says which whitespace characters set a use off and where they stand,
///  because a person who typed a tab or left the arrow bare has to know which character to change.
///  The character and its place are named, never only the shape of a line that already looks like the one typed.
#[test]
fn a_use_line_the_reader_cannot_place_names_the_whitespace_it_needs() {
    for line in [
        "uses src/a->src/b",
        "uses src/a => src/b",
        "uses src/a ->",
        "uses src/a",
    ] {
        let error: String = error_of(format!("{line}\n").as_str());
        assert!(error.contains("Line 1"), "{error} in {line}");
        assert!(
            error.contains("one whitespace character after the word `uses`"),
            "{line} must be told which whitespace character sets its words off: {error}"
        );
        assert!(
            error.contains("one before the arrow and one after it"),
            "{line} must be told where the whitespace stands: {error}"
        );
    }
}

/// A line holds no name before its arrow when the word and the arrow are set off by two spaces,
///  so the line is a misspelling of the shape.
#[test]
fn a_use_line_with_no_name_before_its_arrow_is_an_error() {
    // What: one line with two spaces and one with a single space before the arrow.
    // Why: the single space reaches the reader's default for a name that is not there, and both hold no name.
    for line in ["uses  -> src/b", "uses -> src/b"] {
        let error: String = error_of(format!("{line}\n").as_str());
        assert!(error.contains("Line 1"), "{error}");
        assert!(error.contains("uses <folder> -> <folder>"), "{error}");
        assert!(
            error.contains("No folder name stands before the arrow"),
            "`{line}` must be told no name stands there: {error}"
        );
    }
}

/// A run of two spaces in front of a name reads as that name's own leading space,
///  which the rule prints for a folder whose first name part begins with one,
///  so the four words alone never decide where a name starts.
#[test]
fn a_run_of_two_spaces_in_front_of_a_name_reads_as_its_own_leading_space() {
    let list: AllowList = parsed("uses  src/a -> src/b\n");
    assert!(list.covers_use(Path::new(" src/a"), Path::new("src/b")));
    assert!(!list.covers_use(Path::new("src/a"), Path::new("src/b")));
}

/// A `..` inside a path reads back over the folder before it,
///  so a line that climbs and comes back names the folder the walk names.
#[test]
fn a_climb_inside_a_path_reads_back_over_the_folder_before_it() {
    let list: AllowList = parsed("uses src/a/../b -> src/c\nuses ../a/../b -> src/d\n");
    assert!(list.covers_use(Path::new("src/b"), Path::new("src/c")));
    assert!(list.covers_use(Path::new("../b"), Path::new("src/d")));
}

/// A `uses` line whose arrow the reader cannot place is an error,
///  because a folder name holds no arrow of its own:
///  the line sets a second arrow off with whitespace, so it names no pair of folders.
#[test]
fn a_use_line_with_two_arrows_is_an_error() {
    let error: String = error_of("uses src/a -> b -> src/c\n");
    assert!(error.contains("uses <folder> -> <folder>"), "{error}");
    assert!(error.contains("no folder name holds"), "{error}");
    assert!(
        error.contains("two `->` are set off by whitespace"),
        "the sentence must say what is wrong with the line that was typed: {error}"
    );
}

/// A second arrow set off with tabs is refused as one set off with spaces is,
///  because whitespace sets an arrow off whatever character it is,
///  and a reader that looked for the literal spelling would read a folder name that holds a tab.
#[test]
fn a_use_line_whose_second_arrow_is_set_off_by_tabs_is_an_error() {
    let error: String = error_of("uses src/a -> src/b\t->\tsrc/c\n");
    assert!(error.contains("no folder name holds"), "{error}");
    assert!(
        error.contains("two `->` are set off by whitespace"),
        "{error}"
    );
}

/// A bare `->` inside a folder name is no arrow of the line:
///  the reader takes the first arrow that whitespace sets off on both sides,
///  and the rule prints such a folder name in a line its own reader reads back.
#[test]
fn a_bare_arrow_inside_a_name_is_not_the_lines_arrow() {
    let list: AllowList = parsed("uses src/x->y -> src/z\n");
    assert!(
        list.covers_use(Path::new("src/x->y"), Path::new("src/z")),
        "the arrow after the name is the line's own"
    );
    assert!(
        !list.covers_use(Path::new("src/x"), Path::new("y -> src/z")),
        "the arrow glued inside the name is no separator"
    );
}

/// A folder name whose `->` touches a character holds no arrow word,
///  because only whitespace on both sides sets one off;
///  the folder rule therefore reads a tree holding `x->` or `->x`,
///  where a reader that took one side for enough would stop it.
#[test]
fn a_folder_name_that_touches_the_arrow_word_without_whitespace_holds_none() {
    // What: the names whose arrow touches a character on one side, which a `uses` line still reads.
    // Why: such a name is no shape the reader could mistake for its own arrow, so no line could misread it.
    for name in ["x->", "->x", "x->y"] {
        assert!(
            !name_holds_an_arrow_word(name),
            "`{name}` holds no arrow word"
        );
    }
    // What: the control, names whose arrow whitespace sets off, which the reader really would misread.
    // Why: a check that answers no for every name would pass the loop above and protect nothing.
    for name in ["x -> y", "-> y", "x ->", "->"] {
        assert!(
            name_holds_an_arrow_word(name),
            "`{name}` holds an arrow word"
        );
    }
}

/// A dead `uses` or `file-uses` line that names a folder no source folder of the package is called names it,
///  so a hand-aligned or mistyped line is told to be re-spaced,
///  and a line whose folders the package does hold is told only to be deleted.
///  A folder on disk that holds no module file is no source folder, so the sentence holds for it too.
#[test]
fn a_dead_use_line_names_a_folder_the_package_does_not_hold() {
    let base: &Path = Path::new("package");
    let list: AllowList = parsed(
        "uses src/a -> src/b extra\n\
         file-uses src/c one.rs -> two.rs\n\
         uses src/gone -> src/also-gone\n\
         uses src/a -> src/b\n",
    );
    // What: the folders of the fixture package, as the walk names them.
    // Why: an entry is never used here, so every line is dead and each message can be read.
    let folders: BTreeSet<PathBuf> = ["src/a", "src/b"]
        .iter()
        .map(|name| return base.join(name))
        .collect::<BTreeSet<PathBuf>>();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &folders,
        &BTreeSet::<PathBuf>::new(),
        base,
    );
    assert_eq!(findings.len(), 4, "{findings:#?}");
    assert!(
        findings[0].message.contains(
            "names the folder `src/b extra`. No source folder of this package has that name"
        ),
        "{}",
        findings[0].message
    );
    assert!(
        findings[1]
            .message
            .contains("names the folder `src/c`. No source folder of this package has that name"),
        "{}",
        findings[1].message
    );
    // What: the word that agrees with two folders the line names and the package lacks.
    // Why: a line never read names two folders, and its sentence must not speak of one.
    assert!(
        findings[2].message.contains(
            "names the folders `src/gone` and `src/also-gone`. No source folder of this package has that name"
        ),
        "{}",
        findings[2].message
    );
    assert!(
        findings[3]
            .message
            .contains("tolerates nothing any more. Delete the line"),
        "{}",
        findings[3].message
    );
}

/// A dead line names the folder it read with the whitespace beside that name spelled,
///  because a tab and a no-break space are invisible in a terminal:
///  a reader shown `src/a` would look for a folder the package may well hold,
///  where the line really read one whose name ends in a tab.
#[test]
fn a_dead_use_line_spells_the_whitespace_that_is_part_of_the_name() {
    let base: &Path = Path::new("package");
    let list: AllowList = parsed("uses src/a\t\t-> src/b\nuses src/x ->\u{a0}\u{a0}src/y\n");
    // What: the folders the fixture package holds, as the walk names them.
    // Why: only the two names a doubled gap put whitespace into are absent, so each message names one of them.
    let folders: BTreeSet<PathBuf> = ["src/b", "src/x", "src/y"]
        .iter()
        .map(|name| return base.join(name))
        .collect::<BTreeSet<PathBuf>>();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &folders,
        &BTreeSet::<PathBuf>::new(),
        base,
    );
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings[0]
            .message
            .contains("names the folder `src/a\\u{9}`"),
        "{}",
        findings[0].message
    );
    assert!(
        findings[1]
            .message
            .contains("names the folder `\\u{a0}src/y`"),
        "{}",
        findings[1].message
    );
}

/// A dead `uses` line whose gap holds two whitespace characters names the character the reader kept
///  and which one it took off as the separator,
///  because the line already reads as the shape it spells and its reader has to know which of the two to remove.
#[test]
fn a_dead_use_line_whose_gap_holds_two_characters_says_which_to_remove() {
    let base: &Path = Path::new("package");
    let list: AllowList = parsed("uses  src/alpha  ->  src/b\n");
    // What: the folder of the fixture the package holds, so only the mis-read name before the arrow is absent.
    // Why: a help that spoke of two absent folders would answer about a different line.
    let folders: BTreeSet<PathBuf> = ["src/b"]
        .iter()
        .map(|name| return base.join(name))
        .collect::<BTreeSet<PathBuf>>();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &folders,
        &BTreeSet::<PathBuf>::new(),
        base,
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let help: &str = findings[0].help.as_deref().expect("a dead line has help");
    for part in [
        "One whitespace character sets each word off",
        "took the character nearest the word `uses` or the arrow as the separator",
        "kept `\\u{20}` after the word `uses`, before `src/alpha`",
        "`\\u{20}` after `->`, before `src/b`",
        "remove every character listed there",
    ] {
        assert!(help.contains(part), "{part} missing in {help}");
    }
}

/// A `file-uses` line cannot name a folder whose name holds a space,
///  because its shape is five words set off with one whitespace character each,
///  so the refusal names that folder and says what to do about it
///  rather than sending its reader back to a line that already reads as the shape it names.
///  It does so only while those words name a folder the package holds:
///  the six-word lines below name no folder, so they are told about the shape they misspell.
#[test]
fn a_file_use_line_whose_folder_name_holds_a_space_is_refused_by_that_cause() {
    // What: a package folder holding the folder whose name has the space, and the same fixture without it.
    // Why: the sentence about a spaced folder is true only of a folder the tree really holds.
    let spaced: Fixture = Fixture::new();
    std::fs::create_dir_all(spaced.path.join("src/a b"))
        .expect("create the folder whose name holds a space");
    let error: String = parse_allow_list(
        "file-uses src/a b one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        spaced.path.as_path(),
    )
    .expect_err("no line can name that folder");
    assert!(error.contains("Line 1"), "{error}");
    assert!(error.contains("the folder `src/a b`"), "{error}");
    assert!(error.contains("holds a space"), "{error}");
    assert!(error.contains("Rename the folder"), "{error}");
    assert!(
        !error.contains("A use between two files is written"),
        "{error}"
    );
    // What: what is wrong with each line of six words whose words between the folder's place and the arrow name no folder.
    // Why: a third file name and a doubled arrow are misspellings of the shape, and telling their reader only the shape sends them back to a line that already looks like it.
    let plain: Fixture = Fixture::new();
    std::fs::create_dir_all(plain.path.join("src/ab")).expect("create the folder the line names");
    for (line, wrong) in [
        (
            "file-uses src/ab one.rs two.rs -> three.rs\n",
            "which is five words; this line is 6 words",
        ),
        (
            "file-uses src/ab one.rs -> -> two.rs\n",
            "two `->` are set off by whitespace",
        ),
        (
            "file-uses src/ab one.rs -> two.rs extra\n",
            "which is five words; this line is 6 words",
        ),
    ] {
        // What: the refusal one mistyped line earns, named apart from the spaced folder's own sentence above.
        // Why: one name for each answer keeps the two failures apart in a run's output.
        let mistyped: String =
            parse_allow_list(line, Path::new("allow-list.txt"), plain.path.as_path())
                .expect_err("the line misspells the shape");
        assert!(
            mistyped.contains("A use between two files is written"),
            "`{line}` must be told about the shape it misspells: {mistyped}"
        );
        assert!(
            mistyped.contains(wrong),
            "`{line}` must be told what is wrong with it, `{wrong}`: {mistyped}"
        );
        assert!(
            !mistyped.contains("holds a space"),
            "`{line}` names no folder that holds a space: {mistyped}"
        );
    }
}

/// A `file-uses` line cannot name a folder whose name holds any whitespace character,
///  because its shape is five words set off with one whitespace character each,
///  so a tab, a no-break space or a doubled space in that name makes a sixth word exactly as a space does.
///  The refusal names that folder and says what to do about it,
///  rather than sending its reader back to a line that already reads as the shape it names.
#[test]
fn a_file_use_line_whose_folder_name_holds_other_whitespace_is_refused_by_that_cause() {
    // What: one folder per whitespace class, and its name as the refusal spells it.
    // Why: only a space was held before, so a reader that asked the joined words for a folder stayed green for the others.
    for (named, spelled) in [
        ("src/c\td", "src/c\\u{9}d"),
        ("src/c\u{a0}d", "src/c\\u{a0}d"),
        ("src/c  d", "src/c  d"),
    ] {
        let fixture: Fixture = Fixture::new();
        std::fs::create_dir_all(fixture.path.join(named))
            .expect("create the folder whose name holds the whitespace");
        let error: String = parse_allow_list(
            format!("file-uses {named} one.rs -> two.rs\n").as_str(),
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
        .expect_err("no line can name that folder");
        assert!(error.contains("Line 1"), "{error}");
        assert!(
            error.contains(format!("the folder `{spelled}`").as_str()),
            "the refusal must name `{spelled}`: {error}"
        );
        assert!(error.contains("Rename the folder"), "{error}");
        assert!(
            !error.contains("A use between two files is written"),
            "`{named}` must not be told about the shape: {error}"
        );
    }
}

/// A dead `uses` or `file-uses` line spells a character a reader cannot see that the line holds in a name,
///  because such a character prints as nothing and the name it makes is no folder of the package:
///  a reader shown `src/a` would look for a folder the package may well hold.
#[test]
fn a_dead_use_line_spells_a_character_it_holds_that_a_reader_cannot_see() {
    let base: &Path = Path::new("package");
    // What: one line per place such a character stands, with the name as the message must spell it.
    // Why: a name's own invisible character and the folder of a `file-uses` line are read by two arms of the finding.
    for (line, spelled) in [
        ("uses src/a\u{200b} -> src/b\n", "src/a\\u{200b}"),
        ("uses src/a -> src/b\u{200b}\n", "src/b\\u{200b}"),
        (
            "file-uses src/a\u{200b} one.rs -> two.rs\n",
            "src/a\\u{200b}",
        ),
    ] {
        let list: AllowList = parsed(line);
        let findings: Vec<LayoutFinding> = dead_entry_findings(
            &list,
            &BTreeSet::<usize>::new(),
            &BTreeSet::<PathBuf>::new(),
            &BTreeSet::<PathBuf>::new(),
            base,
        );
        assert_eq!(findings.len(), 1, "{line}");
        assert!(
            findings[0].message.contains(spelled),
            "`{spelled}` must be spelled in the message: {}",
            findings[0].message
        );
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap_or("")
                .contains("take that character out"),
            "the help must say to take it out: {:?}",
            findings[0].help
        );
    }
}

/// A line whose first character is a byte order mark is read as a key and tolerates nothing,
///  so the message names that character and where it stands, and sends its reader to take it out;
///  deleting the line, which the old sentence said, would leave the loop the line records partly unwritten.
#[test]
fn a_byte_order_mark_before_a_line_is_named_where_it_stands_and_taken_out() {
    let base: &Path = Path::new("package");
    let list: AllowList = parsed("\u{feff}uses src/a -> src/b\n");
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &BTreeSet::<PathBuf>::new(),
        &BTreeSet::<PathBuf>::new(),
        base,
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let message: &str = findings[0].message.as_str();
    for part in [
        "\\u{feff}uses src/a -> src/b",
        "byte order mark",
        "character 1",
        "Take that character out of the line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert!(
        !message.contains("Delete the line so the list keeps shrinking"),
        "the line records a use, so deleting it is the wrong edit: {message}"
    );
}

/// A zero-width space where a separator belongs is not whitespace, so the reader takes the word it touches for a key,
///  and the message names the character and its position and says to take it out and put the separator back.
#[test]
fn a_zero_width_space_where_a_separator_belongs_is_named_and_taken_out() {
    let base: &Path = Path::new("package");
    let list: AllowList = parsed("uses\u{200b}src/a -> src/b\n");
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &BTreeSet::<PathBuf>::new(),
        &BTreeSet::<PathBuf>::new(),
        base,
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let message: &str = findings[0].message.as_str();
    for part in [
        "\\u{200b}",
        "zero-width space",
        "character 5",
        "Take that character out of the line",
        "write a whitespace character where it stood",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
}

/// A listed path is read back against the package folder,
///  so a line that climbs out with `..` and comes back in names the use the plain spelling names,
///  on either side of the arrow and on both at once.
#[test]
fn a_listed_path_is_read_back_against_the_package_folder() {
    for lines in [
        "uses ../package/src/a -> src/b\n",
        "uses src/a -> ../package/src/b\n",
        "uses ../package/src/a -> ../package/src/b\n",
    ] {
        let list: AllowList = parsed(lines);
        assert!(
            list.covers_use(Path::new("src/a"), Path::new("src/b")),
            "{lines}"
        );
    }
}

/// Two spellings of one use are the repeat a list refuses,
///  in a `uses` line on either side of the arrow and in a `file-uses` line,
///  because every listed path is read into one spelling before a repeat is decided.
#[test]
fn two_spellings_of_one_use_are_the_repeat_the_list_refuses() {
    for lines in [
        "uses src/a -> src/b\nuses ../package/src/a -> src/b\n",
        "uses src/a -> src/b\nuses src/a -> ../package/src/b\n",
        "uses src/a -> src/b\nuses ../package/src/a -> ../package/src/b\n",
        "file-uses src/ab one.rs -> two.rs\nfile-uses ../package/src/ab one.rs -> two.rs\n",
    ] {
        let error: String = error_of(lines);
        assert!(error.contains("repeats the entry"), "{error} in {lines}");
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
    let error: String =
        read_allow_list(&missing, fixture.path.as_path()).expect_err("an absent list is an error");
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
    let list: AllowList =
        read_allow_list(&written, fixture.path.as_path()).expect("the fixture list reads");
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
    let list: AllowList =
        read_allow_list(&written, fixture.path.as_path()).expect("the dense loop's list reads");
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

/// A key is answered from the index the list read once,
///  so tolerating one finding reads the one entry that holds its key and no entry before it,
///  where a scan over the entries would read each of them.
/// The control fails when the index is replaced by a scan:
///  the same list holds 64 keys and a scan of it reads 64 entries.
#[test]
fn a_key_lookup_reads_one_entry_of_the_list() {
    // What: the number of keys the fixture list holds, named so the last one is no magic number.
    // Why: the list must be long enough that a scan before the last key is unmistakable in the count.
    const KEYS: usize = 64;
    let mut text: String = String::new();
    for index in 0..KEYS {
        text.push_str(format!("glob src/a{index}/lib.rs child::*\n").as_str());
    }
    let list: AllowList = parsed(text.as_str());
    let finding: LayoutFinding = LayoutFinding {
        file: PathBuf::from("src/a63/lib.rs"),
        line: 1,
        message: String::from("a fixture finding about the last glob"),
        help: None,
        // What: the key of the last entry, which is the one the list holds at the end.
        // Why: a scan would read every entry before it, so the counter says which lookup this run made.
        key: format!("glob src/a{}/lib.rs child::*", KEYS - 1),
    };
    // What: the control, one entry counted by hand before the question is asked.
    // Why: a count of zero is only an answer if this counter records work at all.
    counted_work::note_entry();
    assert_eq!(
        counted_work::entries_walked(),
        1,
        "the counter records work"
    );
    counted_work::reset();
    let (kept, used): (Vec<LayoutFinding>, std::collections::BTreeSet<usize>) =
        tolerated(vec![finding], &list);
    assert_eq!(kept.len(), 0, "the last key is on the list");
    assert_eq!(
        used.len(),
        1,
        "the entry that holds it is the one that tolerated it"
    );
    assert_eq!(
        counted_work::entries_walked(),
        1,
        "the index names the one entry that holds the key, and the question reads that one"
    );
}

/// A rule that asks whether one key is tolerated reads no entry at all,
///  because the index the list built answers it,
///  where a scan over the entries would read every one of them.
#[test]
fn a_tolerated_key_is_answered_without_reading_an_entry() {
    // What: the number of keys the fixture list holds, named so the last one is no magic number.
    // Why: a scan would read every one of them for a question the index answers, which the count shows.
    const KEYS: usize = 64;
    let mut text: String = String::new();
    for index in 0..KEYS {
        text.push_str(format!("glob src/a{index}/lib.rs child::*\n").as_str());
    }
    let list: AllowList = parsed(text.as_str());
    // What: the control, one entry counted by hand before the question is asked.
    // Why: a count of zero is only an answer if this counter records work at all.
    counted_work::note_entry();
    assert_eq!(
        counted_work::entries_walked(),
        1,
        "the counter records work"
    );
    counted_work::reset();
    assert!(
        list.covers_key(format!("glob src/a{}/lib.rs child::*", KEYS - 1).as_str()),
        "the last key is on the list"
    );
    assert!(
        !list.covers_key("glob src/other/lib.rs child::*"),
        "a key the list does not hold is answered no"
    );
    assert_eq!(
        counted_work::entries_walked(),
        0,
        "the index answers both questions, so no entry is read"
    );
}

/// What:
///  A list whose key index names an entry that holds another key, and then an entry that holds none,
///  built here because a list read from a file builds its index from its own entries.
/// Why:
///  The lookup checks the entry its index names instead of trusting the number,
///  and only a list made by hand can show that check happening.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const list = { source, entries, uses: new Set(), keys: new Map([['glob ...', 1]]) };
/// ```
fn list_whose_index_names_another_entry(entry: usize) -> AllowList {
    let source: PathBuf = PathBuf::from("package/allow-list.txt");
    let entries: Vec<super::AllowEntry> = vec![
        super::AllowEntry {
            line: 1,
            column: 1,
            text: String::from("glob src/a/lib.rs child::*"),
            kind: AllowKind::Key(String::from("glob src/a/lib.rs child::*")),
        },
        // What: an entry that is no key line, which no key may be answered from.
        // Why: a use line tolerates a use and never a key, so the lookup has to refuse it.
        super::AllowEntry {
            line: 2,
            column: 1,
            text: String::from("uses src/a -> src/b"),
            kind: AllowKind::FolderUse {
                from: PathBuf::from("src/a"),
                to: PathBuf::from("src/b"),
            },
        },
    ];
    let mut keys: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::<String, usize>::new();
    keys.insert(String::from("glob src/b/lib.rs child::*"), entry);
    let mut identities: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::<String, usize>::new();
    for (index, one) in entries.iter().enumerate() {
        identities.insert(one.identity(), index);
    }
    return super::AllowList {
        source,
        entries,
        uses: std::collections::BTreeSet::new(),
        keys,
        identities,
    };
}

/// A lookup refuses an entry its index names that holds another key, and one that holds no key at all,
///  so a number is never taken for an answer on its own.
#[test]
fn a_key_the_index_names_for_another_entry_is_no_line_of_the_list() {
    // What: the position of each entry built above, named so neither is a magic number.
    // Why: the first entry holds another key and the second holds none, and one lookup answers each of them no.
    let holds_another_key: usize = 0;
    let holds_no_key: usize = 1;
    for entry in [holds_another_key, holds_no_key] {
        let list: AllowList = list_whose_index_names_another_entry(entry);
        assert_eq!(
            list.key_entry("glob src/b/lib.rs child::*"),
            None,
            "entry {entry} holds no such key, so no line tolerates the finding"
        );
    }
    // The control: a key the index names no entry for is answered no as well.
    let holding: AllowList = list_whose_index_names_another_entry(holds_another_key);
    assert_eq!(
        holding.key_entry("glob src/a/lib.rs child::*"),
        None,
        "the index names no entry for that key, so nothing is answered"
    );
}

/// A file use is answered by walking the entries,
///  which is what the README says of a file-use lookup,
///  so asking about the last pair of a long list reads every entry before it.
#[test]
fn a_file_use_lookup_walks_the_entries() {
    // What: the number of file-use lines the fixture list holds, named so the last one is no magic number.
    // Why: a walk over every entry would read each of them before the last pair, which the count shows.
    const LINES: usize = 64;
    let mut text: String = String::new();
    for index in 0..LINES {
        text.push_str(format!("file-uses src/a one{index}.rs -> two{index}.rs\n").as_str());
    }
    let list: AllowList = parsed(text.as_str());
    // What: the control, one entry counted by hand before the question is asked.
    // Why: a count of zero is only an answer if this counter records work at all.
    counted_work::note_entry();
    assert_eq!(
        counted_work::entries_walked(),
        1,
        "the counter records work"
    );
    counted_work::reset();
    assert!(
        list.covers_file_use(
            Path::new("src/a"),
            format!("one{}.rs", LINES - 1).as_str(),
            format!("two{}.rs", LINES - 1).as_str()
        ),
        "the last pair is on the list"
    );
    assert_eq!(
        counted_work::entries_walked(),
        LINES,
        "the last pair is found by walking every entry before it"
    );
}

/// The one dead-entry message and help a list earns, against the folders of the package it is read from.
fn one_dead_line(
    lines: &str,
    folders: &[&str],
) -> (String, String) {
    let list: AllowList = parsed(lines);
    // What: each folder the package holds, spelled from the package folder the list is read from.
    // Why: a dead line is judged against the folders the walk names from that base, so a folder given another way is no folder the package holds.
    let known: BTreeSet<PathBuf> = folders
        .iter()
        .map(|one| return Path::new("package").join(one))
        .collect();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &known,
        &BTreeSet::<PathBuf>::new(),
        Path::new("package"),
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    return (
        findings[0].message.clone(),
        findings[0].help.clone().unwrap_or_default(),
    );
}

/// A six-word `file-uses` line whose words name a real folder is refused by naming that folder,
///  whatever whitespace stands in its gaps and whether the name holds an arrow word.
///  The line cannot be satisfied by re-spacing it, because a folder whose name holds whitespace
///  is refused even when every gap is one character,
///  so renaming the folder is the only sentence its reader can act on;
///  and the folder named is one that is there, never another spelling of the words.
#[test]
fn a_six_word_file_use_line_names_the_folder_its_words_name() {
    // What: the tree each line is written for, the line, and the folder its refusal must name.
    // Why: a doubled gap and an arrow-named folder each sent the reader to a sentence about the word count.
    for (folders, line, named) in [
        (
            &["src/c d"][..],
            "file-uses src/c d one.rs  -> two.rs\n",
            "src/c d",
        ),
        (
            &["src/c d"][..],
            "file-uses  src/c d one.rs -> two.rs\n",
            "src/c d",
        ),
        (
            &["src/c d"][..],
            "file-uses src/c d  one.rs -> two.rs\n",
            "src/c d",
        ),
        (
            &["src/c\td"][..],
            "file-uses\t src/c\td one.rs -> two.rs\n",
            "src/c\\u{9}d",
        ),
        (
            &["src/c\td"][..],
            "file-uses  src/c\td one.rs -> two.rs\n",
            "src/c\\u{9}d",
        ),
        (
            &["src/x -> y"][..],
            "file-uses src/x -> y one.rs -> two.rs\n",
            "src/x -> y",
        ),
        (
            &["src/p ->"][..],
            "file-uses src/p -> one.rs -> two.rs\n",
            "src/p ->",
        ),
    ] {
        let fixture: Fixture = Fixture::new();
        for folder in folders {
            std::fs::create_dir_all(fixture.path.join(folder))
                .expect("create the folder the line is written for");
        }
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("no line can carry a folder name that holds whitespace");
        assert!(
            error.contains(format!("the folder `{named}`").as_str()),
            "`{line}` must name the folder `{named}`: {error}"
        );
        assert!(
            error.contains("Rename the folder"),
            "`{line}` must say what renaming gives: {error}"
        );
    }
}

/// A five-word `file-uses` line whose fourth word is no arrow is told which word stands there,
///  because a misspelt or misplaced arrow is the likeliest typo,
///  and "this line is 5 words" about a line of five words names no cause and cannot be acted on.
#[test]
fn a_five_word_file_use_line_whose_fourth_word_is_no_arrow_names_that_word() {
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/ab")).expect("create the folder the line names");
    for (line, fourth) in [
        ("file-uses src/ab one.rs => two.rs\n", "=>"),
        ("file-uses src/ab one.rs --> two.rs\n", "-->"),
        ("file-uses src/ab -> one.rs two.rs\n", "one.rs"),
        ("file-uses -> src/ab one.rs two.rs\n", "one.rs"),
        ("file-uses src/ab one.rs two.rs ->\n", "two.rs"),
    ] {
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("the fourth word is no arrow");
        assert!(
            error.contains("the fourth word must be `->`"),
            "`{line}` must say which word belongs there: {error}"
        );
        assert!(
            error.contains(format!("this line's fourth word is `{fourth}`").as_str()),
            "`{line}` must name the fourth word `{fourth}`: {error}"
        );
    }
}

/// A `file-uses` line whose word count is not five says that number,
///  with a word that agrees with it, because "this line is 1 words" is a sentence its reader stops trusting.
#[test]
fn a_file_use_line_missing_words_says_how_many_it_holds() {
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/ab")).expect("create the folder the line names");
    for (line, counted) in [
        ("file-uses\n", "this line is one word"),
        ("file-uses src/ab\n", "this line is 2 words"),
        (
            "file-uses src/ab one.rs -> two.rs extra\n",
            "this line is 6 words",
        ),
    ] {
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("the line holds the wrong number of words");
        assert!(
            error.contains(counted),
            "`{line}` must say `{counted}`: {error}"
        );
    }
}

/// A line a copied page led with a mark a reader cannot see is told to take that mark out,
///  and the message spells it and names it,
///  because "Delete the line" would leave the use the line records unwritten
///  and the mark itself would print as nothing.
#[test]
fn a_dead_line_led_by_a_character_a_reader_cannot_see_is_told_to_take_it_out() {
    let (message, help) = one_dead_line("\u{200e}uses src/a -> src/b\n", &[]);
    for part in [
        "a left-to-right mark (`\\u{200e}`)",
        "at character 1 of the line",
        "reads as a `uses` line",
        "Take that character out of the line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert!(
        !message.contains('\u{200e}'),
        "the mark itself must not be printed: {message}"
    );
    assert!(
        !message.contains("Delete the line so the list keeps shrinking"),
        "the line records a use, so deleting it is the wrong edit: {message}"
    );
    assert!(help.contains("Take that character out"), "{help}");
}

/// A refusal quotes the line it refuses with every character a reader cannot see spelled,
///  and says where that character stands and what to do with it,
///  because a refusal stops the whole rule, so it is the message a reader most needs to act on.
#[test]
fn a_refusal_spells_the_character_it_quotes_and_says_to_take_it_out() {
    let error: String = error_of("uses src/a\u{200e}-> src/b\n");
    for part in [
        "`uses src/a\\u{200e}-> src/b`",
        "a left-to-right mark (`\\u{200e}`)",
        "at character 11 of the line",
        "take it out",
    ] {
        assert!(error.contains(part), "{part} missing in {error}");
    }
    assert!(
        !error.contains('\u{200e}'),
        "the mark itself must not be printed: {error}"
    );
}

/// A dead `file-uses` line whose file name holds a character a reader cannot see is told to take it out,
///  because the name the line holds is not the name on disk,
///  and deleting the line would leave the use unrecorded.
#[test]
fn a_dead_file_use_line_whose_file_name_holds_an_unseen_character_is_told_to_take_it_out() {
    let (message, help) = one_dead_line("file-uses src/f one.rs\u{200b} -> two.rs\n", &["src/f"]);
    for part in [
        "a zero-width space (`\\u{200b}`)",
        "at character 23 of the line",
        "Take that character out of the line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert!(
        !message.contains("Delete the line so the list keeps shrinking"),
        "taking the character out is the edit: {message}"
    );
    assert!(help.contains("Take that character out"), "{help}");
}

/// A character a reader cannot see is placed by its character count from the line's first one,
///  because the entry's own index is off by the indent and a byte count is off by every two-byte letter before it,
///  while an editor may show a tab as several cells or write one character as more than one byte.
#[test]
fn the_place_a_character_a_reader_cannot_see_stands_at_is_a_column_of_the_line() {
    // What: an indented line, whose mark stands at character 4 of the line but at index 1 of the trimmed entry.
    // Why: the message said "character 1 of the entry it read", which no count from the line's first character gives.
    let (indented, _) = one_dead_line("   \u{feff}uses src/a -> src/b\n", &[]);
    assert!(
        indented.contains("at character 4 of the line"),
        "the place is a column of the line: {indented}"
    );
    // What: a refusal whose line holds a two-byte letter before the mark.
    // Why: counting bytes would answer 13 here, one past the character the line holds at 12.
    let two_byte: String = error_of("uses src/a\u{e9}\u{200e}-> src/b\n");
    assert!(
        two_byte.contains("at character 12 of the line"),
        "characters are counted, not bytes: {two_byte}"
    );
}

/// A dead line that would read as a `file-uses` line once its character a reader cannot see is out
///  names that word, which is the line's own,
///  because telling the reader of a `file-uses` line that its first word is not `uses` names a word the line never had.
#[test]
fn the_word_a_dead_line_would_read_with_is_the_line_s_own_word() {
    let (message, _) = one_dead_line("\u{feff}file-uses src/a one.rs -> two.rs\n", &[]);
    assert!(
        message.contains("reads as a `file-uses` line"),
        "the line's own word is `file-uses`: {message}"
    );
    assert!(
        !message.contains("not `uses`"),
        "the line never begins with `uses`: {message}"
    );
}

/// A line whose first word merely begins with the word `uses` is no `uses` line,
///  so taking its character out is not the edit, and the message does not offer it:
///  deleting the line is, because the line names a finding no rule prints.
#[test]
fn a_key_that_begins_with_the_word_is_not_told_to_take_a_character_out() {
    let (message, help) = one_dead_line("uses\u{e9}\u{200b} src/a -> src/b\n", &[]);
    assert!(
        message.contains("Delete the line so the list keeps shrinking"),
        "a key that reads as nothing must be deleted: {message}"
    );
    assert!(
        !message.contains("Take that character out"),
        "taking the character out leaves a key: {message}"
    );
    assert!(!help.contains("Take that character out"), "{help}");
}

/// The help of a dead line spells the name it says the gap stands before,
///  because that sentence goes on to say the name holds a character a reader cannot see,
///  and a right-to-left override prints nothing while it reorders what follows it.
#[test]
fn the_help_of_a_dead_line_spells_the_name_it_says_the_gap_stands_before() {
    // What: a doubled gap in front of a name holding U+202E, absent from the fixture package.
    // Why: the absent folder is what earns the help that names the gap and the folder it stands before.
    let (message, help) = one_dead_line("uses  src/a\u{202e}b -> src/c\n", &["src/c"]);
    assert!(
        help.contains("before `src/a\\u{202e}b`"),
        "the name is spelled where the gap is named: {help}"
    );
    assert!(
        !help.contains('\u{202e}'),
        "the override itself is not printed: {help}"
    );
    assert!(
        message.contains("`\\u{20}src/a\\u{202e}b`"),
        "the quote spells it too: {message}"
    );
}

/// The help of a dead line names every whitespace character its gap kept,
///  not the first one,
///  because a gap of three characters leaves a dead line for anyone who removes one.
#[test]
fn the_help_of_a_dead_line_names_every_character_its_gap_kept() {
    let (_, help) = one_dead_line("uses   src/a -> src/b\n", &["src/a", "src/b"]);
    for part in [
        "It kept 2 whitespace characters (`\\u{20}` and `\\u{20}`) after the word `uses`, before `src/a`",
        "remove every character listed there",
    ] {
        assert!(help.contains(part), "{part} missing in {help}");
    }
}

/// The refusal of a folder whose name holds whitespace names that character,
///  with a tab called a tab and a no-break space called a no-break space,
///  because the name is what its reader renames and two whitespace characters need the same edit for different names.
#[test]
fn the_refusal_of_a_folder_whose_name_holds_whitespace_names_that_character() {
    for (folder, spelled, named) in [
        ("src/c\td", "src/c\\u{9}d", "a tab (`\\u{9}`)"),
        (
            "src/c\u{a0}d",
            "src/c\\u{a0}d",
            "a no-break space (`\\u{a0}`)",
        ),
        (
            "src/e\u{202f}f",
            "src/e\\u{202f}f",
            "a narrow no-break space (`\\u{202f}`)",
        ),
    ] {
        let fixture: Fixture = Fixture::new();
        std::fs::create_dir_all(fixture.path.join(folder))
            .expect("create the folder whose name holds the whitespace");
        let error: String = parse_allow_list(
            format!("file-uses {folder} one.rs -> two.rs\n").as_str(),
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
        .expect_err("no line can name that folder");
        assert!(
            error.contains(format!("the folder `{spelled}`, which holds {named}").as_str()),
            "`{folder}` must be called `{named}`: {error}"
        );
    }
}

/// A dead line quotes its own text, spelled, and not merely a spelling that another part of the message holds,
///  because the two names of a `uses` line appear again in the sentence about the folder that is absent,
///  and a quote that falls back to the raw text would print the character as nothing.
#[test]
fn a_dead_line_quotes_its_spelled_text_and_no_other_part_of_the_message() {
    let (message, _) = one_dead_line("uses src/a\u{200b} -> src/b\n", &["src/a", "src/b"]);
    assert!(
        message
            .starts_with("The allow-list entry `uses src/a\\u{200b} -> src/b` in allow-list.txt "),
        "the quote is the entry's own text, spelled: {message}"
    );
}

/// A line written for an arrow-named folder names that folder even when a tab or a no-break space sets the
/// folder's own arrow off, or a doubled space stands where that arrow spells one,
/// because those are spellings of the whitespace a printed line uses,
/// and is not told about a second arrow the line does not hold.
#[test]
fn a_six_word_file_use_line_names_an_arrow_named_folder_whose_own_arrow_other_whitespace_sets_off()
{
    for (folder, spelled) in [
        ("src/x\t->\ty", "src/x\\u{9}->\\u{9}y"),
        ("src/x\u{a0}->\u{a0}y", "src/x\\u{a0}->\\u{a0}y"),
        ("src/x  ->  y", "src/x  ->  y"),
    ] {
        let fixture: Fixture = Fixture::new();
        std::fs::create_dir_all(fixture.path.join(folder)).expect("create the arrow-named folder");
        let error: String = parse_allow_list(
            format!("file-uses {folder} one.rs -> two.rs\n").as_str(),
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
        .expect_err("no five-word shape can carry a folder name that holds whitespace");
        assert!(
            error.contains(format!("the folder `{spelled}`").as_str()),
            "`{folder}` must be named `{spelled}`: {error}"
        );
        assert!(
            !error.contains("two `->` are set off by whitespace"),
            "`{folder}` must not be told about a second arrow the line does not hold: {error}"
        );
    }
}

/// A line written for a folder whose own name begins with whitespace names that folder for every spelling of
/// its gaps the reader can tell apart from the name's own whitespace,
/// because the name the line writes is read first and a doubled gap is one character beside it.
#[test]
fn a_six_word_file_use_line_names_a_folder_whose_name_begins_with_whitespace_for_every_gap_spelling()
 {
    let gaps: [&str; 5] = [" ", "  ", "\t", "\u{a0}", "\t\t"];
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join(" src/c d")).expect("create the folder");
    for after_word in gaps {
        for before_file in gaps {
            let line: String =
                format!("file-uses{after_word} src/c d{before_file}one.rs -> two.rs\n");
            let error: String = parse_allow_list(
                line.as_str(),
                Path::new("allow-list.txt"),
                fixture.path.as_path(),
            )
            .expect_err("a folder name holding whitespace makes six words");
            assert!(
                error.contains("the folder `\\u{20}src/c d`"),
                "`{line}` must name the folder that begins with a space: {error}"
            );
        }
    }
}

/// The same text can be the line for two folders: a doubled gap after the word is one separator and one
/// character of a name that may begin with whitespace.
///  The raw reading comes first, so the text names the folder whose own name begins with that character.
#[test]
fn a_line_whose_text_two_folders_share_names_the_folder_its_raw_reading_names() {
    let fixture: Fixture = Fixture::new();
    for folder in ["src/c d", " src/c d"] {
        std::fs::create_dir_all(fixture.path.join(folder)).expect("create the folder");
    }
    let plain: String = parse_allow_list(
        "file-uses src/c d one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("a folder name holding a space makes six words");
    assert!(
        plain.contains("the folder `src/c d`"),
        "the single-gap line names the plain folder: {plain}"
    );
    let shared: String = parse_allow_list(
        "file-uses  src/c d one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("a folder name holding a space makes six words");
    assert!(
        shared.contains("the folder `\\u{20}src/c d`"),
        "the doubled gap names the folder whose own name begins with the space: {shared}"
    );
}

/// A six-word line whose folder is absolute or holds a backslash gets the cause a five-word line gets,
///  because the path is what is wrong with the folder and no rename for a space can mend it.
#[test]
fn a_six_word_file_use_line_whose_folder_is_absolute_or_holds_a_backslash_gets_the_five_word_cause()
{
    let fixture: Fixture = Fixture::new();
    let plain: std::path::PathBuf = fixture.path.join("xy");
    std::fs::create_dir_all(&plain).expect("create the absolute folder");
    let spaced: std::path::PathBuf = fixture.path.join("x y");
    std::fs::create_dir_all(&spaced).expect("create the absolute folder that holds a space");
    for (line, cause) in [
        (
            format!("file-uses {} one.rs -> two.rs\n", plain.display()),
            "names a folder that is absolute",
        ),
        (
            format!("file-uses {} one.rs -> two.rs\n", spaced.display()),
            "names a folder that is absolute",
        ),
    ] {
        let error: String = parse_allow_list(
            line.as_str(),
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
        .expect_err("no absolute path names a folder of the package");
        assert!(
            error.contains(cause),
            "`{line}` must get `{cause}`: {error}"
        );
        assert!(
            !error.contains("Rename the folder so its name holds no space"),
            "`{line}` must not be told to rename a folder for a space: {error}"
        );
    }
    // What: the control, a folder whose first word holds a backslash, at both word counts.
    // Why: the six-word reading must not answer a backslash with a rename for a space.
    std::fs::create_dir_all(fixture.path.join("src\\a"))
        .expect("create the folder holding a backslash");
    std::fs::create_dir_all(fixture.path.join("src\\a b"))
        .expect("create the folder holding a backslash and a space");
    for line in [
        "file-uses src\\a one.rs -> two.rs\n",
        "file-uses src\\a b one.rs -> two.rs\n",
    ] {
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("a folder name holding a backslash is no path");
        assert!(
            error.contains("holds a backslash"),
            "`{line}` must get the backslash cause: {error}"
        );
        assert!(
            !error.contains("Rename the folder so its name holds no space"),
            "`{line}` must not be told to rename a folder for a space: {error}"
        );
    }
}

/// Every refusal the entry reader makes quotes its line, spelled,
///  and a line that holds a character a reader cannot see carries the note that names it and its place,
///  because a refusal stops the whole rule and a person fixes what the one message says.
#[test]
fn every_refusal_quotes_its_line_and_notes_an_unseen_character_with_its_place() {
    // What: a real spaced folder whose file name holds a zero-width space, and the two faults one message must say.
    // Why: the refusal for the folder's space used to leave the character unmentioned, so the next run refused for it.
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/e f"))
        .expect("create the folder holding a space");
    let spaced: String = parse_allow_list(
        "file-uses src/e f one.rs\u{200b} -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("a folder name holding a space makes six words");
    for part in [
        "`file-uses src/e f one.rs\\u{200b} -> two.rs`",
        "holds a space",
        "a zero-width space (`\\u{200b}`)",
        "at character 25 of the line",
    ] {
        assert!(spaced.contains(part), "{part} missing in {spaced}");
    }
    // What: the refusals of a path that is absolute or holds a backslash, on both shapes of line.
    // Why: a quote is what a person copies to find the line, and these three sentences had none.
    for (line, quote) in [
        ("uses /a\u{200e} -> src/b\n", "`uses /a\\u{200e} -> src/b`"),
        (
            "file-uses /nowhere\u{200b} one.rs -> two.rs\n",
            "`file-uses /nowhere\\u{200b} one.rs -> two.rs`",
        ),
        (
            "file-uses src\\a\u{200b} one.rs -> two.rs\n",
            "`file-uses src\\a\\u{200b} one.rs -> two.rs`",
        ),
    ] {
        let error: String = error_of(line);
        assert!(
            error.contains(quote),
            "`{line}` must quote its line: {error}"
        );
        assert!(
            error.contains("at character "),
            "`{line}` must say where the character stands: {error}"
        );
    }
}

/// A sentence that says what the line would read as after the edit says it only when the edited text does read so,
///  because a person who follows it and meets the same refusal learns nothing from it.
#[test]
fn a_line_that_would_still_not_read_as_a_use_line_is_not_told_that_it_would() {
    // What: a key line whose mark stands where the separator belongs but which holds no arrow.
    // Why: with the mark out or replaced by a space the line is still no `uses` line, so the sentence would be false.
    let (message, _) = one_dead_line("uses\u{200b}src/a src/b\n", &[]);
    assert!(
        !message.contains("reads as a `uses` line"),
        "the line would still not read as one: {message}"
    );
    assert!(
        message.contains("Delete the line so the list keeps shrinking"),
        "the line names a key no finding prints, so deleting it is the edit: {message}"
    );
    // What: the control, the same mark with the arrow the shape needs, whose edited text does read as one.
    // Why: a sentence that never appeared would pass the lines above and leave the reader no reading at all.
    let (control, _) = one_dead_line("uses\u{200b}src/a -> src/b\n", &[]);
    assert!(
        control.contains("reads as a `uses` line"),
        "the edited text reads as a `uses` line: {control}"
    );
}

/// A character a reader cannot see that has no plain name is named by its code point in the sentence,
///  because that is where a person looks for it and the quote alone does not say what it is.
#[test]
fn a_character_with_no_plain_name_is_named_by_its_code_point_in_the_sentence() {
    let (message, _) = one_dead_line("\u{70f}k\n", &[]);
    assert!(
        message.contains(
            "a character (`\\u{70f}`) at character 1 of the line, where a reader cannot see it"
        ),
        "the code point stands in the sentence, and the phrase is said once: {message}"
    );
}

/// The place a character a reader cannot see stands at is counted in characters from the line's first one,
///  whatever whitespace makes the indent: a tab, a no-break space and an ideographic space are one character each.
#[test]
fn the_place_counts_characters_under_an_indent_of_a_tab_a_no_break_space_and_an_ideographic_space()
{
    for indent in ["\t", "\u{a0}", "\u{3000}"] {
        let line: String = format!("{indent}uses src/a\u{200e}-> src/b\n");
        let error: String = error_of(line.as_str());
        assert!(
            error.contains("at character 12 of the line"),
            "an indent of {indent:?} is one character: {error}"
        );
    }
}

/// The first of the three readings names the folder, and every later one answers only a reading that names none,
///  so a tree holding two folders the same text could name has the line for the raw one named as that one.
#[test]
fn a_line_naming_two_folders_names_the_one_its_raw_reading_names() {
    let fixture: Fixture = Fixture::new();
    for folder in ["src/c\td", "src/c d"] {
        std::fs::create_dir_all(fixture.path.join(folder)).expect("create the folder");
    }
    let error: String = parse_allow_list(
        "file-uses src/c\td one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert!(
        error.contains("the folder `src/c\\u{9}d`"),
        "the raw reading names the tab folder: {error}"
    );
    assert!(
        !error.contains("the folder `src/c d`"),
        "the joined reading names no folder the raw one names: {error}"
    );
}

/// The help of a dead line names a tab and a no-break space its gap kept, spelled,
///  because half of the characters a hand-aligned line keeps print as nothing.
#[test]
fn the_help_of_a_dead_line_names_a_tab_and_a_no_break_space_its_gap_kept() {
    for (line, spelled) in [
        ("uses\t\tsrc/a -> src/b\n", "\\u{9}"),
        ("uses\u{a0}\u{a0}src/a -> src/b\n", "\\u{a0}"),
    ] {
        let (_, help) = one_dead_line(line, &["src/a", "src/b"]);
        assert!(
            help.contains(
                format!("It kept `{spelled}` after the word `uses`, before `src/a`").as_str()
            ),
            "`{line}` must name the character it kept: {help}"
        );
    }
}

/// The names one six-word `file-uses` line's reading asked the file system about, and the names it built,
///  with a gap of that many spaces between the word and the folder,
///  over the fixture package folder given.
fn names_a_gap_asks_about(
    fixture: &Fixture,
    gap: usize,
) -> (usize, usize) {
    let line: String = format!("file-uses{}src/c d one.rs -> two.rs\n", " ".repeat(gap));
    crate::layout_allow_entries::counted_work::reset();
    let error: String = parse_allow_list(
        line.as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no line can carry a folder name that holds whitespace");
    assert!(error.contains("the folder `src/c d`"), "{error}");
    return (
        crate::layout_allow_entries::counted_work::candidate_names(),
        crate::layout_allow_entries::counted_work::offered_names(),
    );
}

/// A six-word `file-uses` line whose gap is ten thousand whitespace characters is read with one pass over the line,
///  because the names that gap could have left beside the folder are offered one at a time
///  and a name whose first folder is longer than every name the package folder holds is offered to no one:
///  a reader that built one name per character of the gap asked the file system about each of them,
///  which cost the square of the gap in time and in memory
///  (0.25 s and 166 MiB at 16,000 characters, 3.3 s and 2,171 MiB at 64,000, measured on 2026-11-27).
///  The count of names asked about does not grow with the gap,
///  and every name the reading builds is one it asks about:
///  a reader that built an owned name for each character of the run before the bound test copied the square of the gap,
///  and the count of questions never saw it.
#[test]
fn a_six_word_file_use_line_with_a_long_gap_asks_about_few_names() {
    // What: the length of the gap the read measured, whose square the old reader paid.
    // Why: one line of four mebibytes is a list file the reader accepts, so the gap is bounded by nothing but that.
    const THE_GAP: usize = 10_000;
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/c d"))
        .expect("create the folder the line is written for");
    // What: the longest name the package folder holds, which is the most the first folder of such a name can be.
    // Why: a first folder longer than every name there is no folder of the package, so no answer needs that name asked about.
    let longest: usize = std::fs::read_dir(&fixture.path)
        .expect("the package folder is readable")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().len())
        .max()
        .expect("the package folder holds entries");
    let (asked, built): (usize, usize) = names_a_gap_asks_about(&fixture, THE_GAP);
    // What: a bound the package folder's own names set, with no room for a name or two more.
    // Why: `longest + 1` left two names of room, so a bound loosened by two bytes stayed green.
    assert!(
        asked <= longest,
        "a gap of {THE_GAP} characters asks about {asked} names, and the package folder's longest name is {longest} long"
    );
    assert!(
        built <= longest,
        "the reading built {built} names for a gap of {THE_GAP} characters, and the folder's longest name is {longest} long"
    );
    // What: every name the reading built is a name it asked the file system about.
    // Why: a name built and then dropped by the bound test is work the count of questions never saw.
    assert_eq!(
        built, asked,
        "the reading built {built} names and asked about {asked}"
    );
    // What: the same line with twice the gap, which must ask about the same names.
    // Why: a count that grew with the gap would pass a bound written against the folder's names alone.
    let (wider, wider_built): (usize, usize) = names_a_gap_asks_about(&fixture, 2 * THE_GAP);
    assert_eq!(
        wider,
        asked,
        "a gap of {} characters asks about {wider} names where a gap of {THE_GAP} asks about {asked}",
        2 * THE_GAP
    );
    assert_eq!(wider_built, built, "the wider gap builds the same names");
}

/// A refusal for a folder whose own name holds a character a reader cannot see leaves that character out of its advice,
///  because taking it out of the folder's name would name another folder:
///  the refusal already asks its reader to rename the folder, and that rename is what takes the character out.
///  The advice it gave instead sent its reader to a line the next run refused for its word count.
#[test]
fn a_refusal_leaves_the_folder_s_own_character_out_of_its_advice() {
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join(format!("src/c d{}x", '\u{200c}')))
        .expect("create the folder whose name holds a space and a zero-width non-joiner");
    let error: String = parse_allow_list(
        format!("file-uses src/c d{}x one.rs -> two.rs\n", '\u{200c}').as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no line can carry a folder name that holds whitespace");
    for part in [
        "the folder `src/c d\\u{200c}x`",
        "a zero-width non-joiner (`\\u{200c}`) at character 18 of the line",
        "stands in the name of the folder above",
        "renaming that folder is the edit that takes it out",
    ] {
        assert!(error.contains(part), "{part} missing in {error}");
    }
    assert!(
        !error.contains("take it out, or write a whitespace character"),
        "the character is the folder's own name, so taking it out names no folder: {error}"
    );
    // What: the control, a character in a file name of the same line, which is not the folder's name.
    // Why: an advice that never fired would pass the lines above and leave the character unnamed.
    let fixture_file: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture_file.path.join("src/e f"))
        .expect("create the folder holding a space");
    let noted: String = parse_allow_list(
        format!("file-uses src/e f one{}.rs -> two.rs\n", '\u{200b}').as_str(),
        Path::new("allow-list.txt"),
        fixture_file.path.as_path(),
    )
    .expect_err("no line can carry a folder name that holds whitespace");
    assert!(
        noted.contains("take it out, or write a whitespace character where it stood"),
        "a character in a file name is taken out: {noted}"
    );
}

/// A refusal names the take-out edit for the characters the line carries outside the folder's own name
///  and the rename for those inside it,
///  because a sentence that says to take each of them out sends its reader to a line the next run refuses for its word count,
///  and a `that character` that follows a count of two reads as the wrong one of them.
///  Each sentence is asserted whole, because every other control here reads a substring of it.
#[test]
fn a_refusal_names_the_edit_for_each_character_by_where_it_stands() {
    let fixture: Fixture = Fixture::new();
    // What: the folder the lines are written for, whose own name holds a space and a zero-width non-joiner.
    // Why: the reader finds it through the words the line holds, and its character is the one no take-out edit may change.
    std::fs::create_dir_all(fixture.path.join("src/c d\u{200c}x"))
        .expect("create the folder whose name holds a space and a zero-width non-joiner");
    // What: the folder's character second, with one the line carries before it outside the name.
    // Why: the first character stands outside the folder's name and the second inside it, so the two need two edits.
    let second: String = parse_allow_list(
        format!(
            "file-uses src/c{} d{}x one.rs -> two.rs\n",
            '\u{200b}', '\u{200c}'
        )
        .as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert_eq!(
        second,
        "Line 1 reads `file-uses src/c\\u{200b} d\\u{200c}x one.rs -> two.rs`. A use between two files is written `file-uses <folder> <file> -> <file>`, which is five words; this line is 6 words. The line also holds a zero-width space (`\\u{200b}`) at character 16 of the line, where a reader cannot see it, and 1 more character a reader cannot see: the reader does not treat it as whitespace, so take that character out, and write a whitespace character where it stood if it stood where one belongs. A character that stands in the name of a real folder is part of that name, and renaming that folder is the edit that takes it out."
    );
    // What: the folder's character in the middle of three, with one outside it on each side.
    // Why: a sentence that said to take each of them out would take the folder's character out with the two outside it.
    let middle: String = parse_allow_list(
        format!(
            "file-uses src/c{} d{}x on{}e.rs -> two.rs\n",
            '\u{200b}', '\u{200c}', '\u{200d}'
        )
        .as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert_eq!(
        middle,
        "Line 1 reads `file-uses src/c\\u{200b} d\\u{200c}x on\\u{200d}e.rs -> two.rs`. A use between two files is written `file-uses <folder> <file> -> <file>`, which is five words; this line is 6 words. The line also holds a zero-width space (`\\u{200b}`) at character 16 of the line, where a reader cannot see it, and 2 more characters a reader cannot see: the reader does not treat it as whitespace, so take that character out, and write a whitespace character where it stood if it stood where one belongs. A character that stands in the name of a real folder is part of that name, and renaming that folder is the edit that takes it out."
    );
    // What: the folder's character first, with a character in the file name after it.
    // Why: the count clause before the folder clause is what made `that character` read as the wrong one.
    let named_first: String = parse_allow_list(
        format!(
            "file-uses src/c d{}x on{}e.rs -> two.rs\n",
            '\u{200c}', '\u{200b}'
        )
        .as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert_eq!(
        named_first,
        "Line 1 reads `file-uses src/c d\\u{200c}x on\\u{200b}e.rs -> two.rs`. It names the folder `src/c d\\u{200c}x`, which holds a space: a `file-uses` line sets its five words off with one whitespace character each, so it cannot carry a name that holds one. Rename the folder so its name holds no space: it holds no module file, so no loop of files inside it needs such a line. The line also holds a zero-width non-joiner (`\\u{200c}`) at character 18 of the line, where a reader cannot see it, and 1 more character a reader cannot see, and one of them stands in the name of the folder above, so renaming that folder takes it out: take out the one outside the folder's name, and write a whitespace character where it stood if it stood where one belongs."
    );
    // What: the folder the file-name character's line names, which holds a space and no module file.
    // Why: a folder the line names is one the refusal can ask about, so the sentence says every character of it is outside the folder's name.
    std::fs::create_dir_all(fixture.path.join("src/c d"))
        .expect("create the folder whose name holds a space");
    // What: one character outside the folder's name and none inside it.
    // Why: the take-out sentence of a one-character refusal said `where it stands if it stood`, a change no record listed.
    let alone: String = parse_allow_list(
        format!(
            "file-uses src/c d on{}e.rs -> two.rs\n",
            '\u{200b}'
        )
        .as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert_eq!(
        alone,
        "Line 1 reads `file-uses src/c d on\\u{200b}e.rs -> two.rs`. It names the folder `src/c d`, which holds a space: a `file-uses` line sets its five words off with one whitespace character each, so it cannot carry a name that holds one. Rename the folder so its name holds no space: it holds no module file, so no loop of files inside it needs such a line. The line also holds a zero-width space (`\\u{200b}`) at character 21 of the line, where a reader cannot see it: the reader does not treat it as whitespace, so take it out, or write a whitespace character where it stood if it stood where one belongs."
    );
    // What: a line the reader finds no folder in, holding one character.
    // Why: the sentence cannot ask which character of a folder's name this is, so it names the placed character's edit
    // and says what a folder's own character needs instead.
    let no_folder: String = parse_allow_list(
        format!(
            "file-uses src/x y on{}e.rs -> two.rs\n",
            '\u{200b}'
        )
        .as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert_eq!(
        no_folder,
        "Line 1 reads `file-uses src/x y on\\u{200b}e.rs -> two.rs`. A use between two files is written `file-uses <folder> <file> -> <file>`, which is five words; this line is 6 words. The line also holds a zero-width space (`\\u{200b}`) at character 21 of the line, where a reader cannot see it: the reader does not treat it as whitespace, so take it out, or write a whitespace character where it stood if it stood where one belongs. A character that stands in the name of a real folder is part of that name, and renaming that folder is the edit that takes it out."
    );
    // What: the control, a refusal whose only character stands in the folder's own name.
    // Why: the rename is the one edit there, so a sentence that never named the take-out edit would pass the lines above
    // and leave a reader with no edit for that character.
    let in_the_name: String = parse_allow_list(
        format!("file-uses src/c d{}x one.rs -> two.rs\n", '\u{200c}').as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    for part in [
        "and that character stands in the name of the folder above: renaming that folder is the edit that takes it out.",
    ] {
        assert!(in_the_name.contains(part), "{part} missing in {in_the_name}");
    }
    assert!(
        !in_the_name.contains("take out the one outside the folder's name"),
        "the folder's own character is taken out by the rename alone: {in_the_name}"
    );
}

/// A refusal that names a folder holds no loop to untangle when that folder holds no module file,
///  because a loop is made of module files and a folder that holds none has nothing to untangle:
///  the sentence says what is so and leaves the rename as the one edit.
#[test]
fn a_refusal_about_a_folder_with_no_module_file_names_no_loop_to_untangle() {
    // What: a directory of the package and one beside it, each holding a space in its name and no module file.
    // Why: the two are the shapes the read found the sentence was untrue for.
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/docs notes"))
        .expect("create the folder of the package");
    std::fs::write(fixture.path.join("src/docs notes/README.md"), "# notes\n")
        .expect("write a file that is no module");
    // What: the directory beside the package folder, whose walk the line writes from the package folder.
    // Why: a six-word line names it, and the folder holds no module file either.
    let beside: std::path::PathBuf = fixture.path.join("../docs notes");
    std::fs::create_dir_all(&beside).expect("create the directory beside the package");
    for line in [
        "file-uses src/docs notes one.rs -> two.rs\n",
        "file-uses ../docs notes one.rs -> two.rs\n",
    ] {
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("no line can carry a folder name that holds whitespace");
        assert!(
            error.contains("Rename the folder so its name holds no space"),
            "`{line}` must be told the rename: {error}"
        );
        assert!(
            error.contains(
                "it holds no module file, so no loop of files inside it needs such a line"
            ),
            "`{line}` must be told what is so about a folder with no module file: {error}"
        );
        assert!(
            !error.contains("untangle the loop of files inside it"),
            "`{line}` has no loop to untangle: {error}"
        );
    }
    // What: the control, the same shape for a folder that does hold module files, with a loop of two of them.
    // Why: untangling the loop is a real edit there, so a sentence that never offered it would be as wrong as the other way.
    let looped: Fixture = Fixture::new();
    std::fs::create_dir_all(looped.path.join("src/c d"))
        .expect("create the folder whose name holds a space");
    for file in ["one.rs", "two.rs"] {
        std::fs::write(looped.path.join("src/c d").join(file), "/// Module.\n")
            .expect("write a module file of the folder");
    }
    let offered: String = parse_allow_list(
        "file-uses src/c d one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        looped.path.as_path(),
    )
    .expect_err("no line can carry a folder name that holds whitespace");
    assert!(
        offered.contains("or untangle the loop of files inside it"),
        "the folder holds module files, so its loop can be untangled: {offered}"
    );
}

/// What:
///  The one dead-entry message a list earns on the line that holds a character a reader cannot see,
///  with every other line of the list taken for one a finding uses.
/// Why:
///  Such a line is read as one finding's key and never as a use, so it is dead however the rule ran,
///  and a fixture that reported every line would answer about lines the sentence is not about.
fn one_unseen_line(
    lines: &str,
    folders: &[&str],
) -> (String, String) {
    let list: AllowList = parsed(lines);
    // What: every entry of the list that holds no character a reader cannot see, which the rule is taken to have used.
    // Why: only the line the sentence is about stays dead, so the one message is that line's.
    let used: BTreeSet<usize> = list
        .entries()
        .iter()
        .enumerate()
        .filter(|(_index, entry)| {
            return crate::layout_finding::first_unseen_character(entry.text.as_str()).is_none();
        })
        .map(|(index, _entry)| return index)
        .collect();
    let known: BTreeSet<PathBuf> = folders
        .iter()
        .map(|one| return Path::new("package").join(one))
        .collect();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &used,
        &known,
        &BTreeSet::<PathBuf>::new(),
        Path::new("package"),
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    return (
        findings[0].message.clone(),
        findings[0].help.clone().unwrap_or_default(),
    );
}

/// A sentence about what an edited line reads as asks the reader that reads the list itself,
///  so a line whose edited text is absolute, holds a backslash or repeats a line already there is never told it reads,
///  because the next run refuses it and its reader has learned nothing.
#[test]
fn a_sentence_about_the_edited_line_asks_the_reader_that_reads_the_list() {
    // What: the three edited texts the next run refuses for a cause of their own.
    // Why: the reader of the sentence read only the first word and a word count before, so each of them was told it would read.
    for line in [
        format!("{}uses /abs -> src/b\nuses src/a -> src/b\n", '\u{feff}'),
        format!("{}uses src\\a -> src/b\nuses src/a -> src/b\n", '\u{feff}'),
        format!("uses src/a -> src/b\n{}uses src/a -> src/b\n", '\u{feff}'),
    ] {
        let (message, _help): (String, String) = one_unseen_line(line.as_str(), &[]);
        assert!(
            !message.contains("the line reads as a `uses` line"),
            "the edited line is not one the reader takes: {message}"
        );
        assert!(
            message.contains("a byte order mark"),
            "the character is still named where it stands: {message}"
        );
        assert!(
            message.contains("Delete the line so the list keeps shrinking"),
            "a line no edit of which the reader takes is deleted: {message}"
        );
    }
    // What: the control, a mark before a line whose edited text the reader does take.
    // Why: a sentence that never appeared would pass the lines above and leave the reader nothing to do.
    let (reads, _help): (String, String) =
        one_unseen_line(format!("{}uses src/a -> src/b\n", '\u{200e}').as_str(), &[]);
    assert!(
        reads.contains("the line reads as a `uses` line"),
        "the edited text reads as a `uses` line: {reads}"
    );
}

/// A sentence about more than one character a reader cannot see names them all,
///  because the reading it describes takes each of them out or writes a space for each of them,
///  and a sentence that named the first promised a line only the edit of every one gives.
#[test]
fn a_sentence_about_two_characters_a_reader_cannot_see_names_every_one() {
    // What: a byte order mark and a zero-width space, each with the edit that leaves a `uses` line behind.
    // Why: a mark where a separator belongs is replaced by a space, a mark inside a name is taken out,
    // and neither edit gives a line while the other character stays.
    for (line, reading) in [
        (
            format!("{}uses{}src/a -> src/b\n", '\u{feff}', '\u{200b}'),
            "With a whitespace character where each of the two such characters stands, the line reads as a `uses` line",
        ),
        (
            format!("{}uses src/a{} -> src/b\n", '\u{feff}', '\u{200b}'),
            "With both such characters out, the line reads as a `uses` line",
        ),
    ] {
        let (error, _help): (String, String) = one_unseen_line(line.as_str(), &[]);
        for part in [
            "and 1 more character a reader cannot see",
            reading,
            "Take both such characters out of the line",
        ] {
            assert!(error.contains(part), "{part} missing in {error}");
        }
    }
    // What: the control, a line holding one such character, whose sentence speaks of that one.
    // Why: a sentence that always counted would pass the lines above and send a reader looking for a character that is not there.
    let (one, _help): (String, String) =
        one_unseen_line(format!("{}uses src/a -> src/b\n", '\u{feff}').as_str(), &[]);
    assert!(
        one.contains("With that character out, the line reads as a `uses` line"),
        "one character is named as one: {one}"
    );
    assert!(
        !one.contains("more character"),
        "one character is no count: {one}"
    );
}

/// A dead `file-uses` line whose file name holds a character a reader cannot see is told only the edit a file name takes,
///  because a file name holds no whitespace and the line is live again only while the use it records still exists:
///  the sentence offered the advice a key line needs, twice over in one doubled space.
#[test]
fn the_file_name_sentence_names_only_the_edit_a_file_name_takes() {
    // What: a line whose file name holds the zero-width space, in a folder the package holds,
    // with no module file of that folder called the name the line writes.
    // Why: that is the shape the file-name sentence is for, and its words are the sentence's whole subject.
    let (said, help): (String, String) = one_dead_line(
        format!("file-uses src/f one{}.rs -> two.rs\n", '\u{200b}').as_str(),
        &["src/f"],
    );
    let message: &str = said.as_str();
    assert!(
        message.contains(
            "Take that character out of the line, then write the file name the rule prints"
        ),
        "a file name takes the character out and nothing else: {message}"
    );
    assert!(
        message.contains("the entry is live again only if the use still exists"),
        "the sentence promises no live line without the use: {message}"
    );
    assert!(
        !message.contains("write a whitespace character where it stood"),
        "a file name can hold no whitespace: {message}"
    );
    assert!(
        !message.contains("  "),
        "the sentence holds no doubled space: {message}"
    );
    assert!(
        !help.contains("write a whitespace character"),
        "a file name takes no whitespace character: {help}"
    );
}

/// A blank cell of the Braille block is a character with a glyph of its own,
///  so the quote of a line that holds one leaves it as written,
///  and the README says which characters the reader spells and why this one is not among them:
///  the set is Unicode's `Cf`, `Cc` and `Default_Ignorable_Code_Point`, and none of the three names U+2800.
#[test]
fn a_braille_blank_cell_is_left_as_written_and_a_format_character_beside_it_is_spelled() {
    let (blank, _) = one_dead_line("file-uses src/f one.rs -> t\u{2800}wo.rs\n", &["src/f"]);
    assert!(
        blank.contains("`file-uses src/f one.rs -> t\u{2800}wo.rs`"),
        "the cell is a character a reader can see, so the quote keeps it: {blank}"
    );
    assert!(
        !blank.contains("\\u{2800}"),
        "the cell is no character of the set the reader spells: {blank}"
    );
    // What: the control, a zero-width space in the same place, which the reader does spell.
    // Why: a quote that spelled nothing would pass the line above and leave a name a reader cannot compare with the disk.
    let (spelled, _) = one_dead_line("file-uses src/f one.rs -> t\u{200b}wo.rs\n", &["src/f"]);
    assert!(
        spelled.contains("`file-uses src/f one.rs -> t\\u{200b}wo.rs`"),
        "a zero-width space is spelled: {spelled}"
    );
}

/// The second reading takes the leading run off the name one character at a time, from the fewest characters
/// removed to the most, so the name that keeps the most of the run is the first a tree can hold:
///  the other order offers the shortest name first and names a folder the line's reader meant less.
#[test]
fn the_second_reading_offers_the_name_that_keeps_the_most_of_the_run_first() {
    // What: one folder whose name begins with the gap's character and the same name without it.
    // Why: the text's two leading characters are the gap's own plus one, and only the order of the readings tells the two folders apart.
    let fixture: Fixture = Fixture::new();
    for folder in ["src/c d", " src/c d"] {
        std::fs::create_dir_all(fixture.path.join(folder)).expect("create the folder");
    }
    let error: String = parse_allow_list(
        "file-uses   src/c d one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("a folder name holding a space makes six words");
    assert!(
        error.contains("the folder `\\u{20}src/c d`"),
        "the name that keeps the gap's character is offered first: {error}"
    );
    assert!(
        !error.contains("the folder `src/c d`"),
        "the name without the character is the later reading: {error}"
    );
}

/// A written folder whose raw text ends in whitespace names no folder at all,
///  so the reader asks the name its second reading gives with that whole run taken off the end:
///  alpha.22 trimmed both ends there, and the line for the folder it named must still name that folder.
#[test]
fn a_written_folder_whose_raw_text_ends_in_whitespace_names_the_folder_its_second_reading_names() {
    // What: the two folders whose names one text can be the line for, and the text that ends in whitespace.
    // Why: the raw reading names no folder, and the second reading is what leaves the folder whose own name begins with the space.
    let fixture: Fixture = Fixture::new();
    for folder in ["src/c d", " src/c d"] {
        std::fs::create_dir_all(fixture.path.join(folder)).expect("create the folder");
    }
    let error: String = parse_allow_list(
        "file-uses  src/c d  one.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("a folder name holding a space makes six words");
    assert!(
        error.contains("the folder `\\u{20}src/c d`"),
        "the run at the end of the written text is taken off whole: {error}"
    );
}

/// A file name's place in the line is counted in characters, after any whitespace character that separates the words,
///  because a byte count is off by every two-byte letter before the name
///  and a separator taken for a space alone answers no place at all where a tab sets a word off.
#[test]
fn the_place_of_a_file_name_character_counts_characters_after_any_whitespace_separator() {
    // What: a tab before the file name and a two-byte letter before that, with the mark at character 23 of the line.
    // Why: counting bytes answers 24, and taking only a space for a separator answers 21 or no advice at all.
    let (message, _) = one_dead_line(
        format!(
            "file-uses src/caf{}\tone{}.rs -> two.rs\n",
            '\u{e9}', '\u{200b}'
        )
        .as_str(),
        &[format!("src/caf{}", '\u{e9}').as_str()],
    );
    assert!(
        message.contains("at character 23 of the line"),
        "the place counts characters after the tab: {message}"
    );
    assert!(
        message.contains("a zero-width space (`\\u{200b}`)"),
        "the mark in the file name is named at all: {message}"
    );
}

/// Every message the dead lines of one list earn, with the folders and the module files the package is taken to hold,
///  so a list of two such lines can be read as the two messages a run prints.
fn dead_line_messages(
    lines: &str,
    folders: &[&str],
    module_files: &[&str],
) -> Vec<(String, String)> {
    let list: AllowList = parsed(lines);
    // What: each folder and module file the package holds, spelled from the package folder the list is read from.
    // Why: a dead line is judged against the folders and files the walk names from that base.
    let known: BTreeSet<PathBuf> = folders
        .iter()
        .map(|one| return Path::new("package").join(one))
        .collect();
    let files: BTreeSet<PathBuf> = module_files
        .iter()
        .map(|one| return Path::new("package").join(one))
        .collect();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &known,
        &files,
        Path::new("package"),
    );
    return findings
        .iter()
        .map(|one| {
            return (one.message.clone(), one.help.clone().unwrap_or_default());
        })
        .collect();
}

/// The one message a list of one dead line earns, where the folder's own module files are something the message reads.
fn one_dead_line_beside_its_module_files(
    lines: &str,
    folders: &[&str],
    module_files: &[&str],
) -> (String, String) {
    let messages: Vec<(String, String)> = dead_line_messages(lines, folders, module_files);
    assert_eq!(messages.len(), 1, "{messages:#?}");
    return messages.into_iter().next().expect("one message");
}

/// A sentence is read whole: the words around a full stop are set off from it,
///  so `any more.With` is no sentence a person reads as one,
///  and a character with no plain name is called what it is once, not once in its name and again where it stands.
#[test]
fn a_dead_line_s_message_holds_no_two_words_run_together() {
    // What: the sentence a dead key line prints about the line it would read as, whole.
    // Why: every control beside this one checks a substring, and a missing space is a defect no substring sees.
    let (reads, _) = one_dead_line("\u{feff}uses src/a -> src/b\n", &[]);
    assert_eq!(
        reads,
        "The allow-list entry `\\u{feff}uses src/a -> src/b` in allow-list.txt holds a byte order mark or zero-width no-break space (`\\u{feff}`) at character 1 of the line, where a reader cannot see it: the reader took it as part of the word beside it and the line was read as one finding's key; no finding of the rule prints that key, so the entry tolerates nothing any more. With that character out, the line reads as a `uses` line. Take that character out of the line, and write a whitespace character where it stood if it stood where one belongs."
    );
    assert!(
        !reads.contains("any more.With"),
        "a sentence is set off from the one before it: {reads}"
    );
    // What: a character this crate has no plain name for, which its phrase calls by its code point alone.
    // Why: the phrase said `a character a reader cannot see ... where a reader cannot see it`, and the sentence says it twice.
    let (unnamed, _) = one_dead_line("\u{70f}uses src/a -> src/b\n", &[]);
    assert!(
        unnamed.contains(
            "holds a character (`\\u{70f}`) at character 1 of the line, where a reader cannot see it"
        ),
        "the character is named once, by its code point: {unnamed}"
    );
    assert_eq!(
        unnamed.matches("a reader cannot see").count(),
        1,
        "the phrase is said once: {unnamed}"
    );
}

/// A message about a line holding three characters a reader cannot see counts all of them and names the edit that takes every one,
///  because a sentence built only for one and for two sends its reader after the first and leaves two behind.
#[test]
fn a_sentence_about_three_characters_a_reader_cannot_see_names_every_one() {
    let (message, help) = one_unseen_line(
        format!(
            "{}uses{}src/a -> src/b{}\n",
            '\u{feff}', '\u{200b}', '\u{200e}'
        )
        .as_str(),
        &[],
    );
    for part in [
        "and 2 more characters a reader cannot see",
        "Take every such character out of the line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert!(
        !message.contains("and 3 more"),
        "two stand beside the first: {message}"
    );
    assert!(help.contains("Take every such character out"), "{help}");
}

/// The count a file name's edit speaks of covers the file names' characters only,
///  because the edit takes those out and leaves every character of a real folder's own name where it is:
///  a line whose folder holds one and whose file name holds one was told to take both out, and the next run named a folder that is not there.
#[test]
fn the_count_of_a_file_name_s_edit_covers_the_file_names_characters_only() {
    // What: a real folder whose own name holds a zero-width space, and a file name holding one too.
    // Why: the folder is no name the edit touches, so the count is the file name's one.
    let (message, help) = one_dead_line(
        "file-uses src/q\u{200b}r on\u{200b}e.rs -> two.rs\n",
        &["src/q\u{200b}r"],
    );
    for part in [
        "Take that character out of the line, then write the file name the rule prints",
        "at character 21 of the line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert!(
        !message.contains("more character"),
        "one character of a file name is one: {message}"
    );
    assert!(
        !message.contains("Take both such characters"),
        "taking the folder's own character out names no folder: {message}"
    );
    assert!(
        !help.contains("both such characters"),
        "the help counts the file names' characters too: {help}"
    );
    // What: the control, two characters of one line that both stand in file names.
    // Why: a sentence that never counted file names would pass the lines above and never say two.
    let (both, both_help) = one_dead_line(
        "file-uses src/f o\u{200b}ne.rs -> th\u{200c}ree.rs\n",
        &["src/f"],
    );
    for part in [
        ", and 1 more character a reader cannot see",
        "Take both such characters out of the line, then write the file names the rule prints",
    ] {
        assert!(both.contains(part), "{part} missing in {both}");
    }
    assert!(
        both_help.contains("write the file names the rule prints"),
        "the help names the same edit: {both_help}"
    );
    // What: a real module whose own name holds a character, beside a file name the advice changes.
    // Why: the sentence goes on to name the file name the rule prints, which is that module's own name,
    // so a count that included its character asked for a change the same sentence undoes.
    let (module_first, module_help) = one_dead_line_beside_its_module_files(
        "file-uses src/f o\u{200b}ne.rs -> tw\u{200d}o.rs\n",
        &["src/f"],
        &["src/f/o\u{200b}ne.rs"],
    );
    assert_eq!(
        module_first,
        "The allow-list entry `file-uses src/f o\\u{200b}ne.rs -> tw\\u{200d}o.rs` in allow-list.txt holds a zero-width joiner (`\\u{200d}`) at character 30 of the line, where a reader cannot see it: a file name is read exactly as the line writes it, and no module of this folder is called the name it holds, so the entry tolerates nothing any more. Take that character out of the line, then write the file name the rule prints; the entry is live again only if the use still exists."
    );
    assert_eq!(
        module_help,
        "Take that character out of the line, then write the file name the rule prints: a character a reader cannot see makes a file name of its own."
    );
    // What: the real module the second name, where the first name is the one the advice changes.
    // Why: which side of the arrow the module stands on must not change the count.
    let (module_second, _) = one_dead_line_beside_its_module_files(
        "file-uses src/f tw\u{200d}o.rs -> o\u{200b}ne.rs\n",
        &["src/f"],
        &["src/f/o\u{200b}ne.rs"],
    );
    assert_eq!(
        module_second,
        "The allow-list entry `file-uses src/f tw\\u{200d}o.rs -> o\\u{200b}ne.rs` in allow-list.txt holds a zero-width joiner (`\\u{200d}`) at character 19 of the line, where a reader cannot see it: a file name is read exactly as the line writes it, and no module of this folder is called the name it holds, so the entry tolerates nothing any more. Take that character out of the line, then write the file name the rule prints; the entry is live again only if the use still exists."
    );
    // What: two characters inside one file name the advice changes.
    // Why: the noun follows the number of names, not of characters, so one name holding two still speaks of a file name.
    let (one_name, one_name_help) = one_dead_line(
        "file-uses src/g o\u{200b}n\u{200d}e.rs -> two.rs\n",
        &["src/g"],
    );
    assert!(
        one_name.contains("Take both such characters out of the line, then write the file name the rule prints"),
        "two characters of one name are two characters of one file name: {one_name}"
    );
    assert!(
        one_name_help.contains("write the file name the rule prints"),
        "the help names one file name: {one_name_help}"
    );
}

/// A refusal leaves the advice to take a character out of the line out where that character stands in the name of the folder it names,
///  however the line spells the folder's own whitespace:
///  where the gap is a tab, a no-break space or a doubled space the folder's text is not in the line as written,
///  and the note then sent its reader to an edit the next run refused for its word count.
#[test]
fn a_refusal_leaves_the_folder_s_own_character_out_of_its_advice_under_every_spelling_of_its_space()
{
    let fixture: Fixture = Fixture::new();
    // What: the folder the line is written for, whose own name holds a space and a zero-width non-joiner.
    // Why: the reader finds it through the words joined with one space, which no spelling of the gap writes as it is.
    std::fs::create_dir_all(fixture.path.join("src/c d\u{200c}x"))
        .expect("create the folder whose name holds a space and a zero-width non-joiner");
    for (spelling, line) in [
        ("a tab", "file-uses src/c\td\u{200c}x one.rs -> two.rs\n"),
        (
            "two spaces",
            "file-uses src/c  d\u{200c}x one.rs -> two.rs\n",
        ),
        (
            "a no-break space",
            "file-uses src/c\u{a0}d\u{200c}x one.rs -> two.rs\n",
        ),
        ("a space", "file-uses src/c d\u{200c}x one.rs -> two.rs\n"),
    ] {
        let error: String =
            parse_allow_list(line, Path::new("allow-list.txt"), fixture.path.as_path())
                .expect_err("no five-word shape carries a folder name that holds whitespace");
        assert!(
            error.contains("the folder `src/c d\\u{200c}x`"),
            "`{spelling}` must name the folder: {error}"
        );
        for part in [
            "stands in the name of the folder above",
            "renaming that folder is the edit that takes it out",
        ] {
            assert!(
                error.contains(part),
                "`{spelling}`: {part} missing in {error}"
            );
        }
        assert!(
            !error.contains("take it out, or write a whitespace character"),
            "`{spelling}` stands in the folder's own name, so taking it out names no folder: {error}"
        );
    }
}

/// A sentence that names one character a reader cannot see says how many the line holds,
///  in each of the sentences a refusal or a dead line can print,
///  because a person told of one character learns of the second on the next run.
#[test]
fn every_sentence_about_an_unseen_character_says_how_many_the_line_holds() {
    // What: two characters a reader cannot see, one in the folder the refusal names and one in a file name after it.
    // Why: the refusal named the first and said nothing of the second.
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/c d\u{200c}x"))
        .expect("create the folder whose name holds a space and a zero-width non-joiner");
    let refused: String = parse_allow_list(
        "file-uses src/c d\u{200c}x on\u{200b}e.rs -> two.rs\n",
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect_err("no five-word shape carries a folder name that holds whitespace");
    assert!(
        refused.contains(", and 1 more character a reader cannot see"),
        "the refusal counts the second: {refused}"
    );
    // What: the Delete note of a line whose two file names are real modules of the folder.
    // Why: deleting the line is the edit there, and the note named the first character alone.
    let (deleted, _) = one_dead_line_beside_its_module_files(
        "file-uses src/f o\u{200b}ne.rs -> th\u{200c}ree.rs\n",
        &["src/f"],
        &["src/f/o\u{200b}ne.rs", "src/f/th\u{200c}ree.rs"],
    );
    for part in [
        "Delete the line so the list keeps shrinking",
        ", and 1 more character a reader cannot see",
    ] {
        assert!(deleted.contains(part), "{part} missing in {deleted}");
    }
    // What: the help of a dead line whose folder is absent and whose name holds two such characters.
    // Why: the help named the first and told its reader to take that one out, leaving the second for the next run.
    let (_, help) = one_dead_line("uses  src/a\u{200b}b\u{200c}c -> src/c\n", &["src/c"]);
    for part in [
        ", and 1 more character a reader cannot see",
        "take each of them out",
    ] {
        assert!(help.contains(part), "{part} missing in {help}");
    }
}

/// Two dead key lines whose own edits give the same line are not each told that the line reads,
///  because the list refuses the second of the two edits as a repeat,
///  so the edited texts are compared with each other as well as with the lines as written.
#[test]
fn two_dead_key_lines_whose_edits_give_one_line_are_not_each_told_it_reads() {
    // What: a byte order mark and a zero-width space, each before the same `uses` line.
    // Why: taken out, each edit gives the same line, and only one of the two can stay.
    let messages: Vec<(String, String)> = dead_line_messages(
        format!(
            "{}uses src/a -> src/b\n{}uses src/a -> src/b\n",
            '\u{feff}', '\u{200b}'
        )
        .as_str(),
        &[],
        &[],
    );
    assert_eq!(messages.len(), 2, "{messages:#?}");
    // What: the two whole messages, each naming the other line and the edit the two share.
    // Why: deleting every one of them leaves none, and the next run says nothing of the one that could have stayed.
    assert_eq!(
        messages[0].0,
        "The allow-list entry `\\u{feff}uses src/a -> src/b` in allow-list.txt tolerates nothing any more. Delete the line so the list keeps shrinking. The line also holds a byte order mark or zero-width no-break space (`\\u{feff}`) at character 1 of the line, where a reader cannot see it. The line's own edit, taking that character out, gives the same line as line 2: making that edit on one of them and deleting the rest keeps one of them."
    );
    assert_eq!(
        messages[1].0,
        "The allow-list entry `\\u{200b}uses src/a -> src/b` in allow-list.txt tolerates nothing any more. Delete the line so the list keeps shrinking. The line also holds a zero-width space (`\\u{200b}`) at character 1 of the line, where a reader cannot see it. The line's own edit, taking that character out, gives the same line as line 1: making that edit on one of them and deleting the rest keeps one of them."
    );
    for (message, _help) in &messages {
        assert!(
            !message.contains("reads as a `uses` line"),
            "the list refuses the second of the two edits: {message}"
        );
        assert!(
            message.contains("Delete the line so the list keeps shrinking"),
            "the line whose edit repeats another line's edit is deleted: {message}"
        );
    }
    // What: three lines whose edits give one line, each naming the two others.
    // Why: a sentence that named one line of a larger group sent its reader to delete a line a third edit also gives.
    let three: Vec<(String, String)> = dead_line_messages(
        format!(
            "{}uses src/a -> src/b\n{}uses src/a -> src/b\n{}uses src/a -> src/b\n",
            '\u{feff}',
            '\u{200b}',
            '\u{2060}'
        )
        .as_str(),
        &[],
        &[],
    );
    assert_eq!(three.len(), 3, "{three:#?}");
    assert!(
        three[0]
            .0
            .ends_with("gives the same line as lines 2 and 3: making that edit on one of them and deleting the rest keeps one of them."),
        "the first line names the two others: {}",
        three[0].0
    );
    // What: two lines whose edits give one line, one by taking the character out and one by writing a space in its place.
    // Why: each sentence names the edit its own line makes, so a reader knows which of the two edits to keep.
    let two_edits: Vec<(String, String)> = dead_line_messages(
        format!(
            "{}uses src/a -> src/b\nuses{}src/a -> src/b\n",
            '\u{feff}', '\u{200b}'
        )
        .as_str(),
        &[],
        &[],
    );
    assert_eq!(two_edits.len(), 2, "{two_edits:#?}");
    assert!(
        two_edits[0]
            .0
            .ends_with("The line's own edit, taking that character out, gives the same line as line 2: making that edit on one of them and deleting the rest keeps one of them."),
        "the first line's own edit is the removal: {}",
        two_edits[0].0
    );
    assert!(
        two_edits[1]
            .0
            .ends_with("The line's own edit, writing a whitespace character where that character stands, gives the same line as line 1: making that edit on one of them and deleting the rest keeps one of them."),
        "the second line's own edit is the space: {}",
        two_edits[1].0
    );
    // What: the control, one such line with no other line's edit beside it.
    // Why: a sentence that never appeared would pass the lines above and leave the reader no reading to make.
    let (alone, _) = one_dead_line("\u{feff}uses src/a -> src/b\n", &[]);
    assert!(
        alone.contains("reads as a `uses` line"),
        "the edit of one line with no other beside it reads: {alone}"
    );
    assert!(
        !alone.contains("gives the same line as"),
        "one line's edit gives no other line's: {alone}"
    );
}

/// A list of dead key lines whose edits read as `uses` lines hands out the whole list only to the passes that report the lines,
///  because the index of what the list holds answers each line's question in one lookup:
///  a repeat check that walked the entries built two texts per entry per line, which cost the square of the list's length
///  (3,500 such lines took 41 times the previous release's time, measured on 2026-11-27).
#[test]
fn a_list_of_dead_key_lines_that_read_as_uses_scans_no_other_line() {
    // What: enough lines that a scan of the list would read tens of thousands of entries.
    // Why: a count of no entry read says nothing about a list whose lines were never read at all.
    const LINES: usize = 200;
    // What: the times the reporting asks the list for its entries: its length once and its lines twice.
    // Why: the bound below is those passes and no more, and a scan put back for each line adds the whole list per line.
    const PASSES: usize = 3;
    let mut text: String = String::new();
    for index in 0..LINES {
        text.push_str(format!("{}uses src/a -> src/b{index}\n", '\u{feff}').as_str());
    }
    let list: AllowList = parsed(text.as_str());
    counted_work::reset();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &BTreeSet::<PathBuf>::new(),
        &BTreeSet::<PathBuf>::new(),
        Path::new("package"),
    );
    // What: every line is dead and earns its own message, so the list was read and not skipped.
    // Why: the count below is about the questions these lines ask, and a run that asked none would pass it.
    assert_eq!(findings.len(), LINES);
    assert!(
        findings[0].message.contains("reads as a `uses` line"),
        "{}",
        findings[0].message
    );
    // What: the entries the reporting handed out to its callers, which a scan of the entries would grow for each line.
    // Why: the index answers each question, so a scan that came back would hand out the whole list once more for every line.
    let walked: usize = counted_work::entries_walked();
    assert!(
        walked <= PASSES * LINES,
        "{LINES} dead key lines read {walked} entries of the list, where the reporting hands out at most {PASSES} lists"
    );
}

/// Dead key lines whose edited text is a six-word `file-uses` line list each folder they name once for the whole list,
///  because the longest name the package folder holds and whether a named folder holds a module file are answers about the folder
///  and not questions about the line:
///  a listing per line cost 175 times the previous release's time on a folder of 5,000 entries (measured on 2026-11-27),
///  and the same shape over a folder of 20,000 files cost 5.4 s for 400 lines (measured on 2026-11-28).
#[test]
fn a_list_of_six_word_edits_lists_the_package_folder_once() {
    // What: enough lines that a listing per line is a different number from one.
    // Why: the line's edited text is a six-word shape, which is the only text whose reading asks the folder how long its names are.
    const LINES: usize = 50;
    // What: files of the folder the lines name, so a listing of it is work and not an empty answer.
    // Why: the control counted the longest-name listing only, on a folder created empty,
    // so a listing per line of the folder for a module file stayed green.
    const FILES: usize = 8;
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir_all(fixture.path.join("src/c d"))
        .expect("create the folder the six-word text names");
    for index in 0..FILES {
        std::fs::write(
            fixture.path.join("src/c d").join(format!("n{index}.txt")),
            "no module file here\n",
        )
        .expect("write a file of the folder that is no module");
    }
    let mut text: String = String::new();
    for index in 0..LINES {
        text.push_str(
            format!(
                "{}file-uses  src/c d  f{index}.rs -> g{index}.rs\n",
                '\u{feff}'
            )
            .as_str(),
        );
    }
    let list: AllowList = parse_allow_list(
        text.as_str(),
        Path::new("allow-list.txt"),
        fixture.path.as_path(),
    )
    .expect("a key line is no line the reader refuses");
    crate::layout_allow_entries::counted_work::reset();
    let findings: Vec<LayoutFinding> = dead_entry_findings(
        &list,
        &BTreeSet::<usize>::new(),
        &BTreeSet::<PathBuf>::new(),
        &BTreeSet::<PathBuf>::new(),
        fixture.path.as_path(),
    );
    assert_eq!(findings.len(), LINES);
    assert!(
        findings[0].message.contains("tolerates nothing any more"),
        "{}",
        findings[0].message
    );
    // What: the times the package folder was listed for its longest name, which every line asks for.
    // Why: the answer belongs to the folder, so the whole list asks it once.
    let listings: usize = crate::layout_allow_entries::counted_work::folder_listings();
    assert_eq!(
        listings, 1,
        "{LINES} six-word edits listed the package folder {listings} times"
    );
    // What: the times the folder the lines name was listed for a module file, which every line asks for.
    // Why: a folder holding files makes that listing cost something, and a listing per line grows with the list.
    let module_file_listings: usize =
        crate::layout_allow_entries::counted_work::module_file_listings();
    assert_eq!(
        module_file_listings, 1,
        "{LINES} six-word edits listed the folder they name {module_file_listings} times for a module file"
    );
    // What: the control, a list of dead lines whose edits read as no six-word line.
    // Why: a count that was one for every list would pass the line above without showing what asked for the listing.
    crate::layout_allow_entries::counted_work::reset();
    let plain: AllowList = parsed("k\u{200b}m\n");
    let _unused: Vec<LayoutFinding> = dead_entry_findings(
        &plain,
        &BTreeSet::<usize>::new(),
        &BTreeSet::<PathBuf>::new(),
        &BTreeSet::<PathBuf>::new(),
        Path::new("package"),
    );
    assert_eq!(
        crate::layout_allow_entries::counted_work::folder_listings(),
        0,
        "a list whose lines ask nothing about the folder lists it at all"
    );
}
