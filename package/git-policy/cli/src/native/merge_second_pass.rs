//! What:
//!  The word-level second pass over the files a Git merge left conflicted: which conflicted files may be re-merged, their
//!  re-merge by `word_merge`, the objects and the tree that carry the result, and the words that list every merged part
//!  on standard error and in the journal.
//! Why:
//!  The owner decided on 2026-11-03 (`doc/planning/cli-git-merge-auto-resolution.md`) that the merges a person runs and the
//!  wrapper's own unattended merges get a second pass that accepts a file only when every part resolves and shows each part.
//!  Four callers need the same answer from the same stage blobs: the forwarded merge, rebase, cherry-pick and revert
//!  (`merge_second_pass_worktree.rs`), the isolated merge, the auto-push reconcile merge and the commit replay merge, which
//!  work from objects (`merge_second_pass_tree.rs`). They differ in how Git is started, so they hand in a `GitRunner`.
//!  Only a plain file whose three stages exist, with one mode on all three, that is text, small, and not claimed by a merge
//!  driver or the `binary` attribute, is re-merged; everything else is left to the person exactly as Git left it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await secondPass(runner, conflicts); // [{ merged: { path, mode, bytes, parts } } | { left: { path, why } }]
//! ```

use super::client_journal::record_entry_as;
use super::diagnostic_log::debug;
use super::journald_native::Priority;
use super::path_bytes::path_message_text;
use super::transaction_git::{GitContext, GitRequest, run_git};
use super::word_merge::{LineSpan, MergedPart, WordMerge, merge_by_word};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMerge";

/// The words every line about a merged part carries, so a reader finds them in a long output.
pub const REPORT_MARK: &str = "word-level merge";

/// The journal field that names the file of an entry.
pub const FILE_FIELD: &str = "CLI_GIT_WORD_MERGE_FILE";

/// The journal field that names the operation that produced the conflict.
pub const OPERATION_FIELD: &str = "CLI_GIT_WORD_MERGE_OPERATION";

/// The journal field that counts the parts of the file that were merged by word.
pub const PARTS_FIELD: &str = "CLI_GIT_WORD_MERGE_PARTS";

/// The number of leading bytes in which Git looks for a NUL byte to call a file binary: `FIRST_FEW_BYTES` in Git 2.56.0's
///  `xdiff-interface.c` line 197, used by `buffer_is_binary` and by the merge driver choice in `merge-ll.c` lines 120 to 122.
const BINARY_PROBE_BYTES: usize = 8000;

/// The size above which Git's own merge treats a file as binary and does not merge it by text: `MAX_XDIFF_SIZE`, 1023 MiB, in
///  `xdiff-interface.h` line 14, tested in `merge-ll.c` lines 117 to 119.
const GIT_MAX_XDIFF_SIZE: usize = 1024 * 1024 * 1023;

/// The size above which this pass leaves a file to the person, although Git would still merge it by text. Git's value cannot
///  bound the cost of this pass: measured on three texts that share no line, one 1 MiB of lines took 0.08 s and 45 MB, 8 MiB took
///  0.73 s and 352 MB, 32 MiB took 4.12 s and 1320 MB (release build, Myers diff, one core), so a file at Git's limit would take
///  minutes and tens of gigabytes. 8 MiB keeps the worst case under a second.
const PASS_COST_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// The largest blob the second pass re-merges: the smaller of Git's own limit and the one measured above.
const LARGEST_BLOB_BYTES: usize = if PASS_COST_LIMIT_BYTES < GIT_MAX_XDIFF_SIZE { PASS_COST_LIMIT_BYTES } else { GIT_MAX_XDIFF_SIZE };

/// The most characters of a merged part a report line shows.
const EXCERPT_CHARACTERS: usize = 100;

/// The modes of a plain file, with and without the executable bit.
const PLAIN_FILE_MODES: [u32; 2] = [0o100_644, 0o100_755];

/// The values of the `merge` attribute that leave the text merge to Git's own algorithm: none given, `merge` set, `text`.
const TEXT_MERGE_VALUES: [&str; 3] = ["unspecified", "set", "text"];

/// The fields `git check-attr -z` prints for each answer: the path, the attribute and its value.
const ATTRIBUTE_FIELDS: usize = 3;

