//! What:
//!  The findings for the entries of a list that tolerated nothing.
//! Why:
//!  A dead line is the sign the list has not shrunk after the work that made it dead,
//!  and the run must say so on the line itself.
//!  A `uses` or `file-uses` line that names a folder no source folder of the package is called
//!  says which folder it read and that no source folder has that name:
//!  that is what a hand-aligned or mistyped line looks like,
//!  and it is the only thing that tells its reader to re-space the line rather than delete it.
//!  The sentence speaks of source folders rather than of folders,
//!  because the set it is judged against is the module tree's:
//!  a folder on disk that holds no module file (a `docs` folder holding a `README.md`)
//!  is no source folder, and telling its reader that the package holds no such folder would be false.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function deadEntryFindings(list: AllowList, used: Set<number>, folders: Set<Path>, base: Path): LayoutFinding[];
//! ```

/// Import the list and the entry shapes it holds,
///  the finding shape,
///  the spelling of the names in it,
///  and the characters a reader cannot see.
use crate::{
    layout_allow_list::{
        AllowEntry,
        AllowKind,
        AllowList,
    },
    layout_finding::{
        LayoutFinding,
        first_unseen_character,
        key_path,
        prints_as_nothing,
        shown,
        spelled_unreadable,
        unseen_character_phrase,
    },
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The findings for the entries that tolerated nothing.
/// Why:
///  A dead line is the sign the list has not shrunk after the work that made it dead,
///  and the run must say so on the line itself.
///  A `uses` or `file-uses` line that names a folder no source folder of the package is called
///  says which folder it read and that no source folder has that name:
///  that is what a hand-aligned or mistyped line looks like,
///  and it is the only thing that tells its reader to re-space the line rather than delete it.
///  The sentence speaks of source folders rather than of folders,
///  because the set it is judged against is the module tree's:
///  a folder on disk that holds no module file is no source folder,
///  and telling its reader that the package holds no such folder would be false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadEntryFindings(list: AllowList, used: Set<number>, folders: Set<Path>, base: Path): LayoutFinding[];
/// ```
pub fn dead_entry_findings(
    list: &AllowList,
    used: &BTreeSet<usize>,
    folders: &BTreeSet<PathBuf>,
    module_files: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    // What: the folders of the package in the spelling a list line writes them.
    // Why: an entry names its folder that way, so the two spellings must meet before a name can be called absent.
    let mut known: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for folder in folders {
        known.insert(PathBuf::from(key_path(folder, base)));
    }
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        if used.contains(&index) {
            continue;
        }
        // What: the folders this entry names that no folder of the package is called.
        // Why: a name that is not there is a misspelling or a hand-aligned line, and naming it is what tells its reader apart from a use that is simply gone.
        let mut absent: Vec<String> = Vec::<String>::new();
        match &entry.kind {
            AllowKind::FolderUse {
                from,
                to,
            } => {
                for path in [from, to] {
                    if known.contains(path) {
                        continue;
                    }
                    // What: the folder as the message spells it, with every character a reader cannot see written out.
                    // Why: a tab or a no-break space beside a name is invisible, so `src/a` would be shown for a folder the package does hold.
                    absent.push(format!(
                        "`{}`",
                        spelled_unreadable(path.display().to_string().as_str())
                    ));
                }
            }
            AllowKind::FileUse {
                folder,
                ..
            } => {
                if !known.contains(folder) {
                    absent.push(format!(
                        "`{}`",
                        spelled_unreadable(folder.display().to_string().as_str())
                    ));
                }
            }
            AllowKind::Key(_) => {}
        }
        let (message, help): (String, String) = if absent.is_empty() {
            // What: the line as the message quotes it, with every character a reader cannot see spelled.
            // Why: a byte order mark or a zero-width space prints as nothing, so the quoted line would look like a line that reads.
            let spelled_text: String = spelled_unreadable(entry.text.as_str());
            // What: what the line would be with each unseen character taken out, or with each one written as a space.
            // Why: a character standing where a separator belongs makes a use line only once a space stands in its place, and one standing inside a word makes none either way.
            let reading: Option<String> = match &entry.kind {
                AllowKind::Key(_) => what_the_line_would_read_as(entry.text.as_str(), false)
                    .or_else(|| return what_the_line_would_read_as(entry.text.as_str(), true)),
                _ => None,
            };
            // What: the character the edit would be about, what the edit changes, and its place among the entry's characters.
            // Why: taking a character out is the edit only where the edited name is one the tree does not hold,
            // and a line that spells a real module file's own name is dead for another reason.
            let edit: Option<(DeadEdit, char, usize)> = if reading.is_some() {
                first_unseen_character(entry.text.as_str())
                    .map(|(letter, position)| return (DeadEdit::Key, letter, position))
            } else {
                file_name_advice(entry, module_files, base)
                    .map(|(letter, position)| return (DeadEdit::FileName, letter, position))
            };
            if let Some((what, letter, position)) = edit {
                // What: the plain name of that character, or one that still names it by its code point.
                // Why: two of them read the same to a person, and the edits differ.
                let name: String = unseen_character_phrase(letter);
                // What: where that character stands in the line, counted in characters from the line's first one.
                // Why: the entry the reader keeps starts past the indent, and the quote shows the character spelled.
                let place: usize = entry.column + position - 1;
                // What: the sentence about what the line would read as, which is empty when the edit is about a file name.
                // Why: only a key line earns it, and only when its edited text really reads as a line of the list.
                let sentence: &str = reading.as_deref().unwrap_or("");
                // What: what the line was read as before its edit, and the help that says how to make it.
                // Why: a key line was read as a key, and a file-use line as its shape with the name the character made,
                // and the two edits differ because one removes a separator's stand-in and the other a name's extra character.
                let (read_as, help): (&str, String) = match what {
                    DeadEdit::Key => (
                        "the reader took it as part of the word beside it and the line was read as one finding's key. No finding of the rule prints that key, so the entry tolerates nothing any more.",
                        String::from(
                            "Take that character out of the line: it is not whitespace, so nothing on the line can set it off as a separator, and the reader makes it part of the word beside it.",
                        ),
                    ),
                    DeadEdit::FileName => (
                        "A file name is read exactly as the line writes it, and no module of this folder is called the name it holds, so the entry tolerates nothing any more.",
                        String::from(
                            "Take that character out of the line, then write the file name the rule prints: a character a reader cannot see makes a file name of its own.",
                        ),
                    ),
                };
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} holds {name} at character {place} of the line, where a reader cannot see it: {read_as} {sentence} Take that character out of the line, and write a whitespace character where it stood if it stood where one belongs.",
                        shown(list.source(), base)
                    ),
                    help,
                )
            } else {
                // What: the character a line that needs no edit still holds, named with its place.
                // Why: the quote spells it, and a reader who cannot see it still has to find which character the spelling stands for.
                let note: String = match first_unseen_character(entry.text.as_str()) {
                    Some((letter, position)) => format!(
                        " The line also holds {} at character {} of the line, where a reader cannot see it.",
                        unseen_character_phrase(letter),
                        entry.column + position - 1
                    ),
                    None => String::new(),
                };
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} tolerates nothing any more. Delete the line so the list keeps shrinking.{note}",
                        shown(list.source(), base)
                    ),
                    String::from(
                        "Delete this line. The list holds today's findings only, and a line that matches none of them must not wait for a later round.",
                    ),
                )
            }
        } else {
            // What: the word that makes the sentence agree with how many folders are absent.
            // Why: most dead lines name one absent folder, and only a line never read names two.
            let names: &str = if absent.len() == 1 {
                "the folder"
            } else {
                "the folders"
            };
            // What: each whitespace character the line read as part of a name, with where it stands.
            // Why: the line already reads as the shape it spells, so its reader has to know which of two characters to remove.
            let kept: Vec<String> = the_characters_a_dead_line_kept(entry);
            // What: the line as the message quotes it, with every character a reader cannot see spelled.
            // Why: a name holding a zero-width space prints as a name the package does hold, and the reader would look for it in vain.
            let spelled_text: String = spelled_unreadable(entry.text.as_str());
            // What: the sentence about the one edit, which names the character kept and the one taken off.
            // Why: a reader told only that a doubled space is part of a name has to work out which of the two to delete.
            let mut help: String = if kept.is_empty() {
                String::from(
                    "Write the folder name the rule prints, or delete the line if the use is gone: a doubled space, or a tab beside a name, is read as part of that name.",
                )
            } else {
                format!(
                    "Write the folder name the rule prints, or delete the line if the use is gone. One whitespace character sets each word off: the reader took the character nearest the word `uses` or the arrow as the separator and read every character beside the name as part of it. It kept {}; remove every character listed there and leave the one nearest the word or the arrow in place.",
                    kept.join(", and ")
                )
            };
            if let Some((letter, position)) = first_unseen_character(entry.text.as_str()) {
                // What: the plain name of that character, or one that still names it by its code point.
                // Why: the name above is spelled, and its reader still has to find the character inside it.
                let name: String = unseen_character_phrase(letter);
                let place: usize = entry.column + position - 1;
                help.push_str(
                    format!(
                        " The name above also holds {name} at character {place} of the line, where a reader cannot see it: take that character out."
                    )
                    .as_str(),
                );
            }
            (
                format!(
                    "The allow-list entry `{spelled_text}` in {} names {} {}. No source folder of this package has that name, so the entry tolerates nothing any more. Check the line's spelling and the spacing between its words.",
                    shown(list.source(), base),
                    names,
                    absent.join(" and ")
                ),
                help,
            )
        };
        findings.push(LayoutFinding {
            file: list.source().to_path_buf(),
            line: entry.line,
            message,
            help: Some(help),
            key: String::new(),
        });
    }
    return findings;
}

