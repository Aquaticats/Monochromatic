//! What:
//!  Read the list of files that a folder's top doc carries.
//! Why:
//!  The top doc is the `//!` lines at the start of the folder's `mod.rs`.
//!  The file rule asks it to list the folder's files,
//! each after the files it uses.
//!  The format is plain enough for someone who knows only `git commit` and `git push`:
//!  a marker sentence,
//! then one line per file,
//!  and everything else in the doc is free text that the rule never reads.
//!  The marker is read as prose:
//! the Markdown rule breaks a sentence after its comma,
//!  so the marker may wrap anywhere a sentence may,
//!  and a blank line inside it ends the paragraph,
//! so the marker is then not found.
//!  A doc that spells the marker nearly is read as having no list at all,
//! so the reader also records where it nearly wrote the marker,
//!  and the rule reports one finding of its own for each of those places,
//! naming the line, the words found and the marker expected.
//!
//! ```text
//! //! Files in this folder,
//! //! each using only files listed before it:
//! //!
//! //! - `saving_a_file.rs`: writes a file so a crash never leaves half of it.
//! //! - `saving_many_files.rs`: saves several files together.
//! ```
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DocEntry = { name: string; line: number };
//! type TopDoc = { markerLine?: number; entries: DocEntry[]; firstLine: number };
//! function readTopDoc(text: string): TopDoc;
//! ```

/// The sentence that starts the file list in a top doc,
/// which the doc may wrap across lines.
pub const LIST_MARKER: &str = "Files in this folder, each using only files listed before it:";

/// What:
///  One place where a top doc nearly spells the marker,
///  which is why it has no file list at all.
/// Why:
///  The findings about a doc with no list say what it owes its reader
///  and never why the list was not read,
///  so this record is what a finding that names the cause is built from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NearMiss = { line: number; continuesAt?: number; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NearMiss {
    /// One-based line where the near-miss words start.
    pub line: usize,
    /// One-based line where the words go on after a blank line,
    ///  a plain comment or an inner attribute,
    ///  or nothing when the words sit in one paragraph.
    pub continues_at: Option<usize>,
    /// The words found, with every run of whitespace written as one space.
    pub text: String,
}

/// What:
///  One listed file.
/// Why:
///  Findings about an entry point at its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocEntry = { name: string; line: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocEntry {
    /// The file name between the backticks,
    ///  such as `saving_a_file.rs`.
    pub name: String,
    /// One-based line of the entry in the file.
    pub line: usize,
}

/// What:
///  What a top doc says about the folder's files.
/// Why:
///  No marker means the doc has no list at all,
///  which is its own finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TopDoc = { markerLine?: number; entries: DocEntry[]; firstLine: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TopDoc {
    /// One-based line of the marker,
    ///  or nothing when the doc has none.
    pub marker_line: Option<usize>,
    /// Every place where the doc nearly wrote the marker, in the order the doc writes them.
    ///  Empty when the doc holds the marker or nothing close to it.
    pub near_misses: Vec<NearMiss>,
    /// The listed files in order.
    pub entries: Vec<DocEntry>,
    /// One-based line of the first `//!` line,
    ///  or one when the file has none.
    pub first_line: usize,
}

/// What:
///  The file name inside the first pair of backticks of a list line.
/// Why:
///  A list line is `- ` then the name in backticks,
///  then anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryName(text: string): string | undefined;
/// ```
fn entry_name(text: &str) -> Option<String> {
    let after_dash: &str = text.trim_start().strip_prefix("- `")?;
    let end: usize = after_dash.find('`')?;
    // What: the name with the whitespace at its end taken off.
    // Why: a list line is read without its ends,
    // so an entry written `x.rs ` names `x.rs`,
    // and no module file can hold a name that ends in one of those characters anyway.
    return Some(String::from(after_dash[..end].trim_end()));
}

