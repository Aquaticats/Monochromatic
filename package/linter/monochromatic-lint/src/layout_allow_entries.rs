//! What:
//!  One line of a layout rule's allow list:
//!  the shape it reads as,
//!  the words that spell each shape,
//!  and the one way each shape is written.
//! Why:
//!  A line is the unit a person edits,
//!  and a line that reads as no shape must fail the run rather than pass for an entry,
//!  because a silently ignored line would let a finding through.
//!  The list that holds the entries is `layout_allow_list.rs`;
//!  it reads them from here and never walks a line itself.
//!  A `uses` line is read back exactly as the rule prints its key:
//!  the four words the shape is written with,
//!  or, when a folder name holds a space,
//!  the two sides of the arrow the line holds,
//!  so a name the rule prints is a name the reader reads.
//!  A listed path is then read back against the package folder,
//!  so a line that climbs out with `..` and comes back in names the folder every key names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AllowEntry = { line: number; text: string; kind: AllowKind };
//! function parseEntries(text: string, base: Path): AllowEntry[];
//! ```

/// Import the spelling every listed path is read into,
///  the spelling of the characters a message cannot show,
///  and the reading that removes the `.` segments and the folders a `..` reads back over.
use crate::{
    layout_finding::{
        key_path,
        spelled_whitespace,
    },
    markdown_lfs_target::lexical_normal,
};
/// Import the set that keeps one line from repeating another,
///  the one spelling of a listed path,
///  and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

/// The word that starts an entry tolerating one use between two folders.
pub(crate) const FOLDER_USE_WORD: &str = "uses";

/// The word that starts an entry tolerating one use between two files of a folder.
const FILE_USE_WORD: &str = "file-uses";

/// The text that joins the two ends of a tolerated use.
pub(crate) const ARROW: &str = "->";

/// The arrow as a printed line writes it,
///  with the one space that sets it off on each side.
/// Why:
///  A line may set its arrow off with any whitespace character,
///  and this spelling is the one every key and every message quotes,
///  so it is what a shape of a name is named by.
pub(crate) const SET_OFF_ARROW: &str = " -> ";

/// What:
///  What one entry of a list tolerates.
/// Why:
///  A use is suppressed before loops are found;
///  a finding key is matched against the findings that were produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowKind = { kind: 'folder-use'; from: Path; to: Path }
///   | { kind: 'file-use'; folder: Path; from: string; to: string }
///   | { kind: 'key'; key: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllowKind {
    /// One use between two folders of the package.
    FolderUse {
        /// The folder whose code uses.
        from: PathBuf,
        /// The folder it uses.
        to: PathBuf,
    },
    /// One use between two files of one folder.
    FileUse {
        /// The folder both files are in.
        folder: PathBuf,
        /// The file whose code uses.
        from: String,
        /// The file it uses.
        to: String,
    },
    /// The exact key one finding reports.
    Key(
        /// The key text,
        /// as `--debug` prints it.
        String,
    ),
}

/// What:
///  One line of a list,
/// as written and as read.
/// Why:
///  A finding about a dead entry points at that line,
///  and its message quotes the line so the reader can find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowEntry = { line: number; text: string; kind: AllowKind };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowEntry {
    /// One-based line of the list file.
    pub line: usize,
    /// The line as written,
    /// without surrounding spaces.
    pub text: String,
    /// What the line tolerates.
    pub kind: AllowKind,
}

/// What:
///  What one entry tolerates,
///  written the one way every spelling of it reads.
/// Why:
///  No line may repeat another,
///  and `uses ./src/a/ -> src/b` repeats `uses src/a -> src/b`:
///  a `.` segment and a trailing separator name the same folder,
///  so the list must say so rather than hold two lines for one use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entryIdentity(entry): string
/// ```
impl AllowEntry {
    /// The entry's kind in the words the list writes it.
    pub fn identity(&self) -> String {
        return match &self.kind {
            AllowKind::FolderUse {
                from,
                to,
            } => format!(
                "{FOLDER_USE_WORD} {} {ARROW} {}",
                from.display(),
                to.display()
            ),
            AllowKind::FileUse {
                folder,
                from,
                to,
            } => format!("{FILE_USE_WORD} {} {from} {ARROW} {to}", folder.display()),
            AllowKind::Key(key) => key.clone(),
        };
    }
}

