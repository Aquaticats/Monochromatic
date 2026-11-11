//! What:
//!  The intent of a batch:
//! the record of what a landing is about to do,
//! written as `key=value` lines and read back,
//! with the
//!  parts a reader needs (request identities,
//! commits,
//! paths,
//! index,
//! Git directory,
//! cleanup).
//! Why:
//!  The journal (`service_journal.rs`) writes,
//! lists and prunes files of intents;
//! the text form needs no file,
//! so it is a file the
//!  journal uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = intentText(intent); const intent2 = parseIntent(text); // undefined when damaged
//! ```

use crate::path_bytes_in_records::HEX_SUFFIX;
use crate::path_bytes_in_records as path_bytes;
use crate::repo_read_layout::dot_git_target;
use crate::service_protocol::Cleanup;
use std::path::{Path, PathBuf};
/// What:
///  What the service decided to do for a batch,
/// written before the branch moves.
/// Why:
///   Enough to finish the batch after a kill:
/// where the branch was,
/// where it goes,
/// which requests,
/// which index paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Intent = { seq; repository; commonDir; reference; old; new; ids; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    /// Number of the batch,
    /// unique and increasing within one journal.
    pub seq: u64,
    /// Worktree root whose index the batch updates.
    pub repository: PathBuf,
    /// Common git directory.
    pub common_dir: PathBuf,
    /// The branch.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The branch after the batch.
    pub new: String,
    /// Identities of the requests in the batch.
    pub ids: Vec<String>,
    /// The commit each request became,
    /// in the same order as the identities.
    pub commits: Vec<String>,
    /// The command each request says will decide its commit's post-commit checks,
    /// in the same order as the commits,
    /// and empty for a request that named none.
    /// Why:
    ///   The held push of a landed commit says which command will promote it,
    ///   and a batch a restart finishes holds the pushes again:
    ///  the command that would have promoted them is the one the request named,
    /// which no process table of the restart can tell.
    pub holders: Vec<String>,
    /// Paths whose index entries the batch updates.
    pub paths: Vec<Vec<u8>>,
    /// Whether the index entries to update are the paths that differ between `old` and `new`,
    /// read from the repository
    ///  when they are needed (a commit Git built has no list of changes to carry,
    /// and its names are bytes).
    pub derived: bool,
    /// The conclusion state the landing removes after the branch moved;
    /// nothing for any other batch.
    pub cleanup: Option<Cleanup>,
    /// The real index whose lock the landing held,
    /// so a restart can remove its own stale lock and finish the index.
    pub index: PathBuf,
    /// The worktree's own Git directory,
    /// which names the instance that serves the batch;
    /// an intent written before it was
    ///  recorded reads as the Git directory of its repository's worktree.
    pub git_dir: PathBuf,
}

/// What:
///  The lines a path takes in an intent:
/// the plain text under `key` when one line can hold it,
/// and its bytes in hexadecimal under `key` followed by `HEX_SUFFIX` always.
/// Why:
///   A reader prefers the bytes,
/// so a path that is not text survives;
/// the plain line stays for the records of an earlier release and for a person reading the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathLines = (key, path) => [textInOneLine(path) && `${key}=${textInOneLine(path)}`, `${key}Hex=${pathBytesHex(path)}`].filter(Boolean)
/// ```
fn path_lines(key: &str, path: &Path) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if let Some(text) = path_bytes::text_in_one_line(path) {
        lines.push(format!("{key}={text}"));
    }
    lines.push(format!("{key}{HEX_SUFFIX}={}", path_bytes::hex_of_path(path)));
    return lines;
}

