//! What:
//!  One read of the whole Git configuration (`git config --list -z --show-scope`),
//! parsed here,
//!  and the lookups the commit path makes against it instead of starting `git config` for each key.
//! Why:
//!  Every `git config` start costs a process,
//! and a wrapped commit asked Git the same configuration
//!  several times (`core.hooksPath`,
//! the hook switches,
//! the maintenance keys,
//! the ref storage).
//!  Git lists entries in precedence order (system,
//! global,
//! local,
//! worktree,
//! then `GIT_CONFIG_COUNT`,
//!  `GIT_CONFIG_PARAMETERS` and `-c` values,
//! all in the `command` scope),
//! so the last entry of a key is the
//!  value `git config --get` prints.
//!  A lookup answers only when it can reproduce Git's own answer exactly;
//!  any value it does not model (`~` and `%(prefix)` paths,
//! unit suffixes,
//! hexadecimal,
//! a bad boolean) is
//!  [`Lookup::Unhandled`],
//! and the caller starts Git for that key as before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const snapshot = parseConfigList(await git(['config', '--list', '-z', '--show-scope']));
//! const hooksPath = snapshot.pathValue('core.hookspath'); // 'unhandled' means: ask Git
//! ```

/// Debug logging of a snapshot that could not be used.
use crate::diagnostic_log::debug;
/// One required Git start with the caller's context.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// The same start awaited by a caller that is a task of the runtime.
use crate::transaction_git_awaiting::run_git_awaiting;
/// The components one `core.fsync` value makes Git flush.
use crate::config_fsync::FsyncComponents;
/// Ordered map of the four maintenance keys to the text Git prints for them.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Largest decimal integer text a lookup models;
/// longer digit strings fall back to Git.
const DECIMAL_DIGITS_LIMIT: usize = 9;

/// Keys the maintenance decision reads,
/// spelled as `git config --list` prints them.
pub const MAINTENANCE_KEYS: [&str; 4] = ["maintenance.auto", "maintenance.autodetach", "gc.auto", "gc.autodetach"];

/// The scope name Git lists for the values that came from a command line or its environment rather than a file:
/// `-c`,
/// `GIT_CONFIG_COUNT` and `GIT_CONFIG_PARAMETERS`.
const COMMAND_SCOPE: &str = "command";

/// What:
///  One configuration entry in listing order.
/// Why:
///   The scope is kept so a consumer can tell `-c` values from file values,
/// and `None` is a valueless key,
///  which Git reads as `true` for booleans and refuses for paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigEntry = { scope: string; key: string; value: Uint8Array | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    /// `system`,
    /// `global`,
    /// `local`,
    /// `worktree` or `command`.
    pub scope: String,
    /// Canonical key:
    /// lowercase section and variable,
    /// subsection kept as written.
    pub key: String,
    /// Value bytes,
    /// or `None` for a key written without `=`.
    pub value: Option<Vec<u8>>,
}

/// What:
///  The answer of one lookup.
/// Why:
///   "Not set" and "set to this" are Git's answers;
/// "unhandled" is this module declining to guess,
///  so the caller falls back to starting Git,
/// which is never a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Lookup<T> = { kind: 'unset' } | { kind: 'found'; value: T } | { kind: 'unhandled' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lookup<Value> {
    /// No entry has the key,
    /// so `git config --get` exits 1.
    Unset,
    /// The last entry of the key,
    /// converted exactly as Git converts it.
    Found(Value),
    /// Some entry of the key has a shape this module does not reproduce;
    /// ask Git.
    Unhandled,
}

/// What:
///  The parsed listing of one Git configuration read.
/// Why:
///   It is read once per commit with the caller's global options,
/// and reused by the consumers that would
///  otherwise each start `git config`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigSnapshot = { entries: ConfigEntry[]; readWithoutGlobalOptions: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigSnapshot {
    /// Entries in the order Git listed them,
    /// which is precedence order.
    entries: Vec<ConfigEntry>,
    /// Whether the read carried no global options,
    /// so it equals a read made without them.
    without_global_options: bool,
}

