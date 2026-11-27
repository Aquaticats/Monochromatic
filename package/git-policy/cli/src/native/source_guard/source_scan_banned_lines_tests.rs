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
//!  A ban on a call reads the name where the line spells the call:
//!  the last segment of a path (`fs::symlink`),
//!  the call (`symlink(`),
//!  and the call with its types written out (`symlink::<_, _>(`),
//!  so a rename's import and a function taken as a value are findings too,
//!  and the name is read only where no byte of a longer name touches it.
//!  A ban on a name no other use shares is read wherever the line writes it,
//!  because a type is written in a signature and in a `use` list as often as at a call.
//!  An excuse names the one use it covers by the line that use is written on,
//!  because an entry that named only the file and the text excused a second use of that text
//!  added beside the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesNoEntryExcuses(linesHoldingOneOf(sources, TEXTS), MAY_HOLD)).toEqual([]);
//! expect(linesNoEntryExcuses(linesCallingOneOf(sources, CALLS), MAY_HOLD)).toEqual([]);
//! expect(linesNoEntryExcuses(linesNamingOneOf(sources, NAMES), MAY_HOLD)).toEqual([]);
//! expect(anchorsTheScopeNoLongerHolds(sources, ANCHORS)).toEqual([]);
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
///  `MayHold` is one use a ban lets stand anyway:
///  the file it is written in,
///  the banned text or name it holds,
///  the line it is written on,
///  and the reason it may.
/// Why:
///  A ban holds over a part of the tree and still has places it does not reach,
///  so the exception is recorded where the ban is read,
///  and a ban whose exception matched nothing fails rather than outliving the code it excused.
///  The line is what an entry names rather than a count of uses:
///  a use removed and another of the same text added to the same file leaves the line this entry names
///  holding nothing,
///  so the swap is a finding here and a stale entry there,
///  where a count would have kept the file excused.
///  An entry covers one use,
///  so a second line written exactly like an excused one needs an entry of its own.
/// Gotcha:
///  The line is the one as written,
///  without its indentation,
///  so `rustfmt` reflowing it makes the entry stale and the ban reports it;
///  the reflow and the entry are one commit's work.
pub(crate) struct MayHold {
    /// The file's `/`-separated identity under `src/native`.
    pub(crate) file: &'static str,
    /// The banned text or name that file may hold.
    pub(crate) text: &'static str,
    /// The line that use is written on,
    /// as written without its indentation.
    pub(crate) line: &'static str,
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
///  so a ban on a call that a longer name ends in (`link` in `hard_link`)
///  is written through `lines_calling_one_of` instead,
///  and a ban on a name through that or
///  `lines_naming_one_of`.
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
///  `lines_calling_one_of(sources, calls)` is every code line of those sources that spells one of `calls`,
///  once for each call it spells.
///  A call is spelled as its bare name (`symlink(`),
///  as the last segment of a path (`fs::symlink`),
///  or with its types written out (`symlink::<`,
///  `),
///  and it is read only where no byte of a longer name touches the name.
/// Why:
///  A ban on a call has to fail on every spelling of it in one line,
///  and each of those spellings needs the name in scope somewhere:
///  at the call,
///  in the `use` that brings it in,
///  or in the value the function is taken as.
///  So the path's last segment is a finding too:
///  a rename's import (`use std::os::unix::fs::symlink as make_link;`) and a function taken as a value
///  (`let f = std::os::unix::fs::symlink;`) spell the path,
///  while the call they then make (`make_link(a,
///  b)`) cannot be told from any other call.
///  The name alone is the tail of names a file may use freely:
///  `link` ends `hard_link`,
///  `read_link` and `unlink`,
///  and `symlink` ends `is_symlink`,
///  so a name byte on either side means the text is no name of its own.
/// Gotcha:
///  A macro written outside the scope that expands to the call is not read:
///  its body is in another file and the call site spells the macro's name,
///  so a ban over a scope says that in its top doc.
///  A glob import (`use super::*;`) puts a name in scope without writing it, and is named there too.
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
///  `calls_at(line, call)` says whether that line spells `call`,
///  which is the case where its name occurs with no byte of a longer name touching it
///  and the line writes it as a path's last segment,
///  a call,
///  or a call with its types written out.
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
        let end: usize = start + call.len();
        if !continues_a_name(bytes, start, end) && writes_the_call(bytes, start, end) {
            return true;
        }
        from = end;
    }
    return false;
}