/// The stage numbers of the common ancestor, the first side and the second side.
const BASE_STAGE: u8 = 1;
const OURS_STAGE: u8 = 2;
const THEIRS_STAGE: u8 = 3;

/// What a Git start gave back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitReply {
    /// The exit status, or nothing when a signal ended Git.
    pub code: Option<i32>,
    /// Standard output.
    pub stdout: Vec<u8>,
    /// Standard error.
    pub stderr: Vec<u8>,
}

/// How a caller starts Git for the second pass: in the repository it works in, with its environment and global options.
pub trait GitRunner {
    /// Start Git with `arguments`, `input` on standard input and `index_file` as the index when given. `Err` means Git could
    /// not be started; an exit status other than 0 is a reply.
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String>;
}

/// The runner for callers that start Git through `run_git`: one context, one directory and a fixed set of request options.
pub struct ContextRunner<'a> {
    /// The real Git, its overlay and the global options of the invocation.
    pub context: &'a GitContext,
    /// The directory every start runs in.
    pub cwd: &'a Path,
    /// Whether the invocation's global options are left out (the commit replay's private runs).
    pub without_prefix: bool,
    /// The object store that receives written objects, when it is not the repository's own.
    pub object_directory: Option<&'a Path>,
    /// Variables added to every start.
    pub environment: Vec<(OsString, OsString)>,
}

/// Starting Git through the shared runner.
impl GitRunner for ContextRunner<'_> {
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String> {
        let mut request: GitRequest = GitRequest::new(self.cwd, arguments);
        request.without_prefix = self.without_prefix;
        request.object_directory = self.object_directory.map(Path::to_path_buf);
        request.environment.clone_from(&self.environment);
        request.input = input.map(<[u8]>::to_vec);
        request.index = index_file.map(PathBuf::from);
        let output = run_git(self.context, &request).map_err(|error| { return format!("git {} could not start: {error}", arguments.join(" ")); })?;
        return Ok(GitReply { code: output.code, stdout: output.stdout, stderr: output.stderr });
    }
}

/// What:
///  The configuration options (`-c <name>=<value>`) among an invocation's global options, and nothing else.
/// Why:
///   The second pass runs from the top of the worktree, because `git check-attr` and `git checkout-index` read the paths
///  they are given relative to the directory they run in; `-C` and `--git-dir` would move them again. The settings that change
///  what a checkout writes (`core.autocrlf`, filters) still apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configOptions(prefix: string[]): string[]; // ['-c', 'a=b', '-c', 'c=d']
/// ```
pub fn config_options(prefix: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut tokens = prefix.iter();
    while let Some(token) = tokens.next() {
        if token == "-c"
            && let Some(setting) = tokens.next()
        {
            kept.push(token.clone());
            kept.push(setting.clone());
        }
    }
    return kept;
}

/// One side of a conflicted path: its mode and object name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    /// The file mode with its type bits.
    pub mode: u32,
    /// The object name, as hexadecimal text.
    pub oid: String,
}

/// A conflicted path with the stages Git recorded for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    /// The path, `/` separated, as bytes.
    pub path: Vec<u8>,
    /// The common ancestor, absent for a path both sides added.
    pub base: Option<Stage>,
    /// The first side, absent for a path it deleted.
    pub ours: Option<Stage>,
    /// The second side, absent for a path it deleted.
    pub theirs: Option<Stage>,
}