/// What:
///  Whether the bytes of one value are a plain decimal integer the lookups model,
/// and its numeric value.
/// Why:
///   Git parses integers with `strtoimax` base 0 and unit suffixes,
/// so only `0` and digit strings that start with
///  `1` to `9` are decided here;
/// leading zeros (octal),
/// signs,
/// `0x`,
/// spaces and suffixes fall back to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^(0|[1-9][0-9]{0,8})$/.test(text) ? Number(text) : undefined;
/// ```
fn plain_decimal(text: &str) -> Option<u32> {
    if text == "0" {
        return Some(0);
    }
    let first: u8 = *text.as_bytes().first()?;
    if !(b'1'..=b'9').contains(&first) || text.len() > DECIMAL_DIGITS_LIMIT {
        return None;
    }
    if !text.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    return text.parse::<u32>().ok();
}

/// What:
///  Git's boolean word list,
/// case-insensitively:
/// `Some(true)` for true,
/// yes and on,
/// `Some(false)` for false,
/// no
///  and off,
/// `None` for any other text.
/// Why:
///   `git_parse_maybe_bool_text` compares with `strcasecmp`,
/// which is ASCII case folding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ true: true, yes: true, on: true, false: false, no: false, off: false })[text.toLowerCase()];
/// ```
fn boolean_word(text: &str) -> Option<bool> {
    for word in ["true", "yes", "on"] {
        if text.eq_ignore_ascii_case(word) {
            return Some(true);
        }
    }
    for word in ["false", "no", "off"] {
        if text.eq_ignore_ascii_case(word) {
            return Some(false);
        }
    }
    return None;
}

/// What:
///  One byte as a digit of `radix`,
/// or nothing when it is not one.
/// Why:
///   `strtoimax` stops at the first byte that is not a digit of the radix it chose,
/// and the loop that walks the
///  digits needs that answer as a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseInt(String.fromCharCode(byte), radix); // NaN when it is not a digit
/// ```
fn digit_of(byte: u8, radix: u32) -> Option<u32> {
    return (byte as char).to_digit(radix);
}

/// What:
///  One integer as Git's own reader spells it:
/// `strtoimax` with base 0,
/// then an optional `k`,
/// `m` or `g` unit.
/// Why:
///   `git_parse_signed` reads an integer this way and multiplies it by the unit,
/// so `00` is octal zero,
///  `0x0` is zero,
/// `-0` and `+0` are zero,
/// `0k` is zero,
/// `010` is eight,
/// and `1k` is 1024.
///  A text that reader refuses (a trailing space,
/// a digit the radix does not hold,
/// a value past the range of a
///  signed 64-bit integer) is `None`,
/// which means Git reads it as something else,
/// such as a hook path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitInteger('0x10'); // 16
/// ```
fn git_integer(text: &str) -> Option<i64> {
    let bytes: &[u8] = text.as_bytes();
    let mut at: usize = 0;
    // `strtoimax` skips leading whitespace and takes one sign.
    while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    let negative: bool = match bytes.get(at) {
        Some(b'-') => {
            at += 1;
            true
        }
        Some(b'+') => {
            at += 1;
            false
        }
        _ => false,
    };
    let (radix, first_digit): (u32, usize) = match (bytes.get(at), bytes.get(at + 1).map(u8::to_ascii_lowercase)) {
        (Some(b'0'), Some(b'x')) => (16, at + 2),
        (Some(b'0'), _) => (8, at),
        _ => (10, at),
    };
    let mut magnitude: i64 = 0;
    let mut digits: usize = 0;
    let mut cursor: usize = first_digit;
    loop {
        let Some(byte) = bytes.get(cursor) else { break };
        let Some(digit) = digit_of(*byte, radix) else { break };
        magnitude = magnitude.checked_mul(i64::from(radix))?.checked_add(i64::from(digit))?;
        digits += 1;
        cursor += 1;
    }
    if digits == 0 {
        return None;
    }
    let factor: i64 = match bytes.get(cursor..) {
        Some(b"") => 1,
        Some(tail) if tail.eq_ignore_ascii_case(b"k") => 1024,
        Some(tail) if tail.eq_ignore_ascii_case(b"m") => 1024 * 1024,
        Some(tail) if tail.eq_ignore_ascii_case(b"g") => 1024 * 1024 * 1024,
        _ => return None,
    };
    let scaled: i64 = magnitude.checked_mul(factor)?;
    return Some(if negative { -scaled } else { scaled });
}