/// What:
///  What the edit a dead line needs changes: the finding's key, or a file name the folder's modules do not hold.
/// Why:
///  The two edits differ:
///  a character standing where a separator belongs is taken out and a whitespace character written in its place,
///  while a character inside a file name is taken out and the name the rule prints is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeadEdit = 'key' | 'file-name';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeadEdit {
    /// The line was read as one finding's key.
    Key,
    /// The line was read as its shape, with a file name no module of the folder holds.
    FileName,
}

/// What:
///  The first character of a dead `file-uses` line's file names that a reader cannot see,
///  with its place among the entry's characters,
///  when no module of the folder is called that file name.
/// Why:
///  A line that spells a real module file's own name is dead because its use is gone,
///  and the name the finding prints is the one the line already holds:
///  taking the character out would name a file the folder does not have.
///  The folder's own name is no file name, so a character there earns no such advice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileNameAdvice = (entry, moduleFiles, base) => {
///   const { folder, from, to } = entry.kind
///   const words = entry.text.split(/\s+/u)
///   const starts = wordStarts(entry.text)
///   for (const [at, name] of [[2, from], [4, to]]) {
///     const first = [...name].findIndex(printsAsNothing)
///     if (first === -1 || words[at] !== name) continue
///     if (moduleFiles.has(join(base, folder, name))) continue
///     return [name[first], starts[at] + first + 1]
///   }
///   return undefined
/// }
/// ```
fn file_name_advice(
    entry: &AllowEntry,
    module_files: &BTreeSet<PathBuf>,
    base: &Path,
) -> Option<(char, usize)> {
    let AllowKind::FileUse {
        folder,
        from,
        to,
    } = &entry.kind
    else {
        return None;
    };
    // What: the line's words, so a name is answered only where the shape puts it.
    // Why: a file name that happens to appear inside the folder's text is no file word of this line.
    let words: Vec<&str> = entry.text.split_whitespace().collect::<Vec<&str>>();
    // What: where each word begins, counted in characters from the line's first one.
    // Why: the message gives a place in the line, and a name's own position is not that place.
    let starts: Vec<usize> = word_starts(entry.text.as_str());
    for (index, name) in [(2_usize, from), (4_usize, to)] {
        // What: the first character of this file name a reader cannot see, with its place inside the name.
        // Why: a file name that holds none needs no edit.
        let Some((letter, position)) = first_unseen_character(name.as_str()) else {
            continue;
        };
        if words.get(index) != Some(&name.as_str()) {
            continue;
        }
        // What: the path this name would have in the folder, in the spelling the walk gives it.
        // Why: a module the folder holds is a name the line already spells, and its dead line is deleted.
        let path: PathBuf = base.join(folder).join(name);
        if module_files.contains(&path) {
            continue;
        }
        let Some(start) = starts.get(index) else {
            continue;
        };
        return Some((letter, start + position));
    }
    return None;
}

