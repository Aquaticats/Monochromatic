//! What:
//!  Text another program chose,
//!  shown inside one line a person reads on a terminal,
//!  escaped for the characters the sentence it lands in puts around it:
//!  every control character and every character outside printable ASCII
//!  written as an escape,
//!  the backslash escaped,
//!  and the character that would end the wrapping escaped too
//!  (a parenthesis,
//!  or a backtick).
//! Why:
//!  A process sets the name this program prints for it (`/proc/<pid>/comm`)
//!  to any bytes it likes,
//!  and a line belongs to the terminal that reads it:
//!  an escape byte can move the cursor or erase the line this program is writing,
//!  a line break starts a line this program never wrote,
//!  and the character that closes the wrapping ends the name before the name
//!  does,
//!  so the rest of it reads as this program's own words.
//!  The name is shown as text that can do none of those things (`AGENTS.md`,
//!  rule `SYB`:
//!  text crossing a syntax boundary obeys the destination grammar,
//!  encoded where it is written into that grammar),
//!  and a person can still read which program is named.
//!  Nothing here decides where the name sits in the sentence:
//!  the caller writes that around this text and names the wrapping it used,
//!  so this is the last step before the sentence reaches the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shown = shownSoItCannotRewriteTheLine(nameOfTheProgram, TheWrappingAroundTheName.Parentheses);
//! ```

/// What:
///  The characters a sentence puts around a name it shows,
///  which the name inside may not hold.
/// Why:
///  Each sentence of this program names a program inside its own punctuation
///  (`PID 5 (git)`,
///  `` `ssh-keygen` (process 6) ``),
///  so the character that closes that punctuation is escaped for the sentence
///  that uses it and only there:
///  a parenthesis inside a backtick-wrapped name is part of the name,
///  and escaping it there would show a reader a character the program never had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWrappingAroundTheName = 'nothing' | 'parentheses' | 'backticks';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TheWrappingAroundTheName {
    /// A pair of parentheses:
    /// `(git)`.
    Parentheses,
    /// A pair of backticks:
    /// `` `git` ``.
    Backticks,
}

/// What:
///  `text` as characters that cannot act on the line it is going into:
///  a printable ASCII character stays,
///  except `\`,
///  and the character that would close the wrapping it is shown in,
///  which carry a backslash before them,
///  and every other character is written as Rust's own escape for it
///  (`\n`,
///  `\t`,
///  `\u{1b}`).
/// Why:
///  The name of a process or a program is not this program's text,
///  and it lands in a line a person reads to find that program:
///  an escape movement,
///  a line break,
///  a right-to-left override,
///  a closing parenthesis or a closing backtick inside the name
///  would change what the line says.
///  The backslash is escaped in both wrappings because the escapes here are
///  spelled with it:
///  a name holding `\n` as two bytes must not read as the line break this
///  program writes for a real one.
///  The escaped form reads back by eye,
///  so the person still knows which program holds the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shownSoItCannotRewriteTheLine(text: string, wrapping: TheWrappingAroundTheName): string {
///   const closing = wrapping === TheWrappingAroundTheName.Parentheses ? [')', '('] : wrapping === TheWrappingAroundTheName.Backticks ? ['`'] : [];
///   return [...text].map((c) => c === '\\' ? '\\\\' : closing.includes(c) ? `\\${c}` : c >= ' ' && c <= '~' ? c : escapeDefault(c)).join('');
/// }
/// ```
pub(crate) fn shown_so_it_cannot_rewrite_the_line(
    text: &str,
    wrapping: TheWrappingAroundTheName,
) -> String {
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
        // Why:  the arms are the things a terminal or a reader may do with a character:
        //  the backslash is the escape itself, the wrapping's own character says whose name this is,
        //  printable ASCII is safe to show, and everything else (`\n`, `\u{1b}`, `\u{e9}`) is spelled out.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (character === '\\') shown += '\\\\'; else if (closing.includes(character)) shown += `\\${character}`; else if (character >= ' ' && character <= '~') shown += character; else shown += escapeDefault(character);
        // ```
        if character == '\\' {
            shown.push_str("\\\\");
            continue;
        }
        let closes_the_wrapping: bool = match wrapping {
            TheWrappingAroundTheName::Parentheses => character == '(' || character == ')',
            TheWrappingAroundTheName::Backticks => character == '`',
        };
        if closes_the_wrapping {
            shown.push('\\');
            shown.push(character);
            continue;
        }
        if character.is_ascii_graphic() || character == ' ' {
            shown.push(character);
            continue;
        }
        shown.extend(character.escape_default());
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
