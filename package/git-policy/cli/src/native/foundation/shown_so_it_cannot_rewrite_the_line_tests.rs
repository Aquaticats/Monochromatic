//! Controls for the text this program shows of a name another program chose,
//! once for each wrapping a sentence puts around it.
//! Why:
//!  A process sets its own name to any bytes it likes,
//!  and the line that names it is the one place a person finds it,
//!  so each shape a terminal or a reader could take for something else must come back spelled out,
//!  and each wrapping must lose its own closing character and no other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(shownSoItCannotRewriteTheLine('\u001b[2K', TheWrappingAroundTheName.Parentheses)).toBe('\\u{1b}[2K');
//! ```

use super::*;

/// What:
///  The shapes the security rules of the repository list for text that crosses into a line,
///  and the shapes a name could carry into a sentence that wraps it:
///  an escape that starts a terminal sequence,
///  a line break that starts a line this program never wrote,
///  a tab,
///  a backslash,
///  both parentheses,
///  a backtick,
///  both quotation marks,
///  a character that is not ASCII,
///  a right-to-left override and an isolate,
///  and a Unicode line separator.
/// Why:
///  Each is a way a name could change what the line says or where the cursor goes;
///  one list keeps every control below on the same shapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHAPES = ['\u001b[2K', '\n', '\r', '\t', ')', '(', '`', '\\', '"', "'", 'é', '\u202e', '\u2066', '\u2028'];
/// ```
const SHAPES: &[&str] = &[
    "\u{1b}[2K",
    "\n",
    "\r",
    "\t",
    ")",
    "(",
    "`",
    "\\",
    "\"",
    "'",
    "é",
    "\u{202e}",
    "\u{2066}",
    "\u{2028}",
];

/// The wrappings every control below holds the escape to.
const WRAPPINGS: &[TheWrappingAroundTheName] = &[
    TheWrappingAroundTheName::Parentheses,
    TheWrappingAroundTheName::Backticks,
];

/// What:
///  The character that closes one wrapping,
///  which the name inside it may not hold unescaped.
/// Why:
///  A control asserts on the one character its wrapping loses,
///  and the two wrappings differ in exactly that character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const closing = (wrapping) => wrapping === TheWrappingAroundTheName.Parentheses ? ')' : '`';
/// ```
fn the_character_that_closes(wrapping: TheWrappingAroundTheName) -> char {
    return match wrapping {
        TheWrappingAroundTheName::Parentheses => ')',
        TheWrappingAroundTheName::Backticks => '`',
    };
}

/// What:
///  The characters a backslash in the shown text may lead:
///  the escapes this program writes.
/// Why:
///  A lone backslash would leave the rest of the name reading as this program's own text,
///  and the escapes are the only places the shown text may hold a backslash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ESCAPED = ['\\', '(', ')', '`', 'n', 'r', 't', 'u'];
/// ```
const ESCAPED_LEADS: &[char] = &['\\', '(', ')', '`', 'n', 'r', 't', 'u'];