/// What:
///  The lines of an intent's cleanup:
/// the Git directory,
/// the scratch directory and the reftable flag,
/// as tab-separated items,
/// in both forms a path takes.
/// Why:
///   The cleanup is one field of the intent,
/// so its two paths are one line,
/// and a reader prefers the line that holds their bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanupLines = (cleanup) => cleanup ? [`cleanup=${gitDir}\t${scratch}\t${flag}`, `cleanupHex=${hex(gitDir)}\t${hex(scratch)}\t${flag}`] : []
/// ```
fn cleanup_lines(cleanup: Option<&Cleanup>) -> Vec<String> {
    let Some(found) = cleanup else {
        return Vec::new();
    };
    let mut lines: Vec<String> = Vec::new();
    if let (Some(git_dir), Some(scratch)) = (path_bytes::text_in_one_line(found.git_dir.as_path()), path_bytes::text_in_one_line(found.scratch.as_path())) {
        lines.push(format!("cleanup={git_dir}\t{scratch}\t{}", u8::from(found.reftable)));
    }
    lines.push(format!("cleanup{HEX_SUFFIX}={}\t{}\t{}", path_bytes::hex_of_path(found.git_dir.as_path()), path_bytes::hex_of_path(found.scratch.as_path()), u8::from(found.reftable)));
    return lines;
}

/// What a request that named no deciding command is spelled as in an intent.
/// Why:
///   The field is positional,
/// one item per landed commit,
/// and the list reader drops empty items,
/// so silence needs a word of its own
///  or the entries after it move to another commit.
pub const NO_HOLDER: &str = "-";

/// What:
///  The item one holder takes in an intent,
///  or [`NO_HOLDER`] when the request named none.
/// Why:
///   See [`NO_HOLDER`]:
/// an empty item would vanish and shift every later commit's command onto another commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holderText = (holder) => holder === '' ? '-' : holder;
/// ```
fn holder_text(holder: &str) -> String {
    return match holder.is_empty() {
        true => String::from(NO_HOLDER),
        false => holder.to_owned(),
    };
}

/// The text of an intent:
/// `key=value` lines,
/// tabs between list items.
pub fn intent_text(intent: &Intent) -> String {
    let mut lines: Vec<String> = vec![format!("seq={}", intent.seq)];
    lines.extend(path_lines("repository", intent.repository.as_path()));
    lines.extend(path_lines("common", intent.common_dir.as_path()));
    lines.extend([
        format!("ref={}", intent.reference),
        format!("old={}", intent.old),
        format!("new={}", intent.new),
        format!("ids={}", intent.ids.join("\t")),
        format!("commits={}", intent.commits.join("\t")),
        format!("holders={}", intent.holders.iter().map(String::as_str).map(holder_text).collect::<Vec<String>>().join("\t")),
        format!("paths={}", intent.paths.iter().map(Vec::as_slice).map(String::from_utf8_lossy).map(std::borrow::Cow::into_owned).collect::<Vec<String>>().join("\t")),
        format!("paths{HEX_SUFFIX}={}", intent.paths.iter().map(Vec::as_slice).map(path_bytes::hex_of_bytes).collect::<Vec<String>>().join("\t")),
        format!("derived={}", u8::from(intent.derived)),
    ]);
    lines.extend(cleanup_lines(intent.cleanup.as_ref()));
    lines.extend(path_lines("index", intent.index.as_path()));
    lines.extend(path_lines("gitdir", intent.git_dir.as_path()));
    return format!("{}\n", lines.join("\n"));
}

/// What:
///  The worktree's own Git directory,
/// read from the worktree's `.git` entry:
/// the entry itself when it is a directory,
/// the
///  directory a `gitdir:` file names for a linked worktree,
/// and `.git` under the worktree when it cannot be read.
/// Why:
///   An intent written before it recorded its Git directory has only the repository to name the worktree by;
/// the worktree's
///  instance is the hash of this directory,
/// so the recovery that finds the intent must resolve it the way a commit does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDirOfWorktree('/work/repo') // '/work/repo/.git', or '/work/main/.git/worktrees/repo' for a linked worktree
/// ```
pub(crate) fn git_dir_of_worktree(repository: &Path) -> PathBuf {
    return match dot_git_target(repository) {
        Some(Some((git_dir, _))) => git_dir,
        Some(None) | None => repository.join(".git"),
    };
}

