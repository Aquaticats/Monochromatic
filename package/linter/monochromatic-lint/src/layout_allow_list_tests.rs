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
    let findings: Vec<LayoutFinding> =
        dead_entry_findings(&list, &BTreeSet::<usize>::new(), &folders, base);
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
    let findings: Vec<LayoutFinding> =
        dead_entry_findings(&list, &BTreeSet::<usize>::new(), &folders, base);
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
    let findings: Vec<LayoutFinding> =
        dead_entry_findings(&list, &BTreeSet::<usize>::new(), &folders, base);
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
    return super::AllowList {
        source,
        entries,
        uses: std::collections::BTreeSet::new(),
        keys,
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
        (&["src/c d"][..], "file-uses src/c d one.rs  -> two.rs\n", "src/c d"),
        (&["src/c d"][..], "file-uses  src/c d one.rs -> two.rs\n", "src/c d"),
        (&["src/c d"][..], "file-uses src/c d  one.rs -> two.rs\n", "src/c d"),
        (&["src/c\td"][..], "file-uses\t src/c\td one.rs -> two.rs\n", "src/c\\u{9}d"),
        (&["src/c\td"][..], "file-uses  src/c\td one.rs -> two.rs\n", "src/c\\u{9}d"),
        (&["src/x -> y"][..], "file-uses src/x -> y one.rs -> two.rs\n", "src/x -> y"),
        (&["src/p ->"][..], "file-uses src/p -> one.rs -> two.rs\n", "src/p ->"),
    ] {
        let fixture: Fixture = Fixture::new();
        for folder in folders {
            std::fs::create_dir_all(fixture.path.join(folder))
                .expect("create the folder the line is written for");
        }
        let error: String = parse_allow_list(
            line,
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
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
        let error: String = parse_allow_list(
            line,
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
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
        ("file-uses src/ab one.rs -> two.rs extra\n", "this line is 6 words"),
    ] {
        let error: String = parse_allow_list(
            line,
            Path::new("allow-list.txt"),
            fixture.path.as_path(),
        )
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

/// A character a reader cannot see is placed by its column in the line,
///  counted in characters from the line's first one,
///  because that is where a person finds it in an editor:
///  the entry's own index is off by the indent, and a byte count is off by every two-byte letter before it.
#[test]
fn the_place_a_character_a_reader_cannot_see_stands_at_is_a_column_of_the_line() {
    // What: an indented line, whose mark stands at column 4 but at index 1 of the trimmed entry.
    // Why: the message said "character 1 of the entry it read", which a person counting in an editor cannot find.
    let (indented, _) = one_dead_line("   \u{feff}uses src/a -> src/b\n", &[]);
    assert!(
        indented.contains("at character 4 of the line"),
        "the place is a column of the line: {indented}"
    );
    // What: a refusal whose line holds a two-byte letter before the mark.
    // Why: counting bytes would answer 13 here where the editor shows 12.
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
        ("src/c\u{a0}d", "src/c\\u{a0}d", "a no-break space (`\\u{a0}`)"),
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
        message.starts_with(
            "The allow-list entry `uses src/a\\u{200b} -> src/b` in allow-list.txt "
        ),
        "the quote is the entry's own text, spelled: {message}"
    );
}