/// What:
///  Where each whitespace-separated word of a text begins, counted in characters from the text's first one.
/// Why:
///  A place in a line is counted from its first character,
///  and a name's own position inside itself is not that place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordStarts = (text) => {
///   const starts = []
///   let afterASeparator = true
///   for (const [at, letter] of [...text].entries()) {
///     if (/\s/u.test(letter)) { afterASeparator = true; continue }
///     if (afterASeparator) { starts.push(at); afterASeparator = false }
///   }
///   return starts
/// }
/// ```
fn word_starts(text: &str) -> Vec<usize> {
    // What: where each word begins so far.
    // Why: the caller asks for one word's place by its index.
    let mut starts: Vec<usize> = Vec::<usize>::new();
    // What: whether the character before this one was a separator, so this one begins a word.
    // Why: a word begins at the line's first character and after any whitespace run.
    let mut after_a_separator: bool = true;
    for (index, letter) in text.chars().enumerate() {
        if letter.is_whitespace() {
            after_a_separator = true;
            continue;
        }
        if after_a_separator {
            starts.push(index);
            after_a_separator = false;
        }
    }
    return starts;
}

/// What:
///  What the line would be once each character a reader cannot see is taken out,
///  or replaced by a space,
///  as a sentence a dead line's reader can act on,
///  or nothing when it would be no line of the list.
/// Why:
///  The sentence that takes such a character out has to say what the line becomes,
///  and the word it names must be the line's own:
///  a `file-uses` line told that its first word is not `uses` is told about a word the line never held.
///  Replacing the character by a space answers the case the removal cannot:
///  a mark written where the separator belongs leaves one word when it is taken out alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whatItWouldReadAs = (text, asSpace) => {
///   const trimmed = [...text].map((one) => printsAsNothing(one) ? (asSpace ? ' ' : '') : one).join('').trim()
///   if (trimmed === '') return 'the line carries nothing.'
///   if (trimmed.startsWith('#')) return 'the line is a comment.'
///   const first = trimmed.split(/\s/u)[0]
///   const edit = asSpace ? 'With a whitespace character where that character stands' : 'With that character out'
///   return first === 'uses' || first === 'file-uses'
///     ? `${edit}, the line reads as a \`${first}\` line.`
///     : undefined
/// }
/// ```
fn what_the_line_would_read_as(
    text: &str,
    as_a_space: bool,
) -> Option<String> {
    // What: the line with each character a reader cannot see taken out or written as a space.
    // Why: the sentence is about the line the reader would then read, not about the one it read.
    let without: String = text
        .chars()
        .filter_map(|letter| {
            return match (prints_as_nothing(letter), as_a_space) {
                (false, _) => Some(letter),
                (true, true) => Some(' '),
                (true, false) => None,
            };
        })
        .collect();
    // What: the part of that line a reader would see, without the whitespace the list reader trims.
    // Why: a line holding only such a character and spaces would read as blank, which carries nothing.
    let trimmed: &str = without.trim();
    // What: which edit the sentence offers, which is the one this reading applied.
    // Why: a reader told to take the character out of a separator's place would leave one word.
    let edit: &str = if as_a_space {
        "With a whitespace character where that character stands"
    } else {
        "With that character out"
    };
    if trimmed.is_empty() {
        return Some(format!("{edit}, the line carries nothing."));
    }
    if trimmed.starts_with('#') {
        return Some(format!("{edit}, the line is a comment."));
    }
    // What: the word the edited line begins with, when that line reads as one of the list's use shapes.
    // Why: the reader that decides what a line is decides the sentence too, so the sentence says only what reads.
    return match crate::layout_allow_entries::shape_of_a_line(trimmed) {
        Some(word) => Some(format!("{edit}, the line reads as a `{word}` line.")),
        None => None,
    };
}