/// What:
///  `lines_naming_one_of(sources, names)` is every code line of those sources that writes one of `names`,
///  once for each name it writes,
///  wherever the line writes it.
/// Why:
///  A ban whose subject is a name no other use shares (a socket type,
///  a builder,
///  a random name's function) reads it
///  everywhere,
///  because a type is written in a signature and in a `use` list as often as at a call:
///  `use std::os::unix::net::{self, UnixStream};` is a socket by the name it brings in,
///  and a ban that read only calls would miss the type position it then appears in.
///  The name is read only where no byte of a longer name touches it,
///  so `UnixStreams` is another name.
/// Gotcha:
///  A word that is also an ordinary name (`link`,
///  `stderr`) is banned through `lines_calling_one_of`
///  instead:
///  a field named `link` and a local variable named `stderr` are no link and no shared handle.
pub(crate) fn lines_naming_one_of(
    sources: &[(String, String)],
    names: &[&'static str],
) -> Vec<HoldingLine> {
    let mut found: Vec<HoldingLine> = Vec::new();
    for (file, text) in sources {
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for name in names {
                if !names_at(line, name) {
                    continue;
                }
                found.push(HoldingLine {
                    file: file.clone(),
                    text: name,
                    number: index + 1,
                    line: String::from(line.trim()),
                });
            }
        }
    }
    return found;
}

/// What:
///  `names_at(line, name)` says whether that line writes `name` where no byte of a longer name touches it.
/// Why:
///  The boundary of a name is the whole rule here,
///  so it is written once.
fn names_at(
    line: &str,
    name: &str,
) -> bool {
    let bytes: &[u8] = line.as_bytes();
    let mut from: usize = 0;
    while let Some(at) = line[from..].find(name) {
        let start: usize = from + at;
        let end: usize = start + name.len();
        if !continues_a_name(bytes, start, end) {
            return true;
        }
        from = end;
    }
    return false;
}

/// What:
///  `continues_a_name(bytes, start, end)` says whether a byte of a longer name touches the name at those offsets.
/// Why:
///  A name is read where a name begins and ends,
///  so a byte that could be inside a name means the text is the tail or the start of a longer one.
///  Every byte of a character outside ASCII counts as inside a name:
///  a name written in another script is then read as a name and never as the start of a call,
///  which keeps a ban on `link` from reading a name in another script that merely ends in it.
fn continues_a_name(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> bool {
    let before: bool = start > 0 && continues_a_name_byte(bytes[start - 1]);
    let after: bool = end < bytes.len() && continues_a_name_byte(bytes[end]);
    return before || after;
}

/// What:
///  `continues_a_name_byte(byte)` says whether that byte could be part of a Rust name.
/// Why:
///  The one question both ends of an occurrence ask.
fn continues_a_name_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii();
}

