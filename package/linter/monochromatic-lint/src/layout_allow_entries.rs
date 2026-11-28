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
///  the spelling and the name of the characters a message cannot show,
///  and the reading that removes the `.` segments and the folders a `..` reads back over.
use crate::{
    layout_finding::{
        characters_beside_the_first,
        first_unseen_character,
        key_path,
        prints_as_nothing,
        spelled_unreadable,
        unseen_character_count,
        unseen_character_phrase,
    },
    markdown_lfs_target::lexical_normal,
};
/// Import the cells that hold the answers about the package folder's names,
///  the map that keeps one answer per folder,
///  the set that keeps one line from repeating another,
///  the one spelling of a listed path,
///  and native paths.
use std::{
    cell::{
        OnceCell,
        RefCell,
    },
    collections::{
        BTreeMap,
        BTreeSet,
    },
    ops::Range,
    path::{
        Path,
        PathBuf,
    },
};

/// The word that starts an entry tolerating one use between two folders.
pub(crate) const FOLDER_USE_WORD: &str = "uses";

/// The word that starts an entry tolerating one use between two files of a folder.
pub(crate) const FILE_USE_WORD: &str = "file-uses";

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
///  The package folder a list is read against,
///  with the one answer about the longest name it holds,
///  read at most once however many lines ask for it.
/// Why:
///  A six-word `file-uses` line asks how long the longest name in the package folder is,
///  and a dead key line whose edited text is such a line asks the same question:
///  listing the folder once per line cost a list of 1,000 such lines 175 times the previous release's time
///  where the folder holds 5,000 entries (measured on 2026-11-27),
///  so the answer belongs to the folder and not to the line.
///  A list whose lines never ask reads the folder not at all.
///  One refusal also asks whether the folder it names holds a module file,
///  and a dead line whose edited text is such a line asks it once per line:
///  that answer is kept per folder as well, because a list names one folder many times
///  (measured on 2026-11-28: 400 such lines over a folder of 20,000 files cost 5.4 s where the folder was listed per line).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PackageFolder {
///   #longest?: number;
///   constructor(readonly path: Path) {}
///   get longestName() { return (this.#longest ??= longestNameIn(this.path)); }
/// }
/// ```
pub(crate) struct PackageFolder<'base> {
    /// The folder every listed path is read back against.
    path: &'base Path,
    /// The length of the longest name the folder holds, once a line asks for it.
    longest_name: OnceCell<Option<usize>>,
    /// Whether each folder a line names holds a module file, once that line asks.
    module_file: RefCell<BTreeMap<PathBuf, bool>>,
}

/// What:
///  The package folder, and the answers it gives about its own names.
/// Why:
///  A caller makes one of these for one list and passes it to every line it reads,
///  so the folder is listed once and never once per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packageFolder = (base: Path) => new PackageFolder(base);
/// ```
impl<'base> PackageFolder<'base> {
    /// Build the reader of one package folder, which lists it only when a line asks.
    pub(crate) fn at(path: &'base Path) -> PackageFolder<'base> {
        return PackageFolder {
            path,
            longest_name: OnceCell::new(),
            module_file: RefCell::new(BTreeMap::new()),
        };
    }

    /// The folder every listed path is read back against.
    pub(crate) fn path(&self) -> &'base Path {
        return self.path;
    }

    /// The length in bytes of the longest name the folder holds, or nothing when it cannot be read.
    fn longest_name(&self) -> Option<usize> {
        return *self
            .longest_name
            .get_or_init(|| return longest_name_in(self.path));
    }

    /// Whether a folder holds a module file, answered once for each folder a list names.
    /// Why:
    ///  A refusal for a folder whose name holds whitespace asks this of the folder it names,
    ///  and every dead line whose edited text is such a line asks it again:
    ///  listing the folder once per line cost a list of 400 such lines 5.4 s over 20,000 files (measured 2026-11-28),
    ///  where the folder holds the same files for every line of the list.
    ///  The answer belongs to the folder, so the first line that names it lists it and every later line reads that answer.
    pub(crate) fn holds_a_module_file(
        &self,
        folder: &Path,
    ) -> bool {
        // What: the answer this list already has for this folder, if any line asked before.
        // Why: a folder named twice is one question, and the files it holds do not change while a list is read.
        if let Some(answer) = self.module_file.borrow().get(folder) {
            return *answer;
        }
        // What: whether the folder holds a module file, asked of the file system once.
        // Why: the walk must not repeat for the second line that names the same folder.
        let answer: bool = holds_a_module_file(folder);
        self.module_file
            .borrow_mut()
            .insert(folder.to_path_buf(), answer);
        return answer;
    }
}

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
///  The message also says where a character it names stands,
///  counted in characters from the line's first one,
///  so the entry keeps the column it starts at:
///  the reader trims the whitespace around a line, and counting from the trimmed text would be off by the indent.
///  A character is a Unicode scalar value, so a tab, a no-break space and an ideographic space are one each,
///  though an editor may draw them as several cells or write one as more than one byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowEntry = { line: number; column: number; text: string; kind: AllowKind };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowEntry {
    /// One-based line of the list file.
    pub line: usize,
    /// One-based column of the entry's first character in that line,
    ///  one past the whitespace the reader trims.
    pub column: usize,
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
///  The folder a six-word `file-uses` line names,
///  and where the words that name it stand in the line.
/// Why:
///  The refusal names that folder,
///  and a character a reader cannot see that stands inside those words is part of the folder's own name
///  and no character to take out of the line.
///  The words are the line's own,
///  so the place answers where the gap is a tab,
///  a no-break space or a doubled space,
///  which a search for the folder's text answers nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SpacedFolder = { name: string; span: [number, number] };
/// ```
struct SpacedFolder {
    /// The folder a directory of the package is called, as the reader reads it.
    name: String,
    /// The bytes of the line that hold the words naming that folder.
    span: Range<usize>,
}

