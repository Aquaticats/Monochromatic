//! The scan a ban over part of the release tree reads.
//! What:
//!  Which release sources a ban covers,
//!  which of their code lines hold the text it forbids or call the name it forbids,
//!  which entries excuse one,
//!  and the anchor that proves the scope is still the code the ban is about.
//! Why:
//!  Every ban over a part of the tree answers those four questions,
//!  and the alpha.48 splits left several lists that no longer named the files the code moved to
//!  (issue 6599),
//!  so a ban states its scope as prefixes and reads the tree through the one walk,
//!  and a file a later split makes inside the scope is covered without anyone remembering it.
//!  A ban on a call reads the name at the boundary of the name,
//!  because the name alone is the tail of longer names a file may use freely.
//!  An excuse names how many uses of its text the one file may hold,
//!  because an entry that named only the file and the text covered every later use of it too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesNoEntryExcuses(linesHoldingOneOf(releaseSourcesUnder(SCOPE), BANNED), MAY_HOLD)).toEqual([]);
//! expect(linesNoEntryExcuses(linesCallingOneOf(releaseSourcesUnder(SCOPE), CALLS), MAY_HOLD)).toEqual([]);
//! expect(anchorsTheScopeNoLongerHolds(releaseSourcesUnder(SCOPE), ANCHORS)).toEqual([]);
//! ```

use super::release_sources_in;
use super::source_scan_tests;

/// What:
///  `release_sources_under(prefixes)` is every release source whose `/`-separated identity under
///  `src/native` begins with one of `prefixes`,
///  each read as its identity and its text,
///  one file at a time.
/// Why:
///  A ban that holds over a part of the tree names that part as a prefix,
///  so a file a split makes inside it is covered without anyone remembering to add it
///  (issue 6599).
///  The files are read through the one walk,
///  so a `uses` link is entered once and a test-only sidecar is never mistaken for release code.
/// Gotcha:
///  A prefix that matches nothing answers an empty list,
///  which passes every ban,
///  so each ban names an anchor its scope must still hold.
pub(crate) fn release_sources_under(prefixes: &[&str]) -> Result<Vec<(String, String)>, String> {
    let root: std::path::PathBuf = source_scan_tests::source_root();
    let roots: Vec<source_scan_tests::CrateRoot> = source_scan_tests::real_roots();
    let mut found: Vec<(String, String)> = Vec::new();
    for path in release_sources_in(root.as_path(), roots.as_slice())? {
        if !prefixes
            .iter()
            .any(|prefix| return path.starts_with(prefix))
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(path.as_str()))
            .map_err(|error| return format!("cannot read {path}: {error}"))?;
        found.push((path, text));
    }
    return Ok(found);
}

/// What:
///  `release_sources_holding(words)` is every release source that names one of `words` anywhere in its text,
///  each as its identity and its text,
///  read one file at a time.
/// Why:
///  A ban whose subject is a lane of code rather than a folder selects its files by reading each one and keeping only
///  those that name the lane,
///  so the whole tree is never held at once
///  (issue 6598:
///  the sources are 46 MB and one guard that held them all spent 890 MiB).
///  The words are matched in comments too,
///  because a file of the lane says in its doc why it hands its lines over.
pub(crate) fn release_sources_holding(words: &[&str]) -> Result<Vec<(String, String)>, String> {
    let root: std::path::PathBuf = source_scan_tests::source_root();
    let roots: Vec<source_scan_tests::CrateRoot> = source_scan_tests::real_roots();
    let mut found: Vec<(String, String)> = Vec::new();
    for path in release_sources_in(root.as_path(), roots.as_slice())? {
        let text: String = std::fs::read_to_string(root.join(path.as_str()))
            .map_err(|error| return format!("cannot read {path}: {error}"))?;
        if !words.iter().any(|word| return text.contains(*word)) {
            continue;
        }
        found.push((path, text));
    }
    return Ok(found);
}

/// What:
///  `HoldingLine` is one code line of a release source that holds a text a ban forbids:
///  the file's identity,
///  the banned text it holds,
///  the line's number counting from one,
///  and the line.
/// Why:
///  A finding has to name the file and the line to open,
///  and the allow list refuses by file and banned text,
///  so both travel with the line that was found.
pub(crate) struct HoldingLine {
    /// The file's `/`-separated identity under `src/native`.
    pub(crate) file: String,
    /// The banned text the line holds.
    pub(crate) text: &'static str,
    /// The line's number,
    /// counting from one.
    pub(crate) number: usize,
    /// The line as written.
    pub(crate) line: String,
}

