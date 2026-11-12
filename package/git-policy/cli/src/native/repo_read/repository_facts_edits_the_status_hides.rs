//! What:
//!  The tracked edits the index's own `assume-unchanged` mark hides from every status Git answers.
//! Why:
//!  `git status`,
//!  `git diff-files`,
//!  `git diff-index` and `git ls-files --modified` all read a marked entry as "do not compare the
//!  worktree copy",
//!  so an edited path that carries the mark is invisible to the question the guard asks,
//!  while `reset --hard`,
//!  a forced `checkout` or `switch`,
//!  a path `checkout` or `restore`,
//!  `read-tree --reset -u`,
//!  `checkout-index -f -a` and `rm -f` overwrite it anyway
//!  (`oneway_merge` with `CE_MATCH_IGNORE_VALID`).
//!  A marked edit is a tracked edit like any other,
//!  so it gets exactly what the same edit gets without the mark,
//!  which is what reading it here restores.
//!  `skip-worktree` is not read:
//!  Git keeps that file, so no discarding form loses it.
//!  The marks are listed once,
//!  and the worktree entries of those paths alone are compared,
//!  so a worktree that marks thousands of files costs one listing and one stat each,
//!  and only a stat that differs costs one more query.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // editsTheStatusHides(facts) === ['a.txt']  when a.txt is marked and its bytes differ from the index
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The captured-query runner's output type,
///  and the read ceiling every query has.
use crate::git_metadata::MetadataOutput;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The mark `git ls-files -v` prints for an `assume-unchanged` entry:
/// its tag is the lower-case letter of the entry's state.
/// Why:
///   `-v` lower-cases the tag of exactly the entries Git was told not to look at,
///  and every other entry keeps an upper-case tag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isMarked = (tag: string) => tag.length === 1 && tag === tag.toLowerCase();
/// ```
fn is_marked(tag: &[u8]) -> bool {
    return tag.len() == 1 && tag[0].is_ascii_lowercase();
}

/// What:
///  The paths whose index entry carries the `assume-unchanged` mark and whose worktree entry
///  no longer holds what the index records.
///  `Result<Vec<String>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   A listing that failed says nothing,
///  and a safety check never reads a failure as "no marked edit";
///  a marked path with no worktree file is no edit of the person's,
///  because Git writes that file back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editsTheStatusHides(facts, worktreeRoot): Promise<string[]>; // throws
/// ```
pub(super) fn edits_the_status_hides(facts: &mut GitFacts, worktree_root: &Path) -> Result<Vec<String>, String> {
    let marked: Vec<Vec<u8>> = marked_paths(facts)?;
    if marked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    let mut edited: Vec<String> = Vec::<String>::new();
    for entry in marked_entries(facts, marked.as_slice())? {
        if !differs_from_the_index(facts, worktree_root, &entry)? {
            continue;
        }
        edited.push(String::from_utf8_lossy(entry.path.as_slice()).into_owned());
    }
    return Ok(edited);
}

/// What:
///  The paths `git ls-files -v` reports with a mark that hides them from status.
/// Why:
///   One listing names every marked entry,
///  and the marks of the paths nothing edits cost nothing more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedPaths(facts): Promise<Buffer[]>; // throws
/// ```
fn marked_paths(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-files", "-v", "-z", "--full-name"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the paths the index marks assume-unchanged")?;
    if !output.success {
        return Err(failed(&output, "the paths the index marks assume-unchanged"));
    }
    let mut marked: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        // Each entry is `<tag> <path>`; a tag of one lower-case letter is the mark.
        let Some(space) = entry.iter().position(|byte| *byte == b' ') else {
            continue;
        };
        if !is_marked(&entry[..space]) {
            continue;
        }
        marked.push(entry[space + 1..].to_vec());
    }
    return Ok(marked);
}

/// What:
///  One marked entry as the index records it:
///  the object its bytes make,
///  the mode it has,
///  and the stat Git kept for it.
/// Why:
///   The stat says in one query which marked entries are worth comparing byte for byte,
///  and the object and the mode are what the comparison needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkedEntry = { mode: string; object: string; path: Buffer; size: u64; modified: [i64, i64] };
/// ```
struct MarkedEntry {
    /// The mode the index records.
    mode: String,
    /// The object the index holds for the path.
    object: String,
    /// The path,
    ///  as the index holds it.
    path: Vec<u8>,
    /// The size the index recorded.
    size: u64,
    /// The modification time the index recorded,
    ///  in seconds and nanoseconds.
    modified: (i64, i64),
}

