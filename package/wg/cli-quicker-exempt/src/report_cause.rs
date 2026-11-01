//! The cause of a failure line.
//! A channel delivers the first few distinct causes of each failure kind and counts the rest,
//! so what makes two lines the same cause decides what a person sees.
//! Two lines are the same cause when they differ only in numbers
//! (process ids,
//! counts,
//! durations)
//! or in the names a user chose
//! (cgroup paths and unit names).
//! A different reason is another cause,
//! so one error cannot hide another of the same kind.
//! The whole line counts up to a bound,
//! because every loud line starts with a fixed phrase of over a hundred characters
//! that would otherwise use up a short bound before the reason begins.
//! A name that can contain spaces ends where the line says it ends:
//! producers put such a name between backticks with `quote_name`,
//! and the cause replaces everything between the backticks.
//! A name that is not quoted is replaced word by word,
//! which is right for names without spaces,
//! such as unit names.

/// Characters of a normalized line that make up its cause.
const MAX_CAUSE_CHARS: usize = 1024;

/// What a user-chosen name is replaced with.
const NAME_PLACEHOLDER: &str = "<name>";

/// The character that delimits a name a user chose.
const NAME_QUOTE: char = '`';

/// Characters that may surround a word without being part of a name.
const WORD_EDGE: &[char] = &['(', ')', '[', ']', ',', ';', ':', '\'', '"', '`', '.'];

/// Endings of the names of systemd units,
/// which are derived from names a user or an application chose.
const UNIT_SUFFIXES: [&str; 5] = [".service", ".scope", ".slice", ".socket", ".mount"];

/// Start of the note appended to a delivery that follows held-back repeats.
const HELD_SUFFIX_HEAD: &str = " (repeated ";
/// End of the note appended to a delivery that follows held-back repeats.
const HELD_SUFFIX_TAIL: &str = " more time(s) since the last report)";

/// The note appended to a delivery after `held` repeats were held back,
/// so no occurrence is lost from the count.
pub fn held_suffix(held: u32) -> String {
    return format!("{HELD_SUFFIX_HEAD}{held}{HELD_SUFFIX_TAIL}");
}

/// A name a user chose,
/// ready to put in a line:
/// between backticks,
/// with a backtick inside the name written as an escape so it cannot end the name early.
pub fn quote_name(name: &str) -> String {
    return format!("{NAME_QUOTE}{}{NAME_QUOTE}", name.replace(NAME_QUOTE, "\\u{0060}"));
}

/// Whether a character is an ASCII digit.
fn is_digit(character: char) -> bool {
    return character.is_ascii_digit();
}

/// The line without the note of held-back repeats,
/// when it ends with one,
/// so a line and its repeat with a count appended have the same cause.
pub fn without_held_suffix(line: &str) -> &str {
    if let Some(body) = line.strip_suffix(HELD_SUFFIX_TAIL)
        && let Some(index) = body.rfind(HELD_SUFFIX_HEAD)
    {
        let count = &body[index + HELD_SUFFIX_HEAD.len()..];
        if !count.is_empty() && count.chars().all(is_digit) {
            return &line[..index];
        }
    }
    return line;
}

/// Whether a word or a quoted span,
/// without its surrounding punctuation,
/// is a name a user chose:
/// an absolute path or the name of a unit.
fn is_user_chosen_name(core: &str) -> bool {
    if core.starts_with('/') {
        return true;
    }
    for suffix in UNIT_SUFFIXES {
        if core.len() > suffix.len() && core.ends_with(suffix) {
            return true;
        }
    }
    return false;
}

/// Appends a word with every run of digits made alike.
fn push_with_alike_numbers(cause: &mut String, word: &str) {
    let mut in_digits = false;
    for character in word.chars() {
        if character.is_ascii_digit() {
            if !in_digits {
                cause.push('#');
            }
            in_digits = true;
        } else {
            in_digits = false;
            cause.push(character);
        }
    }
}

/// Appends one word of a line:
/// a name a user chose becomes a placeholder and keeps its surrounding punctuation,
/// anything else keeps its text with the numbers made alike.
fn push_word(cause: &mut String, word: &str) {
    let without_lead = word.trim_start_matches(WORD_EDGE);
    let lead = &word[..word.len() - without_lead.len()];
    let core = without_lead.trim_end_matches(WORD_EDGE);
    let tail = &without_lead[core.len()..];
    if !core.is_empty() && is_user_chosen_name(core) {
        cause.push_str(lead);
        cause.push_str(NAME_PLACEHOLDER);
        cause.push_str(tail);
        return;
    }
    push_with_alike_numbers(cause, word);
}

/// Appends text that holds no quoted name,
/// word by word.
fn push_words(cause: &mut String, text: &str) {
    for (index, word) in text.split(' ').enumerate() {
        if cause.chars().count() >= MAX_CAUSE_CHARS {
            return;
        }
        if index > 0 {
            cause.push(' ');
        }
        push_word(cause, word);
    }
}

/// The cause of a failure line:
/// its text with the numbers made alike and the names a user chose replaced,
/// at most the first `MAX_CAUSE_CHARS` characters.
pub fn normalized_cause(line: &str) -> String {
    let mut cause = String::new();
    let mut rest = line;
    while !rest.is_empty() && cause.chars().count() < MAX_CAUSE_CHARS {
        let Some(open) = rest.find(NAME_QUOTE) else {
            push_words(&mut cause, rest);
            break;
        };
        push_words(&mut cause, &rest[..open]);
        let after_open = &rest[open + NAME_QUOTE.len_utf8()..];
        let Some(close) = after_open.find(NAME_QUOTE) else {
            // No closing backtick: the rest is ordinary text.
            cause.push(NAME_QUOTE);
            push_words(&mut cause, after_open);
            break;
        };
        let inside = &after_open[..close];
        if is_user_chosen_name(inside) {
            cause.push_str(NAME_PLACEHOLDER);
        } else {
            cause.push(NAME_QUOTE);
            push_with_alike_numbers(&mut cause, inside);
            cause.push(NAME_QUOTE);
        }
        rest = &after_open[close + NAME_QUOTE.len_utf8()..];
    }
    if cause.chars().count() > MAX_CAUSE_CHARS {
        cause = cause.chars().take(MAX_CAUSE_CHARS).collect();
    }
    return cause;
}