/// What:
///  How many leading lines of the given doc lines make up the list marker.
/// Why:
///  The Markdown rule breaks prose after a comma,
///  so the marker may wrap anywhere a sentence may.
///  The lines are read as one paragraph:
///  a blank line,
///  a plain comment or a different word ends it,
///  and any run of spaces matches one space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markerLineCount(markerWords: string[], bodies: string[]): number | undefined;
/// ```
fn marker_line_count(
    marker_words: &[&str],
    bodies: &[(usize, &str)],
) -> Option<usize> {
    let mut matched: usize = 0;
    for (covered, (_, body)) in bodies.iter().enumerate() {
        // The words of this line, with every run of spaces gone.
        let words: Vec<&str> = body.split_whitespace().collect::<Vec<&str>>();
        // This line must be the next words of the marker, so a blank line or a different word ends the try.
        if words.is_empty()
            || marker_words.get(matched..matched + words.len()) != Some(words.as_slice())
        {
            // The marker is not here.
            return None;
        }
        matched += words.len();
        if matched == marker_words.len() {
            // The marker ended exactly at the end of this line.
            return Some(covered + 1);
        }
    }
    // The doc ended inside the marker.
    return None;
}

/// What:
///  Where the first marker starts among the doc lines and how many lines it covers.
/// Why:
///  The marker may begin at any line,
///  so every line is tried as its start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findMarker(markerWords: string[], bodies: string[]): { start: number; covered: number } | undefined;
/// ```
fn find_marker(
    marker_words: &[&str],
    bodies: &[(usize, &str)],
) -> Option<(usize, usize)> {
    for start in 0..bodies.len() {
        if let Some(covered) = marker_line_count(marker_words, &bodies[start..]) {
            // The marker starts at this line and covers `covered` lines.
            return Some((start, covered));
        }
    }
    // No line starts the marker.
    return None;
}

