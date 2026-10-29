//! What:
//!  One read of the whole Git configuration (`git config --list -z --show-scope`), parsed here,
//!  and the lookups the commit path makes against it instead of starting `git config` for each key.
//! Why:
//!  Every `git config` start costs a process, and a wrapped commit asked Git the same configuration
//!  several times (`core.hooksPath`, the hook switches, the maintenance keys, the ref storage).
//!  Git lists entries in precedence order (system, global, local, worktree, then `GIT_CONFIG_COUNT`,
//!  `GIT_CONFIG_PARAMETERS` and `-c` values, all in the `command` scope), so the last entry of a key is the
//!  value `git config --get` prints.
//!  A lookup answers only when it can reproduce Git's own answer exactly;
//!  any value it does not model (`~` and `%(prefix)` paths, unit suffixes, hexadecimal, a bad boolean) is
//!  [`Lookup::Unhandled`], and the caller starts Git for that key as before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const snapshot = parseConfigList(await git(['config', '--list', '-z', '--show-scope']));
//! const hooksPath = snapshot.pathValue('core.hookspath'); // 'unhandled' means: ask Git
//! ```

/// Debug logging of a snapshot that could not be used.
use super::diagnostic_log::debug;
/// One required Git start with the caller's context.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// Ordered map of the four maintenance keys to the text Git prints for them.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Largest decimal integer text a lookup models; longer digit strings fall back to Git.
const DECIMAL_DIGITS_LIMIT: usize = 9;

/// Keys the maintenance decision reads, spelled as `git config --list` prints them.
pub const MAINTENANCE_KEYS: [&str; 4] = ["maintenance.auto", "maintenance.autodetach", "gc.auto", "gc.autodetach"];

/// What:
///  One configuration entry in listing order.
/// Why:
///   The scope is kept so a consumer can tell `-c` values from file values, and `None` is a valueless key,
///  which Git reads as `true` for booleans and refuses for paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigEntry = { scope: string; key: string; value: Uint8Array | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    /// `system`, `global`, `local`, `worktree` or `command`.
    pub scope: String,
    /// Canonical key: lowercase section and variable, subsection kept as written.
    pub key: String,
    /// Value bytes, or `None` for a key written without `=`.
    pub value: Option<Vec<u8>>,
}

/// What:
///  The answer of one lookup.
/// Why:
///   "Not set" and "set to this" are Git's answers; "unhandled" is this module declining to guess,
///  so the caller falls back to starting Git, which is never a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Lookup<T> = { kind: 'unset' } | { kind: 'found'; value: T } | { kind: 'unhandled' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lookup<Value> {
    /// No entry has the key, so `git config --get` exits 1.
    Unset,
    /// The last entry of the key, converted exactly as Git converts it.
    Found(Value),
    /// Some entry of the key has a shape this module does not reproduce; ask Git.
    Unhandled,
}

/// What:
///  The parsed listing of one Git configuration read.
/// Why:
///   It is read once per commit with the caller's global options, and reused by the consumers that would
///  otherwise each start `git config`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigSnapshot = { entries: ConfigEntry[]; readWithoutGlobalOptions: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigSnapshot {
    /// Entries in the order Git listed them, which is precedence order.
    entries: Vec<ConfigEntry>,
    /// Whether the read carried no global options, so it equals a read made without them.
    without_global_options: bool,
}

/// What:
///  Whether the bytes of one value are a plain decimal integer the lookups model, and its numeric value.
/// Why:
///   Git parses integers with `strtoimax` base 0 and unit suffixes, so only `0` and digit strings that start with
///  `1` to `9` are decided here; leading zeros (octal), signs, `0x`, spaces and suffixes fall back to Git.
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
    if !text.bytes().all(|byte| { return byte.is_ascii_digit(); }) {
        return None;
    }
    return text.parse::<u32>().ok();
}

/// What:
///  Git's boolean word list, case-insensitively: `Some(true)` for true, yes and on, `Some(false)` for false, no
///  and off, `None` for any other text.
/// Why:
///   `git_parse_maybe_bool_text` compares with `strcasecmp`, which is ASCII case folding.
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
///  `--type=bool` of one value: a valueless key is true, empty text is false, a word from the list is itself,
///  and a plain decimal integer is whether it is not zero.
/// Why:
///   Anything else (a bad boolean that Git refuses, a unit suffix, hexadecimal, octal) is not modelled.
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
    return plain_decimal(text).map(|number| { return number != 0; });
}