/// What:
///  `writes_the_call(bytes, start, end)` says whether those bytes write the name as a path's last segment,
///  a call with its types written out,
///  or a call.
/// Why:
///  The three spellings a line holds are the whole answer:
///  `::name` is the path a call is made by,
///  imported,
///  or taken as a value from,
///  `name::<` is the call with its types written out,
///  and `name (` is the call,
///  with the whitespace Rust allows before its arguments.
fn writes_the_call(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> bool {
    let last_segment_of_a_path: bool =
        start >= 2 && bytes[start - 1] == b':' && bytes[start - 2] == b':';
    if last_segment_of_a_path {
        return true;
    }
    if bytes[end..].starts_with(b"::<") {
        return true;
    }
    let mut at: usize = end;
    while at < bytes.len() && bytes[at] == b' ' {
        at += 1;
    }
    return at < bytes.len() && bytes[at] == b'(';
}

/// What:
///  `lines_no_entry_excuses(found, may_hold)` is every holding line no `may_hold` entry covers,
///  each as `file:line (banned text) the line`.
///  An entry covers one finding:
///  the one whose file,
///  text and line are the entry's,
///  and each entry covers one finding only.
/// Why:
///  This is the finding list a ban's control asserts empty,
///  and it names the line and the text so a reader opens the place rather than searching for it.
///  The use an entry names is the line it is written on,
///  so a use removed and another of the same text added to the same file fails twice over:
///  the line the entry named holds nothing,
///  which `entries_that_match_nothing` reports,
///  and the new use is excused by no entry here.
///  An entry covers one finding,
///  so a second line written exactly like an excused one must be excused by an entry of its own.
pub(crate) fn lines_no_entry_excuses(
    found: &[HoldingLine],
    may_hold: &[MayHold],
) -> Vec<String> {
    let chosen: Vec<Option<usize>> = the_entry_of_each_finding(found, may_hold);
    let mut findings: Vec<String> = Vec::new();
    for (at, holding) in found.iter().enumerate() {
        if chosen[at].is_some() {
            continue;
        }
        findings.push(format!(
            "{}:{} ({}) {}",
            holding.file, holding.number, holding.text, holding.line
        ));
    }
    return findings;
}

/// What:
///  `entries_that_match_nothing(found, may_hold)` is every `may_hold` entry no finding took,
///  each as `file (text) line: why`.
/// Why:
///  An exception whose code went still excuses the whole file,
///  so the list may only shrink and a stale entry fails.
///  An entry whose text stands elsewhere in the file is stale in the same way:
///  the use it named is gone,
///  which is what a swap leaves behind.
pub(crate) fn entries_that_match_nothing(
    found: &[HoldingLine],
    may_hold: &[MayHold],
) -> Vec<String> {
    let chosen: Vec<Option<usize>> = the_entry_of_each_finding(found, may_hold);
    let mut findings: Vec<String> = Vec::new();
    for (at, entry) in may_hold.iter().enumerate() {
        if chosen.iter().any(|one| return *one == Some(at)) {
            continue;
        }
        findings.push(format!(
            "{} ({}) {}: {}",
            entry.file, entry.text, entry.line, entry.why
        ));
    }
    return findings;
}

/// What:
///  `the_entry_of_each_finding(found, may_hold)` answers,
///  for each finding in order,
///  the index of the entry that covers it,
///  and gives each entry to one finding.
/// Why:
///  The two questions a ban asks of its excuse list (what no entry explains,
///  and which entry is stale) are one
///  pairing,
///  so the pairing is made once and read twice.
fn the_entry_of_each_finding(
    found: &[HoldingLine],
    may_hold: &[MayHold],
) -> Vec<Option<usize>> {
    let mut taken: Vec<bool> = vec![false; may_hold.len()];
    let mut chosen: Vec<Option<usize>> = Vec::with_capacity(found.len());
    for holding in found {
        let mut cover: Option<usize> = None;
        for (at, entry) in may_hold.iter().enumerate() {
            if taken[at] {
                continue;
            }
            let same_use: bool = entry.file == holding.file.as_str()
                && entry.text == holding.text
                && entry.line == holding.line.as_str();
            if !same_use {
                continue;
            }
            cover = Some(at);
            taken[at] = true;
            break;
        }
        chosen.push(cover);
    }
    return chosen;
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

/// What:
///  `planted(file, code)` is one planted source file,
///  as a ban's reader takes it.
/// Why:
///  Every control here asks the same question about one line,
///  so the file's identity and its text are the whole of what it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planted = (file: string, code: string) => [[file, code]];
/// ```
fn planted(
    file: &str,
    code: &str,
) -> Vec<(String, String)> {
    return vec![(String::from(file), String::from(code))];
}

/// What:
///  One line of code,
///  in a body,
///  so a finding stands on the line the control names.
/// Why:
///  The readers count the line a spelling is written on,
///  and a spelling outside a body reads the same to them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneLine = (line: string) => `fn planted() {\n    ${line}\n}\n`;
/// ```
fn one_line(line: &str) -> String {
    return format!("fn planted() {{\n    {line}\n}}\n");
}

/// What:
///  Every spelling a line holds of a call to `symlink`,
///  and every reading of that name that makes no link.
/// Why:
///  The review of release 0.6.1-alpha.50 found that the guard of the commit path missed a rename,
///  a turbofish and a
///  function taken as a value (issue 6613),
///  and the controls that should have caught them planted none of the three.
const SPELLINGS_OF_A_CALL: &[&str] = &[
    "let _ = symlink(from, to);",
    "let _ = std::os::unix::fs::symlink(from, to);",
    "let _ = libc::symlink(from, to);",
    "let _ = std::os::unix::fs::symlink::<_, _>(from, to);",
    "use std::os::unix::fs::symlink as make_link;",
    "let f = std::os::unix::fs::symlink;",
    "let _ = std::os::unix::fs::symlink (from, to);",
];

/// What:
///  The names that end in a banned call's name,
///  and the paths that end in it without calling it.
/// Why:
///  A ban that read a name wherever its text occurs would fail every file that asks what a name is,
///  and the guard of the commit path reads one of these on the line beside its excused link.
const READINGS_OF_THE_NAME: &[&str] = &[
    "let _ = kind.is_symlink();",
    "let _ = std::fs::symlink_metadata(path);",
    "let _ = std::fs::read_link(path);",
    "let _ = libc::unlink(path);",
    "pub link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,",
    "let _ = (platform.link)(staged, lock);",
    "// a symlink here makes nothing, and this line is no code",
];

/// A call is a finding however one line spells it,
/// and a name that ends in it,
/// or a path that does,
/// is none.
#[test]
fn a_call_is_read_however_one_line_spells_it() {
    for spelling in SPELLINGS_OF_A_CALL {
        let sources: Vec<(String, String)> =
            planted("commit/planted.rs", one_line(spelling).as_str());
        let found: Vec<HoldingLine> = lines_calling_one_of(sources.as_slice(), &["symlink"]);
        assert_eq!(
            found.len(),
            1,
            "{spelling} spells one call and is {} finding(s)",
            found.len()
        );
    }
    for reading in READINGS_OF_THE_NAME {
        let sources: Vec<(String, String)> =
            planted("commit/planted.rs", one_line(reading).as_str());
        assert!(
            lines_calling_one_of(sources.as_slice(), &["symlink", "link"]).is_empty(),
            "{reading} makes no link"
        );
    }
}

/// What:
///  A type's name is read wherever a line writes it,
///  and a longer name that ends in it is another name.
/// Why:
///  A socket is named in a `use` list and in a signature as often as at a call,
///  and a ban on the type that read only calls would miss the position it then appears in.
const SPELLINGS_OF_A_NAME: &[&str] = &[
    "use std::os::unix::net::{self, UnixStream};",
    "let _ = UnixStream::pair();",
    "fn socket_of(it: UnixStream) -> UnixStream { it }",
    "use std::io::{self, Stderr};",
];

/// A name is a finding wherever a line writes it,
/// and a longer name is another name.
#[test]
fn a_name_is_read_wherever_a_line_writes_it() {
    for spelling in SPELLINGS_OF_A_NAME {
        let sources: Vec<(String, String)> =
            planted("service/planted.rs", one_line(spelling).as_str());
        let found: Vec<HoldingLine> =
            lines_naming_one_of(sources.as_slice(), &["UnixStream", "Stderr"]);
        assert_eq!(
            found.len(),
            1,
            "{spelling} writes one name and is {} finding(s)",
            found.len()
        );
    }
    let longer: Vec<(String, String)> = planted(
        "service/planted.rs",
        one_line("let _ = UnixStreams::pair();").as_str(),
    );
    assert!(
        lines_naming_one_of(longer.as_slice(), &["UnixStream"]).is_empty(),
        "a longer name that begins with the banned one is another name"
    );
}

/// What:
///  An excuse covers the use it names and nothing else,
///  and a finding on a line the entry does not name is a finding.
/// Why:
///  The review found that an entry naming a count of uses kept a file excused when the excused use was removed and
///  another of the same text was added (issue 6613).
const ONE_EXCUSED_USE: &[MayHold] = &[MayHold {
    file: "landing/one.rs",
    text: "symlink",
    line: "let _ = std::fs::symlink(a, b);",
    why: "a planted excuse, in a control that asks what an entry covers",
}];

/// The line an entry names is excused,
/// and every other line holding that text is a finding.
#[test]
fn an_excuse_covers_the_one_use_it_names() {
    let excused: Vec<(String, String)> = planted(
        "landing/one.rs",
        "fn made(a: &Path, b: &Path) {\n    let _ = std::fs::symlink(a, b);\n}\n",
    );
    let held: Vec<HoldingLine> = lines_calling_one_of(excused.as_slice(), &["symlink"]);
    assert_eq!(
        lines_no_entry_excuses(held.as_slice(), ONE_EXCUSED_USE),
        Vec::<String>::new(),
        "the line the entry names is the use it excuses"
    );
    assert_eq!(
        entries_that_match_nothing(held.as_slice(), ONE_EXCUSED_USE),
        Vec::<String>::new(),
        "the entry matched the use it names"
    );
    // The excused use is removed and another of the same text is added to the same file.
    let swapped: Vec<(String, String)> = planted(
        "landing/one.rs",
        "fn made(a: &Path, b: &Path) {\n    let _ = std::fs::symlink(b, a);\n}\n",
    );
    let after: Vec<HoldingLine> = lines_calling_one_of(swapped.as_slice(), &["symlink"]);
    assert_eq!(
        lines_no_entry_excuses(after.as_slice(), ONE_EXCUSED_USE).len(),
        1,
        "the use the swap added is excused by no entry"
    );
    assert_eq!(
        entries_that_match_nothing(after.as_slice(), ONE_EXCUSED_USE).len(),
        1,
        "the entry names a line that is gone"
    );
}

/// A second line written exactly like an excused one needs an entry of its own.
#[test]
fn a_second_line_like_an_excused_one_needs_its_own_entry() {
    let twice: Vec<(String, String)> = planted(
        "landing/one.rs",
        "fn made(a: &Path, b: &Path) {\n    let _ = std::fs::symlink(a, b);\n}\nfn also(a: &Path, b: &Path) {\n    let _ = std::fs::symlink(a, b);\n}\n",
    );
    let held: Vec<HoldingLine> = lines_calling_one_of(twice.as_slice(), &["symlink"]);
    assert_eq!(
        lines_no_entry_excuses(held.as_slice(), ONE_EXCUSED_USE).len(),
        1,
        "one entry excuses one use, so the second identical line is a finding"
    );
}