/// What:
///  Why a path an entry names may not be used,
///  or nothing when it may.
/// Why:
///  A finding shows paths relative to the package,
///  and an absolute path would mean a different file on every machine.
///  A relative path may climb out of the package with `..`,
///  because a module a `#[path]` names elsewhere is a folder of the package's tree
///  and every key about it writes that walk from the package folder;
///  a key the rule prints and the reader refuses would be the rule contradicting itself.
///  Keys hold `/` between segments on every platform,
///  so an entry written with a backslash would name a path no key matches,
///  and a folder whose name holds a backslash cannot be listed at all:
///  saying so is better than a line that tolerates nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unusableReason(path: string): string | undefined;
/// ```
fn unusable_reason(path: &str) -> Option<&'static str> {
    if path.is_empty() {
        return Some("names no folder");
    }
    if path.contains('\\') {
        return Some("holds a backslash, and every path is written with `/` on every machine");
    }
    if Path::new(path).is_absolute() {
        return Some("is absolute");
    }
    return None;
}

/// What:
///  One path as an entry names it:
///  with `.` segments, a trailing separator and the folders a `..` reads back over gone,
///  so `./src/a` and `src/a/` name the folder a finding names.
/// Why:
///  A use is counted from the path the code makes,
///  while an entry is read from its own words,
///  so a spelling difference between them would make a line that records a use tolerate nothing.
///  A `.` on its own names the package folder itself,
///  which is how a key writes it,
///  and `./` is the same folder.
///  A `..` that names nothing before it stays,
///  because a folder a `#[path]` names outside the package is written by that walk from the package folder:
///  `../outside` is the folder of a module declared `#[path = "../../outside/mod.rs"]`,
///  and reading it away would name a folder inside the package instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listedPath = (text) => {
///   const normal = lexicalNormalKeepingARisingDotDot(text)
///   return normal === '' && text !== '' ? '.' : normal
/// }
/// ```
fn listed_path(text: &str) -> PathBuf {
    let mut path: PathBuf = PathBuf::new();
    for part in text.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            // What: whether this `..` reads back over the folder before it, or names one above the package.
            // Why: a `..` with no folder before it is the walk out of the package, and reading it away would rename the folder.
            let over_a_folder: bool = matches!(
                path.components().next_back(),
                Some(std::path::Component::Normal(_))
            );
            if over_a_folder {
                path.pop();
            } else {
                path.push(part);
            }
            continue;
        }
        path.push(part);
    }
    // What: the spelling a key uses for the package folder, kept instead of being read away.
    // Why: an entry about the package folder must compare equal to the key that names it.
    if path.as_os_str().is_empty() && !text.is_empty() {
        return PathBuf::from(".");
    }
    return path;
}

