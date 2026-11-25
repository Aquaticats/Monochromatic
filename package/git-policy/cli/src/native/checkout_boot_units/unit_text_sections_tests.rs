//! What:
//!  The reading every unit text's control shares:
//!  one INI section's directive lines,
//!  with comments and blank lines dropped,
//!  and the shape every file of this folder must have.
//! Why:
//!  an assertion that searched the whole file would find `[Unit]`'s
//!  `After=` inside `[Service]`'s text or a line a comment carries,
//!  so each control names the section it read.
//!  The shape check is one place because a file missing its last newline,
//!  or one still carrying a `<placeholder>`,
//!  is wrong whichever unit it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesOf(text, '[Unit]')).toContain('Before=umount.target');
//! ```

/// The directive lines of one section,
/// trimmed,
/// with comments and blank lines dropped.
///
/// Why:
///  the caller compares whole `key=value` lines,
///  so whitespace around them must not decide the comparison.
pub(crate) fn lines_of(text: &str, section: &str) -> Vec<String> {
    let mut inside: bool = false;
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            inside = trimmed == section;
            continue;
        }
        if !inside || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        lines.push(String::from(trimmed));
    }
    return lines;
}

/// Whether one section carries a line that begins with the given words.
///
/// Why:
///  a line such as `Last=` names an argument a control does not want to
///  repeat, so the assertion reads the beginning and the rest is the text's.
pub(crate) fn carries(text: &str, section: &str, beginning: &str) -> bool {
    return lines_of(text, section).iter().any(|line| return line.starts_with(beginning));
}

/// Whether one section carries the whole line.
pub(crate) fn holds(text: &str, section: &str, wanted: &str) -> bool {
    return lines_of(text, section).iter().any(|line| return line == wanted);
}

/// Assert one file of this folder has the shape every one of them must have:
/// one final newline,
/// no second one,
/// no carriage return,
/// no tab,
/// and no `<placeholder>` left in it.
///
/// Why:
///  a leftover placeholder is a unit line naming a file nothing installed,
///  and a tab or carriage return is a byte systemd and a reader disagree about.
pub(crate) fn assert_file_shape(text: &str) {
    assert!(text.ends_with('\n'), "the file ends with one newline");
    assert!(!text.ends_with("\n\n"), "the file carries no blank tail");
    assert!(!text.contains('\r'), "the file carries no carriage return");
    assert!(!text.contains('\t'), "the file carries no tab");
    assert!(!text.contains('<'), "the file carries no placeholder: {text}");
    assert!(!text.contains('>'), "the file carries no placeholder: {text}");
}

#[test]
fn the_reader_names_a_section_and_drops_comments_and_blanks() {
    let text: &str = "# a comment\n[Unit]\nDescription=x\n\n# inside\nWants=y\n[Service]\nType=oneshot\n";
    assert_eq!(lines_of(text, "[Unit]"), vec![String::from("Description=x"), String::from("Wants=y")]);
    assert_eq!(lines_of(text, "[Service]"), vec![String::from("Type=oneshot")]);
    assert!(carries(text, "[Unit]", "Wants="));
    assert!(!holds(text, "[Unit]", "Wants=y "), "a line is compared whole");
    assert!(holds(text, "[Unit]", "Wants=y"));
}