/// What:
///  The conflicted paths a merge-tree answer lists, with their stages, in the order Git printed them.
/// Why:
///   `git merge-tree --write-tree -z` prints the tree, then one `<mode> <object> <stage>\t<path>` field per stage of every
///  conflicted path, then an empty field, then the messages. The name-only form does not carry the object names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsOfMergeTree(stdout: Uint8Array): { tree: string; conflicts: Conflict[] };
/// ```
pub fn conflicts_of_merge_tree(stdout: &[u8]) -> (String, Vec<Conflict>) {
    let mut fields = stdout.split(|byte| { return *byte == 0; });
    let tree: String = String::from_utf8_lossy(fields.next().unwrap_or_default()).into_owned();
    let mut by_path: BTreeMap<Vec<u8>, Conflict> = BTreeMap::new();
    let mut order: Vec<Vec<u8>> = Vec::new();
    for field in fields {
        if field.is_empty() {
            break;
        }
        let Some((facts, path)) = split_once(field, b'\t') else { continue };
        let words: Vec<&[u8]> = facts.split(|byte| { return *byte == b' '; }).collect();
        let [mode_bytes, oid_bytes, stage_bytes] = words.as_slice() else { continue };
        let (Ok(mode_text), Ok(oid_text), Ok(stage_text)) = (std::str::from_utf8(mode_bytes), std::str::from_utf8(oid_bytes), std::str::from_utf8(stage_bytes)) else { continue };
        let (Ok(mode), Ok(stage)) = (u32::from_str_radix(mode_text, 8), stage_text.parse::<u8>()) else { continue };
        if !by_path.contains_key(path) {
            order.push(path.to_vec());
        }
        let conflict: &mut Conflict = by_path.entry(path.to_vec()).or_insert_with(|| { return Conflict { path: path.to_vec(), base: None, ours: None, theirs: None }; });
        let side: Option<Stage> = Some(Stage { mode, oid: oid_text.to_owned() });
        match stage {
            BASE_STAGE => conflict.base = side,
            OURS_STAGE => conflict.ours = side,
            THEIRS_STAGE => conflict.theirs = side,
            _ => {}
        }
    }
    return (tree, order.into_iter().filter_map(|path| { return by_path.remove(&path); }).collect());
}

/// The text before the first `separator` and the text after it.
fn split_once(bytes: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at: usize = bytes.iter().position(|byte| { return *byte == separator; })?;
    return Some((&bytes[..at], &bytes[at + 1..]));
}

/// Why a conflicted path is left to the person.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Left {
    /// A stage is missing: both sides added the path, or one side deleted it.
    MissingStage,
    /// The stages differ in mode, or the file is a link, a submodule or not a plain file.
    NotAPlainFile,
    /// A stage holds a NUL byte near its start.
    Binary,
    /// A stage is larger than the pass takes.
    TooLarge,
    /// The `merge` attribute names a driver, `binary` or is unset, so the merge belongs to that setting.
    Driver(String),
    /// At least one line region holds two different changes of the same words.
    Overlap(usize),
}

/// A conflicted file that the word pass merged completely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merged {
    /// The path as bytes.
    pub path: Vec<u8>,
    /// The mode all three stages share.
    pub mode: u32,
    /// The merged bytes.
    pub bytes: Vec<u8>,
    /// The line regions that needed the word pass.
    pub parts: Vec<MergedPart>,
    /// The three stages the file was merged from, kept to put the conflict back if a later step fails.
    pub stages: Conflict,
}

/// What became of one conflicted path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Merged completely.
    Merged(Box<Merged>),
    /// Left as Git left it.
    Left {
        /// The path as bytes.
        path: Vec<u8>,
        /// Why.
        why: Left,
    },
}

/// What:
///  Whether the bytes are text the pass may merge: no NUL byte near the start, and not larger than the limit.
/// Why:
///   Git's own binary test looks at the first bytes only; a file it would call binary is never a candidate for a
///  line-and-word merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = (bytes) => bytes.length > LARGEST ? 'too-large' : bytes.subarray(0, PROBE).includes(0) ? 'binary' : 'text';
/// ```
fn unfit(bytes: &[u8]) -> Option<Left> {
    if bytes.len() > LARGEST_BLOB_BYTES {
        return Some(Left::TooLarge);
    }
    if bytes[..bytes.len().min(BINARY_PROBE_BYTES)].contains(&0) {
        return Some(Left::Binary);
    }
    return None;
}

/// The mode all three stages share, when it is a plain file's.
fn shared_plain_mode(stages: [&Stage; 3]) -> Option<u32> {
    let mode: u32 = stages[0].mode;
    if stages.iter().all(|stage| { return stage.mode == mode; }) && PLAIN_FILE_MODES.contains(&mode) {
        return Some(mode);
    }
    return None;
}