/// What:
///  The value of the first `key=value` line of an intent text whose key is `key`.
/// Why:
///  `parse_intent` reads every field of the intent this way;
/// a line with the key but no equals sign is not the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').map((line) => line.startsWith(key + '=') ? line.slice(key.length + 1) : undefined).find((value) => value !== undefined)
/// ```
pub(crate) fn intent_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(key)
            && let Some(found) = rest.strip_prefix('=')
        {
            return Some(found);
        }
    }
    return None;
}

/// What:
///  The path an intent holds under `key`:
/// the bytes of the line that follows it with `HEX_SUFFIX` when the intent has one,
/// the plain text otherwise,
/// and nothing when it holds neither.
/// Why:
///   A path that is not text survives only in the bytes,
/// and an intent an earlier release wrote holds only the plain line.
/// An error is damage:
/// a line of bytes that is there and does not decode is no path,
/// and a reader that took it for an absent line would name the place the older form names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intentPath = (text, key) => pathOfTwoFields(value(text, key + 'Hex'), value(text, key));
/// ```
pub(crate) fn intent_path(text: &str, key: &str) -> Result<Option<PathBuf>, String> {
    return path_bytes::path_of_the_two_fields(intent_value(text, format!("{key}{HEX_SUFFIX}").as_str()), intent_value(text, key));
}

/// What:
///  The tab-separated items of an intent field,
/// empty items left out.
/// Why:
///  The request identities,
/// commits and paths of an intent are lists in one line each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value(key)?.split('\t').filter((item) => item !== '')
/// ```
pub(crate) fn intent_list(text: &str, key: &str) -> Option<Vec<String>> {
    let found: &str = intent_value(text, key)?;
    let mut items: Vec<String> = Vec::new();
    for item in found.split('\t') {
        if !item.is_empty() {
            items.push(item.to_string());
        }
    }
    return Some(items);
}

/// What:
///  The holders an intent's line holds,
///  with [`NO_HOLDER`] read back as a request that named none.
/// Why:
///   The item is the command the request named,
/// so the word that stands for silence is never a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdersOf = (text) => intentList(text, 'holders')?.map((item) => item === '-' ? '' : item);
/// ```
fn holders_of(text: &str) -> Option<Vec<String>> {
    let items: Vec<String> = intent_list(text, "holders")?;
    return Some(items.iter().map(String::as_str).map(holder_of_item).collect());
}

/// What:
///  The command one item of an intent's holder line names,
///  and nothing when the item is the word for silence.
/// Why:
///   [`NO_HOLDER`] stands for a request that named no deciding command,
/// and the item is read back as that silence rather
///  than as a command called `-`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holderOfItem = (item) => item === '-' ? '' : item;
/// ```
fn holder_of_item(item: &str) -> String {
    return match item {
        NO_HOLDER => String::new(),
        command => command.to_owned(),
    };
}

/// What:
///  The paths of an intent's list:
/// the items of the line of bytes when the intent has one,
/// the items of the plain line otherwise.
/// Why:
///   A path that is not text survives only in its bytes,
/// and an intent an earlier release wrote holds only the plain line.
/// An item that does not decode is damage and rejects the intent,
/// because the entries it names would be updated under another name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intentPathList = (text, key) => { const hex = value(text, key + 'Hex'); return (hex ?? value(text, key))?.split('\t').filter(Boolean).map((item) => hex === undefined ? item : pathOfHex(item)); };
/// ```
pub(crate) fn intent_path_list(text: &str, key: &str) -> Result<Option<Vec<Vec<u8>>>, String> {
    let items: Vec<&str> = match intent_value(text, format!("{key}{HEX_SUFFIX}").as_str()) {
        Some(bytes) => {
            let mut paths: Vec<Vec<u8>> = Vec::new();
            for item in bytes.split('\t') {
                if item.is_empty() {
                    continue;
                }
                let Some(path) = path_bytes::path_of_hex(item) else {
                    return Err(String::from("an item of a list of bytes is not the bytes of a path"));
                };
                paths.push(crate::path_bytes::path_bytes_of(path.as_path()).to_vec());
            }
            return Ok(Some(paths));
        }
        None => match intent_value(text, key) {
            Some(plain) => plain.split('\t').collect(),
            None => return Ok(None),
        },
    };
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for item in items {
        if !item.is_empty() {
            paths.push(item.as_bytes().to_vec());
        }
    }
    return Ok(Some(paths));
}