/// What:
///  The index's own record of each marked path,
///  asked for those paths alone.
/// Why:
///   `--debug` prints the stat Git kept,
///  so a marked entry nothing touched is recognised without reading its bytes;
///  a query that failed says nothing about them and is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedEntries(facts, marked: Buffer[]): Promise<MarkedEntry[]>; // throws
/// ```
fn marked_entries(facts: &mut GitFacts, marked: &[Vec<u8>]) -> Result<Vec<MarkedEntry>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "ls-files", "-v", "--stage", "--debug", "-z", "--full-name", "--"] {
        arguments.push(OsString::from(token));
    }
    for path in marked {
        arguments.push(path_argument(path));
    }
    let output: MetadataOutput = facts.query(arguments, "the index's own record of the marked paths")?;
    if !output.success {
        return Err(failed(&output, "the index's own record of the marked paths"));
    }
    return Ok(parse_marked_entries(output.stdout.as_slice()));
}

/// What:
///  The entries one `git ls-files -v --stage --debug -z` answer holds.
/// Why:
///   Git writes each entry's header up to a NUL and the five lines of its stat after that NUL,
///  so a chunk after the first starts with the previous entry's stat and holds this entry's header
///  after the fifth newline.
///  Each entry is therefore read from its own header and the stat block in front of the next one,
///  and a chunk without those five lines stops the reading instead of naming a path nobody checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMarkedEntries(stdout: Buffer): MarkedEntry[];
/// ```
fn parse_marked_entries(stdout: &[u8]) -> Vec<MarkedEntry> {
    let mut entries: Vec<MarkedEntry> = Vec::<MarkedEntry>::new();
    let chunks: Vec<&[u8]> = stdout.split(|byte| *byte == 0).collect();
    for (index, chunk) in chunks.iter().enumerate() {
        // The first chunk is the first entry's header; every later one starts with the stat of the entry before it.
        let header: &[u8] = if index == 0 {
            chunk
        } else {
            match after_stat_lines(chunk) {
                Some(found) => found,
                None => return entries,
            }
        };
        // The stat of this entry is the head of the chunk after it, and the last entry's stat is the last chunk.
        let Some(stat) = chunks.get(index + 1) else {
            continue;
        };
        if let Some(entry) = entry_of(header, stat) {
            entries.push(entry);
        }
    }
    return entries;
}

/// What:
///  The bytes after the five stat lines of one entry,
///  which is where the next entry's header starts.
///  `None` when the chunk holds fewer lines than that,
///  which is an answer this reader cannot trust.
/// Why:
///  `--debug` writes exactly those five lines after each header,
///  so the boundary between two entries is found without splitting a path
///  that holds a newline of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterStatLines(chunk: Buffer): Buffer | undefined;
/// ```
fn after_stat_lines(chunk: &[u8]) -> Option<&[u8]> {
    let mut rest: &[u8] = chunk;
    for _ in 0..STAT_LINES {
        let newline: usize = rest.iter().position(|byte| *byte == b'\n')?;
        rest = &rest[newline + 1..];
    }
    return Some(rest);
}

/// How many lines `--debug` writes after one entry's header.
/// Why:
///   `ctime`,
///  `mtime`,
///  `dev`/`ino`,
///  `uid`/`gid` and `size`/`flags` are one line each,
///  so the next entry's header always follows the fifth.
const STAT_LINES: usize = 5;

/// What:
///  One entry's header and the stat block Git writes after it,
///  read as the index records them,
///  or nothing when the bytes are not that.
///  `&[u8]` borrows both parts of the answer.
/// Why:
///   A chunk that names no path or no stat is not an entry,
///  and reading it as one would compare the wrong file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(header: Buffer, stat: Buffer): MarkedEntry | undefined;
/// ```
fn entry_of(header: &[u8], stat: &[u8]) -> Option<MarkedEntry> {
    let tab: usize = header.iter().position(|byte| *byte == b'\t')?;
    let fields: Vec<&[u8]> = header[..tab].split(|byte| *byte == b' ').collect();
    let [_tag, mode, object, stage] = fields.as_slice() else {
        return None;
    };
    if *stage != b"0" {
        return None;
    }
    return Some(MarkedEntry {
        mode: String::from_utf8_lossy(mode).into_owned(),
        object: String::from_utf8_lossy(object).into_owned(),
        path: header[tab + 1..].to_vec(),
        size: number_after(stat, b"size: ")?,
        modified: modified_after(stat, b"mtime: ")?,
    });
}