/// What:
///  Whether two word lists hold the same words
///  when the capital letters are ignored.
/// Why:
///  A doc that writes the marker in another case is a near miss,
///  and the marker's own words are what such a doc has to match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sameWords(left: string[], right: string[]): boolean;
/// ```
fn same_words(
    left: &[&str],
    right: &[&str],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (one, other) in left.iter().zip(right.iter()) {
        // What: `eq_ignore_ascii_case` compares two words without their capital letters.
        // Why: a doc that writes `Files` as `files` is the same sentence to a reader.
        if !one.eq_ignore_ascii_case(other) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the longer word list is the shorter one
///  with exactly one word taken out of it,
///  with the capital letters ignored.
/// Why:
///  One word added and one word dropped are the two slips of the pen,
///  and either one leaves the doc unread as a list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneWordRemoved = (longer, shorter) =>
///   longer.length === shorter.length + 1 &&
///   longer.some((_, skipped) => sameWords(longer.filter((_, at) => at !== skipped), shorter));
/// ```
fn one_word_removed(
    longer: &[&str],
    shorter: &[&str],
) -> bool {
    if longer.len() != shorter.len() + 1 {
        return false;
    }
    for skipped in 0..longer.len() {
        // What: the longer list with the word at this position left out.
        // Why: the added or dropped word may sit anywhere, so each position is tried.
        let mut rest: Vec<&str> = Vec::<&str>::new();
        for (index, word) in longer.iter().enumerate() {
            if index == skipped {
                continue;
            }
            // What: `*word` is the borrowed word itself rather than a reference to it.
            // Why: the list holds the words, not references into the longer list.
            rest.push(*word);
        }
        if same_words(rest.as_slice(), shorter) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a word list is the marker's words
///  with at most one word added, dropped or written in another case.
/// Why:
///  These are the slips a person makes when copying the sentence,
///  and each one leaves the doc with no list at all.
///  A text two words away is a different sentence that its author wrote on purpose,
///  so no finding asks for it to be changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oneWordApart(found: string[], marker: string[]): boolean;
/// ```
fn one_word_apart(
    found: &[&str],
    marker: &[&str],
) -> bool {
    if found.len() == marker.len() {
        // What: how many positions hold a word that differs once the capital letters are ignored.
        // Why: no difference is the marker in another case, and one is a word put in place of another.
        let mut different: usize = 0;
        for (left, right) in found.iter().zip(marker.iter()) {
            if !left.eq_ignore_ascii_case(right) {
                different += 1;
            }
        }
        return different <= 1;
    }
    if found.len() + 1 == marker.len() {
        // What: a word of the marker is missing from the doc.
        // Why: the marker is the longer list and the doc's words are the shorter.
        return one_word_removed(marker, found);
    }
    // What: a word the marker does not hold stands in the doc.
    // Why: the doc's words are the longer list here.
    return one_word_removed(found, marker);
}

/// What:
///  Some doc lines as one text:
///  the words of each line in order, with one space between them.
/// Why:
///  The marker reader matches words and never spacing,
///  so a wrap onto the next line and a run of spaces between words are one space,
///  and this text is also what a finding shows its reader.
///  The words are copied into a new text, so the answer borrows nothing from the doc.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesText = (lines) => lines.flatMap((line) => line.body.split(/\s+/).filter(Boolean)).join(' ');
/// ```
fn lines_text(lines: &[(usize, &str)]) -> String {
    let mut words: Vec<&str> = Vec::<&str>::new();
    for &(_, body) in lines {
        let line_words: Vec<&str> = body.split_whitespace().collect::<Vec<&str>>();
        // What: `extend` moves every word of this line into the list.
        // Why: the lines are read in the order the doc writes them.
        words.extend(line_words);
    }
    return words.join(" ");
}

/// What:
///  How much work one run of the near-miss search did,
///  for the control that pins its cost.
/// Why:
///  A cost has no test of its own,
///  and a control that timed one would read this machine's load rather than the code,
///  so a control counts the doc lines the search read instead and fails when one line is read once per line after it.
///  Test builds only, so a release artifact carries no counter and no branch on one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let linesRead = 0;
/// ```
#[cfg(test)]
pub(crate) mod counted_work {
    /// Import the cell the counter lives in.
    use std::cell::Cell;
    // What: one counter per thread, because the test runner runs several tests at once in one process.
    // Why: a counter shared between tests would add two tests' work together and make both controls meaningless;
    // the siblings a reader might expect are a plain `static` and a lock, and neither keeps the tests apart.
    thread_local! {
        /// The doc lines the near-miss search read.
        static LINES_READ: Cell<usize> = const { Cell::new(0) };
    }

    /// Count one doc line the search read.
    pub(crate) fn note_line() {
        LINES_READ.with(|count| return count.set(count.get() + 1));
    }

    /// How many doc lines the search read on this thread.
    pub(crate) fn lines_read() -> usize {
        return LINES_READ.with(|count| return count.get());
    }

    /// Start one control from nothing counted.
    pub(crate) fn reset() {
        LINES_READ.with(|count| return count.set(0));
    }
}

/// What:
///  The near miss that starts at one doc line and covers whole lines after it,
///  with the number of lines it covers,
///  or nothing when the words there are not within one word of the marker.
/// Why:
///  A near miss starts where a line starts, as the marker does,
///  and it ends at the line where its words reach the marker's length,
///  so the list lines and the prose below it are never read as part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nearMissAt(bodies, start, markerWords): { near: NearMiss; covered: number } | undefined;
/// ```
fn near_miss_at(
    bodies: &[(usize, &str)],
    start: usize,
    marker_words: &[&str],
) -> Option<(NearMiss, usize)> {
    for end in start..bodies.len() {
        // What: one doc line this attempt read, which a control counts.
        // Why: an attempt that reads past the marker's own length is no attempt, so a control holds that it stops there.
        #[cfg(test)]
        counted_work::note_line();
        if bodies[end].1.trim().is_empty() {
            // What: `None` is the absent variant of `Option`.
            // Why: the words of an unsplit near miss sit in one paragraph, so a blank line, a plain comment or an inner attribute ends the attempt.
            return None;
        }
        let text: String = lines_text(&bodies[start..=end]);
        let words: Vec<&str> = text.split_whitespace().collect::<Vec<&str>>();
        if words.len() > marker_words.len() + 1 {
            // What: more words than the marker holds plus the one word it may be off by.
            // Why: a line further on only adds words, so no later end can match either.
            return None;
        }
        if one_word_apart(words.as_slice(), marker_words) {
            // What: `Some(...)` is the present variant of `Option`, wrapping the record and the line count.
            // Why: the words here nearly hold the marker, so the finding names this line and these words.
            return Some((
                NearMiss {
                    line: bodies[start].0,
                    continues_at: None,
                    text,
                },
                end - start + 1,
            ));
        }
    }
    return None;
}

/// What:
///  The near miss whose words a paragraph break splits,
///  with the number of lines it covers,
///  or nothing when the two halves are not the marker's words.
/// Why:
///  A blank `//!` line, a plain comment or an inner attribute inside the marker
///  ends the paragraph the marker reader joins,
///  so the doc spells the marker with a break in it.
///  Both halves are read exactly, because a break is not a word,
///  and the words before the break may wrap over several lines.
///  The start line must hold words,
///  because the words before the break are the marker's first words,
///  so a start on a blank line is rejected before the run of blank lines after it is read at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitNearMissAt(bodies, start, markerWords): { near: NearMiss; covered: number } | undefined;
/// ```
fn split_near_miss_at(
    bodies: &[(usize, &str)],
    start: usize,
    marker_words: &[&str],
) -> Option<(NearMiss, usize)> {
    // What: the start line holds no word, so the words before the break are empty and cannot be the marker's first words.
    // Why: the walk below over the run of blank lines after the start found that the first half was empty only after reading the rest of the run,
    // so a doc of N blank lines cost the rest of the run once for each of its lines, which is N squared reads of one file.
    if bodies[start].1.trim().is_empty() {
        // What: `None` is the absent variant of `Option`.
        // Why: a start on a blank line is no split near miss, with the same answer the walk gave.
        return None;
    }
    let mut break_at: Option<usize> = None;
    for end in start..bodies.len() {
        // What: one doc line this attempt read, which a control counts.
        // Why: more words than the marker holds cannot be its first words, so a control holds that the read stops there.
        #[cfg(test)]
        counted_work::note_line();
        if bodies[end].1.trim().is_empty() {
            break_at = Some(end);
            break;
        }
        // What: the words of the first half so far, which must be the marker's first words.
        // Why: more words than the marker holds cannot be its first words, so the attempt ends.
        let so_far: String = lines_text(&bodies[start..=end]);
        if so_far.split_whitespace().count() > marker_words.len() {
            return None;
        }
    }
    // What: `break_at?` hands over the line the break sits on, or ends this function with `None`.
    // Why: a run of lines with no break in it is no split near miss.
    let end: usize = break_at?;
    // What: the line the words go on at, the first line after the break.
    // Why: one or more blank lines, plain comments or inner attributes may sit between the two halves.
    let mut next: usize = end + 1;
    while next < bodies.len() && bodies[next].1.trim().is_empty() {
        // What: one blank line this attempt stepped over, which a control counts.
        // Why: a start that can never match walked the rest of its blank run here, so a control holds that this stops.
        #[cfg(test)]
        counted_work::note_line();
        next += 1;
    }
    if next >= bodies.len() {
        return None;
    }
    let first_text: String = lines_text(&bodies[start..end]);
    let first_words: Vec<&str> = first_text.split_whitespace().collect::<Vec<&str>>();
    // What: how many of the marker's words stand before the break.
    // Why: the doc may break the marker after any of its words, and the first half fixes which break this is.
    let before_break: usize = first_words.len();
    if before_break == 0
        || before_break >= marker_words.len()
        || !same_words(first_words.as_slice(), &marker_words[..before_break])
    {
        return None;
    }
    for second_end in next..bodies.len() {
        // What: one doc line this attempt read, which a control counts.
        // Why: the second half is the rest of the marker, so the read stops once it holds more words than that.
        #[cfg(test)]
        counted_work::note_line();
        if bodies[second_end].1.trim().is_empty() {
            return None;
        }
        let second_text: String = lines_text(&bodies[next..=second_end]);
        let second_words: Vec<&str> = second_text.split_whitespace().collect::<Vec<&str>>();
        if second_words.len() > marker_words.len() - before_break {
            return None;
        }
        if same_words(second_words.as_slice(), &marker_words[before_break..]) {
            return Some((
                NearMiss {
                    line: bodies[start].0,
                    continues_at: Some(bodies[next].0),
                    text: format!("{first_text} {second_text}"),
                },
                second_end + 1 - start,
            ));
        }
    }
    return None;
}

/// What:
///  Every place where a top doc nearly wrote the marker,
///  in the order the doc writes them.
/// Why:
///  One doc can hold more than one attempt,
///  and each attempt is a finding of its own with a key of its own.
///  The lines one attempt covers are never searched again,
///  so one text is one finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nearMisses(bodies, markerWords): NearMiss[];
/// ```
fn near_misses(
    bodies: &[(usize, &str)],
    marker_words: &[&str],
) -> Vec<NearMiss> {
    let mut near: Vec<NearMiss> = Vec::<NearMiss>::new();
    let mut index: usize = 0;
    while index < bodies.len() {
        if let Some((found, covered)) = near_miss_at(bodies, index, marker_words) {
            near.push(found);
            index += covered;
            continue;
        }
        if let Some((found, covered)) = split_near_miss_at(bodies, index, marker_words) {
            near.push(found);
            index += covered;
            continue;
        }
        index += 1;
    }
    return near;
}

/// What:
///  Read the top doc of a Rust file.
/// Why:
///  Only the leading `//!` lines count;
///  blank lines,
/// ordinary comments and inner attributes between them are skipped,
///  and the first other line ends the doc.
///  A skipped line still ends a paragraph,
///  so it never joins two halves of the marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTopDoc(text: string): TopDoc;
/// ```
pub fn read_top_doc(text: &str) -> TopDoc {
    let mut doc: TopDoc = TopDoc {
        first_line: 1,
        ..TopDoc::default()
    };
    let mut seen_doc: bool = false;
    let mut bodies: Vec<(usize, &str)> = Vec::<(usize, &str)>::new();
    // What: the marker's words, split once here rather than once per line tried and once per paragraph read.
    // Why: the search for the marker and the search for a near miss compare the same words.
    let marker_words: Vec<&str> = LIST_MARKER.split_whitespace().collect::<Vec<&str>>();
    for (index, line) in text.lines().enumerate() {
        let number: usize = index + 1;
        let trimmed: &str = line.trim();
        let Some(body) = trimmed.strip_prefix("//!") else {
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#![") {
                if seen_doc {
                    bodies.push((number, ""));
                }
                continue;
            }
            break;
        };
        if !seen_doc {
            seen_doc = true;
            doc.first_line = number;
        }
        bodies.push((number, body));
    }
    let Some((start, covered)) = find_marker(marker_words.as_slice(), &bodies) else {
        // What: the doc has no list, so record every place where it nearly wrote the marker.
        // Why: the finding about a near miss names the cause, where the missing list names only its effect.
        doc.near_misses = near_misses(bodies.as_slice(), marker_words.as_slice());
        return doc;
    };
    doc.marker_line = Some(bodies[start].0);
    for &(number, body) in &bodies[start + covered..] {
        if body.trim().is_empty() || body.starts_with("  ") {
            continue;
        }
        match entry_name(body) {
            Some(name) => doc.entries.push(DocEntry {
                name,
                line: number,
            }),
            None => break,
        }
    }
    return doc;
}

/// Controls of the top doc reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_top_doc_tests.rs"]
mod tests;
