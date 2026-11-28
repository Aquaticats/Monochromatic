//! What:
//!  The shape of one finding of a layout rule before it gets a severity and a rule name.
//! Why:
//!  Both layout rules report a file,
//!  a line and a message,
//!  and the step that runs them adds the severity and the diagnostic code.
//!  A finding also carries the key that an allow-list entry writes to tolerate it.
//!  A loop of folders, of entries or of files carries none,
//!  because it is tolerated one use at a time and never as a whole;
//!  the one loop finding with a key is the use written in a group's own file,
//!  which no `uses` line can record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LayoutFinding = { file: Path; line: number; message: string; help?: string; key: string };
//! ```

/// Import the native path type.
use std::path::PathBuf;

/// What:
///  A finding of a layout rule.
/// Why:
///  A line of zero means the finding is about a folder or a link and has no line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LayoutFinding = { file: Path; line: number; message: string; help?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutFinding {
    /// The file the finding is reported on:
    ///  the file holding the proving reference,
    ///  or the link or folder the finding is about.
    pub file: PathBuf,
    /// One-based line,
    ///  or zero for no line.
    pub line: usize,
    /// What is wrong,
    ///  naming every folder or file involved.
    pub message: String,
    /// What to do about it.
    pub help: Option<String>,
    /// The stable name of this one finding,
    ///  which an allow-list entry writes to tolerate it;
    ///  empty for a finding no list may tolerate,
    ///  such as a loop of folders, entries or files,
    ///  which is tolerated by listing the uses inside it;
    ///  the loop a group's own file closes carries one,
    ///  because no `uses` line can name that use.
    pub key: String,
}

/// What:
///  A path as findings show it:
///  relative to the package folder when it is inside it.
/// Why:
///  Absolute paths of a checkout would make every message long and different on every machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shown(path: Path, base: Path): string;
/// ```
pub fn shown(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    return match path.strip_prefix(base) {
        Ok(below) if below.components().next().is_some() => below.to_string_lossy().into_owned(),
        _ => path.to_string_lossy().into_owned(),
    };
}

/// What:
///  A name with the whitespace a reader cannot see spelled as its code point,
///  and the characters that print as nothing wherever they stand.
/// Why:
///  A message names a folder, a file or an entry so its reader can find it,
///  and a tab, a no-break space or a space at either end of a name prints as nothing a reader can see:
///  a message that showed `src/a` for a folder named `src/a ` would send its reader to a name the package does hold.
///  A byte order mark, a zero-width space or a soft hyphen prints as nothing wherever it stands,
///  and a name holding one is no name the reader can compare with the names on disk,
///  so each is spelled too.
///  A space between two words stays a space, because the message already reads as words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spelledUnreadable = (name) => [...name]
///   .map((letter, index) =>
///     isFormat(letter) || (letter.isWhitespace() && (letter !== ' ' || index === 0 || index === name.length - 1))
///       ? `\\u{${letter.codePointAt(0)}}`
///       : letter)
///   .join('')
/// ```
pub(crate) fn spelled_unreadable(name: &str) -> String {
    // What: the position of the name's last character, which is an end like the first one is.
    // Why: a space at either end is invisible in a message, where a space between two words is not.
    let last: usize = name.chars().count().saturating_sub(1);
    let mut spelled: String = String::with_capacity(name.len());
    for (index, letter) in name.chars().enumerate() {
        // What: whether this character prints as nothing a reader could measure.
        // Why: every whitespace character but a space between two words disappears into the line it stands in, and a format character disappears wherever it stands.
        let invisible: bool = prints_as_nothing(letter)
            || (letter.is_whitespace() && (letter != ' ' || index == 0 || index == last));
        if !invisible {
            spelled.push(letter);
            continue;
        }
        spelled.push_str(format!("\\u{{{:x}}}", letter as u32).as_str());
    }
    return spelled;
}

/// What:
///  The phrase a message calls one character a reader cannot see:
///  its plain name where this crate has one, and the character's code point either way.
/// Why:
///  A reader told only that a character cannot be seen has nothing to search for,
///  and a character with no common name is still found by its code point in the line.
///  The two spellings differ, so the phrase keeps the name where one exists and never invents one.
///  A character with no name is called `a character` and nothing more,
///  because every sentence that prints the phrase goes on to say that a reader cannot see it:
///  a phrase that said so itself printed the same words twice in one sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const phrase = (letter) =>
///   unseenCharacterName(letter) ?? `a character (\`\\u{${letter.codePointAt(0)!.toString(16)}}\`)`
/// ```
pub(crate) fn unseen_character_phrase(letter: char) -> String {
    return match unseen_character_name(letter) {
        Some(name) => String::from(name),
        None => format!("a character (`\\u{{{:x}}}`)", letter as u32),
    };
}