/// What:
///  `--type=bool` of one value:
/// a valueless key is true,
/// empty text is false,
/// a word from the list is itself,
///  and any other text Git's integer reader accepts is whether the integer is not zero.
/// Why:
///   Git asks its integer reader for everything that is not a boolean word,
/// so the integer spellings
///  (octal,
/// hexadecimal,
/// signs,
/// unit suffixes) are the person's answer too,
/// and a value the reader refuses is
///  the only one left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// canonicalBool(value); // undefined means: ask Git
/// ```
pub fn canonical_bool(value: Option<&[u8]>) -> Option<bool> {
    let Some(bytes) = value else {
        return Some(true);
    };
    let text: &str = std::str::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return Some(false);
    }
    if let Some(word) = boolean_word(text) {
        return Some(word);
    }
    let number: i64 = git_integer(text)?;
    return Some(number != 0);
}

/// What:
///  `--type=bool-or-int` of one value,
/// as the text Git prints.
/// Why:
///   Git prints `true` or `false` for a boolean word,
/// valueless key or empty text,
/// and the decimal number for an
///  integer;
/// the plain decimal subset prints unchanged,
/// and anything else is not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// canonicalBoolOrInt(value); // undefined means: ask Git
/// ```
pub fn canonical_bool_or_int(value: Option<&[u8]>) -> Option<String> {
    let Some(bytes) = value else {
        return Some(String::from("true"));
    };
    let text: &str = std::str::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return Some(String::from("false"));
    }
    if let Some(word) = boolean_word(text) {
        return Some(String::from(if word { "true" } else { "false" }));
    }
    let number: u32 = plain_decimal(text)?;
    return Some(number.to_string());
}

/// What:
///  `--type=path` of one value,
/// when Git returns the text unchanged.
/// Why:
///   Git expands a leading `~`,
/// `~user/` and `%(prefix)/`,
/// treats `:(optional)` specially,
/// and refuses a valueless
///  key;
/// those,
/// an empty value and non-UTF-8 bytes are not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plainPath(value); // undefined means: ask Git
/// ```
pub fn plain_path(value: Option<&[u8]>) -> Option<String> {
    let text: &str = std::str::from_utf8(value?).ok()?;
    if text.is_empty() || text.starts_with('~') || text.starts_with("%(") || text.starts_with(":(") {
        return None;
    }
    return Some(String::from(text));
}

/// What:
///  Split one listing record into its key and value.
/// Why:
///   With `-z` a record is `key\nvalue`,
/// or just `key` when the key has no value;
/// the value itself may hold
///  newlines,
/// so only the first one separates the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = record.indexOf(0x0a); at < 0 ? [record, undefined] : [record.slice(0, at), record.slice(at + 1)];
/// ```
fn split_record(record: &[u8]) -> (&[u8], Option<&[u8]>) {
    return match record.iter().position(is_line_feed_byte) {
        Some(newline) => (&record[..newline], Some(&record[newline + 1..])),
        None => (record, None),
    };
}

/// What:
///  Whether one byte is the line feed that separates a key from its value in one record.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeedByte = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one byte is the NUL byte that ends each field of the listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

impl ConfigSnapshot {
    /// What:
    ///  The last entry whose key is exactly `key`,
    /// or `None` when no entry has it.
    ///  `Option<&ConfigEntry>` is "a borrowed entry,
    /// or no answer".
    /// Why:
    ///   Git uses the last value of a key,
    /// and four readers ask for it,
    /// so one loop from the end answers them all
    ///  where closures given to `rfind` and `any` stood.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.entries.findLast((entry) => entry.key === key);
    /// ```
    fn last_entry_named(&self, key: &str) -> Option<&ConfigEntry> {
        for entry in self.entries.iter().rev() {
            if entry.key == key {
                return Some(entry);
            }
        }
        return None;
    }