/// What:
///  The shown text holds no control character,
///  no raw line separator,
///  no byte that is not ASCII,
///  and no backslash that leads no escape of this program's.
/// Why:
///  These hold for every wrapping,
///  so each control asks them of the same text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingInTheShownTextCanAct(shown: string, shape: string, name: string): void
/// ```
fn the_shown_text_holds_no_byte_that_can_act(
    name: &str,
    shown: &str,
    shape: &str,
) {
    assert!(
        !shown.chars().any(char::is_control),
        "{shape:?}: no control character is left in {shown:?} for {name:?}"
    );
    assert!(
        !shown.contains('\u{2028}') && !shown.contains('\u{2029}'),
        "{shape:?}: no raw line separator is left in {shown:?} for {name:?}"
    );
    assert!(
        shown.is_ascii(),
        "{shape:?}: every byte of the shown name is ASCII: {shown:?} for {name:?}"
    );
    let characters: Vec<char> = shown.chars().collect();
    let mut at: usize = 0;
    while at < characters.len() {
        let character: char = characters[at];
        if character != '\\' {
            at += 1;
            continue;
        }
        let leads: Option<&char> = characters.get(at + 1);
        assert!(
            leads.is_some_and(|next| return ESCAPED_LEADS.contains(next)),
            "{shape:?}: the backslash at {at} leads no escape of this program's: {shown:?} for {name:?}"
        );
        // What: how far past the backslash this escape runs.
        // Why:  `\u{...}` is longer than the one character every other escape leads.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const length = leads === 'u' ? shown.indexOf('}', at) - at + 1 : 2;
        // ```
        let length: usize = if leads == Some(&'u') {
            let closing: Option<usize> = characters
                .iter()
                .enumerate()
                .skip(at)
                .find(|(_, found)| return **found == '}')
                .map(|(found_at, _)| return found_at);
            assert!(
                closing.is_some(),
                "{shape:?}: a unicode escape ends with its brace: {shown:?} for {name:?}"
            );
            closing.unwrap_or(at) - at + 1
        } else {
            2
        };
        at += length;
    }
}

/// What:
///  The shown text holds no unescaped character of `closing`,
///  which is the character that would end the wrapping,
///  and every one it had is shown behind a backslash.
/// Why:
///  A name that closes the wrapping speaks the rest of the sentence as its own,
///  which is the finding this escape exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingClosesTheWrapping(name: string, shown: string, closing: string): void
/// ```
fn nothing_closes_the_wrapping(
    name: &str,
    shown: &str,
    closing: char,
) {
    let characters: Vec<char> = shown.chars().collect();
    for (at, character) in characters.iter().enumerate() {
        if *character != closing {
            continue;
        }
        assert!(
            at > 0 && characters.get(at - 1) == Some(&'\\'),
            "the {closing:?} at {at} is shown without the backslash that keeps it inside the name: \
             {shown:?} for {name:?}"
        );
    }
    assert_eq!(
        shown.matches(closing).count(),
        name.matches(closing).count(),
        "every {closing:?} the name holds is still shown once: {shown:?} for {name:?}"
    );
}

/// A closing parenthesis inside a name cannot end the parentheses that show it,
/// so the rest of the name cannot read as this program's own words.
#[test]
fn a_parenthesis_inside_a_name_cannot_close_the_parentheses_that_show_it() {
    let name: &str = "x) FORGED\u{1b}[2K";
    let shown: String =
        shown_so_it_cannot_rewrite_the_line(name, TheWrappingAroundTheName::Parentheses);
    assert_eq!(shown, "x\\) FORGED\\u{1b}[2K");
    the_shown_text_holds_no_byte_that_can_act(name, shown.as_str(), "parenthesis");
    nothing_closes_the_wrapping(name, shown.as_str(), ')');
}

/// A backtick inside a name cannot end the backticks that show it,
/// so the rest of the name cannot read as this program's own words.
#[test]
fn a_backtick_inside_a_name_cannot_end_the_backticks_that_show_it() {
    let name: &str = "x` FORGED\u{1b}[2K";
    let shown: String =
        shown_so_it_cannot_rewrite_the_line(name, TheWrappingAroundTheName::Backticks);
    assert_eq!(shown, "x\\` FORGED\\u{1b}[2K");
    the_shown_text_holds_no_byte_that_can_act(name, shown.as_str(), "backtick");
    nothing_closes_the_wrapping(name, shown.as_str(), '`');
}

/// Each wrapping escapes its own character and leaves the other's as the name spells it,
/// so a person reads the name the program chose and not this program's punctuation.
#[test]
fn each_wrapping_escapes_its_own_character_and_no_other() {
    let name: &str = "a)b`c(d";
    assert_eq!(
        shown_so_it_cannot_rewrite_the_line(name, TheWrappingAroundTheName::Parentheses),
        "a\\)b`c\\(d"
    );
    assert_eq!(
        shown_so_it_cannot_rewrite_the_line(name, TheWrappingAroundTheName::Backticks),
        "a)b\\`c(d"
    );
}