/// What:
///  The value of the `merge` attribute of each path, from one `git check-attr` start.
/// Why:
///   A path with a driver, with `-merge` or with `merge=binary` must be merged the way its setting says, not by a second
///  pass that ignores the setting. The caller's runner decides where attributes come from: the worktree for a merge a person
///  runs, the attribute tree for a merge from objects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeAttributes(runner, paths: Uint8Array[]): Promise<Map<string, string>>;
/// ```
fn merge_attributes(runner: &dyn GitRunner, paths: &[&[u8]]) -> Result<BTreeMap<Vec<u8>, String>, String> {
    if paths.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path);
        input.push(0);
    }
    let reply: GitReply = runner.run(&["check-attr", "-z", "--stdin", "merge"], Some(input.as_slice()), None)?;
    if reply.code != Some(0) {
        return Err(format!("git check-attr failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    let fields: Vec<&[u8]> = reply.stdout.split(|byte| { return *byte == 0; }).collect();
    let mut values: BTreeMap<Vec<u8>, String> = BTreeMap::new();
    for triple in fields.as_chunks::<ATTRIBUTE_FIELDS>().0 {
        values.insert(triple[0].to_vec(), String::from_utf8_lossy(triple[2]).into_owned());
    }
    return Ok(values);
}

/// What:
///  The bytes of the named blobs, from one `git cat-file --batch` start.
/// Why:
///   One start reads every stage of every candidate file; the answer for a name is its header line and then its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readBlobs(runner, oids: string[]): Promise<Map<string, Uint8Array>>;
/// ```
fn read_blobs(runner: &dyn GitRunner, oids: &[&str]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut input: Vec<u8> = Vec::new();
    for oid in oids {
        input.extend_from_slice(oid.as_bytes());
        input.push(b'\n');
    }
    let reply: GitReply = runner.run(&["cat-file", "--batch"], Some(input.as_slice()), None)?;
    if reply.code != Some(0) {
        return Err(format!("git cat-file failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    let mut blobs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut rest: &[u8] = reply.stdout.as_slice();
    for oid in oids {
        let Some((header_bytes, after)) = split_once(rest, b'\n') else { return Err(format!("git cat-file ended before {oid}")) };
        let header: String = String::from_utf8_lossy(header_bytes).into_owned();
        let words: Vec<&str> = header.split(' ').collect();
        let [name, _kind, size_text] = words.as_slice() else { return Err(format!("git cat-file could not read {oid}: {header}")) };
        let size: usize = size_text.parse().map_err(|_| { return format!("git cat-file printed an unreadable size for {oid}: {header}"); })?;
        let Some(content) = after.get(..size) else { return Err(format!("git cat-file ended inside {oid}")) };
        blobs.insert((*name).to_owned(), content.to_vec());
        rest = after.get(size + 1..).unwrap_or_default();
    }
    return Ok(blobs);
}

/// What:
///  Re-merge every conflicted path that may be, and say what became of each.
/// Why:
///   A path is merged only when all three stages exist with one plain mode, are small text, and the `merge` attribute leaves
///  the text merge to Git; and only when every line region resolves. Anything else is `Left`, so the caller keeps the conflict.
///   `Err` means a Git start failed; the caller then leaves every conflict as Git left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function secondPass(runner: GitRunner, conflicts: Conflict[]): Promise<Verdict[]>;
/// ```
pub fn second_pass(runner: &dyn GitRunner, conflicts: &[Conflict]) -> Result<Vec<Verdict>, String> {
    let mut verdicts: BTreeMap<usize, Verdict> = BTreeMap::new();
    let mut candidates: Vec<(usize, u32)> = Vec::new();
    for (index, conflict) in conflicts.iter().enumerate() {
        let (Some(base), Some(ours), Some(theirs)) = (&conflict.base, &conflict.ours, &conflict.theirs) else {
            verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::MissingStage });
            continue;
        };
        match shared_plain_mode([base, ours, theirs]) {
            Some(mode) => candidates.push((index, mode)),
            None => {
                verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::NotAPlainFile });
            }
        }
    }
    let attributes: BTreeMap<Vec<u8>, String> = merge_attributes(
        runner,
        candidates.iter().map(|(index, _)| { return conflicts[*index].path.as_slice(); }).collect::<Vec<&[u8]>>().as_slice(),
    )?;
    let mut wanted: Vec<&str> = Vec::new();
    for (index, _) in &candidates {
        let conflict: &Conflict = &conflicts[*index];
        for stage in [&conflict.base, &conflict.ours, &conflict.theirs].into_iter().flatten() {
            if !wanted.contains(&stage.oid.as_str()) {
                wanted.push(stage.oid.as_str());
            }
        }
    }
    let blobs: BTreeMap<String, Vec<u8>> = if wanted.is_empty() { BTreeMap::new() } else { read_blobs(runner, wanted.as_slice())? };
    for (index, mode) in candidates {
        let conflict: &Conflict = &conflicts[index];
        if let Some(value) = attributes.get(&conflict.path).filter(|value| { return !TEXT_MERGE_VALUES.contains(&value.as_str()); }) {
            verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::Driver(value.clone()) });
            continue;
        }
        let sides: Vec<&Vec<u8>> = [&conflict.base, &conflict.ours, &conflict.theirs]
            .into_iter()
            .filter_map(|stage| { return stage.as_ref().and_then(|found| { return blobs.get(&found.oid); }); })
            .collect();
        let [base, ours, theirs] = sides.as_slice() else { return Err(format!("a stage of {} was not read", path_message_text(conflict.path.as_slice()))) };
        let verdict: Verdict = match [base, ours, theirs].into_iter().find_map(|bytes| { return unfit(bytes); }) {
            Some(why) => Verdict::Left { path: conflict.path.clone(), why },
            None => match merge_by_word(base, ours, theirs) {
                WordMerge::Resolved { merged, parts } => Verdict::Merged(Box::new(Merged { path: conflict.path.clone(), mode, bytes: merged, parts, stages: conflict.clone() })),
                WordMerge::Conflict { unresolved } => Verdict::Left { path: conflict.path.clone(), why: Left::Overlap(unresolved) },
            },
        };
        verdicts.insert(index, verdict);
    }
    return Ok(verdicts.into_values().collect());
}