    /// What:
    ///  Parse the bytes of `git config --list -z --show-scope`.
    /// Why:
    ///   The listing is `scope NUL key [LF value] NUL` repeated;
    /// a listing in any other shape is refused whole,
    ///  so a half-understood configuration is never used.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parseConfigList(bytes, { withoutGlobalOptions });
    /// ```
    pub fn parse(listing: &[u8], without_global_options: bool) -> Option<Self> {
        let mut entries: Vec<ConfigEntry> = Vec::new();
        if listing.is_empty() {
            return Some(Self { entries, without_global_options });
        }
        let body: &[u8] = listing.strip_suffix(&[0])?;
        let mut fields = body.split(is_nul_byte);
        while let Some(scope) = fields.next() {
            let record: &[u8] = fields.next()?;
            let (key, value): (&[u8], Option<&[u8]>) = split_record(record);
            let key_text: &str = std::str::from_utf8(key).ok()?;
            if key_text.is_empty() {
                return None;
            }
            entries.push(ConfigEntry {
                scope: String::from(std::str::from_utf8(scope).ok()?),
                key: String::from(key_text),
                value: value.map(<[u8]>::to_vec),
            });
        }
        return Some(Self { entries, without_global_options });
    }

    /// What:
    ///  The components `core.fsync` makes Git flush in this repository;
    /// `None` when the value is one this does not model.
    /// Why:
    ///   Git's own writes follow `core.fsync` and the service's in-process writes follow the same setting.
    /// The last entry
    ///  decides alone and starts from the default (`FsyncComponents::default_set`);
    /// the reading is `FsyncComponents::parse`.
    ///  A key written without a value is refused by Git,
    /// so it is not modelled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const value = entries('core.fsync').at(-1); // undefined: the default
    /// ```
    pub fn fsync_components(&self) -> Option<FsyncComponents> {
        let Some(entry) = self.last_entry_named("core.fsync") else { return Some(FsyncComponents::default_set()) };
        let value: &str = std::str::from_utf8(entry.value.as_deref()?).ok()?;
        return Some(FsyncComponents::parse(value));
    }

    /// What:
    ///  Whether `core.fsync` names the index,
    /// so native Git flushes its index writes in this repository;
    /// `None` when the value is
    ///  one this does not model.
    /// Why:
    ///   The service writes the index in process,
    /// so it follows the setting Git's own index write follows.
    /// The components that
    ///  include the index are `index`,
    /// `added` and `all`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const hardens = fsyncComponents()?.has('index');
    /// ```
    pub fn hardens_index(&self) -> Option<bool> {
        return self.fsync_components().map(FsyncComponents::names_index);
    }

    /// What:
    ///  The `core.fsync` value to give the one `git fast-import` that imports a landing's commits,
    /// or `None` when it is to be
    ///  left as it is.
    /// Why:
    ///   Native Git writes the objects of a commit as loose objects,
    /// which its default `core.fsync` does not flush,
    /// while
    ///  `fast-import` flushes the temporary pack it writes (and deletes again when the batch is small).
    /// A repository that
    ///  names `loose-object` (or sets the deprecated `core.fsyncObjectFiles`) gets its set with the pack and its metadata
    ///  added when it lacks them,
    /// so a pack the import keeps is flushed as the loose objects would be;
    /// any other gets the set
    ///  it has without the packs;
    /// a value this does not model is left alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// set.has('loose-object') || legacyObjectFiles ? undefined : set.without('pack', 'pack-metadata').toValue()
    /// ```
    pub fn import_fsync_value(&self) -> Option<String> {
        let set: FsyncComponents = self.fsync_components()?;
        // The deprecated switch makes Git flush every loose object file; a value that is not a plain boolean is not modelled.
        let legacy = self.last_entry_named("core.fsyncobjectfiles");
        let legacy_flush: bool = match legacy {
            None => false,
            Some(entry) => canonical_bool(entry.value.as_deref())?,
        };
        if set.names_loose_object() || legacy_flush {
            // Native Git flushes the objects it writes, and a pack that the import keeps instead of unpacking holds the same
            //  objects, so the pack and its index are flushed too; the unpacked case flushes its loose objects as the repository says.
            return if set.names_pack_and_metadata() { None } else { Some(set.with_packs().value()) };
        }
        if !set.names_pack() {
            return None;
        }
        return Some(set.without_packs().value());
    }