/// What:
///  Whether a character prints as nothing wherever it stands,
///  so a name holding one is no name a reader can see.
/// Why:
///  A byte order mark before a line's first word makes that word `\u{feff}uses`,
///  which is no word the reader knows, a zero-width space inside a name is no separator it reads,
///  and a right-to-left override reorders every character after it on a terminal that draws bidirectional text;
///  each is a character a person cannot see, so a message about one must spell it.
///  Which characters those are is `layout_unseen_characters`'s answer,
///  which takes the format and control categories from the Unicode tables this crate links
///  and the rest of Unicode's `Default_Ignorable_Code_Point` from the data file it names:
///  a list written here by hand missed the marks, the embeddings and the overrides a copied line carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const printsAsNothing = (letter) => !/\s/u.test(letter) && (/[\p{Cf}\p{Cc}]/u.test(letter) || isDefaultIgnorable(letter))
/// ```
pub(crate) fn prints_as_nothing(letter: char) -> bool {
    return crate::layout_unseen_characters::prints_as_nothing(letter);
}

/// What:
///  The plain name of a character a reader cannot see, with its code point,
///  or nothing when this crate has no plain name for it.
/// Why:
///  A sentence that called a byte order mark "a character a reader cannot see" would leave its reader no way to tell it from a zero-width space,
///  and the two need different edits: a byte order mark at the start of a line is taken out, a zero-width space where a separator belongs is replaced by one.
///  The name given covers the characters a copied line carries:
///  what a word processor and a web page put in front of, inside and around a word,
///  which is where such a line comes from.
///  A character with no name here is still spelled with its code point by the message,
///  so its reader can find it even though the sentence cannot call it anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unseenName = (letter) => ({ '\u{feff}': 'a byte order mark', '\u{200b}': 'a zero-width space', ... })[letter]
/// ```
pub(crate) fn unseen_character_name(letter: char) -> Option<&'static str> {
    return match letter {
        '\u{feff}' => Some("a byte order mark or zero-width no-break space (`\\u{feff}`)"),
        '\u{200b}' => Some("a zero-width space (`\\u{200b}`)"),
        '\u{200c}' => Some("a zero-width non-joiner (`\\u{200c}`)"),
        '\u{200d}' => Some("a zero-width joiner (`\\u{200d}`)"),
        '\u{2060}' => Some("a word joiner (`\\u{2060}`)"),
        '\u{ad}' => Some("a soft hyphen (`\\u{ad}`)"),
        '\u{200e}' => Some("a left-to-right mark (`\\u{200e}`)"),
        '\u{200f}' => Some("a right-to-left mark (`\\u{200f}`)"),
        '\u{202a}' => Some("a left-to-right embedding (`\\u{202a}`)"),
        '\u{202b}' => Some("a right-to-left embedding (`\\u{202b}`)"),
        '\u{202c}' => Some("a pop directional formatting (`\\u{202c}`)"),
        '\u{202d}' => Some("a left-to-right override (`\\u{202d}`)"),
        '\u{202e}' => Some("a right-to-left override (`\\u{202e}`)"),
        '\u{2066}' => Some("a left-to-right isolate (`\\u{2066}`)"),
        '\u{2067}' => Some("a right-to-left isolate (`\\u{2067}`)"),
        '\u{2068}' => Some("a first strong isolate (`\\u{2068}`)"),
        '\u{2069}' => Some("a pop directional isolate (`\\u{2069}`)"),
        '\u{61c}' => Some("an arabic letter mark (`\\u{61c}`)"),
        '\u{180e}' => Some("a mongolian vowel separator (`\\u{180e}`)"),
        '\u{34f}' => Some("a combining grapheme joiner (`\\u{34f}`)"),
        '\u{7f}' => Some("a delete character (`\\u{7f}`)"),
        _ => None,
    };
}

/// What:
///  The first character of a text that a reader cannot see, with its one-based position among the text's characters,
///  or nothing when every character of the text is one a reader can see.
/// Why:
///  A message that names such a character must say where it stands,
///  because the line's words look right without it and its reader has to know which character to take out.
///  The position counts characters rather than bytes,
///  because a byte offset names a place no person can count, and an editor may show a tab as several cells or a letter as more than one byte:
///  `usés` holds five characters and six bytes, and a byte offset would name a place the file does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstUnseen = (text) => {
///   const at = [...text].findIndex(printsAsNothing)
///   return at === -1 ? undefined : [text[at], at + 1]
/// }
/// ```
pub(crate) fn first_unseen_character(text: &str) -> Option<(char, usize)> {
    for (index, letter) in text.chars().enumerate() {
        if prints_as_nothing(letter) {
            return Some((letter, index + 1));
        }
    }
    return None;
}