/// What:
///  The decimal number one debug line records after a named label.
/// Why:
///  `size: 2\tflags: 8000` carries the size and the flags on one line,
///  and the number ends at the first byte that is not a digit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function numberAfter(block: Buffer, label: Buffer): number | undefined;
/// ```
fn number_after(block: &[u8], label: &[u8]) -> Option<u64> {
    let at: usize = find(block, label)? + label.len();
    let mut end: usize = at;
    while end < block.len() && block[end].is_ascii_digit() {
        end += 1;
    }
    return std::str::from_utf8(&block[at..end]).ok()?.parse::<u64>().ok();
}

/// What:
///  The `mtime` one debug block records as seconds and nanoseconds.
/// Why:
///  `mtime: 1794466647:495117187` is the pair Git's own refresh compares,
///  and a block that carries no pair is no evidence about the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modifiedAfter(block: Buffer, label: Buffer): [number, number] | undefined;
/// ```
fn modified_after(block: &[u8], label: &[u8]) -> Option<(i64, i64)> {
    let at: usize = find(block, label)? + label.len();
    let colon: usize = at + block[at..].iter().position(|byte| *byte == b':')?;
    let seconds: i64 = std::str::from_utf8(&block[at..colon]).ok()?.trim().parse::<i64>().ok()?;
    let mut end: usize = colon + 1;
    while end < block.len() && block[end].is_ascii_digit() {
        end += 1;
    }
    let nanos: i64 = std::str::from_utf8(&block[colon + 1..end]).ok()?.parse::<i64>().ok()?;
    return Some((seconds, nanos));
}

/// What:
///  Where one label starts inside a block,
///  as an index.
/// Why:
///   A debug block is searched by the label Git prints,
///  and one search keeps every reader of it the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function find(block: Buffer, label: Buffer): number | undefined;
/// ```
fn find(block: &[u8], label: &[u8]) -> Option<usize> {
    if label.len() > block.len() {
        return None;
    }
    return block.windows(label.len()).position(|window| window == label);
}

/// What:
///  Whether one marked entry's worktree copy differs from what the index records,
///  by its stat,
///  its mode or its bytes.
/// Why:
///  The stat answers for every entry nothing touched,
///  which is the whole of a worktree that marks thousands of files,
///  and a stat that differs costs one query that reads the bytes through the same filters
///  the index was written with,
///  so a `touch` of an unchanged file is no edit and a real one is.
///  A worktree file that is gone is no edit of the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function differsFromTheIndex(facts, root, entry): Promise<boolean>; // throws
/// ```
fn differs_from_the_index(facts: &mut GitFacts, root: &Path, entry: &MarkedEntry) -> Result<bool, String> {
    let Some(relative) = relative_path(entry.path.as_slice()) else {
        // A name this process cannot spell is no name to read bytes from, so the stat's own answer stands.
        return Ok(true);
    };
    let location: PathBuf = root.join(relative.as_path());
    let Ok(metadata) = std::fs::symlink_metadata(location.as_path()) else {
        return Ok(false);
    };
    let modified: (i64, i64) = modified_of_metadata(&metadata);
    let mode: &str = if is_executable(&metadata) { "100755" } else { "100644" };
    // A symbolic link in the worktree where the index holds a file is an edit whatever its stat says.
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    if !metadata.file_type().is_file() {
        return Ok(true);
    }
    if metadata.len() == entry.size && modified == entry.modified && mode == entry.mode.as_str() {
        return Ok(false);
    }
    return Ok(worktree_object(facts, entry)? != entry.object);
}