    /// What:
    ///  The entries in listing order.
    /// Why:
    ///   Consumers that scan for a family of keys (hook definitions) read them directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.entries
    /// ```
    pub fn entries(&self) -> &[ConfigEntry] {
        return self.entries.as_slice();
    }

    /// What:
    ///  Whether the read carried no global options,
    /// so a consumer that runs Git without them may reuse it.
    /// Why:
    ///   A `-c`,
    /// `-C` or `--git-dir` in front of the read changes what Git lists,
    /// and a consumer that ignores the
    ///  caller's options must not see their effect.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.withoutGlobalOptions
    /// ```
    pub fn without_global_options(&self) -> bool {
        return self.without_global_options;
    }

    /// What:
    ///  The last entry of a key converted by `convert`,
    /// or why no answer is given.
    /// Why:
    ///   `git config --get` formats every entry of the key and fails when any is bad,
    /// so one unconvertible entry
    ///  makes the whole lookup unhandled even when it is not the last.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastConverted(entries.filter((entry) => entry.key === key), convert);
    /// ```
    fn last_converted<Value>(
        &self,
        key: &str,
        convert: impl Fn(Option<&[u8]>) -> Option<Value>,
    ) -> Lookup<Value> {
        let mut last: Lookup<Value> = Lookup::Unset;
        for entry in &self.entries {
            if entry.key != key {
                continue;
            }
            match convert(entry.value.as_deref()) {
                Some(converted) => last = Lookup::Found(converted),
                None => return Lookup::Unhandled,
            }
        }
        return last;
    }

    /// What:
    ///  `git config --type=path --get <key>` for values Git returns unchanged.
    /// Why:
    ///   `core.hooksPath` is read this way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.pathValue('core.hookspath');
    /// ```
    pub fn path_value(&self, key: &str) -> Lookup<String> {
        return self.last_converted(key, plain_path);
    }

    /// What:
    ///  `git config --type=bool --get <key>`.
    /// Why:
    ///   Every value Git would read is read here,
    /// whatever scope it came from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.boolValue('hook.pre-commit.enabled');
    /// ```
    pub fn bool_value(&self, key: &str) -> Lookup<bool> {
        return self.last_converted(key, canonical_bool);
    }

    /// What:
    ///  `git config --type=bool --get <key>` over the entries the configuration files hold,
    /// the `command` scope left out.
    /// Why:
    ///   The `command` scope is where `-c`,
    /// `GIT_CONFIG_COUNT` and
    /// `GIT_CONFIG_PARAMETERS` land,
    /// so it holds the switches of whoever
    ///  started this process:
    /// the wrapper itself starts every Git with every hook event switched off,
    /// and a Git child passes
    ///  those values on to the commands it runs.
    /// A question about what the person configured must not be
    ///  answered by them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.boolValueFromFiles('hook.pre-commit.enabled');
    /// ```
    pub fn bool_value_from_files(&self, key: &str) -> Lookup<bool> {
        let mut last: Lookup<bool> = Lookup::Unset;
        for entry in &self.entries {
            if entry.key != key || entry.scope == COMMAND_SCOPE {
                continue;
            }
            match canonical_bool(entry.value.as_deref()) {
                Some(converted) => last = Lookup::Found(converted),
                None => return Lookup::Unhandled,
            }
        }
        return last;
    }

