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
///  the characters a reader cannot see,
///  and the two words a use line starts with.
use crate::{
    layout_allow_entries::{
        FILE_USE_WORD,
        FOLDER_USE_WORD,
    },
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
        unseen_character_name,
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
            // What: the first character of the line that a reader cannot see, with its place among the entry's characters.
            // Why: a line holding one may be the line the reader took for a key instead of the use it spells, and the place is what its reader has to find.
            let invisible: Option<(char, usize)> = first_unseen_character(entry.text.as_str());
            // What: the plain name of that character, so its reader knows which one it is.
            // Why: two of them read the same to a person, and the edits differ.
            let name: &str = invisible.map_or("", |(letter, _)| {
                return unseen_character_name(letter).unwrap_or("a character a reader cannot see");
            });
            // What: where that character stands in the line, counted from the line's first character.
            // Why: a person counting in an editor counts from there, and the entry the reader keeps starts past the indent.
            let place: usize = entry.column
                + invisible.map_or(0, |(_, position)| {
                    return position;
                })
                - 1;
            // What: what the line would be with each unseen character taken out, or with each one written as a space.
            // Why: a character standing where a separator belongs makes a use line only once a space stands in its place, and one standing inside a word makes none either way.
            let advice: Option<String> = if let AllowKind::Key(_) = &entry.kind {
                what_the_line_would_read_as(entry.text.as_str(), false)
                    .or_else(|| return what_the_line_would_read_as(entry.text.as_str(), true))
            } else {
                None
            };
            if let Some(sentence) = advice.as_deref()
                && invisible.is_some()
            {
                // What: the edit the character needs, which the sentence chose by the reading that found a line of the list.
                // Why: a separator written as a mark a reader cannot see looks like no separator at all, and only a space in its place makes the line read.
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} holds {name} at character {place} of the line, where a reader cannot see it, so the reader took it as part of the word beside it and the line was read as one finding's key. No finding of the rule prints that key, so the entry tolerates nothing any more. {sentence} Take that character out of the line, and write a whitespace character where it stood if it stood where one belongs.",
                        shown(list.source(), base)
                    ),
                    String::from(
                        "Take that character out of the line: it is not whitespace, so nothing on the line can set it off as a separator, and the reader makes it part of the word beside it.",
                    ),
                )
            } else if let AllowKind::FileUse {
                ..
            } = &entry.kind
                && invisible.is_some()
            {
                // What: the same advice for a line the reader took as its shape, whose file name therefore holds the character.
                // Why: a file name is read exactly as the line writes it, so the name the use belongs to is the name with the character gone.
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} holds {name} at character {place} of the line, where a reader cannot see it. A file name is read exactly as the line writes it, so the name this line holds is not the name the finding prints, and the entry tolerates nothing any more. Take that character out of the line.",
                        shown(list.source(), base)
                    ),
                    String::from(
                        "Take that character out of the line, then write the file name the rule prints: a character a reader cannot see makes a file name of its own.",
                    ),
                )
            } else {
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} tolerates nothing any more. Delete the line so the list keeps shrinking.",
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
                let name: &str =
                    unseen_character_name(letter).unwrap_or("a character a reader cannot see");
                // What: where that character stands in the line, counted from its first character.
                // Why: the name above is spelled, and its reader still has to find the character inside it.
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
    return match trimmed.split_whitespace().next() {
        Some(first) if first == FOLDER_USE_WORD || first == FILE_USE_WORD => {
            Some(format!("{edit}, the line reads as a `{first}` line."))
        }
        _ => None,
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
