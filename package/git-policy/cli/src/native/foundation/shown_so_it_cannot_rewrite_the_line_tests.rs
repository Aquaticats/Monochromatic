//! Controls for the text this program shows of a name another program chose.
//! Why:
//!  A process sets its own name to any bytes it likes,
//!  and the line that names it is the one place a person finds it,
//!  so each shape a terminal or a reader could take for something else must come back spelled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(shownSoItCannotRewriteTheLine('\u001b[2K')).toBe('\\u{1b}[2K');
//! ```

use super::*;

/// What:
///  The shapes the security rules of the repository list for text that crosses into a line:
///  an escape that starts a terminal sequence,
///  a line break that starts a line this program never wrote,
///  the quotes and the parentheses that would end the name,
///  a tab,
///  a backslash,
///  and a character that is not ASCII.
/// Why:
///  Each is a way a name could change what the line says or where the cursor goes;
///  one list keeps every control below on the same shapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHAPES = ['\u001b[2K', '\n', '\r', '\t', ')', '(', '\\', '"', "'", 'é'];
/// ```
const SHAPES: &[&str] = &[
    "\u{1b}[2K",
    "\n",
    "\r",
    "\t",
    ")",
    "(",
    "\\",
    "\"",
    "'",
    "é",
];

/// A name that holds an escape, a line break or a closing parenthesis comes back as text that can do none of those
/// things,
/// and a name of printable ASCII comes back as it is.
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
    // What: the characters a backslash in the shown text may lead: the escapes this program writes.
    // Why:  a lone backslash would leave the rest of the name reading as this program's own text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ESCAPED = ['\\', '(', ')', 'n', 'r', 't', 'u'];
    // ```
    const ESCAPED: &[char] = &['\\', '(', ')', 'n', 'r', 't', 'u'];
    // What: the shown text's characters, read one escape at a time.
    // Why:  a parenthesis is safe only while a backslash leads it, so the walk has to know where each escape ends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const characters = [...shown];
    // ```
    let characters: Vec<char> = shown.chars().collect();
    // What: where the walk stands in the shown text.
    // Why:  an escape is one unit and the loop moves over it whole, so the character after it is read as itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let at = 0;
    // ```
    let mut at: usize = 0;
    while at < characters.len() {
        let character: char = characters[at];
        // What: an escape, which is the backslash and the characters that spell what it leads.
        // Why:  the parenthesis inside an escape is the one the reader takes as part of the name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (character === '\\') { const leads = characters[at + 1]; assert(ESCAPED.includes(leads)); at += leads === 'u' ? characters.indexOf('}', at) - at + 1 : 2; continue; }
        // ```
        if character == '\\' {
            let leads: Option<&char> = characters.get(at + 1);
            assert!(
                leads.is_some_and(|next| return ESCAPED.contains(next)),
                "{shape:?}: the backslash at {at} leads no escape of this program's: {shown:?} for {name:?}"
            );
            // What: how far past the backslash this escape runs.
            // Why:  `\\u{...}` is longer than the one character every other escape leads.
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
            continue;
        }
        assert!(
            character != '(' && character != ')',
            "{shape:?}: the parenthesis at {at} stands outside an escape, so it could end the name: {shown:?} for {name:?}"
        );
        at += 1;
    }
}

/// A terminal escape that would erase the line comes back spelled out,
/// so the person reads the name and the line is the length this program wrote.
#[test]
fn an_escape_that_would_erase_the_line_is_spelled_out() {
    let shown: String = shown_so_it_cannot_rewrite_the_line("\u{1b}[2Kx");
    assert_eq!(shown, "\\u{1b}[2Kx");
    the_shown_text_holds_no_byte_that_can_act("\u{1b}[2Kx", shown.as_str(), "escape");
}

/// A line break inside the name cannot start a line,
/// which is how a name would come to look like this program's own words.
#[test]
fn a_line_break_inside_the_name_cannot_start_a_line() {
    let shown: String = shown_so_it_cannot_rewrite_the_line("\nmonochromatic-git-policy-cli: x");
    assert_eq!(shown, "\\nmonochromatic-git-policy-cli: x");
    assert_eq!(shown.matches('\n').count(), 0, "{shown:?}");
    the_shown_text_holds_no_byte_that_can_act(
        "\nmonochromatic-git-policy-cli: x",
        shown.as_str(),
        "newline",
    );
}

/// The parentheses and the backslash are escaped,
/// so a name cannot end the sentence's own punctuation and go on speaking as this program.
#[test]
fn the_parentheses_and_the_backslash_are_escaped() {
    assert_eq!(shown_so_it_cannot_rewrite_the_line(") (x"), "\\) \\(x");
    assert_eq!(shown_so_it_cannot_rewrite_the_line("a\\b"), "a\\\\b");
}

/// A name of printable ASCII is what every program names itself in the ordinary case,
/// and it is shown unchanged,
/// so the escape costs a reader nothing.
#[test]
fn a_printable_name_is_shown_as_it_is() {
    assert_eq!(shown_so_it_cannot_rewrite_the_line("git-lfs"), "git-lfs");
    assert_eq!(
        shown_so_it_cannot_rewrite_the_line("Web Content"),
        "Web Content"
    );
}

/// Every shape the repository's security rules list comes back as text that can act on nothing.
#[test]
fn every_shape_of_the_list_cannot_act_on_the_line() {
    for shape in SHAPES {
        let shown: String = shown_so_it_cannot_rewrite_the_line(shape);
        the_shown_text_holds_no_byte_that_can_act(shape, shown.as_str(), shape);
    }
}