/// What:
///  The folder a `file-uses` line names,
///  and the words that name it,
///  when its words are more than the shape has and the ones in the folder's place hold whitespace.
/// Why:
///  The shape is five words set off with one whitespace character each,
///  so a folder whose name holds a space, a tab, a no-break space or a doubled space makes six
///  and the line cannot be read at all.
///  The five-word sentence alone would send its reader back to a line that already reads as the shape it names,
///  so the refusal names the folder and the whitespace in its name instead.
///  A file name never holds whitespace in a tree the rules accept (that is `space-in-file-name`),
///  so the raw text between the word and the file before the arrow is the folder's own name.
///  That raw text is read first rather than the words joined with one space,
///  because joining them would name a folder the package does not hold for every whitespace character but a space,
///  and a tree can hold both `src/c<TAB>d` and `src/c d`.
///  The text a doubled gap left beside the name is taken off next,
///  because that character is what a re-spaced line would remove,
///  and the name it leaves is the one the reader meant.
///  The run before the file is taken off whole, because no folder name a tree the rules accept holds ends in whitespace;
///  at the start the run is taken off one character at a time,
///  because a folder name may begin with whitespace and only the name the tree holds can say how much of that run is its own.
///  The words joined with one space are read last,
///  because a folder name whose whitespace is a tab or a no-break space is no text any folder holds,
///  and a reader who wrote such a line meant the folder those characters stand for.
///  Those words are the folder's own only while they name one:
///  a third file name and a doubled arrow make six words whose middle ones are no folder,
///  and telling that reader to rename a folder that is fine sends them to the wrong place.
///  Only a folder of the tree can hold the whitespace, so the package folder the words are read from is the caller's.
///  The line's own arrow is the last one whitespace sets off:
///  a file name is one word and can hold no whitespace, so no arrow of a file name follows it,
///  while the folder before it may hold an arrow word of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spacedFolder = (text: string, words: string[], package: PackageFolder): SpacedFolder | undefined => {
///   if (words.length < 6 || words[words.length - 2] !== '->') return undefined
///   const afterWord = text.slice('file-uses'.length).replace(/^\s/, '')
///   const arrow = lastArrowBetweenWhitespace(afterWord)
///   if (arrow === undefined) return undefined
///   const beforeArrow = afterWord.slice(0, arrow).replace(/\s+$/, '')
///   const file = words[words.length - 3]
///   const written = beforeArrow.endsWith(file) ? beforeArrow.slice(0, -file.length).replace(/\s$/, '') : undefined
///   const found = firstReading(written, package.longestName, (one) => /\s/.test(one) && isDirectory(package.path + one))
///   if (found) return { name: found.name, span: [namesBegin + found.offset, namesBegin + found.offset + found.name.length] }
///   const joined = words.slice(1, -3).join(' ')
///   if (/\s/.test(joined) && isDirectory(package.path + joined)) return { name: joined, span: spanOfWords(words, text, 1, words.length - 3) }
///   return undefined
/// }
/// ```
fn spaced_file_use_folder(
    text: &str,
    words: &[&str],
    package: &PackageFolder<'_>,
) -> Option<SpacedFolder> {
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
    // What: where the words that name the folder begin in the line.
    // Why: a span the message compares a character's place with is counted in the line, and these words begin one character after the line's own word.
    let names_begin: usize = text.len() - after_the_word.len();
    // What: where the line's own arrow begins, which is the last one whitespace sets off on both sides.
    // Why: a folder name before it may hold an arrow word of its own, while a file name is one word and holds none.
    let arrow: usize = last_arrow_between_whitespace(after_the_word)?;
    // What: everything before that arrow, without the whitespace run that sets it off.
    // Why: a doubled gap between the file and the arrow is no part of any name the line can carry, and the folder before the file is still the one it writes.
    let before_arrow: &str = after_the_word[..arrow].trim_end_matches(char::is_whitespace);
    // What: the longest name the package folder holds, which is the longest first folder a name beside the gap can have.
    // Why: a first folder longer than every name there is no folder of the package, so a name that would need one is no name to ask about.
    let longest_name: Option<usize> = package.longest_name();
    // What: the first name the written text spells that a folder of the package is called, offered one at a time and never kept.
    // Why: the text the line writes is the folder its reader meant whenever the tree holds it, and each name the gap left beside it answers one reading of that text.
    if let Some(before_file) = before_arrow.strip_suffix(file)
        && let Some(written) = before_file.strip_suffix(char::is_whitespace)
        && let Some((offset, found)) =
            first_reading_a_folder_holds(written, longest_name, &mut |name| {
                // What: one name this read asks the file system about, which a control counts.
                // Why: a read that asked about one name per character of the gap cost the square of the gap, and the count is what holds that.
                #[cfg(test)]
                counted_work::note_candidate_name();
                // What: whether this name is one a folder of the package is called.
                // Why: a name with no whitespace in it leaves the line readable as the shape has it, and a name no tree holds is no folder the message can send its reader to.
                return name.chars().any(char::is_whitespace) && package.path().join(name).is_dir();
            })
    {
        return Some(SpacedFolder {
            name: String::from(found),
            span: names_begin + offset..names_begin + offset + found.len(),
        });
    }
    // What: the words between the word and that file, joined with one space.
    // Why: a folder whose name holds a tab or a no-break space is no text any folder holds, and its line names the folder those characters stand for.
    let joined: String = words[1..words.len() - 3].join(" ");
    #[cfg(test)]
    counted_work::note_offered_name();
    #[cfg(test)]
    counted_work::note_candidate_name();
    // What: whether those words name a folder of the package that holds whitespace, which is the one reason the shape's five words become six.
    // Why: a name with no whitespace in it leaves the line readable as the shape has it, and a name no tree holds is no folder the message can send its reader to.
    if joined.chars().any(char::is_whitespace) && package.path().join(joined.as_str()).is_dir() {
        // What: where those words stand in the line.
        // Why: the joined name is no text the line holds, so the place a character's own is compared with is the words it was joined from.
        let starts: Vec<usize> = word_offsets(text);
        let first_word: usize = *starts.get(1)?;
        let last_word: &str = words.get(words.len() - 4)?;
        let last_end: usize = *starts.get(words.len() - 4)? + last_word.len();
        return Some(SpacedFolder {
            name: joined,
            span: first_word..last_end,
        });
    }
    return None;
}