/// What:
///  The folder a `file-uses` line names,
///  when its words are more than the shape has and the ones in the folder's place hold whitespace.
/// Why:
///  The shape is five words set off with one whitespace character each,
///  so a folder whose name holds a space, a tab, a no-break space or a doubled space makes six
///  and the line cannot be read at all.
///  The five-word sentence alone would send its reader back to a line that already reads as the shape it names,
///  so the refusal names the folder and the whitespace in its name instead.
///  A file name never holds whitespace in a tree the rules accept (that is `space-in-file-name`),
///  so the raw text between the word and the file before the arrow is the folder's own name.
///  That raw text is read rather than the words joined with one space,
///  because joining them would name a folder the package does not hold for every whitespace character but a space.
///  Those words are the folder's own only while they name one:
///  a third file name and a doubled arrow make six words whose middle ones are no folder,
///  and telling that reader to rename a folder that is fine sends them to the wrong place.
///  Only a folder of the tree can hold the whitespace, so the caller gives the package folder the words are read from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spacedFolder = (text: string, words: string[], base: Path) => {
///   if (words.length < 6 || words[words.length - 2] !== '->') return undefined
///   const afterWord = text.slice('file-uses'.length + 1)
///   const file = words[words.length - 3]
///   const folder = afterWord.slice(0, -file.length).slice(0, -1)
///   return /\s/.test(folder) && isDirectory(base.join(folder)) ? folder : undefined
/// }
/// ```
fn spaced_file_use_folder(
    text: &str,
    words: &[&str],
    base: &Path,
) -> Option<String> {
    // What: whether the arrow stands where the shape puts it, with one file name after it.
    // Why: a line whose arrow stands anywhere else is a misspelling of the shape and not a spaced folder.
    if words.len() < 6 || words[words.len() - 2] != ARROW {
        return None;
    }
    // What: the one file name the shape puts just before the arrow.
    // Why: the folder is the raw text between the word that starts the line and that file.
    let file: &str = words[words.len() - 3];
    // What: the line after the word `file-uses`, with the one whitespace character that sets the folder off.
    // Why: exactly one character is the separator, and a whitespace character beside the name is part of the name.
    let after_the_word: &str = text
        .strip_prefix(FILE_USE_WORD)?
        .strip_prefix(char::is_whitespace)?;
    // What: where the line's own arrow begins, which is the first one whitespace sets off on both sides.
    // Why: a file name is read as written and may hold a bare `->`, so only the set-off arrow ends the folder.
    let arrow: usize = arrow_between_whitespace(after_the_word)?;
    // What: the words before the arrow, with the whitespace that sets the arrow off taken away.
    // Why: the folder and the file before the arrow are what those words hold.
    let before_arrow: &str = after_the_word[..arrow].strip_suffix(char::is_whitespace)?;
    // What: the folder's name exactly as the line writes it, whitespace and all.
    // Why: a joined spelling would turn `src/c<TAB>d` into `src/c d`, which is no folder the package holds.
    let folder: &str = before_arrow
        .strip_suffix(file)?
        .strip_suffix(char::is_whitespace)?;
    // What: whether that name holds any whitespace character, which is the one reason the shape's five words become six.
    // Why: a line whose middle words are no folder at all is a misspelling of the shape, so the sentence about the folder would be false.
    if !folder.chars().any(char::is_whitespace) {
        return None;
    }
    // What: whether those words name a folder of the package.
    // Why: only a folder that is really there can be the one the line cannot carry.
    if !base.join(folder).is_dir() {
        return None;
    }
    // What: the folder name the line cannot carry.
    // Why: the caller's sentence names it, so its reader sees which name to change.
    return Some(String::from(folder));
}

/// What:
///  The plain name of the whitespace character a folder name holds,
///  with its code point.
/// Why:
///  A tab or a no-break space prints as nothing in a terminal,
///  so a sentence that called it "a whitespace character" would leave its reader no way to tell it from a space.
///  The first character that is not a space is the one named, because a space is the one whitespace a reader can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whitespaceName = (name: string) => {
///   const letter = [...name].find((one) => /\s/.test(one) && one !== ' ')
///   if (!letter) return 'a whitespace character'
///   return `${plainName(letter)} (\u{${letter.codePointAt(0)!.toString(16)}})`
/// }
/// ```
fn whitespace_character_description(name: &str) -> String {
    // What: the first whitespace character that is not a space, or nothing when the name holds none.
    // Why: a space is visible between two words, so only another whitespace character needs its code point.
    let Some(letter) = name
        .chars()
        .find(|letter| return letter.is_whitespace() && *letter != ' ')
    else {
        return String::from("a whitespace character");
    };
    // What: the plain name of that character.
    // Why: a reader who is told `\u{9}` alone still has to look the code point up.
    let named: &str = match letter {
        '\t' => "a tab",
        '\u{a0}' => "a no-break space",
        '\u{202f}' => "a narrow no-break space",
        '\u{3000}' => "an ideographic space",
        '\u{2000}'..='\u{200a}' => "a typographic space",
        '\u{1680}' => "an ogham space mark",
        '\u{205f}' => "a medium mathematical space",
        '\u{2028}' => "a line separator",
        '\u{2029}' => "a paragraph separator",
        '\u{85}' => "a next-line character",
        '\u{b}' => "a vertical tab",
        '\u{c}' => "a form feed",
        '\r' => "a carriage return",
        _ => "a whitespace character",
    };
    // What: the character's code point, in the spelling every message of this rule uses.
    // Why: the name above is worth little without the exact character the line holds.
    let code_point: u32 = letter as u32;
    return format!("{named} (`\\u{{{code_point:x}}}`)");
}