    /// What:
    ///  The maintenance keys as `git config --type=bool-or-int --null --get-regexp` prints them:
    /// each set key
    ///  with the text of its last value.
    /// Why:
    ///   The maintenance decision parses that listing;
    /// `None` when any maintenance entry is not modelled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.maintenanceRecords();
    /// ```
    pub fn maintenance_records(&self) -> Option<BTreeMap<String, String>> {
        let mut records: BTreeMap<String, String> = BTreeMap::new();
        for key in MAINTENANCE_KEYS {
            match self.last_converted(key, canonical_bool_or_int) {
                Lookup::Unset => {}
                Lookup::Found(text) => {
                    records.insert(String::from(key), text);
                }
                Lookup::Unhandled => return None,
            }
        }
        return Some(records);
    }

    /// What:
    ///  Whether the ref storage is proven to be the files backend.
    /// Why:
    ///   `git rev-parse --show-ref-format` prints `reftable` only when `extensions.refStorage` is set,
    /// so a
    ///  listing without that key,
    /// read without global options,
    /// proves `files`;
    /// any listing that names the key
    ///  (in any scope,
    /// from any file) is left to Git,
    /// which also applies the repository-format rules.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.withoutGlobalOptions && !snapshot.entries.some((entry) => entry.key === 'extensions.refstorage');
    /// ```
    pub fn proves_files_ref_storage(&self) -> bool {
        return self.without_global_options && self.last_entry_named("extensions.refstorage").is_none();
    }
}

/// What:
///  Read the configuration once with the caller's context.
/// Why:
///   A start failure,
/// a non-zero exit (a malformed configuration file) or a listing this module cannot parse
///  gives `None`,
/// and every consumer then starts Git for its own key,
/// so the result is Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshot = await readConfigSnapshot({ context, cwd }); // undefined means: ask Git per key
/// ```
pub fn read_config_snapshot(context: &GitContext, cwd: &Path) -> Option<ConfigSnapshot> {
    let request: GitRequest = config_listing_request(cwd);
    return snapshot_of(run_git(context, &request), context);
}

/// What:
///  The same read,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The synchronous face enters the runtime for the pipes of a step that is ended at its ceiling,
/// and that entering is refused from inside the runtime,
/// so a consumer that has become a future asks for the read here.
///  The request,
/// the parse and the debug lines are the synchronous face's,
/// so the two cannot differ in what they read or record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshot = await readConfigSnapshotOnTheRuntime({ context, cwd });
/// ```
pub async fn read_config_snapshot_awaiting(context: &GitContext, cwd: &Path) -> Option<ConfigSnapshot> {
    let request: GitRequest = config_listing_request(cwd);
    return snapshot_of(run_git_awaiting(context, &request).await, context);
}

/// What:
///  The command line one configuration read puts to Git.
/// Why:
///   Both faces of the read ask exactly this question,
/// and a listing whose shape this module does not model is refused whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitRequest(cwd, ['config', '--list', '-z', '--show-scope']);
/// ```
fn config_listing_request(cwd: &Path) -> GitRequest {
    return GitRequest::new(cwd, &["config", "--list", "-z", "--show-scope"]);
}

/// What:
///  The snapshot of one finished read,
/// or nothing when the read failed or its listing has a shape this module does not model.
/// Why:
///   The outcome is the same whichever face ran the read,
/// so the deciding and the debug lines are written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshotOf(await read, context); // undefined means: ask Git per key
/// ```
fn snapshot_of(run: std::io::Result<GitOutput>, context: &GitContext) -> Option<ConfigSnapshot> {
    let output: GitOutput = match run {
        Ok(finished) => finished,
        Err(error) => {
            debug("readConfigSnapshot", format!("git config --list could not start: {error}").as_str());
            return None;
        }
    };
    if !output.succeeded() {
        debug(
            "readConfigSnapshot",
            format!("git config --list failed: {}", output.error_text()).as_str(),
        );
        return None;
    }
    let parsed: Option<ConfigSnapshot> =
        ConfigSnapshot::parse(output.stdout.as_slice(), context.global_prefix.is_empty());
    if parsed.is_none() {
        debug("readConfigSnapshot", "the configuration listing has an unexpected shape");
    }
    return parsed;
}

/// Snapshot controls:
/// parsing,
/// each lookup against real Git,
/// and the generated differential cases.
#[cfg(test)]
#[path = "config_snapshot_tests.rs"]
mod tests;