/// What:
///  The object the worktree file's bytes make,
///  read through the clean filter of the path it is at.
/// Why:
///  Git records the filtered object in the index,
///  so an unfiltered hash would differ from it for every file a filter touches.
///  A query that failed says nothing about the bytes and is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeObject(facts, entry): Promise<string>; // throws
/// ```
fn worktree_object(facts: &mut GitFacts, entry: &MarkedEntry) -> Result<String, String> {
    let Ok(shown) = String::from_utf8(entry.path.clone()) else {
        // A path that is not text has no `--path` spelling, and its stat already said it changed.
        return Ok(String::new());
    };
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    arguments.push(OsString::from("--no-optional-locks"));
    arguments.push(OsString::from("hash-object"));
    arguments.push(OsString::from(format!("--path={shown}")));
    arguments.push(OsString::from("--"));
    arguments.push(path_argument(entry.path.as_slice()));
    let output: MetadataOutput = facts.query(arguments, "the object of the worktree copy")?;
    if !output.success {
        return Err(failed(&output, "the object of the worktree copy"));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  One repository path as the filesystem path below the worktree root,
///  or nothing when its bytes are not a name this process can spell.
/// Why:
///   The stat has to name the file the index names,
///  and a name that cannot be spelled here is one whose stat is no evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePath(bytes: Buffer): string | undefined;
/// ```
#[cfg(unix)]
fn relative_path(bytes: &[u8]) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let text: OsString = OsString::from_vec(bytes.to_vec());
    return Some(PathBuf::from(text));
}

/// What:
///  One repository path as the filesystem path below the worktree root.
/// Why:
///  No gate compiles this branch (the crate does not build for Windows; README, Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePath(bytes: Buffer): string | undefined;
/// ```
#[cfg(not(unix))]
fn relative_path(bytes: &[u8]) -> Option<PathBuf> {
    let text: String = String::from_utf8(bytes.to_vec()).ok()?;
    return Some(PathBuf::from(text));
}

/// What:
///  The modification time of one worktree entry as seconds and nanoseconds since the epoch.
/// Why:
///   That pair is what the index records and what Git's own refresh compares,
///  and a clock the system will not report is no evidence of an edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modifiedOfMetadata(metadata): [number, number];
/// ```
fn modified_of_metadata(metadata: &std::fs::Metadata) -> (i64, i64) {
    let since: std::time::Duration = match metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()) {
        Some(found) => found,
        None => std::time::Duration::ZERO,
    };
    return (i64::try_from(since.as_secs()).unwrap_or(i64::MAX), i64::from(since.subsec_nanos()));
}

/// What:
///  Whether a file carries the executable bit Git records.
/// Why:
///   A `100755` entry and a `100644` one are different files to Git,
///  so a chmod is an edit like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isExecutable = (metadata) => (metadata.mode & 0o111) !== 0;
/// ```
#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    return metadata.permissions().mode() & 0o111 != 0;
}

/// What:
///  Whether a file carries the executable bit Git records.
/// Why:
///   No gate compiles this branch (the crate does not build for Windows; README, Requirements),
///  and the answer doubles as the one the Unix branch gives an unreadable mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isExecutable = (metadata) => false;
/// ```
#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    return false;
}

/// What:
///  One path as an argument Git reads exactly as it is.
/// Why:
///   A path is bytes and not text,
///  so a name that is not UTF-8 reaches Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (bytes: Buffer): string => bytes;
/// ```
#[cfg(unix)]
fn path_argument(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    return OsString::from_vec(bytes.to_vec());
}

/// What:
///  One path as an argument Git reads exactly as it is.
/// Why:
///  No gate compiles this branch (the crate does not build for Windows; README, Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (bytes: Buffer): string => String(bytes);
/// ```
#[cfg(not(unix))]
fn path_argument(bytes: &[u8]) -> OsString {
    return OsString::from(String::from_utf8_lossy(bytes).into_owned());
}

/// What:
///  One failed query as this fact's own failure.
/// Why:
///  A reading that could not be made says nothing about the worktree,
///  and the words that stopped it are what the person acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `Git could not read ${fact}: ${firstStderrLine}`;
/// ```
fn failed(output: &MetadataOutput, fact: &str) -> String {
    let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    return format!("Git could not read {fact}: {detail}");
}

/// Reading the marks and comparing what the worktree holds with what the index records.
#[cfg(test)]
#[path = "repository_facts_edits_the_status_hides_tests.rs"]
mod tests;