/// What:
///  Every whitespace character a dead `uses` line read as part of a name, with where it stands,
///  one phrase per run of them.
/// Why:
///  One whitespace character on each side of a name is the separator and every character beside it belongs to the name,
///  so a reader whose line is dead has to remove that run and leave one separator,
///  and a help that named only the first character would leave a gap of three characters dead after one edit,
///  with the same help again.
///  A `file-uses` line has no such character, because its folder is one word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptCharacters = (entry) => {
///   if (entry.kind !== 'folder-use') return []
///   const kept: string[] = []
///   const leading = /^\s+/u.exec(entry.from)?.[0]
///   const trailing = /\s+$/u.exec(entry.from)?.[0]
///   const secondLeading = /^\s+/u.exec(entry.to)?.[0]
///   if (leading) kept.push(run(leading, `after the word \`uses\`, before \`${entry.from.trim()}\``))
///   if (trailing) kept.push(run(trailing, `after \`${entry.from.trim()}\`, before \`->\``))
///   if (secondLeading) kept.push(run(secondLeading, `after \`->\`, before \`${entry.to.trim()}\``))
///   return kept
/// }
/// ```
fn the_characters_a_dead_line_kept(entry: &AllowEntry) -> Vec<String> {
    // What: the two names of a folder use, or nothing when the line is of another shape.
    // Why: only a `uses` line sets two folder names off with whitespace.
    let AllowKind::FolderUse {
        from,
        to,
    } = &entry.kind
    else {
        return Vec::new();
    };
    // What: the two names as the reader read them, whitespace included.
    // Why: the whitespace at either end is the character the line kept, and its place says which one to remove.
    let from_text: String = from.display().to_string();
    let to_text: String = to.display().to_string();
    // What: the part of each name a reader sees, which is the folder the line was trying to name.
    // Why: the sentence names that folder so its reader knows which of two identical spaces the phrase is about.
    let from_name: &str = from_text.trim();
    let to_name: &str = to_text.trim();
    // What: the run of whitespace at each place a separator belongs, and the phrase that names it.
    // Why: a line can keep a run on each side of the arrow, and each run needs its own edit.
    let mut kept: Vec<String> = Vec::<String>::new();
    for (run, place) in [
        (
            leading_whitespace(from_text.as_str()),
            format!("after the word `uses`, before `{from_name}`"),
        ),
        (
            trailing_whitespace(from_text.as_str()),
            format!("after `{from_name}`, before `->`"),
        ),
        (
            leading_whitespace(to_text.as_str()),
            format!("after `->`, before `{to_name}`"),
        ),
    ] {
        if let Some(phrase) = kept_run(run, place.as_str()) {
            kept.push(phrase);
        }
    }
    return kept;
}

