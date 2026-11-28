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
///  and the reading that removes the `.` segments and the folders a `..` reads back over.
use crate::{
    layout_allow_entries::PackageFolder,
    layout_allow_list::{
        AllowEntry,
        AllowKind,
        AllowList,
    },
    layout_finding::{
        LayoutFinding,
        characters_beside_the_first,
        first_unseen_character,
        key_path,
        prints_as_nothing,
        shown,
        spelled_unreadable,
        the_characters_as_spaces,
        the_characters_named,
        unseen_character_count,
        unseen_character_phrase,
    },
    markdown_lfs_target::lexical_normal,
};
/// Import the map that counts the lines one edit gives, ordered collections and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  What a dead key line's own edit would leave,
///  when the reader reads that text as a line of a use shape.
/// Why:
///  The sentence about the edit names what the line would read,
///  and the line the edit would leave is also what tells two dead lines whose edits give one line apart from two whose edits give two:
///  both questions read the one reading this line's edit has,
///  and reading it twice cost the reader's second pass over the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { sentence: string; identity?: string };
/// ```
struct Reading {
    /// The sentence that says what the edit leaves, such as `With that character out, the line reads as a `uses` line.`.
    sentence: String,
    /// The edit itself, named so a sentence beside it can say what the edit does without repeating the sentence above it.
    edit: String,
    /// The line the edit leaves, in the words a list writes an entry, when it leaves a line of a use shape.
    identity: Option<String>,
}

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
///  a folder on disk that holds no module file (a `docs` folder holding a `README.md`)
///  is no source folder,
///  and telling its reader that the package holds no such folder would be false.
///  Every dead key line's edit is read once, before any message is written,
///  because the sentence about the edit and the check that no other dead line's edit gives the same line are one reading:
///  reading it once per line per question cost the square of the list's length,
///  and a scan of the list for a repeated line cost the same square again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadEntryFindings(list: AllowList, used: Set<number>, folders: Set<Path>, moduleFiles: Set<Path>, base: Path): LayoutFinding[];
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
    // What: the package folder the lines are read against, listed at most once however many lines ask about its names.
    // Why: a dead key line's edited text is read as any line is, and one such text asks how long the longest name the folder holds is.
    let package: PackageFolder<'_> = PackageFolder::at(base);
    // What: what each dead key line's own edit would leave, one entry per line of the list.
    // Why: the message pass and the collision check both read it, and reading it twice per line is work the square of the list does not need.
    let mut readings: Vec<Option<Reading>> = Vec::<Option<Reading>>::new();
    readings.resize_with(list.entries().len(), || return None);
    // What: the dead key lines whose own edit gives each line, one list per line the edits give.
    // Why: a list refuses two lines that hold one thing, so two edits that give one line cannot both be written,
    // and a sentence that names the edit of one such line has to say which others give the same line.
    let mut edits: BTreeMap<String, Vec<usize>> = BTreeMap::<String, Vec<usize>>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        // What: only a line no finding used, of the shape that is one finding's key, has an edit a sentence can describe.
        // Why: a `uses` or `file-uses` line that tolerated nothing names a use that is gone, which is no edit of its text.
        if used.contains(&index) || !matches!(entry.kind, AllowKind::Key(_)) {
            continue;
        }
        // What: the text this line's edit leaves, by the removal of each such character first and by a space in its place second.
        // Why: the sentence names the edit a person would make first, and a mark where a separator belongs reads only once a space stands there.
        let edit: Option<Reading> = what_the_line_would_read_as(
            entry.text.as_str(),
            false,
            &package,
            list,
            index,
        )
        .or_else(|| {
            return what_the_line_would_read_as(entry.text.as_str(), true, &package, list, index);
        });
        let Some(reading) = edit else {
            continue;
        };
        if let Some(identity) = &reading.identity {
            edits
                .entry(identity.clone())
                .or_default()
                .push(index);
        }
        readings[index] = Some(reading);
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
            // What: this line's own reading, while no other dead line's edit gives the same line.
            // Why: a text another line's edit also gives is refused as the repeat it is once both are written, so the sentence may not promise the line reads.
            let reading: Option<&Reading> = readings[index].as_ref().filter(|one| {
                return one.identity.as_ref().is_none_or(|identity| {
                    return edits
                        .get(identity)
                        .is_none_or(|lines| return lines.len() == 1);
                });
            });
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
                // What: every character of the line the edit this sentence names takes out, and how many names it changes.
                // Why: a key's edit takes out every such character of the line,
                // and a file name's takes out those of the file names the advice changes and of no other name:
                // a character of a real folder's name is part of that folder's name, and a character of a real module's own name is part of the name the rule prints.
                let (unseen, names): (usize, usize) = match what {
                    DeadEdit::Key => (unseen_character_count(entry.text.as_str()), 0),
                    DeadEdit::FileName => {
                        unseen_characters_the_file_name_edit_changes(entry, module_files, base)
                    }
                };
                // What: the characters this line holds beside the one the sentence names, when it holds more than one.
                // Why: a reader told of one character where two stand looks for one and leaves the other, and the next run names it again.
                let more: String = characters_beside_the_first(unseen);
                // What: what the line was read as before its edit, the sentence about what the edit leaves, and the help that says how to make it.
                // Why: a key line was read as a key, and a file-use line as its shape with the name the character made,
                // and the two edits differ because one removes a separator's stand-in and the other a name's extra character.
                let (read_as, sentence, edit_sentence, help): (&str, &str, String, String) =
                    match what {
                        DeadEdit::Key => (
                            "the reader took it as part of the word beside it and the line was read as one finding's key; no finding of the rule prints that key, so the entry tolerates nothing any more.",
                            reading
                                .and_then(|one| return Some(one.sentence.as_str()))
                                .unwrap_or_default(),
                            take_out_what_a_reader_cannot_see(unseen),
                            take_out_what_a_reader_cannot_see(unseen),
                        ),
                        DeadEdit::FileName => (
                            "a file name is read exactly as the line writes it, and no module of this folder is called the name it holds, so the entry tolerates nothing any more.",
                            "",
                            format!(
                                "Take {} out of the line, then write the {} the rule prints; the entry is live again only if the use still exists.",
                                the_characters_named(unseen),
                                file_names_the_rule_prints(names)
                            ),
                            format!(
                                "Take {} out of the line, then write the {} the rule prints: a character a reader cannot see makes a file name of its own.",
                                the_characters_named(unseen),
                                file_names_the_rule_prints(names)
                            ),
                        ),
                    };
                // What: the space and the sentence about the reading, which a file name's edit does not have.
                // Why: a sentence run into the one before it is no sentence: `any more.With` is what a missing space prints.
                let what_the_edit_leaves: String = if sentence.is_empty() {
                    String::new()
                } else {
                    format!(" {sentence}")
                };
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} holds {name} at character {place} of the line, where a reader cannot see it{more}: {read_as}{what_the_edit_leaves} {edit_sentence}",
                        shown(list.source(), base)
                    ),
                    help,
                )
            } else {
                // What: the character a line that needs no edit still holds, with its place and how many stand beside it.
                // Why: the quote spells it, and a reader who cannot see it still has to find which character the spelling stands for.
                let note: String = match first_unseen_character(entry.text.as_str()) {
                    Some((letter, position)) => format!(
                        " The line also holds {} at character {} of the line, where a reader cannot see it{}.",
                        unseen_character_phrase(letter),
                        entry.column + position - 1,
                        characters_beside_the_first(unseen_character_count(entry.text.as_str()))
                    ),
                    None => String::new(),
                };
                // What: the other dead key lines whose own edit gives the same line as this one's, and the edit they share.
                // Why: following each line's own edit is refused as a repeat, so the sentence has to say that one of them may be kept
                // and that the edit is made there; deleting every one of them leaves none, and the next run says nothing of the one that could have stayed.
                let shared: String = readings[index]
                    .as_ref()
                    .and_then(|one| return one.identity.as_ref())
                    .and_then(|identity| return edits.get(identity))
                    .map(|lines| {
                        return lines
                            .iter()
                            .copied()
                            .filter(|other| return *other != index)
                            .collect::<Vec<usize>>();
                    })
                    .filter(|others| return !others.is_empty())
                    .and_then(|others| {
                        let shared_reading: &Reading = readings[index].as_ref()?;
                        let listed: Vec<usize> = others
                            .iter()
                            .map(|other| return list.entries()[*other].line)
                            .collect();
                        return Some(format!(
                            " The line's own edit, {}, gives the same line as {}: making that edit on one of them and deleting the rest keeps one of them.",
                            shared_reading.edit,
                            lines_listed(&listed)
                        ));
                    })
                    .unwrap_or_default();
                (
                    format!(
                        "The allow-list entry `{spelled_text}` in {} tolerates nothing any more. Delete the line so the list keeps shrinking.{note}{shared}",
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
                // What: how many such characters the line holds, and the clause that says so.
                // Why: a help that named the first left the second for the next run to name.
                let count: usize = unseen_character_count(entry.text.as_str());
                let more: String = characters_beside_the_first(count);
                // What: the edit, in the words that agree with how many characters it changes.
                // Why: "take that character out" about two of them leaves one behind.
                let edit: &str = if count == 1 {
                    "take that character out"
                } else {
                    "take each of them out"
                };
                help.push_str(
                    format!(
                        " The name above also holds {name} at character {place} of the line, where a reader cannot see it{more}: {edit}."
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
///  The line numbers of the other dead lines whose edit gives the same line, written the way a sentence lists them.
/// Why:
///  A sentence that names one line says `line 2`, two say `lines 2 and 5`, and three say `lines 2, 5 and 9`:
///  a list of numbers with no word between them is no sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesListed = (lines) => lines.length === 1
///   ? `line ${lines[0]}`
///   : `lines ${lines.slice(0, -1).join(', ')} and ${lines[lines.length - 1]}`
/// ```
fn lines_listed(lines: &[usize]) -> String {
    // What: one line is named on its own, and two or more are a list.
    // Why: `lines 2` is wrong and `line 2 and 5` is wrong, so the two shapes need two spellings.
    let Some((last, others)) = lines.split_last() else {
        return String::new();
    };
    if others.is_empty() {
        return format!("line {last}");
    }
    // What: the numbers before the last one, set off from each other by a comma.
    // Why: the last one joins the list with `and`, which is how a sentence names a list of lines.
    let mut listed: String = String::new();
    for (at, line) in others.iter().enumerate() {
        if at > 0 {
            listed.push_str(", ");
        }
        listed.push_str(format!("{line}").as_str());
    }
    return format!("lines {listed} and {last}");
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
        // What: the path this name would have in the folder, in the spelling the walk gives the file it read.
        // Why: a module the folder holds is a name the line already spells, and its dead line is deleted,
        // and the tree writes a file outside the package as the walk from the package folder, so the line's own `..` must be read back before the two spellings meet.
        let path: PathBuf = lexical_normal(&base.join(folder).join(name));
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
///  and the line the edit leaves,
///  or nothing when the reader that reads the list would not read it as a line of a use shape.
/// Why:
///  The sentence that takes such characters out has to say what the line becomes,
///  and the word it names must be the line's own:
///  a `file-uses` line told that its first word is not `uses` is told about a word the line never held.
///  The answer is the same reader the list is read with, asked about the edited text,
///  so a sentence that says the line would read is never said about a line the next run refuses:
///  an absolute folder, a backslash and a repeat are refused there, and only a reader that asks is not fooled by them.
///  Replacing the character by a space answers the case the removal cannot:
///  a mark written where the separator belongs leaves one word when it is taken out alone.
///  The sentence names every character it edits, because it edits them all:
///  one that named the first promised a line only the edit of every one gives.
///  The line the edit leaves is answered with the sentence,
///  because the collision of two dead lines' edits is that line and nothing of the sentence.
///  The repeat is asked of the list's own index, which holds every line once,
///  where a scan for it cost the square of the list's length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whatItWouldReadAs = (text, asSpace, package, list, at) => {
///   const count = unseenCharacters(text).length
///   const edit = asSpace ? `With ${asSpaces(count)}, the line` : `With ${named(count)} out, the line`
///   const trimmed = [...text].map((one) => printsAsNothing(one) ? (asSpace ? ' ' : '') : one).join('').trim()
///   if (trimmed === '') return { sentence: `${edit} carries nothing.` }
///   if (trimmed.startsWith('#')) return { sentence: `${edit} is a comment.` }
///   const entry = entryOf(1, 1, trimmed, package)
///   if (entry === undefined || entry.kind === 'key') return undefined
///   if (list.identityIsHeldByAnother(entry.identity(), at)) return undefined
///   return { sentence: `${edit} reads as a \`${word(entry)}\` line.`, identity: entry.identity() }
/// }
/// ```
fn what_the_line_would_read_as(
    text: &str,
    as_a_space: bool,
    package: &PackageFolder<'_>,
    list: &AllowList,
    entry_at: usize,
) -> Option<Reading> {
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
    // What: how many characters of the line the reading edits.
    // Why: the sentence names what the edit does to each of them, and one of two is not one.
    let count: usize = unseen_character_count(text);
    // What: which edit the sentence offers, which is the one this reading applied.
    // Why: a reader told to take the character out of a separator's place would leave one word, and a line holding two such characters needs both edits.
    let edit: String = if as_a_space {
        format!("With {}", the_characters_as_spaces(count))
    } else {
        format!("With {} out", the_characters_named(count))
    };
    // What: the same edit named as something to do, which a sentence beside this one can name.
    // Why: a line whose edit another dead line's edit repeats is not told the line reads, and its sentence names the edit it shares instead.
    let doing: String = if as_a_space {
        format!("writing {}", the_characters_as_spaces(count))
    } else {
        format!("taking {} out", the_characters_named(count))
    };
    if trimmed.is_empty() {
        return Some(Reading {
            sentence: format!("{edit}, the line carries nothing."),
            edit: doing,
            identity: None,
        });
    }
    if trimmed.starts_with('#') {
        return Some(Reading {
            sentence: format!("{edit}, the line is a comment."),
            edit: doing,
            identity: None,
        });
    }
    // What: the entry the reader makes of the edited text, or nothing when the reader refuses it.
    // Why: the reader that decides what a line is decides the sentence too, so the sentence says only what reads.
    let read: AllowEntry = crate::layout_allow_entries::entry_of(1, 1, trimmed, package).ok()?;
    // What: the word the edited line reads with, when it reads as one of the list's use shapes.
    // Why: a key line is no use shape, and the sentence is about a use the edited line would stand for.
    let word: &str = match &read.kind {
        AllowKind::FolderUse {
            ..
        } => crate::layout_allow_entries::FOLDER_USE_WORD,
        AllowKind::FileUse {
            ..
        } => crate::layout_allow_entries::FILE_USE_WORD,
        AllowKind::Key(_) => return None,
    };
    // What: the line this edit leaves, in the words a list writes an entry.
    // Why: two dead lines whose edits give one line are refused as a repeat once both are written.
    let identity: String = read.identity();
    // What: whether a line of the list already holds what this edit would make.
    // Why: a line the list refuses as a repeat reads as a use line and is no line its reader can keep.
    if list.identity_is_held_by_another(identity.as_str(), entry_at) {
        return None;
    }
    return Some(Reading {
        sentence: format!("{edit}, the line reads as a `{word}` line."),
        edit: doing,
        identity: Some(identity),
    });
}

/// What:
///  How many characters of a dead `file-uses` line's file names the advice changes, and how many file names hold them.
/// Why:
///  A file name's edit takes such a character out of the names the advice names and out of nothing else,
///  so the count a sentence about that edit gives is theirs alone:
///  a character of the folder's own name is part of a name the package holds,
///  and a character of a real module's own name is part of the name the rule prints,
///  which the same sentence goes on to ask its reader to write.
///  The number of names chooses the noun, so two characters in one name still speak of one file name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changedByTheFileNameEdit = (entry, moduleFiles, base) => {
///   let characters = 0, names = 0
///   for (const name of [entry.from, entry.to]) {
///     const held = unseenCharacters(name).length
///     if (held === 0) continue
///     if (moduleFiles.has(normalise(join(base, entry.folder, name)))) continue
///     characters += held; names += 1
///   }
///   return [characters, names]
/// }
/// ```
fn unseen_characters_the_file_name_edit_changes(
    entry: &AllowEntry,
    module_files: &BTreeSet<PathBuf>,
    base: &Path,
) -> (usize, usize) {
    // What: the two file names of the line, or nothing when it is of another shape.
    // Why: a key line and a folder use name no file.
    let AllowKind::FileUse {
        folder,
        from,
        to,
        ..
    } = &entry.kind
    else {
        return (0, 0);
    };
    // What: the characters the advice changes and the names that hold them.
    // Why: the sentence counts characters and names the noun after the names.
    let mut characters: usize = 0;
    let mut names: usize = 0;
    for name in [from, to] {
        // What: how many characters of this name a reader cannot see.
        // Why: a name that holds none needs no edit and is no name the sentence speaks of.
        let held: usize = unseen_character_count(name.as_str());
        if held == 0 {
            continue;
        }
        // What: the path this name would have in the folder, in the spelling the walk gives the file it read.
        // Why: a module the folder holds is a name the line already spells, and its dead line is deleted,
        // so the advice leaves the name the rule prints where it is.
        if module_files.contains(&lexical_normal(&base.join(folder).join(name.as_str()))) {
            continue;
        }
        characters += held;
        names += 1;
    }
    return (characters, names);
}

/// What:
///  The words that name how many file names the rule prints: one, or more than one.
/// Why:
///  A line whose two file names hold such a character takes the character out of both,
///  and a sentence that spoke of "the file name" over two of them sent its reader to one name.
///  The noun follows the number of names the edit changes,
///  so two characters inside one file name still speak of that one file name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const printedNames = (count) => (count === 1 ? 'file name' : 'file names');
/// ```
fn file_names_the_rule_prints(count: usize) -> &'static str {
    return match count {
        0 | 1 => "file name",
        _ => "file names",
    };
}

/// What:
///  The edit a dead key line takes when the characters a reader cannot see stand in its words.
/// Why:
///  Such a character is not whitespace, so nothing on the line sets it off as a separator
///  and the reader makes it part of the word beside it: taking it out is the one edit,
///  and a whitespace character goes where it stood only if it stood where one belongs.
///  A line holding two of them takes both edits, and a sentence naming one would leave the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeOut = (count) => `Take ${named(count)} out of the line, and write a whitespace character where ${count === 1 ? 'it' : 'each'} stood if ${count === 1 ? 'it' : 'it'} stood where one belongs.`
/// ```
fn take_out_what_a_reader_cannot_see(count: usize) -> String {
    if count == 1 {
        return String::from(
            "Take that character out of the line, and write a whitespace character where it stood if it stood where one belongs.",
        );
    }
    return format!(
        "Take {} out of the line, and write a whitespace character where each of them stood if it stood where one belongs.",
        the_characters_named(count)
    );
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
    // What: the part of each name a reader sees, which is the folder the line was trying to name,
    // with every character a reader cannot see spelled.
    // Why: the sentence names that folder so its reader knows which of two identical spaces the phrase is about,
    // and the sentence that follows it says the same name holds that character, which a raw name would print as nothing.
    let from_name: String = spelled_unreadable(from_text.trim());
    let to_name: String = spelled_unreadable(to_text.trim());
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