/// What:
///  `--type=bool-or-int` of one value, as the text Git prints.
/// Why:
///   Git prints `true` or `false` for a boolean word, valueless key or empty text, and the decimal number for an
///  integer; the plain decimal subset prints unchanged, and anything else is not modelled.
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
    return plain_decimal(text).map(|number| { return number.to_string(); });
}

/// What:
///  `--type=path` of one value, when Git returns the text unchanged.
/// Why:
///   Git expands a leading `~`, `~user/` and `%(prefix)/`, treats `:(optional)` specially, and refuses a valueless
///  key; those, an empty value and non-UTF-8 bytes are not modelled.
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
///   With `-z` a record is `key\nvalue`, or just `key` when the key has no value; the value itself may hold
///  newlines, so only the first one separates the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = record.indexOf(0x0a); at < 0 ? [record, undefined] : [record.slice(0, at), record.slice(at + 1)];
/// ```
fn split_record(record: &[u8]) -> (&[u8], Option<&[u8]>) {
    return match record.iter().position(|byte| { return *byte == b'\n'; }) {
        Some(newline) => (&record[..newline], Some(&record[newline + 1..])),
        None => (record, None),
    };
}

impl ConfigSnapshot {
    /// What:
    ///  Parse the bytes of `git config --list -z --show-scope`.
    /// Why:
    ///   The listing is `scope NUL key [LF value] NUL` repeated; a listing in any other shape is refused whole,
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
        let mut fields = body.split(|byte| { return *byte == 0; });
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
    ///  Whether the read carried no global options, so a consumer that runs Git without them may reuse it.
    /// Why:
    ///   A `-c`, `-C` or `--git-dir` in front of the read changes what Git lists, and a consumer that ignores the
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
    ///  The last entry of a key converted by `convert`, or why no answer is given.
    /// Why:
    ///   `git config --get` formats every entry of the key and fails when any is bad, so one unconvertible entry
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
        for entry in self.entries.iter().filter(|candidate| { return candidate.key == key; }) {
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
    ///   The hook switches are read this way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.boolValue('hook.pre-commit.enabled');
    /// ```
    pub fn bool_value(&self, key: &str) -> Lookup<bool> {
        return self.last_converted(key, canonical_bool);
    }

    /// What:
    ///  The maintenance keys as `git config --type=bool-or-int --null --get-regexp` prints them: each set key
    ///  with the text of its last value.
    /// Why:
    ///   The maintenance decision parses that listing; `None` when any maintenance entry is not modelled.
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
    ///  Whether any entry defines a hook by name: a `hook.<name>.event` or `hook.<name>.command` key.
    /// Why:
    ///   Git runs such a hook for its event whatever the hooks directory holds, so a repository with one keeps
    ///  every dispatcher entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries.some((entry) => /^hook\..*\.(event|command)$/.test(entry.key));
    /// ```
    pub fn defines_named_hook(&self) -> bool {
        return self.entries.iter().any(|entry| {
            return entry.key.starts_with("hook.")
                && (entry.key.ends_with(".event") || entry.key.ends_with(".command"));
        });
    }

    /// What:
    ///  Whether the ref storage is proven to be the files backend.
    /// Why:
    ///   `git rev-parse --show-ref-format` prints `reftable` only when `extensions.refStorage` is set, so a
    ///  listing without that key, read without global options, proves `files`; any listing that names the key
    ///  (in any scope, from any file) is left to Git, which also applies the repository-format rules.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot.withoutGlobalOptions && !snapshot.entries.some((entry) => entry.key === 'extensions.refstorage');
    /// ```
    pub fn proves_files_ref_storage(&self) -> bool {
        return self.without_global_options
            && !self.entries.iter().any(|entry| { return entry.key == "extensions.refstorage"; });
    }
}

/// What:
///  Read the configuration once with the caller's context.
/// Why:
///   A start failure, a non-zero exit (a malformed configuration file) or a listing this module cannot parse
///  gives `None`, and every consumer then starts Git for its own key, so the result is Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshot = await readConfigSnapshot({ context, cwd }); // undefined means: ask Git per key
/// ```
pub fn read_config_snapshot(context: &GitContext, cwd: &Path) -> Option<ConfigSnapshot> {
    let request: GitRequest = GitRequest::new(cwd, &["config", "--list", "-z", "--show-scope"]);
    let output = match run_git(context, &request) {
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

/// Snapshot controls: parsing, each lookup against real Git, and the generated differential cases.
#[cfg(test)]
#[path = "config_snapshot_tests.rs"]
mod tests;
