//! Controls for the marker lines, the marker size and the open-operation line.

use super::*;

/// A marker of `size` characters `kind`, followed by `rest`.
fn marker(kind: char, size: usize, rest: &str) -> String {
    return format!("{}{rest}\n", kind.to_string().repeat(size));
}

/// An opening or closing line of Git's size is a marker, whatever follows the white space, and nothing else is.
#[test]
fn opening_and_closing_lines_of_the_given_size_are_markers() {
    for size in [DEFAULT_MARKER_SIZE, DEFAULT_MARKER_SIZE + 2] {
        let opening: String = format!("text\n{}", marker('<', size, " HEAD"));
        let closing: String = format!("text\n{}", marker('>', size, " other"));
        let bare: String = format!("text\n{}", marker('<', size, ""));
        assert!(has_marker_lines(opening.as_bytes(), size), "{size}: opening");
        assert!(has_marker_lines(closing.as_bytes(), size), "{size}: closing");
        assert!(has_marker_lines(bare.as_bytes(), size), "{size}: a bare marker line");
        let shorter: String = marker('<', size - 1, " HEAD");
        let longer: String = marker('<', size + 1, "");
        assert!(!has_marker_lines(shorter.as_bytes(), size), "{size}: a shorter run is no marker");
        assert!(!has_marker_lines(longer.as_bytes(), size), "{size}: a longer run is no marker of this size");
    }
}

/// A run of `=` alone underlines a heading and is no sign of a conflict, a marker needs white space after it, and a last line
///  without a line feed is too short to be one, as in Git.
#[test]
fn equals_signs_alone_and_unterminated_lines_are_not_markers() {
    let heading: String = format!("Title\n{}\n", "=".repeat(DEFAULT_MARKER_SIZE));
    assert!(!has_marker_lines(heading.as_bytes(), DEFAULT_MARKER_SIZE));
    let glued: String = format!("{}x\n", "<".repeat(DEFAULT_MARKER_SIZE));
    assert!(!has_marker_lines(glued.as_bytes(), DEFAULT_MARKER_SIZE), "no white space after the run");
    let unterminated: String = "<".repeat(DEFAULT_MARKER_SIZE);
    assert!(!has_marker_lines(unterminated.as_bytes(), DEFAULT_MARKER_SIZE), "no line feed after the run");
    assert!(!has_marker_lines(b"", DEFAULT_MARKER_SIZE));
}

/// The attribute is a number, and Git's default stands for anything else, zero included.
#[test]
fn the_attribute_value_is_a_number_or_the_default() {
    let nine: usize = DEFAULT_MARKER_SIZE + 2;
    assert_eq!(marker_size_of(nine.to_string().as_str()), nine);
    for value in ["unspecified", "set", "unset", "0", "-3", "", "seven"] {
        assert_eq!(marker_size_of(value), DEFAULT_MARKER_SIZE, "{value:?}");
    }
}

/// The open-operation line names the commands that finish the operation.
#[test]
fn the_open_line_names_how_to_finish() {
    let merge: String = open_operation_line(true);
    assert!(merge.contains("merge is still in progress") && merge.contains("git merge --continue") && merge.contains("git commit"), "{merge}");
    let pick: String = open_operation_line(false);
    assert!(pick.contains("cherry-pick is still in progress") && pick.contains("git cherry-pick --continue"), "{pick}");
}