/// What:
///  How many characters of a text a reader cannot see.
/// Why:
///  Every sentence that names one such character says how many the line holds,
///  because a person told of one and left to find the second on the next run learns nothing from the sentence,
///  and one count answers for the sentences of the reader,
///  the refusal it makes and the dead line it reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unseenCount = (text) => [...text].filter((one) => printsAsNothing(one)).length;
/// ```
pub(crate) fn unseen_character_count(text: &str) -> usize {
    return text
        .chars()
        .filter(|letter| return prints_as_nothing(*letter))
        .count();
}

/// What:
///  The clause that says how many characters a line holds beside the one a sentence names,
///  or nothing when it holds none beside it.
/// Why:
///  A sentence says how many characters its edit changes and where the first of them stands,
///  so the count is what tells its reader that the edit touches more than the character it placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const besideTheFirst = (count) =>
///   count < 2 ? '' : `, and ${count - 1} more character${count === 2 ? '' : 's'} a reader cannot see`;
/// ```
pub(crate) fn characters_beside_the_first(count: usize) -> String {
    return match count {
        0 | 1 => String::new(),
        2 => String::from(", and 1 more character a reader cannot see"),
        more => format!(", and {} more characters a reader cannot see", more - 1),
    };
}

/// What:
///  The characters a sentence names as the ones its edit changes: one, both of two, or every one of more.
/// Why:
///  A sentence that said "that character" about two of them sent its reader after the first and left the other,
///  and the next run named the other again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (count) => (count === 1 ? 'that character' : count === 2 ? 'both such characters' : 'every such character');
/// ```
pub(crate) fn the_characters_named(count: usize) -> &'static str {
    return match count {
        1 => "that character",
        2 => "both such characters",
        _ => "every such character",
    };
}

/// What:
///  The phrase that writes a whitespace character where each character a reader cannot see stands.
/// Why:
///  A character where a separator belongs is replaced and never removed,
///  and two of them need one replacement each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asSpaces = (count) => count === 1
///   ? 'a whitespace character where that character stands'
///   : count === 2
///     ? 'a whitespace character where each of the two such characters stands'
///     : 'a whitespace character where each such character stands';
/// ```
pub(crate) fn the_characters_as_spaces(count: usize) -> &'static str {
    return match count {
        1 => "a whitespace character where that character stands",
        2 => "a whitespace character where each of the two such characters stands",
        _ => "a whitespace character where each such character stands",
    };
}

/// What:
///  A path as one finding's key writes it:
///  relative to the package folder when it is inside it,
///  the walk from the package folder when it is outside (a module a `#[path]` names elsewhere),
///  and always with `/` between segments,
/// whatever the platform's own separator is.
/// Why:
///  A list is written on one machine and read on another,
///  and this crate ships binaries for every platform,
///  so a key built from the platform's separator would match on the machine that wrote the line
///  and nowhere else.
///  Each segment is written as the file system spells it,
///  so a name that holds a backslash keeps it:
///  that is what a file name and an entry of a `uses` folder are written as,
///  and a folder name holding one is refused before any check,
///  because a `uses` line writes a folder as a path.
///  An absolute path would make the line match only one checkout,
///  so a file outside the package is named by the folders between it and the package:
///  `../outside x.rs` reads the same on every machine that holds the repository.
///  The package folder itself is written `.`,
///  because the empty path would end a key in a space that the list reader trims away,
///  and no line could hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // keyPath(p, base) = climbingParts(p, base).join('/'), or '.' for the base folder
/// ```
pub fn key_path(
    path: &std::path::Path,
    base: &std::path::Path,
) -> String {
    // What: the path's own segments, each as the file system spells it, joined with `/`.
    // Why: joining the segments is what keeps a name's own backslash and still writes every separator the same on every platform.
    let written: String = climbing_parts(path, base).join("/");
    // What: the one spelling of the package folder itself.
    // Why: a key that ends in an empty path holds a trailing space, which the list reader trims away.
    if written.is_empty() {
        return String::from(".");
    }
    return written;
}