/// What:
///  Where each whitespace-separated word of a text begins, in bytes.
/// Why:
///  The words a line names its folder with are the line's own,
///  and their span is what says whether a character a message names stands inside the folder's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordOffsets = (text) => {
///   const starts: number[] = []
///   let afterASeparator = true
///   for (const [at, letter] of [...text].entries()) {
///     if (/\s/u.test(letter)) { afterASeparator = true; continue }
///     if (afterASeparator) { starts.push(at); afterASeparator = false }
///   }
///   return starts
/// }
/// ```
fn word_offsets(text: &str) -> Vec<usize> {
    // What: where each word begins so far.
    // Why: the caller asks for one word's place by its index.
    let mut starts: Vec<usize> = Vec::<usize>::new();
    // What: whether the character before this one was a separator, so this one begins a word.
    // Why: a word begins at the text's first character and after any whitespace run.
    let mut after_a_separator: bool = true;
    for (offset, letter) in text.char_indices() {
        if letter.is_whitespace() {
            after_a_separator = true;
            continue;
        }
        if after_a_separator {
            starts.push(offset);
            after_a_separator = false;
        }
    }
    return starts;
}

/// What:
///  The names a written folder text may have been meant as,
///  the text itself first and then the same text with the whitespace a doubled gap left beside it taken off one character at a time,
///  offered to a caller one at a time and never kept,
///  and a name whose first folder is longer than every name the package folder holds offered to no one.
/// Why:
///  The reader takes exactly one whitespace character as the separator at each end of a name,
///  so a gap of two characters or more leaves one beside the name on either side.
///  A folder name a tree the rules accept holds never ends in whitespace,
///  so the whole run before the file came from that gap and is taken off whole.
///  At the start the run is taken off one character at a time,
///  because a folder name may begin with whitespace and only the name a tree holds says how much of the run is its own,
///  and the name that keeps the most of the run is the one offered first.
///  Nothing more is read away than a doubled gap put there:
///  a name that the line did not write is named only by the reading of the words joined with one space.
///  No name is kept and the whole run costs one pass over the text:
///  a reader that built one name per character of the gap, each as long as the line,
///  cost the square of the gap in time and in memory (0.25 s and 166 MiB at 16,000 characters,
///  3.3 s and 2,171 MiB at 64,000, measured on 2026-11-27 against 0.00 s and 11 MiB of the release before),
///  while the list reader accepts a file of four mebibytes.
///  The names a folder of the tree can begin with bound the run:
///  a name whose first folder is longer than the longest name the package folder holds is no folder of the package,
///  so the caller is asked about the few names near the end of the run and about nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstReading = (written, longestName, accepts) => {
///   const withoutTrailing = written.replace(/\s+$/u, '')
///   if (written !== withoutTrailing && accepts(written)) return { offset: 0, name: written }
///   const leading = [...withoutTrailing].findIndex((one) => !/\s/u.test(one))
///   const slash = withoutTrailing.indexOf('/')
///   for (let removed = 0; removed <= leading; removed++) {
///     const name = [...withoutTrailing].slice(removed).join('')
///     const first = (slash === -1 ? name.length : slash - [...withoutTrailing].slice(0, removed).join('').length)
///     if (removed < leading && longestName !== undefined && first > longestName) continue
///     if (accepts(name)) return { offset: [...withoutTrailing].slice(0, removed).join('').length, name }
///   }
///   return undefined
/// }
/// ```
fn first_reading_a_folder_holds<'text>(
    written: &'text str,
    longest_name: Option<usize>,
    accepts: &mut impl FnMut(&str) -> bool,
) -> Option<(usize, &'text str)> {
    // What: the text with the whitespace run at its end taken off.
    // Why: no folder name a tree the rules accept holds ends in whitespace, so everything at that end came from the gap before the file.
    let without_trailing: &str = written.trim_end_matches(char::is_whitespace);
    // What: the text as the line writes it, offered first when the end it writes holds whitespace of its own.
    // Why: the name the line writes is the one its reader meant whenever the tree holds it.
    if written != without_trailing && accepts(written) {
        return Some((0, written));
    }
    // What: where the text's leading whitespace run ends, which is where the name's own first character stands.
    // Why: the loop below takes the run off one character at a time, so each name a tree could hold is offered once.
    let leading_end: usize = without_trailing.len()
        - without_trailing
            .trim_start_matches(char::is_whitespace)
            .len();
    // What: where the text's first folder name ends, or nothing when the text is one folder name.
    // Why: a leading run is added to that first folder, so its length is what a name the package folder holds bounds.
    let first_folder_end: Option<usize> = without_trailing.find('/');
    // What: how many bytes of the leading run have been taken off, which is where the walk starts.
    // Why: each step removes one character, so the next name begins where this one's first character ends,
    // and every offset before the first name the folder could hold names a first folder longer than any name it holds,
    // so offering each of them would build the gap's worth of names and ask about none of them.
    // Gotcha: the offset must stand on a character boundary, because the name offered is a slice of the text.
    let mut offset: usize = 0;
    if let (Some(end), Some(longest)) = (first_folder_end, longest_name)
        && end > longest
    {
        offset = end - longest;
        while offset > 0 && !without_trailing.is_char_boundary(offset) {
            offset -= 1;
        }
        offset = offset.min(leading_end);
    }
    while offset <= leading_end {
        let name: &str = &without_trailing[offset..];
        // What: one name this reading built, which a control counts.
        // Why: a reading that built a name for every character of the gap copied the square of the gap, and the count is what holds that.
        #[cfg(test)]
        counted_work::note_offered_name();
        // What: whether this name still begins with the gap's whitespace, so its first folder is that run followed by one.
        // Why: only then does the length of a name the package folder holds say anything about it.
        let keeps_whitespace: bool = offset < leading_end;
        // What: how long that first folder would be, in bytes, which is the length the file system counts.
        // Why: a run of whitespace longer than every name in the package folder names no folder it holds.
        let first_folder: usize = match first_folder_end {
            Some(end) => end - offset,
            None => name.len(),
        };
        // What: whether a folder of the package could be called this name at all.
        // Why: a name beginning with no whitespace is one the line wrote whole, and its length says nothing about where it is.
        let a_folder_could_be_called_this: bool =
            !keeps_whitespace || longest_name.is_none_or(|longest| return first_folder <= longest);
        if a_folder_could_be_called_this && accepts(name) {
            return Some((offset, name));
        }
        if offset >= leading_end {
            return None;
        }
        // What: one character of the leading run, which is one step of the removal the reader takes.
        // Why: the run is counted in characters, so a tab, a no-break space and a space are one step each.
        offset += without_trailing[offset..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
    }
    return None;
}

/// What:
///  The length in bytes of the longest name a folder holds, or nothing when the folder cannot be read.
/// Why:
///  A name of the package folder is the longest first folder a name beside a gap can have,
///  and the reader asks about a name whose first folder is longer than every one of them for nothing:
///  no folder of the package can be called it.
///  A name that is not valid UTF-8 is counted as the lossy spelling of it, which is longer than the file system's name,
///  so the bound is looser than it could be and never skips a name a folder could be called.
///  Nothing is read when the folder cannot be listed, and then every name is offered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const longestNameIn = (folder) => {
///   try { return Math.max(0, ...readdirSync(folder).map((one) => Buffer.byteLength(one))) } catch { return undefined }
/// }
/// ```
fn longest_name_in(folder: &Path) -> Option<usize> {
    // What: one listing of a folder, which a control counts.
    // Why: the answer is one per list, so a control that counts listings holds a reader that asks the folder again for every line.
    #[cfg(test)]
    counted_work::note_folder_listing();
    // What: the names the folder holds, or nothing when it cannot be read.
    // Why: a folder that cannot be listed gives no bound, and every name must then be offered.
    let entries: std::fs::ReadDir = std::fs::read_dir(folder).ok()?;
    let mut longest: usize = 0;
    for entry in entries.flatten() {
        // What: the name as one text, which is what the file system compares a folder's name with.
        // Why: the byte length of that text is the length a name of the folder has.
        let length: usize = entry.file_name().to_string_lossy().len();
        if length > longest {
            longest = length;
        }
    }
    return Some(longest);
}

/// What:
///  Whether a folder holds a module file, which is what a loop of files inside it is made of.
/// Why:
///  The refusal for a folder whose name holds whitespace offers two edits:
///  rename the folder, or untangle the loop of files inside it,
///  and a folder that holds no module file has no loop to untangle.
///  A `.rs` file is what a module file is, and a folder that cannot be listed holds none the rules could read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsAModuleFile = (folder) => {
///   try { return readdirSync(folder).some((one) => one.endsWith('.rs')) } catch { return false }
/// }
/// ```
fn holds_a_module_file(folder: &Path) -> bool {
    // What: one listing of a folder for a module file, which a control counts.
    // Why: the answer is kept per folder and list, so a control that counts listings holds a reader that asks the folder again for every line.
    #[cfg(test)]
    counted_work::note_module_file_listing();
    // What: the names the folder holds, or nothing when it cannot be read.
    // Why: a folder no reader can list holds no module file that any check would read.
    let Ok(entries) = std::fs::read_dir(folder) else {
        return false;
    };
    for entry in entries.flatten() {
        // What: the name of one entry, which is a module file exactly when it ends in `.rs`.
        // Why: both places a module lives (`x.rs` and `x/mod.rs`) end in that extension.
        let name: std::ffi::OsString = entry.file_name();
        if Path::new(&name)
            .extension()
            .is_some_and(|extension| return extension == "rs")
        {
            return true;
        }
    }
    return false;
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
///  The sentence a refusal adds about a character a reader cannot see, or nothing when the line holds none,
///  with the edit its reader can make for each character the line itself carries.
/// Why:
///  A refusal stops the whole rule, so it is the message a reader most needs to act on,
///  and a character that prints as nothing is the hardest fault to find: the line looks right.
///  The place counts characters from the line's first one rather than the entry's,
///  because the reader trims the indent away and a person counting what the file holds starts at its first character.
///  The sentence says how many characters the line holds,
///  because a person told of the first and left to find the second on the next run learns nothing from it.
///  The edit offered is both the one that works when the character stands in a word and the one that works when it stands where a separator belongs:
///  the reader does not treat it as whitespace either way, and which of the two it is depends on where it stands.
///  A character that stands in the folder's own name is no character the line can take out,
///  because the name it would spell is another folder's: that one is taken out by the rename the refusal asks for,
///  and the sentence says so beside the edit for the characters outside the name.
///  The edit is named for the characters the line carries outside the folder's name and for no others,
///  because a sentence that says to take each of them out sends its reader to a line the next run refuses for its word count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unseenNote = (text, column) => noteOf(text, column, undefined)
/// ```
fn unseen_character_note(
    text: &str,
    column: usize,
) -> String {
    return note_of_an_unseen_character(text, column, None);
}

/// What:
///  The sentence a refusal adds about a character a reader cannot see
///  when one or more of them stand in the name of the folder the refusal names.
/// Why:
///  Taking such a character out of the folder's name would name another folder,
///  and the refusal already asks its reader to rename that folder:
///  the rename is what takes those characters out, so the note names that one edit for them
///  and the take-out edit for the characters the line carries elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unseenNoteInFolder = (text, column, span) => noteOf(text, column, span)
/// ```
fn unseen_character_note_in_a_folder_name(
    text: &str,
    column: usize,
    folder: Range<usize>,
) -> String {
    return note_of_an_unseen_character(text, column, Some(folder));
}

/// What:
///  The sentence a refusal adds about the first character of the line a reader cannot see,
///  how many the line holds beside it,
///  which of them stand in the word-named folder's own name,
///  and the edit its reader can make for each one outside that name.
/// Why:
///  One character is enough for the sentence, and its place is what the reader has to find;
///  the count is what says the line holds more, which the next run would otherwise name;
///  and the edit depends on where each one stands, because a character the line wrote as part of a folder's name is no character to take out.
///  The question is asked of every character and not of the first alone:
///  a line whose first character stands within the name and whose second stands outside it was told to take both out,
///  and the next run answered the edited line with a word count and named no folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteOf = (text, column, folder) => {
///   const first = firstUnseen(text)
///   const place = column + first[1] - 1
///   const more = besideTheFirst(unseenCount(text))
///   const inside = folder === undefined ? 0 : unseenInSpan(text, column, folder)
///   ...
/// }
/// ```
fn note_of_an_unseen_character(
    text: &str,
    column: usize,
    folder: Option<Range<usize>>,
) -> String {
    // What: the first character of the line a reader cannot see, with its place among the entry's characters.
    // Why: one character is enough for the sentence, and its place is what the reader has to find.
    let Some((letter, position)) = first_unseen_character(text) else {
        return String::new();
    };
    // What: the plain name of that character, or a phrase that still names it by its code point.
    // Why: a reader who is told only that a character cannot be seen has nothing to look for.
    let name: String = unseen_character_phrase(letter);
    let place: usize = column + position - 1;
    // What: how many characters of the line a reader cannot see, and the clause that says how many stand beside the first.
    // Why: an edit that takes one out leaves the other, and the next run names it again.
    let count: usize = unseen_character_count(text);
    let more: String = characters_beside_the_first(count);
    // What: how many of them stand inside the words the refusal names the folder with.
    // Why: each of those is taken out by renaming the folder, and each of the others by the edit the line can take out itself.
    let inside: usize = folder.as_ref().map_or(0, |span| {
        return unseen_characters_in_span(text, column, span.clone());
    });
    // What: the edit for a refusal that names no folder, where the sentence cannot say which character stands in a folder's own name.
    // Why: a sentence that said to take each of them out sent its reader to a line the next run refused for its word count,
    // so it names the edit for the one character it placed, says how many stand beside it, and says what a folder's own character needs instead.
    if folder.is_none() {
        let edit: &str = if count == 1 {
            "the reader does not treat it as whitespace, so take it out, or write a whitespace character where it stood if it stood where one belongs"
        } else {
            "the reader does not treat it as whitespace, so take that character out, and write a whitespace character where it stood if it stood where one belongs"
        };
        return format!(
            " The line also holds {name} at character {place} of the line, where a reader cannot see it{more}: {edit}. A character that stands in the name of a real folder is part of that name, and renaming that folder is the edit that takes it out."
        );
    }
    // What: the edit for characters every one of which stands outside the folder the refusal names.
    // Why: with the folder known and no character of its name among them, the take-out edit is the one every character there needs.
    if inside == 0 {
        let edit: &str = if count == 1 {
            "the reader does not treat it as whitespace, so take it out, or write a whitespace character where it stood if it stood where one belongs"
        } else {
            "the reader does not treat them as whitespace, so take each of them out, and write a whitespace character where one stood if it stood where one belongs"
        };
        return format!(
            " The line also holds {name} at character {place} of the line, where a reader cannot see it{more}: {edit}."
        );
    }
    let outside: usize = count - inside;
    let take_out: &str = if outside == 1 {
        "take out the one outside the folder's name, and write a whitespace character where it stood if it stood where one belongs"
    } else {
        "take out the {outside} outside the folder's name, and write a whitespace character where one stood if it stood where one belongs"
    };
    // What: which characters the rename takes out, said as a group so the clause never points at `that character` after a count.
    // Why: a sentence whose `that character` follows `2 more characters` reads as the wrong one of them.
    let in_the_folder: String = match (inside, count - inside) {
        (1, 0) => String::from("that character stands in the name of the folder above"),
        (1, _) => String::from("one of them stands in the name of the folder above"),
        (_, 0) => String::from("every one of them stands in the name of the folder above"),
        (_, _) => format!("{inside} of them stand in the name of the folder above"),
    };
    let them: &str = if inside == 1 { "it" } else { "them" };
    if count == inside {
        return format!(
            " The line also holds {name} at character {place} of the line, where a reader cannot see it{more}, and {in_the_folder}: renaming that folder is the edit that takes {them} out."
        );
    }
    return format!(
        " The line also holds {name} at character {place} of the line, where a reader cannot see it{more}, and {in_the_folder}, so renaming that folder takes {them} out: {take_out}."
    );
}

/// What:
///  How many characters of a line a reader cannot see that stand inside a span of it.
/// Why:
///  A message names a character by its place in the whole line and the folder's words are a span of the entry,
///  and the edit a character needs depends on which of the two it is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unseenInSpan = (text, column, span) =>
///   [...text].filter((one, at) => printsAsNothing(one) && placeStandsIn(column + at, column, text, span)).length
/// ```
fn unseen_characters_in_span(
    text: &str,
    column: usize,
    span: Range<usize>,
) -> usize {
    // What: how many characters of the line that print as nothing stand inside the span.
    // Why: each of them is a character the rename takes out, and each one outside it is left for the line's own edit.
    return text
        .chars()
        .enumerate()
        .filter(|(index, letter)| {
            return prints_as_nothing(*letter)
                && place_stands_in(column + index, column, text, span.clone());
        })
        .count();
}

/// What:
///  Whether the character at a place among a line's characters stands inside the words a span of the entry covers.
/// Why:
///  A message names a character by its place in the whole line and a folder's words are a span of the entry,
///  and the two have to be compared in one count: the line's characters.
///  The span is the words the line itself holds,
///  so a folder whose gap is a tab, a no-break space or a doubled space answers here,
///  where a search for the folder's own text answered nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placeStandsIn = (place, column, text, span) => {
///   const first = column + [...text.slice(0, span[0])].length
///   const last = first + [...text.slice(span[0], span[1])].length - 1
///   return place >= first && place <= last
/// }
/// ```
fn place_stands_in(
    place: usize,
    column: usize,
    text: &str,
    span: Range<usize>,
) -> bool {
    // What: the place of the span's first character in the line, counted as the message counts it.
    // Why: the places the message gives and the places this count gives must be one count, or the two sentences disagree.
    let first: usize = column + text[..span.start].chars().count();
    // What: the place of the span's last character in the line.
    // Why: a place between the two is inside the name, and one outside them is not.
    let last: usize = first + text[span.clone()].chars().count().saturating_sub(1);
    return (first..=last).contains(&place);
}

/// What:
///  How many words a count of words says, with a noun that agrees with it.
/// Why:
///  "this line is 1 words" is a sentence its reader stops trusting,
///  and a line of one word is exactly what a half-written entry is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const countedWords = (count) => count === 1 ? 'one word' : `${count} words`
/// ```
fn counted_words(count: usize) -> String {
    return if count == 1 {
        String::from("one word")
    } else {
        format!("{count} words")
    };
}

/// What:
///  The line as a refusal quotes it, and the sentence about a character a reader cannot see.
/// Why:
///  One spelling serves every refusal of the reader,
///  and a line that reads pays for neither:
///  a list of many thousands of keys would otherwise build a sentence it never prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoteOf = (text, column) => [spelledUnreadable(text), unseenCharacterNote(text, column)]
/// ```
fn quote_of(
    text: &str,
    column: usize,
) -> (String, String) {
    return (
        spelled_unreadable(text),
        unseen_character_note(text, column),
    );
}

/// What:
///  Read one line as an entry,
///  or say why it names nothing.
/// Why:
///  A line that is a misspelling must fail the run,
///  because a silently ignored line would let a finding through.
///  The package folder reads every listed path back into the one spelling a key writes,
///  so what a line names and what a finding names meet in one place.
///  Every sentence quotes the line with the characters a reader cannot see spelled,
///  because a line a copied page led with a mark looks like a line that reads.
///
/// The package folder is passed as one reader of it,
///  so a list whose lines ask how long the longest name it holds is asks a folder that is listed once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(line: number, column: number, text: string, package: PackageFolder): AllowEntry;
/// ```
pub(crate) fn entry_of(
    line: usize,
    column: usize,
    text: &str,
    package: &PackageFolder<'_>,
) -> Result<AllowEntry, String> {
    // What: the folder every listed path is read back against.
    // Why: a path in an entry is relative to the package folder, and a key writes that walk.
    let base: &Path = package.path();
    let words: Vec<&str> = text.split_whitespace().collect::<Vec<&str>>();
    let kind: AllowKind = if words.first() == Some(&FOLDER_USE_WORD) {
        let (from, to): (&str, &str) = use_ends(line, column, text)?;
        for path in [from, to] {
            if let Some(reason) = unusable_reason(path) {
                let (spelled, note): (String, String) = quote_of(text, column);
                return Err(format!(
                    "Line {line} reads `{spelled}`. It names a folder that {reason}. A path is relative to the package folder, as findings show it.{note}"
                ));
            }
        }
        AllowKind::FolderUse {
            from: spelled_path(base, &listed_path(from)),
            to: spelled_path(base, &listed_path(to)),
        }
    } else if words.first() == Some(&FILE_USE_WORD) {
        // What: whether the first word of the folder's place is a path this shape cannot carry.
        // Why: an absolute path or a backslash is a cause no rename for a space mends,
        // and a line of more than five words is read for that cause as a five-word one is.
        if words.len() > 5
            && let Some(reason) = unusable_reason(words[1])
        {
            let (spelled, note): (String, String) = quote_of(text, column);
            return Err(format!(
                "Line {line} reads `{spelled}`. It names a folder that {reason}. A path is relative to the package folder, as findings show it.{note}"
            ));
        }
        if let Some(folder) = spaced_file_use_folder(text, words.as_slice(), package) {
            // What: the folder's name with every character a reader cannot see spelled, so the message names the folder that is there.
            // Why: a tab or a no-break space prints as nothing, and a reader shown `src/cd` would look for a folder the package may well hold.
            let spelled_folder: String = spelled_unreadable(folder.name.as_str());
            // What: whether every whitespace character in the name is a space.
            // Why: a space is the one whitespace character a reader can see, so only another one needs its name and code point.
            let holds_only_spaces: bool = folder
                .name
                .chars()
                .all(|letter| return !letter.is_whitespace() || letter == ' ');
            // What: what the name holds, and what renaming must give.
            // Why: a name holding a tab is fixed by a rename that also keeps spaces, and the sentence must not send its reader to the wrong place.
            let (holds, fix): (String, &str) = if holds_only_spaces {
                (String::from("which holds a space"), "space")
            } else {
                (
                    format!(
                        "which holds {}",
                        whitespace_character_description(folder.name.as_str())
                    ),
                    "whitespace character",
                )
            };
            let (spelled, _note): (String, String) = quote_of(text, column);
            // What: whether the folder holds a module file, which is what a loop of files inside it is made of.
            // Why: a folder that holds none has no loop to untangle, and sending its reader to untangle one sends them to something that is not there.
            let holds_a_module: bool =
                package.holds_a_module_file(base.join(folder.name.as_str()).as_path());
            // What: the edit open to the line's reader, which is the rename either way and the untangling only while the folder holds files to untangle.
            // Why: the two edits are the only two ways a loop through such a folder stops needing a line.
            let remedy: String = if holds_a_module {
                format!(
                    "Rename the folder so its name holds no {fix}, or untangle the loop of files inside it."
                )
            } else {
                format!(
                    "Rename the folder so its name holds no {fix}: it holds no module file, so no loop of files inside it needs such a line."
                )
            };
            // What: the sentence about a character the line holds that a reader cannot see.
            // Why: a character in the folder's own name is no character its reader can take out, and the rename above is the edit.
            let note: String = unseen_character_note_in_a_folder_name(text, column, folder.span);
            return Err(format!(
                "Line {line} reads `{spelled}`. It names the folder `{spelled_folder}`, {holds}: a `file-uses` line sets its five words off with one whitespace character each, so it cannot carry a name that holds one. {remedy}{note}"
            ));
        }
        if words.len() != 5 || words[3] != ARROW {
            let (spelled, note): (String, String) = quote_of(text, column);
            // What: whether the line sets more than one arrow off with whitespace, which is one way its five words become more.
            // Why: a doubled arrow is a different mistake from an extra word, and its reader has to know which one is theirs.
            if arrows_between_whitespace(text) > 1 {
                return Err(format!(
                    "Line {line} reads `{spelled}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`, and it holds one `{ARROW}` only: two `{ARROW}` are set off by whitespace.{note}"
                ));
            }
            // What: whether the line holds the shape's five words with another word in the arrow's place.
            // Why: a misspelt or misplaced arrow is the likeliest typo, and a count of five words the line already has names no cause.
            if words.len() == 5 {
                return Err(format!(
                    "Line {line} reads `{spelled}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`, which is five words, and the fourth word must be `{ARROW}`: this line's fourth word is `{}`.{note}",
                    spelled_unreadable(words[3])
                ));
            }
            return Err(format!(
                "Line {line} reads `{spelled}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`, which is five words; this line is {}.{note}",
                counted_words(words.len())
            ));
        }
        if let Some(reason) = unusable_reason(words[1]) {
            let (spelled, note): (String, String) = quote_of(text, column);
            return Err(format!(
                "Line {line} reads `{spelled}`. It names a folder that {reason}. A path is relative to the package folder, as findings show it.{note}"
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
        column,
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
///  Where the last `->` a text sets off with one whitespace character on each side begins,
///  or nothing when it sets none off that way.
/// Why:
///  In a `file-uses` line the line's own arrow is the last one set off that way:
///  a file name is one word and holds no whitespace, so no arrow of a file name can follow it,
///  while the folder before it may hold an arrow word of its own.
///  The folder's arrow is read first by `arrow_between_whitespace`, which is what a `uses` line needs,
///  and a `file-uses` line needs this one so a name such as `src/x -> y` stays part of the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastArrowBetweenWhitespace = (text) => {
///   let found = undefined
///   for (let at = text.indexOf('->'); at !== -1; at = text.indexOf('->', at + 1))
///     if (isWhitespace(text[at - 1]) && isWhitespace(text[at + 2])) found = at
///   return found
/// }
/// ```
fn last_arrow_between_whitespace(text: &str) -> Option<usize> {
    // What: where the last set-off arrow found so far begins, or nothing when none is.
    // Why: the loop keeps the latest answer rather than stopping at the first.
    let mut found: Option<usize> = None;
    for (index, _arrow) in text.match_indices(ARROW) {
        if set_off_by_whitespace(text, index) {
            found = Some(index);
        }
    }
    return found;
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
    // Gotcha: neither `?` can give nothing here: `arrow_between_whitespace` answered only where a whitespace character stands on both sides of the arrow, so the two strips below always find one.
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
    column: usize,
    text: &str,
) -> Result<(&str, &str), String> {
    // What: the line as every sentence below quotes it, and the sentence about a character a reader cannot see.
    // Why: a refusal that printed such a character would show a line other than the one the file holds, and only a line that holds one pays for the note.
    let spelled: String = spelled_unreadable(text);
    let note: String = unseen_character_note(text, column);
    // What: the two names the line sets off from its arrow, or nothing when it sets no arrow off with whitespace.
    // Why: one rule reads every spelling, so a tab or a no-break space beside a name is read as a space is.
    let Some((before_arrow, after_arrow)) = arrow_sides(text) else {
        return Err(format!(
            "Line {line} reads `{spelled}`. A use between two folders is written `uses <folder> {ARROW} <folder>`, with one whitespace character after the word `uses`, one before the arrow and one after it, and a folder name on each side.{note}"
        ));
    };
    // What: whether the name after the arrow holds an arrow word of its own.
    // Why: the reader takes the first arrow as the line's own, so a second one names a folder no tree holds.
    if arrow_between_whitespace(after_arrow).is_some() {
        return Err(format!(
            "Line {line} reads `{spelled}`. A use between two folders is written `uses <folder> {ARROW} <folder>`, and no folder name holds an arrow word: two `{ARROW}` are set off by whitespace.{note}"
        ));
    }
    // What: the line before the arrow, which begins with the word every `uses` line starts with.
    // Why: the caller dispatched on that word, so only the name and its separator can be missing here.
    let Some(after_the_word) = before_arrow.strip_prefix(FOLDER_USE_WORD) else {
        // What: a refusal no line reaches: the caller dispatched on this word as the line's first word.
        // Why: it stays a sentence rather than a panic, so a caller that one day does not dispatch still fails with an answer instead of a crash.
        return Err(format!(
            "Line {line} reads `{spelled}`. A use between two folders is written `uses <folder> {ARROW} <folder>`.{note}"
        ));
    };
    // What: the name before the arrow, with the one whitespace character that sets it off the word taken away.
    // Why: what stands between the word and the arrow is the name, so no whitespace there means no name.
    // Gotcha: `uses -> src/b` reaches the default, because the arrow's own whitespace is all that follows the word.
    let from: &str = after_the_word
        .strip_prefix(char::is_whitespace)
        .unwrap_or_default();
    // What: a line that sets its word and its arrow off with two whitespace characters, which leaves no name at all.
    // Why: the reader takes exactly one character as the separator, so what stands between them is the name, and nothing is no folder.
    if from.is_empty() {
        return Err(format!(
            "Line {line} reads `{spelled}`. No folder name stands before the arrow: a use between two folders is written `uses <folder> {ARROW} <folder>`.{note}"
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
    // What: the package folder every line is read against, listed at most once however many lines ask.
    // Why: a line's folder is read back against the package folder, and a six-word line asks how long its longest name is.
    let package: PackageFolder<'_> = PackageFolder::at(base);
    for (index, raw) in text.lines().enumerate() {
        let line: usize = index + 1;
        let trimmed: &str = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // What: the column the entry starts at, counted in characters from the line's first one.
        // Why: the reader trims the whitespace around a line, and counting from the trimmed text would be off by the indent.
        let column: usize = raw.chars().count() - raw.trim_start().chars().count() + 1;
        let entry: AllowEntry = entry_of(line, column, trimmed, &package)?;
        if !seen.insert(entry.identity()) {
            // What: the repeated line as the sentence quotes it, with every character a reader cannot see spelled.
            // Why: a line copied from another file can hold a mark that prints as nothing, and the quote would look like the line above it.
            let spelled: String = spelled_unreadable(entry.text.as_str());
            return Err(format!(
                "Line {line} repeats the entry `{spelled}` that an earlier line already holds.{}",
                unseen_character_note(entry.text.as_str(), column)
            ));
        }
        entries.push(entry);
    }
    return Ok(entries);
}

/// What:
///  How much work one read did on the names a gap could have left beside a folder and on the names the package folder holds,
///  for the controls that pin the cost.
/// Why:
///  The gap is bounded only by the list file's four mebibytes, so the read must stay linear in the line,
///  and a control that timed it would read this machine's load rather than the code,
///  so a control counts the names the file system was asked about and the listings of the package folder,
///  and fails when either grows with the gap or with the number of lines.
///  Test builds only, so a release artifact carries no counter and no branch on one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let candidateNames = 0;
/// let folderListings = 0;
/// ```
#[cfg(test)]
pub(crate) mod counted_work {
    /// Import the cell the counters live in.
    use std::cell::Cell;
    // What: one counter per thread, because the test runner runs several tests at once in one process.
    // Why: a counter shared between tests would add two tests' work together and make both controls meaningless.
    thread_local! {
        /// The names of a written folder that one read asked the file system about.
        static CANDIDATE_NAMES: Cell<usize> = const { Cell::new(0) };
        /// The times a read listed a folder to learn the longest name it holds.
        static FOLDER_LISTINGS: Cell<usize> = const { Cell::new(0) };
        /// The times a read listed a folder to learn whether it holds a module file.
        static MODULE_FILE_LISTINGS: Cell<usize> = const { Cell::new(0) };
        /// The names a reading built out of a written folder text.
        static OFFERED_NAMES: Cell<usize> = const { Cell::new(0) };
    }

    /// Count one name a read asked the file system about.
    pub(crate) fn note_candidate_name() {
        CANDIDATE_NAMES.with(|count| return count.set(count.get() + 1));
    }

    /// How many names the reads of this thread asked the file system about.
    pub(crate) fn candidate_names() -> usize {
        return CANDIDATE_NAMES.with(|count| return count.get());
    }

    /// Count one name a reading built out of a written folder text.
    pub(crate) fn note_offered_name() {
        OFFERED_NAMES.with(|count| return count.set(count.get() + 1));
    }

    /// How many names the readings of this thread built out of a written folder text.
    pub(crate) fn offered_names() -> usize {
        return OFFERED_NAMES.with(|count| return count.get());
    }

    /// Count one listing of a folder for the longest name it holds.
    pub(crate) fn note_folder_listing() {
        FOLDER_LISTINGS.with(|count| return count.set(count.get() + 1));
    }

    /// How many times the reads of this thread listed a folder for the longest name it holds.
    pub(crate) fn folder_listings() -> usize {
        return FOLDER_LISTINGS.with(|count| return count.get());
    }

    /// Count one listing of a folder for whether it holds a module file.
    pub(crate) fn note_module_file_listing() {
        MODULE_FILE_LISTINGS.with(|count| return count.set(count.get() + 1));
    }

    /// How many times the reads of this thread listed a folder for whether it holds a module file.
    pub(crate) fn module_file_listings() -> usize {
        return MODULE_FILE_LISTINGS.with(|count| return count.get());
    }

    /// Start one control from nothing counted.
    pub(crate) fn reset() {
        CANDIDATE_NAMES.with(|count| return count.set(0));
        FOLDER_LISTINGS.with(|count| return count.set(0));
        MODULE_FILE_LISTINGS.with(|count| return count.set(0));
        OFFERED_NAMES.with(|count| return count.set(0));
    }
}
