//! What:
//!  Text another program chose,
//!  shown inside one line a person reads on a terminal:
//!  every control character written as an escape,
//!  the parentheses and the backslash escaped,
//!  and every character that is not printable ASCII written as an escape too.
//! Why:
//!  A process sets the name this program prints for it (`/proc/<pid>/comm`) to any bytes it likes,
//!  and a line belongs to the terminal that reads it:
//!  an escape byte can move the cursor or erase the line this program is writing,
//!  a line break starts a line this program never wrote,
//!  and a `)` ends the parentheses that say the name is another program's,
//!  so the rest of the name then reads as this program's own words.
//!  The name is shown as text that can do none of those things (`AGENTS.md`, rule `SYB`),
//!  and a person can still read which program is named.
//!  Nothing here decides where the name sits in the sentence: the caller writes that around this text,
//!  and this is the step before the sentence reaches the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shown = shownSoItCannotRewriteTheLine(nameOfTheProgram);
//! ```

/// What:
///  `text` as characters that cannot act on a terminal line:
///  a printable ASCII character stays,
///  except `(`, `)` and `\`, which carry a backslash before them,
///  and every other character is written as Rust's own escape for it
///  (`\n`, `\t`, `\u{1b}`).
/// Why:
///  The name of a process or a program is not this program's text,
///  and it lands in a line a person reads to find that program:
///  an escape movement, a line break or a closing parenthesis inside the name would change what the line says.
///  The escaped form reads back by eye,
///  so the person still knows which program holds the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shownSoItCannotRewriteTheLine(text: string): string {
///   return [...text].map((c) => {
///     if (c === '(') return '\\(';
///     if (c === ')') return '\\)';
///     if (c === '\\') return '\\\\';
///     if (c >= ' ' && c <= '~') return c;
///     return escapeDefault(c);
///   }).join('');
/// }
/// ```
pub fn shown_so_it_cannot_rewrite_the_line(text: &str) -> String {
    // What: the text as it is being built, one character at a time.
    // Why:  a process name is fifteen bytes at most and a program path is short,
    //  so one pass into one buffer is the whole cost and the pass stays linear in the text's length.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let shown = '';
    // ```
    let mut shown: String = String::with_capacity(text.len());
    // What: one character of the text.
    // Why:  each is judged alone, because one escape byte anywhere in the name is the whole hazard.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const character of text) { /* judge it below */ }
    // ```
    for character in text.chars() {
        // What: the character as it goes into the line: itself, or the characters that spell it.
        // Why:  the four arms are the four things a terminal or a reader may do with a character:
        //  the parentheses say whose name this is, the backslash is the escape itself,
        //  printable ASCII is safe to show, and everything else (`\n`, `\u{1b}`, `\u{e9}`) is spelled out.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (character === '(') shown += '\\('; else if (character === ')') shown += '\\)'; else if (character === '\\') shown += '\\\\'; else if (character >= ' ' && character <= '~') shown += character; else shown += escapeDefault(character);
        // ```
        match character {
            '(' => shown.push_str("\\("),
            ')' => shown.push_str("\\)"),
            '\\' => shown.push_str("\\\\"),
            ' '..='~' => shown.push(character),
            other => shown.extend(other.escape_default()),
        }
    }
    // What: the built text, handed back as one `String`.
    // Why:  the caller puts it straight into the sentence it writes, so the escape is the last step before that line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return shown;
    // ```
    return shown;
}

/// Controls of this escape stay out of the release executable.
#[cfg(test)]
#[path = "shown_so_it_cannot_rewrite_the_line_tests.rs"]
mod tests;