/// What:
///  Read one line as an entry,
///  or say why it names nothing.
/// Why:
///  A line that is a misspelling must fail the run,
///  because a silently ignored line would let a finding through.
///  The package folder reads every listed path back into the one spelling a key writes,
///  so what a line names and what a finding names meet in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(line: number, text: string, base: Path): AllowEntry;
/// ```
fn entry_of(
    line: usize,
    text: &str,
    base: &Path,
) -> Result<AllowEntry, String> {
    let words: Vec<&str> = text.split_whitespace().collect::<Vec<&str>>();
    let kind: AllowKind = if words.first() == Some(&FOLDER_USE_WORD) {
        let (from, to): (&str, &str) = use_ends(line, text)?;
        for path in [from, to] {
            if let Some(reason) = unusable_reason(path) {
                return Err(format!(
                    "Line {line} names a folder that {reason}. A path is relative to the package folder, as findings show it."
                ));
            }
        }
        AllowKind::FolderUse {
            from: spelled_path(base, &listed_path(from)),
            to: spelled_path(base, &listed_path(to)),
        }
    } else if words.first() == Some(&FILE_USE_WORD) {
        if let Some(folder) = spaced_file_use_folder(text, words.as_slice(), base) {
            // What: the folder's name with every character a reader cannot see spelled, so the message names the folder that is there.
            // Why: a tab or a no-break space prints as nothing, and a reader shown `src/cd` would look for a folder the package may well hold.
            let spelled_folder: String = spelled_whitespace(folder.as_str());
            // What: whether every whitespace character in the name is a space.
            // Why: a space is the one whitespace character a reader can see, so only another one needs its name and code point.
            let holds_only_spaces: bool = folder
                .chars()
                .all(|letter| return !letter.is_whitespace() || letter == ' ');
            // What: what the name holds, and what renaming must give.
            // Why: a name holding a tab is fixed by a rename that also keeps spaces, and the sentence must not send its reader to the wrong place.
            let (holds, fix): (String, &str) = if holds_only_spaces {
                (String::from("which holds a space"), "space")
            } else {
                (
                    format!("which holds {}", whitespace_character_description(folder.as_str())),
                    "whitespace character",
                )
            };
            return Err(format!(
                "Line {line} names the folder `{spelled_folder}`, {holds}: a `file-uses` line sets its five words off with one whitespace character each, so it cannot carry a name that holds one. Rename the folder so its name holds no {fix}, or untangle the loop of files inside it."
            ));
        }
        if words.len() != 5 || words[3] != ARROW {
            // What: whether the line sets more than one arrow off with whitespace, which is one way its five words become more.
            // Why: a doubled arrow is a different mistake from an extra word, and its reader has to know which one is theirs.
            if arrows_between_whitespace(text) > 1 {
                return Err(format!(
                    "Line {line} reads `{text}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`, and it holds one `{ARROW}` only: two `{ARROW}` are set off by whitespace."
                ));
            }
            return Err(format!(
                "Line {line} reads `{text}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`, which is five words; this line is {} words.",
                words.len()
            ));
        }
        if let Some(reason) = unusable_reason(words[1]) {
            return Err(format!(
                "Line {line} names a folder that {reason}. A path is relative to the package folder, as findings show it."
            ));
        }
        AllowKind::FileUse {
            folder: spelled_path(base, &listed_path(words[1])),
            from: String::from(words[2]),
            to: String::from(words[4]),
        }
    } else {
        AllowKind::Key(String::from(text))
    };
    return Ok(AllowEntry {
        line,
        text: String::from(text),
        kind,
    });
}

/// What:
///  One listed path written the way a finding's key writes that folder:
///  read back against the package folder, then shown from it.
/// Why:
///  A line may climb out of the package with `..` and come back into it:
///  with the package folder named `p`, `uses ../p/src/a -> src/b` names the use `src/a -> src/b`,
///  which is the spelling every key prints.
///  Reading every listed path into that spelling once, as the list is read,
///  gives tolerance, death and the repeat check one answer for one use,
///  where the listed spelling and the key's disagreed:
///  a repeat written two ways stayed unreported,
///  and a `file-uses` folder the walk names one way tolerated nothing when the line spelled it the other.
///  A line's own text stays as written, because a dead entry quotes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spelled = (base: Path, listed: string) => keyPath(lexicalNormal(base.join(listed)), base)
/// ```
fn spelled_path(
    base: &Path,
    listed: &Path,
) -> PathBuf {
    return PathBuf::from(key_path(&lexical_normal(&base.join(listed)), base).as_str());
}