/// What:
///  Write one merged file as a blob and return its object name.
/// Why:
///   `--no-filters` keeps the bytes exactly as merged; the checkout filters apply when a worktree file is made, not here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeBlob(runner: GitRunner, bytes: Uint8Array): Promise<string>;
/// ```
pub fn write_blob(runner: &dyn GitRunner, bytes: &[u8]) -> Result<String, String> {
    let reply: GitReply = runner.run(&["hash-object", "-w", "--no-filters", "--stdin"], Some(bytes), None)?;
    if reply.code != Some(0) {
        return Err(format!("git hash-object failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    return Ok(String::from_utf8_lossy(&reply.stdout).trim().to_owned());
}

/// The `update-index --index-info` record that puts a blob at a path as a resolved entry.
pub fn resolved_record(mode: u32, oid: &str, path: &[u8]) -> Vec<u8> {
    let mut record: Vec<u8> = format!("{mode:06o} {oid}\t").into_bytes();
    record.extend_from_slice(path);
    record.push(0);
    return record;
}

/// What:
///  The tree that is `tree` with the named files replaced, from a scratch index.
/// Why:
///   The tree `git merge-tree` wrote holds the conflicted file with markers; replacing it by the merged blob makes the tree
///  of the merge as if Git had merged cleanly. A scratch index is the way to edit a tree with Git's own commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function treeWithFiles(runner, tree: string, files: { path, mode, oid }[], scratchIndex: string): Promise<string>;
/// ```
pub fn tree_with_files(runner: &dyn GitRunner, tree: &str, files: &[(&[u8], u32, &str)], scratch_index: &Path) -> Result<String, String> {
    let read: GitReply = runner.run(&["read-tree", tree], None, Some(scratch_index))?;
    if read.code != Some(0) {
        return Err(format!("git read-tree failed: {}", String::from_utf8_lossy(&read.stderr).trim()));
    }
    let mut records: Vec<u8> = Vec::new();
    for (path, mode, oid) in files {
        records.extend_from_slice(resolved_record(*mode, oid, path).as_slice());
    }
    let updated: GitReply = runner.run(&["update-index", "-z", "--index-info"], Some(records.as_slice()), Some(scratch_index))?;
    if updated.code != Some(0) {
        return Err(format!("git update-index failed: {}", String::from_utf8_lossy(&updated.stderr).trim()));
    }
    let written: GitReply = runner.run(&["write-tree"], None, Some(scratch_index))?;
    if written.code != Some(0) {
        return Err(format!("git write-tree failed: {}", String::from_utf8_lossy(&written.stderr).trim()));
    }
    return Ok(String::from_utf8_lossy(&written.stdout).trim().to_owned());
}

/// The words for a run of lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// span.count === 0 ? `no line (before line ${span.first})` : span.count === 1 ? `line ${span.first}` : `lines ${span.first} to ${span.first + span.count - 1}`
/// ```
fn span_text(span: &LineSpan) -> String {
    return match span.count {
        0 => format!("no line (before line {})", span.first),
        1 => format!("line {}", span.first),
        count => format!("lines {} to {}", span.first, span.first + count - 1),
    };
}

/// What:
///  What a merged part became, as one escaped line of at most `EXCERPT_CHARACTERS` characters.
/// Why:
///   The reader sees the new text without opening the file; escaping keeps a terminal control sequence in a file from
///  reaching the terminal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(text).slice(0, EXCERPT_CHARACTERS)
/// ```
fn excerpt(merged: &[u8]) -> String {
    let escaped: String = String::from_utf8_lossy(merged).escape_debug().to_string();
    return match escaped.char_indices().nth(EXCERPT_CHARACTERS) {
        Some((end, _)) => format!("\"{}...\"", &escaped[..end]),
        None => format!("\"{escaped}\""),
    };
}

/// One part as a sentence fragment: where it was in the three texts and what it became.
pub fn part_text(part: &MergedPart) -> String {
    return format!(
        "base {}, ours {}, theirs {} became {}",
        span_text(&part.base),
        span_text(&part.ours),
        span_text(&part.theirs),
        excerpt(part.merged.as_slice())
    );
}

/// What:
///  The lines that list a merged file: one per part, or one for the file when it needed no word pass.
/// Why:
///   The owner's decision is that every merged part is visible by file and by part.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportLines(prefix: string, merged: Merged): string[];
/// ```
pub fn report_lines(prefix: &str, merged: &Merged) -> Vec<String> {
    let path: String = path_message_text(merged.path.as_slice());
    if merged.parts.is_empty() {
        return vec![format!("{prefix}{REPORT_MARK}: {path}: resolved without a word pass (every region took one side)")];
    }
    return merged
        .parts
        .iter()
        .enumerate()
        .map(|(at, part)| { return format!("{prefix}{REPORT_MARK}: {path}: part {} of {}: {}", at + 1, merged.parts.len(), part_text(part)); })
        .collect();
}

/// What:
///  The message and the fields of the journal entry for a merged file.
/// Why:
///   One entry names the file, the operation and every part, so `journalctl` finds a merge by file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalEntry('merge', merged) // { message, fields }
/// ```
pub fn journal_entry(operation: &str, merged: &Merged) -> (String, Vec<(&'static str, String)>) {
    let path: String = path_message_text(merged.path.as_slice());
    let message: String = report_lines("", merged).join("; ");
    return (message, vec![(FILE_FIELD, path), (OPERATION_FIELD, operation.to_owned()), (PARTS_FIELD, merged.parts.len().to_string())]);
}

/// What:
///  Send one journal entry for a merged file under the identifier of the program that merged it.
/// Why:
///   The wrapper's standard error is a terminal or a capture that is gone when the command ends; the journal keeps the
///  file and each part for later. Code that runs inside the service sends under the service's identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordEntryAs(identifier, 'warning', `word-level merge: ${path}: ${parts.join('; ')}`, { CLI_GIT_WORD_MERGE_FILE: path, ... });
/// ```
pub fn journal_merged(identifier: &str, operation: &str, merged: &Merged) {
    let (message, fields) = journal_entry(operation, merged);
    let sent: bool = record_entry_as(identifier, Priority::Warning, message.as_str(), fields.as_slice());
    debug(TAG, format!("journal entry for a merged file: sent={sent}").as_str());
}

/// Controls for the parsing, the eligibility rules, the objects and the report words.
#[cfg(test)]
#[path = "merge_second_pass_tests.rs"]
pub(crate) mod tests;