/// What:
///  `MayHold` is one file a ban lets hold one of its texts anyway,
///  with how many uses of that text it holds and the reason it may.
/// Why:
///  A ban holds over a part of the tree and still has places it does not reach,
///  so the exception is recorded where the ban is read,
///  and a ban whose exception matched nothing fails rather than outliving the code it excused.
///  The count is what an exception names rather than the excused lines:
///  a file that gains the text fails,
///  and a file whose excused line is reflowed or moved to another line
///  keeps the count it had and passes,
///  which is the case a list of lines would fail for no reason.
///  The count is of uses,
///  so a second use written on the line that already holds one is a second use.
pub(crate) struct MayHold {
    /// The file's `/`-separated identity under `src/native`.
    pub(crate) file: &'static str,
    /// The banned text that file may hold.
    pub(crate) text: &'static str,
    /// How many uses of that text the file holds,
    /// which is the most it may hold.
    pub(crate) uses: usize,
    /// Why it may,
    /// and what removes the entry.
    pub(crate) why: &'static str,
}

/// What:
///  `lines_holding_one_of(sources, banned)` is every code line of those sources that holds one of
///  `banned`,
///  once for each text it holds.
/// Why:
///  A comment explains a ban as often as the code it rules on,
///  and a file whose doc names the forbidden text is no finding,
///  so a line that starts with `//` is not code.
/// Gotcha:
///  A text is matched wherever it occurs in a line,
///  so a ban on a call that a longer name ends in (`link(` in `hard_link(`)
///  is written through `lines_calling_one_of` instead.
pub(crate) fn lines_holding_one_of(
    sources: &[(String, String)],
    banned: &[&'static str],
) -> Vec<HoldingLine> {
    let mut found: Vec<HoldingLine> = Vec::new();
    for (file, text) in sources {
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for forbidden in banned {
                if !line.contains(*forbidden) {
                    continue;
                }
                found.push(HoldingLine {
                    file: file.clone(),
                    text: forbidden,
                    number: index + 1,
                    line: String::from(line.trim()),
                });
            }
        }
    }
    return found;
}

/// What:
///  `lines_calling_one_of(sources, calls)` is every code line of those sources that calls one of `calls`,
///  once for each call it makes.
///  A call is written as its name and its `(`,
///  and it is read where the byte before the name cannot continue a name.
/// Why:
///  A ban on a call has to fail on every spelling of it (`fs::symlink(`,
///  `libc::symlink(`,
///  and an import's bare
///  `symlink(`),
///  and the name alone is the tail of names a file may use freely:
///  `link(` ends `hard_link(`,
///  `read_link(` and `unlink(`,
///  and `symlink(` ends `is_symlink(`.
///  Reading the byte before the name is what tells the call from the longer name,
///  so one token bans the call without banning the names that end in it.
/// Gotcha:
///  A byte that is not ASCII counts as part of a name,
///  so a name written in another script is never read as the start of a call.
///  A call whose name and `(` are split across two lines is not read:
///  the scan reads one line at a time.
pub(crate) fn lines_calling_one_of(
    sources: &[(String, String)],
    calls: &[&'static str],
) -> Vec<HoldingLine> {
    let mut found: Vec<HoldingLine> = Vec::new();
    for (file, text) in sources {
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for call in calls {
                if !calls_at(line, call) {
                    continue;
                }
                found.push(HoldingLine {
                    file: file.clone(),
                    text: call,
                    number: index + 1,
                    line: String::from(line.trim()),
                });
            }
        }
    }
    return found;
}

/// What:
///  `calls_at(line, call)` says whether that line calls `call`,
///  which is the case wherever its text occurs with no byte of a name before it.
/// Why:
///  One rule answers for every spelling,
///  so the ban lists the name and this function holds the one reading of what makes it a call.
fn calls_at(
    line: &str,
    call: &str,
) -> bool {
    let bytes: &[u8] = line.as_bytes();
    let mut from: usize = 0;
    while let Some(at) = line[from..].find(call) {
        let start: usize = from + at;
        let before_is_a_name: bool = match start.checked_sub(1) {
            None => false,
            Some(index) => continues_a_name(bytes[index]),
        };
        if !before_is_a_name {
            return true;
        }
        from = start + call.len();
    }
    return false;
}

