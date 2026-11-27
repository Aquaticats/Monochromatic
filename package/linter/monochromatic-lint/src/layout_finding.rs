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
        let invisible: bool = is_format_character(letter)
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
///  Whether a character prints as nothing wherever it stands,
///  so a name holding one is no name a reader can see.
/// Why:
///  A byte order mark before a line's first word makes that word `\u{feff}uses`,
///  which is no word the reader knows, and a zero-width space inside a name is no separator it reads;
///  both are characters a person cannot see, so a message about one must spell it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFormat = (letter) => ['\u{ad}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}'].includes(letter)
/// ```
pub(crate) fn is_format_character(letter: char) -> bool {
    return matches!(
        letter,
        '\u{ad}' | '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}'
    );
}

/// What:
///  The plain name of a character a reader cannot see, with its code point,
///  or nothing when the character is one a reader can see.
/// Why:
///  A sentence that called a byte order mark "a character a reader cannot see" would leave its reader no way to tell it from a zero-width space,
///  and the two need different edits: a byte order mark at the start of a line is taken out, a zero-width space where a separator belongs is replaced by one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatName = (letter) => ({ '\u{feff}': 'a byte order mark', '\u{200b}': 'a zero-width space', ... })[letter]
/// ```
pub(crate) fn format_character_name(letter: char) -> Option<&'static str> {
    return match letter {
        '\u{feff}' => Some("a byte order mark (`\\u{feff}`)"),
        '\u{200b}' => Some("a zero-width space (`\\u{200b}`)"),
        '\u{200c}' => Some("a zero-width non-joiner (`\\u{200c}`)"),
        '\u{200d}' => Some("a zero-width joiner (`\\u{200d}`)"),
        '\u{2060}' => Some("a word joiner (`\\u{2060}`)"),
        '\u{ad}' => Some("a soft hyphen (`\\u{ad}`)"),
        _ => None,
    };
}

/// What:
///  The first character of a text that a reader cannot see, with its one-based position,
///  or nothing when every character of the text is one a reader can see.
/// Why:
///  A message that names such a character must say where it stands,
///  because the line's words look right without it and its reader has to know which character to take out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstFormat = (text) => {
///   const at = [...text].findIndex(isFormat)
///   return at === -1 ? undefined : [text[at], at + 1]
/// }
/// ```
pub(crate) fn first_format_character(text: &str) -> Option<(char, usize)> {
    for (index, letter) in text.chars().enumerate() {
        if is_format_character(letter) {
            return Some((letter, index + 1));
        }
    }
    return None;
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
