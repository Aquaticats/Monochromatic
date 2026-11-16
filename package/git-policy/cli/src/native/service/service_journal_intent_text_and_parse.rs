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
    /// The branch,
    /// by the bytes Git printed for its name.
    pub reference: crate::ref_name::RefName,
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
/// the plain text under `key`,
/// escaped by [`path_bytes::encode_field`],
/// and its bytes in hexadecimal under `key` followed by `HEX_SUFFIX` always.
/// Why:
///   A reader prefers the bytes,
/// so a path that is not text survives;
/// the plain line stays for the records of an earlier release and for a person reading the file,
/// and its escape is what keeps a path's line break from ending the line,
/// or its tab from ending the field,
/// and starting one nobody wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathLines = (key, path) => [`${key}=${encodeField(pathBytes(path))}`, `${key}Hex=${pathBytesHex(path)}`]
/// ```
fn path_lines(key: &str, path: &Path) -> Vec<String> {
    let bytes: &[u8] = crate::path_bytes::path_bytes_of(path);
    return vec![format!("{key}={}", path_bytes::encode_field(bytes)), format!("{key}{HEX_SUFFIX}={}", path_bytes::hex_of_bytes(bytes))];
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
/// const cleanupLines = (cleanup) => cleanup ? [`cleanup=${encodeField(gitDir)}\t${encodeField(scratch)}\t${flag}`, `cleanupHex=${hex(gitDir)}\t${hex(scratch)}\t${flag}`] : []
/// ```
fn cleanup_lines(cleanup: Option<&Cleanup>) -> Vec<String> {
    let Some(found) = cleanup else {
        return Vec::new();
    };
    let git_dir: &[u8] = crate::path_bytes::path_bytes_of(found.git_dir.as_path());
    let scratch: &[u8] = crate::path_bytes::path_bytes_of(found.scratch.as_path());
    return vec![
        format!("cleanup={}\t{}\t{}", path_bytes::encode_field(git_dir), path_bytes::encode_field(scratch), u8::from(found.reftable)),
        format!("cleanup{HEX_SUFFIX}={}\t{}\t{}", path_bytes::hex_of_bytes(git_dir), path_bytes::hex_of_bytes(scratch), u8::from(found.reftable)),
    ];
}

/// What:
///  The reference name of an intent,
///  as the two lines a reader takes it from.
/// Why:
///   A recovery moves this branch,
/// so `refHex=` always holds the whole name and `ref=` holds it escaped,
/// which is the field a release before the bytes field reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const referenceField = (name) => `ref=${encodeField(name.bytes)}\nrefHex=${name.hex}`
/// ```
fn reference_field(reference: &crate::ref_name::RefName) -> String {
    return format!("ref={}\nref{HEX_SUFFIX}={}", path_bytes::encode_field(reference.bytes()), reference.hex());
}

/// What:
///  One text value of an intent's line,
///  escaped, so the line stays one line and one field.
/// Why:
///   Every value of an intent's line is written by this one rule,
///  so a field added later cannot forget it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldText = (text) => encodeField(Buffer.from(text))
/// ```
fn field_text(text: &str) -> String {
    return path_bytes::encode_field(text.as_bytes());
}

/// What:
///  A list value of an intent's line:
/// its items escaped and joined by tabs.
/// Why:
///   A tab separates the items,
/// so an item that holds one would add an item nobody wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldList = (items) => items.map(fieldText).join('\t')
/// ```
fn field_list(items: &[String]) -> String {
    return items.iter().map(|item| { return field_text(item.as_str()); }).collect::<Vec<String>>().join("\t");
}

/// What:
///  The bytes one value of an intent's line holds,
///  undone from its escape when the record declares it,
///  and the text it spelled when the record is an earlier release's.
/// Why:
///   The escape is the writer's ([`path_bytes::encode_field`]),
///  so undoing it is the reader's,
///  and a value that does not undo is damage:
///  a path read wrongly names another place at recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldBytes = (text, key, encoded) => { const value = value(text, key); return value === undefined ? undefined : encoded ? decodeField(value) : Buffer.from(value) }
/// ```
fn field_bytes(text: &str, key: &str, encoded: bool) -> Option<Vec<u8>> {
    let value: &str = intent_value(text, key)?;
    return match encoded {
        true => path_bytes::decode_field(value),
        false => Some(value.as_bytes().to_vec()),
    };
}

/// What:
///  The text one value of an intent's line holds,
///  and nothing when the record holds no such field or its value does not decode.
/// Why:
///   Every text value of a record this release writes is escaped,
/// and one an earlier release wrote is the text itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldTextOf = (text, key, encoded) => { const bytes = fieldBytes(text, key, encoded); return bytes && new TextDecoder().decode(bytes) }
/// ```
fn field_text_of(text: &str, key: &str, encoded: bool) -> Option<String> {
    return Some(String::from_utf8(field_bytes(text, key, encoded)?).ok()?);
}

/// What:
///  The reference name an intent holds,
///  from its bytes field when it has one,
///  and from its text otherwise.
/// Why:
///   An intent written before the bytes field existed holds text,
/// and an intent whose bytes do not decode is damage,
///  which its reader rejects:
/// a name read wrongly moves another branch at recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readReference = (text, encoded) => text.refHex !== undefined ? refNameOfHex(text.refHex) : refNameOf(fieldTextOf(text, 'ref', encoded))
/// ```
fn read_reference(text: &str, encoded: bool) -> Option<crate::ref_name::RefName> {
    if let Some(hexadecimal) = intent_value(text, "refHex") {
        return crate::ref_name::RefName::from_hex(hexadecimal);
    }
    return Some(crate::ref_name::RefName::from_bytes(field_bytes(text, "ref", encoded)?.as_slice()));
}

/// How many of a cleanup field's tab-separated items are paths; the last one is the reftable flag.
const CLEANUP_PATH_ITEMS: usize = 2;

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

/// What:
///  The text of an intent:
/// the format's header line,
/// then `key=value` lines,
/// tabs between list items,
/// and every value escaped by [`path_bytes::encode_field`].
/// Why:
///   The header is what tells a reader that these values are escaped,
///  so a record an earlier release wrote is still read as the text it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intentText = (intent) => FIELD_ENCODING_HEADER + [seq, ...paths, ...cleanup, ref, old, new, list(ids), list(commits), list(holders), list(paths), list(pathsHex), derived].join('\n') + '\n'
/// ```
pub fn intent_text(intent: &Intent) -> String {
    let mut lines: Vec<String> = vec![format!("seq={}", intent.seq)];
    lines.extend(path_lines("repository", intent.repository.as_path()));
    lines.extend(path_lines("common", intent.common_dir.as_path()));
    lines.extend([
        reference_field(&intent.reference),
        format!("old={}", field_text(intent.old.as_str())),
        format!("new={}", field_text(intent.new.as_str())),
        format!("ids={}", field_list(intent.ids.as_slice())),
        format!("commits={}", field_list(intent.commits.as_slice())),
        format!("holders={}", field_list(intent.holders.iter().map(|holder| { return holder_text(holder.as_str()); }).collect::<Vec<String>>().as_slice())),
        format!("paths={}", intent.paths.iter().map(|path| { return path_bytes::encode_field(path.as_slice()); }).collect::<Vec<String>>().join("\t")),
        format!("paths{HEX_SUFFIX}={}", intent.paths.iter().map(Vec::as_slice).map(path_bytes::hex_of_bytes).collect::<Vec<String>>().join("\t")),
        format!("derived={}", u8::from(intent.derived)),
    ]);
    lines.extend(cleanup_lines(intent.cleanup.as_ref()));
    lines.extend(path_lines("index", intent.index.as_path()));
    lines.extend(path_lines("gitdir", intent.git_dir.as_path()));
    return format!("{}{}\n", path_bytes::FIELD_ENCODING_HEADER, lines.join("\n"));
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
/// the plain value otherwise,
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
/// const intentPath = (text, key, encoded) => pathOfTwoFields(value(text, key + 'Hex'), fieldBytes(text, key, encoded));
/// ```
pub(crate) fn intent_path(text: &str, key: &str, encoded: bool) -> Result<Option<PathBuf>, String> {
    if let Some(hexadecimal) = intent_value(text, format!("{key}{HEX_SUFFIX}").as_str()) {
        return path_bytes::path_of_the_two_fields(Some(hexadecimal), None);
    }
    return match field_bytes(text, key, encoded) {
        Some(bytes) => Ok(Some(path_bytes::path_of_bytes(bytes.as_slice()))),
        None if intent_value(text, key).is_none() => Ok(None),
        None => Err(format!("the {key} field is not the bytes of a path")),
    };
}

/// What:
///  The tab-separated items of an intent field,
///  each undone from its escape,
/// empty items left out.
/// Why:
///  The request identities,
/// commits and paths of an intent are lists in one line each,
/// and an item that does not undo is damage,
///  which its reader rejects rather than reading as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intentList = (text, key, encoded) => value(text, key)?.split('\t').filter((item) => item !== '').map((item) => fieldTextOfItem(item, encoded))
/// ```
pub(crate) fn intent_list(text: &str, key: &str, encoded: bool) -> Option<Vec<String>> {
    let found: &str = intent_value(text, key)?;
    let mut items: Vec<String> = Vec::new();
    for item in found.split('\t') {
        if item.is_empty() {
            continue;
        }
        let bytes: Vec<u8> = match encoded {
            true => path_bytes::decode_field(item)?,
            false => item.as_bytes().to_vec(),
        };
        items.push(String::from_utf8(bytes).ok()?);
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
/// const holdersOf = (text, encoded) => intentList(text, 'holders', encoded)?.map((item) => item === '-' ? '' : item);
/// ```
fn holders_of(text: &str, encoded: bool) -> Option<Vec<String>> {
    let items: Vec<String> = intent_list(text, "holders", encoded)?;
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
/// const intentPathList = (text, key, encoded) => { const hex = value(text, key + 'Hex'); const plain = value(text, key); return hex !== undefined ? items(hex).map(pathOfHex) : plain === undefined ? undefined : items(plain).map((item) => encoded ? decodeField(item) : Buffer.from(item)) };
/// ```
pub(crate) fn intent_path_list(text: &str, key: &str, encoded: bool) -> Result<Option<Vec<Vec<u8>>>, String> {
    if let Some(bytes) = intent_value(text, format!("{key}{HEX_SUFFIX}").as_str()) {
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
    let Some(plain) = intent_value(text, key) else {
        return Ok(None);
    };
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for item in plain.split('\t') {
        if item.is_empty() {
            continue;
        }
        let path: Vec<u8> = match encoded {
            true => match path_bytes::decode_field(item) {
                Some(bytes) => bytes,
                None => return Err(String::from("an item of a list of paths does not decode")),
            },
            false => item.as_bytes().to_vec(),
        };
        paths.push(path);
    }
    return Ok(Some(paths));
}

/// The three parts a cleanup field's line is read by.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CleanupForm {
    /// The line of hexadecimal bytes this release writes.
    Hexadecimal,
    /// The line of escaped values this release writes,
    /// which is what a person reads.
    Encoded,
    /// The line of plain text an earlier release wrote.
    Plain,
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
/// const [gitDir, scratch, reftable] = entry.split('\t'); return reftable === undefined ? undefined : { gitDir: pathOf(gitDir), scratch: pathOf(scratch), reftable: reftable === '1' };
/// ```
pub(crate) fn parse_cleanup(entry: &str, form: CleanupForm) -> Option<Cleanup> {
    let mut parts = entry.split('\t');
    let (written_git_dir, written_scratch, reftable) = (parts.next()?, parts.next()?, parts.next()?);
    let (git_dir, scratch): (PathBuf, PathBuf) = match form {
        CleanupForm::Hexadecimal => (path_bytes::path_of_hex(written_git_dir)?, path_bytes::path_of_hex(written_scratch)?),
        CleanupForm::Encoded => (
            path_bytes::path_of_bytes(path_bytes::decode_field(written_git_dir)?.as_slice()),
            path_bytes::path_of_bytes(path_bytes::decode_field(written_scratch)?.as_slice()),
        ),
        CleanupForm::Plain => (PathBuf::from(written_git_dir), PathBuf::from(written_scratch)),
    };
    return Some(Cleanup { git_dir, scratch, reftable: reftable == "1" });
}

/// What:
///  Whether every field this release writes twice — as the text a person reads and as the bytes it acts on —
///  names the same value both ways.
/// Why:
///   The two fields of one value are written from the same bytes,
/// so a record whose two disagree was written by another program or cut apart,
///  and the field a reader acts on would then name something nobody chose.
/// A line break in a path of a record an earlier release wrote is exactly that case:
///  its plain field ends early and the rest of it is read as a field of its own.
/// The fields are tab-separated lists of one or more items,
///  so they are compared item by item,
///  each against the lossy text the earlier release spelled it with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bothFieldsNameOneValue = (text) => ['repository', 'common', 'index', 'gitdir', 'ref', 'paths', 'cleanup'].every((key) => agree(value(text, key), value(text, key + 'Hex')))
/// ```
fn both_fields_name_one_value(text: &str, encoded: bool) -> bool {
    for key in ["repository", "common", "index", "gitdir", "ref", "paths", "cleanup"] {
        let (Some(plain), Some(hexadecimal)) = (intent_value(text, key), intent_value(text, format!("{key}{HEX_SUFFIX}").as_str())) else {
            continue;
        };
        let spelled: Vec<&str> = plain.split('\t').collect();
        let bytes: Vec<&str> = hexadecimal.split('\t').collect();
        // A cleanup's last item is the reftable flag and not a path, so the two fields are compared item by item.
        if matches!(key, "cleanup") {
            if spelled.len() != bytes.len() {
                return false;
            }
            for (number, (written, digits)) in spelled.iter().zip(bytes).enumerate() {
                // The last item is the reftable flag, which is the same word on both lines and spells no bytes.
                if number >= CLEANUP_PATH_ITEMS {
                    if written.as_bytes() != digits.as_bytes() {
                        return false;
                    }
                    continue;
                }
                let Some(decoded) = path_bytes::bytes_of_hex(digits) else {
                    return false;
                };
                if !one_value_is_the_bytes(written, decoded.as_slice(), encoded) {
                    return false;
                }
            }
            continue;
        }
        // The items are compared as the one field they are, so the tab between them is part of what is compared: a path of the
        //  earlier release may hold one itself.
        let mut items: Vec<Vec<u8>> = Vec::new();
        for digits in &bytes {
            let Some(found) = path_bytes::bytes_of_hex(digits) else {
                return false;
            };
            items.push(found);
        }
        if !one_value_is_the_bytes(plain, items.join(&b'\t').as_slice(), encoded) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether one plain value of a record spells the bytes beside it.
/// Why:
///   A record of this release escapes the value, so undoing the escape is what it must spell;
///  a record of the earlier release wrote the lossy text of those bytes,
///  and a line reader takes the carriage return that ends a line off that text,
///  so a value that ends there names the same bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneValueIsTheBytes = (written, bytes, encoded) => encoded ? decodeField(written)?.equals(bytes) ?? false : written === lossy(bytes) || `${written}\r` === lossy(bytes)
/// ```
fn one_value_is_the_bytes(written: &str, bytes: &[u8], encoded: bool) -> bool {
    return match encoded {
        true => path_bytes::decode_field(written).as_deref() == Some(bytes),
        false => {
            let spelled: String = path_bytes::lossy_text_of(bytes);
            written == spelled || format!("{written}\r") == spelled
        }
    };
}

/// What:
///  Reads an intent's text.
/// Why:
///   A damaged file is no intent:
/// recovery drops it,
///  which is the safe side of every question it asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIntent(text) // undefined when damaged
/// ```
pub fn parse_intent(text: &str) -> Option<Intent> {
    // Whether the values of this record's lines were escaped is what its first line says; a record of an earlier release
    //  holds them as the text it wrote.
    let encoded: bool = path_bytes::carries_encoded_fields(text);
    if !both_fields_name_one_value(text, encoded) {
        return None;
    }
    let repository: PathBuf = intent_path(text, "repository", encoded).ok().flatten()?;
    // An intent written before the index was recorded names it as the repository's own; one whose line of bytes is damaged
    //  is no intent, because the ordinary index is not where that batch was prepared.
    let index: PathBuf = match intent_path(text, "index", encoded) {
        Ok(Some(named)) => named,
        Ok(None) => repository.join(".git").join("index"),
        Err(_damaged) => return None,
    };
    // An intent written before the Git directory was recorded names its worktree by the repository it was made for, never
    //  by the directory its index sits in, which `GIT_INDEX_FILE` can put anywhere.
    let worktree_git_dir: PathBuf = match intent_path(text, "gitdir", encoded) {
        Ok(Some(named)) => named,
        Ok(None) => git_dir_of_worktree(repository.as_path()),
        Err(_damaged) => return None,
    };
    // The line that holds the paths' bytes is the one a reader takes; the plain line is a person's, and an earlier release's.
    let cleanup: Option<Cleanup> = match intent_value(text, format!("cleanup{HEX_SUFFIX}").as_str()) {
        Some(hexadecimal) => Some(parse_cleanup(hexadecimal, CleanupForm::Hexadecimal)?),
        None => match intent_value(text, "cleanup") {
            // An empty field names no cleanup, as an earlier release wrote it.
            Some("") => None,
            // A cleanup field that is there and does not read is damage, and never a batch that named none.
            Some(plain) => Some(parse_cleanup(plain, match encoded {
                true => CleanupForm::Encoded,
                false => CleanupForm::Plain,
            })?),
            None => None,
        },
    };
    return Some(Intent {
        seq: field_text_of(text, "seq", encoded)?.parse().ok()?,
        repository,
        common_dir: intent_path(text, "common", encoded).ok().flatten()?,
        reference: read_reference(text, encoded)?,
        old: field_text_of(text, "old", encoded)?,
        new: field_text_of(text, "new", encoded)?,
        ids: intent_list(text, "ids", encoded)?,
        commits: intent_list(text, "commits", encoded)?,
        // Intents written before this field existed name no command, and one whose field is there and does not undo is damage.
        holders: match intent_value(text, "holders") {
            Some(_present) => holders_of(text, encoded)?,
            None => Vec::new(),
        },
        paths: intent_path_list(text, "paths", encoded).ok()??,
        // Intents written before these fields existed name neither.
        derived: field_text_of(text, "derived", encoded).as_deref() == Some("1"),
        index,
        git_dir: worktree_git_dir,
        cleanup,
    });
}
