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
        first_format_character,
        format_character_name,
        is_format_character,
        key_path,
        shown,
        spelled_unreadable,
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
            // What: the first character of the line that a reader cannot see, with its position.
            // Why: a line holding one may be the line the reader took for a key instead of the use it spells.
            let invisible: Option<(char, usize)> = first_format_character(entry.text.as_str());
            // What: whether the line would read as a use, a comment or a blank line once its format characters are gone.
            // Why: a key a rule really prints can hold a zero-width character, and telling its reader to take it out would be wrong.
            let reads_as_a_line: bool =
                reads_as_a_line_without_format_characters(entry.text.as_str());
            if let AllowKind::Key(_) = &entry.kind
                && let Some((letter, position)) = invisible
                && reads_as_a_line
            {
                // What: the plain name of that character, so its reader knows which one to take out.
                // Why: two of them read the same to a person, and the edits differ.
                let name: &str =
                    format_character_name(letter).unwrap_or("a character a reader cannot see");
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} holds {name} at character {position} of the entry it read, and the reader does not treat that character as whitespace. The line's first word is therefore not `{FOLDER_USE_WORD}`, so the line was read as one finding's key, and no finding of the rule prints that key, so the entry tolerates nothing any more. Take that character out of the line, and write a whitespace character where it stood if it stood where one belongs.",
                        shown(list.source(), base)
                    ),
                    String::from(
                        "Take that character out of the line: it is not whitespace, so nothing on the line can set it off as a separator, and the reader makes it part of the word beside it.",
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
            // What: the first character of the line that a reader cannot see, when it holds one.
            // Why: such a character is no whitespace, so the name it joined is no name on disk, and taking it out is the edit that makes the line read.
            let invisible: Option<char> = entry
                .text
                .chars()
                .find(|letter| return is_format_character(*letter));
            // What: the sentence about the one edit, which names the character kept and the one taken off.
            // Why: a reader told only that a doubled space is part of a name has to work out which of the two to delete.
            let mut help: String = if kept.is_empty() {
                String::from(
                    "Write the folder name the rule prints, or delete the line if the use is gone: a doubled space, or a tab beside a name, is read as part of that name.",
                )
            } else {
                format!(
                    "Write the folder name the rule prints, or delete the line if the use is gone. One whitespace character sets each word off: the reader took the character nearest the word `uses` or the arrow as the separator and read the character beside the name as part of it. It kept {}; remove that character and leave the one nearest the word or the arrow in place.",
                    kept.join(", and ")
                )
            };
            if let Some(letter) = invisible {
                let name: &str =
                    format_character_name(letter).unwrap_or("a character a reader cannot see");
                help.push_str(
                    format!(
                        " The name above also holds {name} where a reader cannot see it: take that character out."
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
///  Whether a line would read as a use, a comment or a blank line once its characters a reader cannot see are gone.
/// Why:
///  A byte order mark before the first word is the common way such a character hides a real line,
///  while a key a rule prints can hold a zero-width character as content,
///  and only the first reader needs the advice to take it out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readsAsALine = (text) => {
///   const without = [...text].filter((letter) => !isFormat(letter)).join('').trim()
///   return without === '' || without.startsWith('#') || without.startsWith('uses') || without.startsWith('file-uses')
/// }
/// ```
fn reads_as_a_line_without_format_characters(text: &str) -> bool {
    // What: the line with every character a reader cannot see taken out.
    // Why: what the line spells is what stands beside those characters, and that is what the reader compares.
    let without: String = text
        .chars()
        .filter(|letter| return !is_format_character(*letter))
        .collect();
    // What: the part of that line a reader would see, without the whitespace the list reader trims.
    // Why: a line holding only a byte order mark and spaces would read as blank, which carries nothing.
    let trimmed: &str = without.trim();
    return trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with(FOLDER_USE_WORD)
        || trimmed.starts_with(FILE_USE_WORD);
}

/// What:
///  Each whitespace character a dead `uses` line read as part of a name, with where it stands.
/// Why:
///  One whitespace character on each side of a name is the separator and a second belongs to the name,
///  so a reader whose line is dead has to remove the second one and leave the first,
///  and a help that named only the character would leave them to work out which of two identical spaces it meant.
///  A `file-uses` line has no such character, because its folder is one word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptCharacters = (entry) => {
///   if (entry.kind !== 'folder-use') return []
///   const kept: string[] = []
///   if (/\s/.test(entry.from[0])) kept.push(`\`\\u{...}\` after the word \`uses\`, before \`${entry.from.trim()}\``)
///   if (/\s/.test(entry.from.at(-1))) kept.push(`\`\\u{...}\` after \`${entry.from.trim()}\`, before \`->\``)
///   if (/\s/.test(entry.to[0])) kept.push(`\`\\u{...}\` after \`->\`, before \`${entry.to.trim()}\``)
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
    // What: the characters kept so far, each written with its place.
    // Why: a line can hold one on each side of the arrow, and each needs its own edit.
    let mut kept: Vec<String> = Vec::<String>::new();
    if let Some(letter) = from_text.chars().next()
        && letter.is_whitespace()
    {
        kept.push(format!(
            "`{}` after the word `uses`, before `{}`",
            spelled_unreadable(letter.to_string().as_str()),
            from_text.trim()
        ));
    }
    if let Some(letter) = from_text.chars().next_back()
        && letter.is_whitespace()
    {
        kept.push(format!(
            "`{}` after `{}`, before `->`",
            spelled_unreadable(letter.to_string().as_str()),
            from_text.trim()
        ));
    }
    if let Some(letter) = to_text.chars().next()
        && letter.is_whitespace()
    {
        kept.push(format!(
            "`{}` after `->`, before `{}`",
            spelled_unreadable(letter.to_string().as_str()),
            to_text.trim()
        ));
    }
    return kept;
}