/// What:
///  `continues_a_name(byte)` says whether that byte could be the middle of a Rust name.
/// Why:
///  A call begins where a name begins,
///  so a byte that could be inside a name means the text after it is the tail of a longer name.
///  Every byte of a character outside ASCII is counted as inside a name:
///  a name this scan cannot spell in ASCII is then read as a name and never as the start of a call,
///  which keeps a ban on `link(` from reading a name in another script that merely ends in it.
fn continues_a_name(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii();
}

/// What:
///  `lines_no_entry_excuses(found, may_hold)` is every holding line no `may_hold` entry covers,
///  each as `file:line (banned text) the line`.
///  An entry covers a line only where its file,
///  its text and its count all hold:
///  the file holds exactly as many uses of that text as the entry names,
///  where a line that holds the text twice counts twice.
/// Why:
///  This is the finding list a ban's control asserts empty,
///  and it names the line and the text so a reader opens the place rather than searching for it.
///  An entry that named only the file and the text excused a second use of that text
///  added beside the one it recorded,
///  so the count is asked of both sides and a line whose count disagrees is a finding
///  that says what the file holds and what the entry names.
///  The uses are counted rather than the lines,
///  because a second use written on the line that already holds one is a second use.
pub(crate) fn lines_no_entry_excuses(
    found: &[HoldingLine],
    may_hold: &[MayHold],
) -> Vec<String> {
    let mut findings: Vec<String> = Vec::new();
    for holding in found {
        let held: usize = found
            .iter()
            .filter(|other| {
                return other.file == holding.file && other.text == holding.text;
            })
            .map(|other| return other.line.matches(holding.text).count())
            .sum();
        let entry: Option<&MayHold> = may_hold.iter().find(|entry| {
            return entry.file == holding.file && entry.text == holding.text;
        });
        match entry {
            Some(named) if named.uses == held => continue,
            None => findings.push(format!(
                "{}:{} ({}) {}",
                holding.file, holding.number, holding.text, holding.line
            )),
            Some(named) => findings.push(format!(
                "{}:{} ({}) {} [the file holds {} use(s) of this text and the entry beside it names {}]",
                holding.file, holding.number, holding.text, holding.line, held, named.uses
            )),
        }
    }
    return findings;
}

/// What:
///  `entries_that_match_nothing(found, may_hold)` is every `may_hold` entry no holding line matches,
///  each as `file (text): why`.
/// Why:
///  An exception whose code went still excuses the whole file,
///  so the list may only shrink and a stale entry fails.
///  An entry that names more uses than the file holds is caught by `lines_no_entry_excuses`,
///  which asks both sides for the count.
pub(crate) fn entries_that_match_nothing(
    found: &[HoldingLine],
    may_hold: &[MayHold],
) -> Vec<String> {
    return may_hold
        .iter()
        .filter(|entry| {
            return !found.iter().any(|holding| {
                return entry.file == holding.file && entry.text == holding.text;
            });
        })
        .map(|entry| {
            return format!("{} ({}): {}", entry.file, entry.text, entry.why);
        })
        .collect();
}

/// What:
///  `anchors_the_scope_no_longer_holds(sources, anchors)` is every `(text, why)` of `anchors` that none of those
///  sources holds,
///  each as `text: why`.
/// Why:
///  A ban scoped by a prefix passes every tree when the prefix reaches nothing,
///  as it does when a split or a rename empties it,
///  so a ban states the text that proves its scope is still the code it is about.
pub(crate) fn anchors_the_scope_no_longer_holds(
    sources: &[(String, String)],
    anchors: &[(&'static str, &'static str)],
) -> Vec<String> {
    return anchors
        .iter()
        .filter(|(text, _)| {
            return !sources.iter().any(|(_file, source)| {
                return source.contains(*text);
            });
        })
        .map(|(text, why)| {
            return format!("{text}: {why}");
        })
        .collect();
}