/// An escape that would erase the line comes back spelled out,
/// so the person reads the name and the line is the length this program wrote.
#[test]
fn an_escape_that_would_erase_the_line_is_spelled_out() {
    for wrapping in WRAPPINGS {
        let name: &str = "\u{1b}[2Kx";
        let shown: String = shown_so_it_cannot_rewrite_the_line(name, *wrapping);
        assert_eq!(shown, "\\u{1b}[2Kx", "{wrapping:?}");
        the_shown_text_holds_no_byte_that_can_act(name, shown.as_str(), "escape");
    }
}

/// A line break inside the name cannot start a line,
/// which is how a name would come to look like this program's own words.
#[test]
fn a_line_break_inside_the_name_cannot_start_a_line() {
    for wrapping in WRAPPINGS {
        let name: &str = "\nmonochromatic-git-policy-cli: x";
        let shown: String = shown_so_it_cannot_rewrite_the_line(name, *wrapping);
        assert_eq!(shown, "\\nmonochromatic-git-policy-cli: x", "{wrapping:?}");
        assert_eq!(shown.matches('\n').count(), 0, "{shown:?}");
        the_shown_text_holds_no_byte_that_can_act(name, shown.as_str(), "newline");
    }
}

/// A bidirectional control inside the name is spelled out,
/// so it cannot reorder the words of the line a person reads.
#[test]
fn a_bidirectional_control_cannot_reorder_the_line_that_shows_the_name() {
    for wrapping in WRAPPINGS {
        let name: &str = "git\u{202e}ecitca";
        let shown: String = shown_so_it_cannot_rewrite_the_line(name, *wrapping);
        assert_eq!(shown, "git\\u{202e}ecitca", "{wrapping:?}");
        the_shown_text_holds_no_byte_that_can_act(name, shown.as_str(), "override");
    }
}

/// The backslash is escaped in both wrappings,
/// so a name holding `\n` as two characters does not read as the line break this program writes.
#[test]
fn the_backslash_is_escaped_in_every_wrapping() {
    for wrapping in WRAPPINGS {
        assert_eq!(
            shown_so_it_cannot_rewrite_the_line("a\\nb", *wrapping),
            "a\\\\nb",
            "{wrapping:?}"
        );
    }
}

/// A name of printable ASCII is what every program names itself in the ordinary case,
/// and the quotation marks are not the wrapping of either sentence,
/// so they are shown unchanged and the escape costs a reader nothing.
#[test]
fn a_printable_name_is_shown_as_it_is() {
    for wrapping in WRAPPINGS {
        assert_eq!(
            shown_so_it_cannot_rewrite_the_line("git-lfs", *wrapping),
            "git-lfs"
        );
        assert_eq!(
            shown_so_it_cannot_rewrite_the_line("Web Content", *wrapping),
            "Web Content"
        );
        assert_eq!(
            shown_so_it_cannot_rewrite_the_line("\"x\" 'y'", *wrapping),
            "\"x\" 'y'",
            "{wrapping:?}"
        );
    }
}

/// Every shape the repository's security rules list comes back as text that can act on nothing,
/// in every wrapping.
#[test]
fn every_shape_of_the_list_cannot_act_on_the_line() {
    for wrapping in WRAPPINGS {
        for shape in SHAPES {
            let shown: String = shown_so_it_cannot_rewrite_the_line(shape, *wrapping);
            the_shown_text_holds_no_byte_that_can_act(shape, shown.as_str(), shape);
            nothing_closes_the_wrapping(
                shape,
                shown.as_str(),
                the_character_that_closes(*wrapping),
            );
        }
    }
}