/// What:
///  Whether the `->` at a position is set off with one whitespace character on each side.
/// Why:
///  A `uses` or `file-uses` line sets its arrow off that way,
///  and both the arrow search and the doubled-arrow check must read the same rule,
///  or a line one of them refuses would be a line the other reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setOff = (text: string, at: number) =>
///   /\s/.test(text[at - 1] ?? '') && /\s/.test(text[at + 2] ?? '')
/// ```
fn set_off_by_whitespace(
    text: &str,
    at: usize,
) -> bool {
    // What: the character before this `->`, or nothing when the text begins with it.
    // Why: a name's own `->` has no whitespace in front of it, so it is never taken for the arrow.
    let before: Option<char> = text[..at].chars().next_back();
    // What: the character after this `->`, or nothing when the text ends with it.
    // Why: an arrow at the end of a line names nothing, so the line holds no such arrow.
    let after: Option<char> = text[at + ARROW.len()..].chars().next();
    // What: whether whitespace stands on both sides, which is what sets an arrow off.
    // Why: that is the shape a printed line writes and a reader takes as its own arrow.
    return before.is_some_and(char::is_whitespace) && after.is_some_and(char::is_whitespace);
}

/// What:
///  Where the first `->` a text sets off with one whitespace character on each side begins,
///  or nothing when it sets none off that way.
/// Why:
///  A `uses` line sets its arrow off with whitespace,
///  and that whitespace may be a tab or a no-break space as well as a space,
///  so the arrow is found by the characters around it and never by a literal spelling.
///  A folder name in front of the arrow holds no arrow word (no line could name such a folder),
///  so the first `->` set off this way is the line's own arrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arrowBetweenWhitespace = (text) => {
///   const at = text.indexOf('->')
///   return at !== -1 && isWhitespace(text[at - 1]) && isWhitespace(text[at + 2]) ? at : undefined
/// }
/// ```
pub(crate) fn arrow_between_whitespace(text: &str) -> Option<usize> {
    for (index, _found) in text.match_indices(ARROW) {
        if set_off_by_whitespace(text, index) {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  How many `->` a text sets off with one whitespace character on each side.
/// Why:
///  A `file-uses` line holds one arrow only,
///  and a line of six words is either a doubled arrow or an extra word:
///  counting the arrows is what tells its reader which of the two mistakes it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arrowsSetOff = (text) => [...text.matchAll(/->/g)].filter((one) => setOff(text, one.index!)).length
/// ```
fn arrows_between_whitespace(text: &str) -> usize {
    // What: the number of arrows set off on both sides found so far.
    // Why: the caller compares it with one, so it counts rather than stops at the first.
    let mut count: usize = 0;
    for (index, _found) in text.match_indices(ARROW) {
        if set_off_by_whitespace(text, index) {
            count += 1;
        }
    }
    return count;
}

/// What:
///  The two names of a `uses` line,
///  each with the one whitespace character that sets it off from the arrow taken away.
/// Why:
///  The line is read from the two sides of its arrow,
///  and exactly one whitespace character on each side is the separator:
///  a second one beside a name belongs to that name,
///  because a folder name may start with one and a hand-aligned line is read rather than refused.
///  Neither side can be missing, because the arrow was found with whitespace on both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arrowSides = (text) => {
///   const at = arrowBetweenWhitespace(text)
///   return at === undefined ? undefined : [text.slice(0, at - 1), text.slice(at + 3)]
/// }
/// ```
fn arrow_sides(text: &str) -> Option<(&str, &str)> {
    // What: where the line's own arrow begins, or nothing when it sets none off with whitespace.
    // Why: the arrow is the one word a folder name in front of it cannot be mistaken for.
    let arrow: usize = arrow_between_whitespace(text)?;
    // What: the name before the arrow and the name after it.
    // Why: the whitespace the arrow was found with is the separator, and what stands beside it is the name.
    let before: &str = text[..arrow].strip_suffix(char::is_whitespace)?;
    let after: &str = text[arrow + ARROW.len()..].strip_prefix(char::is_whitespace)?;
    return Some((before, after));
}

/// What:
///  Whether a folder name, written into a `uses` line, would hold an arrow word of its own.
/// Why:
///  Such a line would not say which of its arrows is the line's own,
///  so its reader would take the name for another pair of folders.
///  The test is the line itself rather than a pattern over the name,
///  so the check cannot drift from what the reader reads:
///  the name is written where a printed line writes it, between the word and its arrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameHoldsAnArrowWord = (name) => arrowBetweenWhitespace(`uses ${name} `) !== undefined
/// ```
pub(crate) fn name_holds_an_arrow_word(name: &str) -> bool {
    return arrow_between_whitespace(format!("{FOLDER_USE_WORD} {name} ").as_str()).is_some();
}

/// What:
///  The two folders one `uses` line names.
/// Why:
///  The shape is four words,
///  and a folder name may hold a space,
///  so the words alone cannot say where one name ends:
///  the arrow the line holds is the one word a name in front of it cannot be mistaken for,
///  and the rule prints a key in exactly this shape,
///  so what a run prints is what the next run reads.
///  The four words are read when the line sets them off with one whitespace character each;
///  every other line is read from the two sides of its arrow,
///  with exactly the one whitespace character on each side taken off,
///  whatever that character is,
///  because a folder name may start with one and the space a line sets its words off with must never be read as part of a name or the other way round.
///  One rule therefore holds for both sides of the arrow and for the space after the word:
///  a second whitespace character beside a name is part of that name and not a separator.
///  No folder name a tree the rules accept can hold ends in whitespace,
///  so a line that leaves one before the arrow names no folder of any tree and is reported dead,
///  which costs its own line and not the rule's whole report.
///  A name that holds the arrow itself is refused before any check,
///  because the line would not say which folder the one used is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function useEnds(line: number, text: string): [string, string];
/// ```
fn use_ends(
    line: usize,
    text: &str,
) -> Result<(&str, &str), String> {
    // What: the two names the line sets off from its arrow, or nothing when it sets no arrow off with whitespace.
    // Why: one rule reads every spelling, so a tab or a no-break space beside a name is read as a space is.
    let Some((before_arrow, after_arrow)) = arrow_sides(text) else {
        return Err(format!(
            "Line {line} reads `{text}`. A use between two folders is written `uses <folder> {ARROW} <folder>`, with one whitespace character after the word `uses`, one before the arrow and one after it, and a folder name on each side."
        ));
    };
    // What: whether the name after the arrow holds an arrow word of its own.
    // Why: the reader takes the first arrow as the line's own, so a second one names a folder no tree holds.
    if arrow_between_whitespace(after_arrow).is_some() {
        return Err(format!(
            "Line {line} reads `{text}`. A use between two folders is written `uses <folder> {ARROW} <folder>`, and no folder name holds an arrow word: two `{ARROW}` are set off by whitespace."
        ));
    }
    // What: the line before the arrow, which begins with the word every `uses` line starts with.
    // Why: the caller dispatched on that word, so only the name and its separator can be missing here.
    let Some(after_the_word) = before_arrow.strip_prefix(FOLDER_USE_WORD) else {
        return Err(format!(
            "Line {line} reads `{text}`. A use between two folders is written `uses <folder> {ARROW} <folder>`."
        ));
    };
    // What: the name before the arrow, with the one whitespace character that sets it off the word taken away.
    // Why: what stands between the word and the arrow is the name, so no whitespace there means no name.
    let from: &str = after_the_word
        .strip_prefix(char::is_whitespace)
        .unwrap_or_default();
    // What: a line that sets its word and its arrow off with two whitespace characters, which leaves no name at all.
    // Why: the reader takes exactly one character as the separator, so what stands between them is the name, and nothing is no folder.
    if from.is_empty() {
        return Err(format!(
            "Line {line} reads `{text}`. No folder name stands before the arrow: a use between two folders is written `uses <folder> {ARROW} <folder>`."
        ));
    }
    return Ok((from, after_arrow));
}

/// What:
///  Read every line of a list from its text.
/// Why:
///  A blank line and a line that starts with `#` carry nothing;
///  every other line must read as one entry,
///  and no line may repeat another,
///  because a list that names one thing twice is not a list of distinct findings.
///  A repeat is one use however it is spelled,
///  so the two spellings a `..` walk allows are caught here,
///  where every listed path already stands in the spelling a key writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEntries(text: string, base: Path): AllowEntry[];
/// ```
pub fn parse_entries(
    text: &str,
    base: &Path,
) -> Result<Vec<AllowEntry>, String> {
    let mut entries: Vec<AllowEntry> = Vec::<AllowEntry>::new();
    let mut seen: BTreeSet<String> = BTreeSet::<String>::new();
    for (index, raw) in text.lines().enumerate() {
        let line: usize = index + 1;
        let trimmed: &str = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let entry: AllowEntry = entry_of(line, trimmed, base)?;
        if !seen.insert(entry.identity()) {
            return Err(format!(
                "Line {line} repeats the entry `{}` that an earlier line already holds.",
                entry.text
            ));
        }
        entries.push(entry);
    }
    return Ok(entries);
}