/// What:
///  The run of whitespace a text begins with, or nothing when it begins with another character.
/// Why:
///  A name the reader kept begins with the characters a doubled gap left, and the sentence names all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leading = (text) => /^\s+/u.exec(text)?.[0]
/// ```
fn leading_whitespace(text: &str) -> &str {
    return &text[..text.len() - text.trim_start().len()];
}

/// What:
///  The run of whitespace a text ends with, or nothing when it ends with another character.
/// Why:
///  A name the reader kept can end with the characters a doubled gap left before the arrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trailing = (text) => /\s+$/u.exec(text)?.[0]
/// ```
fn trailing_whitespace(text: &str) -> &str {
    return &text[text.trim_end().len()..];
}

/// What:
///  The phrase that names one run of whitespace a dead line kept and where it stands,
///  or nothing when the run is empty.
/// Why:
///  A gap of one character needs one removal and a gap of three needs two,
///  so the sentence counts the characters and spells each of them:
///  a reader told "remove that character" leaves a gap of three dead and reads the same help again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptRun = (run, place) => {
///   if (run === '') return undefined
///   const spelled = [...run].map((one) => `\`${spelled(one)}\``)
///   return spelled.length === 1
///     ? `${spelled[0]} ${place}`
///     : `${spelled.length} whitespace characters (${spelled.join(' and ')}) ${place}`
/// }
/// ```
fn kept_run(
    run: &str,
    place: &str,
) -> Option<String> {
    if run.is_empty() {
        return None;
    }
    // What: each character of the run, spelled so a tab and a no-break space read as what they are.
    // Why: the reader has to find these characters in the line, and half of them print as nothing.
    let spelled: Vec<String> = run
        .chars()
        .map(|letter| return format!("`{}`", spelled_unreadable(letter.to_string().as_str())))
        .collect();
    if let [one] = spelled.as_slice() {
        return Some(format!("{one} {place}"));
    }
    return Some(format!(
        "{} whitespace characters ({}) {place}",
        spelled.len(),
        spelled.join(" and ")
    ));
}