/// What:
///  The cleanup of an intent from its field:
/// the Git directory,
/// the scratch directory and the reftable flag,
/// tab-separated.
/// Why:
///  `parse_intent` reads it with `?`,
/// so a cleanup whose bytes do not decode is no intent;
/// a field with fewer than three parts is no cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [gitDir, scratch, reftable] = entry.split('\t'); return reftable === undefined ? undefined : { gitDir, scratch, reftable: reftable === '1' };
/// ```
pub(crate) fn parse_cleanup(entry: &str, hexadecimal: bool) -> Option<Cleanup> {
    let mut parts = entry.split('\t');
    let (git_dir, scratch, reftable) = (parts.next()?, parts.next()?, parts.next()?);
    if hexadecimal {
        return Some(Cleanup { git_dir: path_bytes::path_of_hex(git_dir)?, scratch: path_bytes::path_of_hex(scratch)?, reftable: reftable == "1" });
    }
    return Some(Cleanup { git_dir: PathBuf::from(git_dir), scratch: PathBuf::from(scratch), reftable: reftable == "1" });
}

/// What:
///  Reads an intent's text.
/// Why:
///   A damaged file is no intent:
/// recovery drops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIntent(text) // undefined when damaged
/// ```
pub fn parse_intent(text: &str) -> Option<Intent> {
    let repository: PathBuf = intent_path(text, "repository").ok().flatten()?;
    // An intent written before the index was recorded names it as the repository's own; one whose line of bytes is damaged
    //  is no intent, because the ordinary index is not where that batch was prepared.
    let index: PathBuf = match intent_path(text, "index") {
        Ok(Some(named)) => named,
        Ok(None) => repository.join(".git").join("index"),
        Err(_damaged) => return None,
    };
    // An intent written before the Git directory was recorded names its worktree by the repository it was made for, never
    //  by the directory its index sits in, which `GIT_INDEX_FILE` can put anywhere.
    let worktree_git_dir: PathBuf = match intent_path(text, "gitdir") {
        Ok(Some(named)) => named,
        Ok(None) => git_dir_of_worktree(repository.as_path()),
        Err(_damaged) => return None,
    };
    // The line that holds the paths' bytes is the one a reader takes; the plain line is an earlier release's.
    let cleanup: Option<Cleanup> = match intent_value(text, format!("cleanup{HEX_SUFFIX}").as_str()) {
        Some(hexadecimal) => Some(parse_cleanup(hexadecimal, true)?),
        None => match intent_value(text, "cleanup") {
            Some(plain) => parse_cleanup(plain, false),
            None => None,
        },
    };
    return Some(Intent {
        seq: intent_value(text, "seq")?.parse().ok()?,
        repository,
        common_dir: intent_path(text, "common").ok().flatten()?,
        reference: intent_value(text, "ref")?.to_string(),
        old: intent_value(text, "old")?.to_string(),
        new: intent_value(text, "new")?.to_string(),
        ids: intent_list(text, "ids")?,
        commits: intent_list(text, "commits")?,
        // Intents written before this field existed name no command.
        holders: holders_of(text).unwrap_or_default(),
        paths: intent_path_list(text, "paths").ok()??,
        // Intents written before these fields existed name neither.
        derived: intent_value(text, "derived") == Some("1"),
        index,
        git_dir: worktree_git_dir,
        cleanup,
    });
}