/// What:
///  How one folder is written from another:
///  one `..` for each folder they do not share, then the folders of `path` below that.
/// Why:
///  A key must read the same in every checkout,
///  and only the distance between two folders of one package is the same everywhere.
///  A relative path and an absolute one share no folder,
///  so they keep the plain spelling the caller gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function climbingParts(path: Path, base: Path): string[];
/// ```
fn climbing_parts(
    path: &std::path::Path,
    base: &std::path::Path,
) -> Vec<String> {
    if path.is_absolute() != base.is_absolute() {
        // What: the path's own spelling as one part, separators and all.
        // Why: the two share no folder to count `..` from, so the caller keeps what it gave.
        return vec![path.to_string_lossy().into_owned()];
    }
    // What: the folders of each path, so the ones they share can be counted once.
    // Why: each folder below the shared part becomes one `..` of the answer.
    let base_parts: Vec<std::path::Component<'_>> = base.components().collect::<Vec<_>>();
    let path_parts: Vec<std::path::Component<'_>> = path.components().collect::<Vec<_>>();
    let mut shared: usize = 0;
    while shared < base_parts.len()
        && shared < path_parts.len()
        && base_parts[shared] == path_parts[shared]
    {
        shared += 1;
    }
    let mut parts: Vec<String> = Vec::<String>::new();
    for _ in shared..base_parts.len() {
        parts.push(String::from(".."));
    }
    for part in &path_parts[shared..] {
        parts.push(part.as_os_str().to_string_lossy().into_owned());
    }
    return parts;
}

/// What:
///  Every part of one path below a folder, as the file system spells it.
/// Why:
///  The folders below the package folder are the names a key writes,
///  and a name's own backslash must stay visible to the check that refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesOf(path: Path): string[];
/// ```
fn names_of(path: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::<String>::new();
    for part in path.components() {
        names.push(part.as_os_str().to_string_lossy().into_owned());
    }
    return names;
}

/// What:
///  Each name a key writes for one path, in order,
///  with a name's own backslash kept instead of being spelled as a separator.
/// Why:
///  A key writes `/` for every separator,
///  so the names of a written key cannot show a name that holds a backslash;
///  the check for a name no list line can hold must see the file system's own spelling,
///  or the name it refuses would be invisible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyNames(path: Path, base: Path): string[];
/// ```
pub(crate) fn key_names(
    path: &std::path::Path,
    base: &std::path::Path,
) -> Vec<String> {
    if let Ok(below) = path.strip_prefix(base)
        && below.components().next().is_some()
    {
        return names_of(below);
    }
    return climbing_parts(path, base);
}

/// What:
///  Make every non-empty key name exactly one finding,
///  by numbering a finding that would share an earlier finding's key.
/// Why:
///  One line of a list tolerates every finding whose key it holds,
///  so two findings with one key would let a line that tolerates today's finding
///  also tolerate a second one added later.
///  A rule tells its findings apart by what each one holds where it can;
///  only findings that hold the same words reach this state,
///  such as two empty `impl` blocks of one type or two glob re-exports of one module.
///  The first keeps the plain key and a later one gains `#2`,
/// `#3` and so on,
/// in the rule's own order,
///  so an entry written before the numbering existed still names its finding,
///  and only an identical finding added above renumbers the ones below it.
///  No line number enters a key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // for each finding: n = ++taken[key]; if (n > 1) finding.key = numbered(key, n)
/// ```
pub fn unique_keys(findings: &mut [LayoutFinding]) {
    // How many findings have already taken each key, in the order the rule produced them.
    let mut taken: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::<String, usize>::new();
    for finding in findings.iter_mut() {
        if finding.key.is_empty() {
            continue;
        }
        let base: String = finding.key.clone();
        let count: usize = match taken.get(&base) {
            Some(seen) => *seen + 1,
            None => 1,
        };
        taken.insert(base.clone(), count);
        if count > 1 {
            finding.key = numbered(base.as_str(), count);
        }
    }
    return;
}

/// What:
///  The key of the `count`th finding of a shape whose own words an earlier finding already holds.
/// Why:
///  The number goes into the key's first word,
/// right after the shape it repeats,
///  because no literal key holds `#` there:
///  every key a rule prints starts with a shape word and a space,
///  so `foreign-name#2 src/lib.rs x` can never be read as a doc entry named `x#2`,
///  which prints `foreign-name src/lib.rs x#2`.
///  Two findings therefore never share a key,
///  and a line written for the first cannot tolerate the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numbered = (base, count) => base.replace(' ', `#${count} `)
/// ```
fn numbered(
    base: &str,
    count: usize,
) -> String {
    return match base.split_once(' ') {
        Some((shape, rest)) => format!("{shape}#{count} {rest}"),
        None => format!("{base}#{count}"),
    };
}

/// Controls of the path display stay outside release artifacts.
#[cfg(test)]
#[path = "layout_finding_tests.rs"]
mod tests;
