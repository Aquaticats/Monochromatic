//! What:
//!  What the in-process Git queries share: the request (`Query`), the answer (`Answer`), the parsed command line (`Parsed`),
//!  the exact-flags check, and the context a query runs in (layout, index path, object directory, environment).
//! Why:
//!  `repo_read_query.rs` dispatches the commands and `repo_read_query_quiet.rs` and `repo_read_query_refs.rs` answer some of
//!  them; all three need these, and none of the three may depend on a file that depends on it, so they sit in a file that
//!  depends on none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Query = { arguments: string[]; directory?: string; variables: [string, string][]; input?: Uint8Array };
//! type Answer = { stdout: Uint8Array; stderr: Uint8Array; code: number };
//! ```

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_index::{INTENT_TO_ADD, IndexEntry, ParsedIndex, parse_index_with_tree};
use crate::repo_read_layout::RepoLayout;
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::index_cache_tree::CacheTree;
use crate::repo_read_tree::{FlatEntry, parse_commit};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The command line and surroundings of one query.
pub struct Query<'a> {
    /// The arguments given to Git, global options first.
    pub arguments: &'a [OsString],
    /// The directory Git would run in; `None` is the process directory.
    pub directory: Option<&'a Path>,
    /// The variables added to Git's environment for this run.
    pub variables: &'a [(OsString, OsString)],
    /// The bytes sent to Git's standard input, for the commands that read object names from it.
    pub input: Option<&'a [u8]>,
}

/// What Git would have printed and returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Answer {
    /// Standard output.
    pub stdout: Vec<u8>,
    /// Standard error.
    pub stderr: Vec<u8>,
    /// The exit code.
    pub code: i32,
}

/// The parts of a command line after its global options.
pub(crate) struct Parsed<'a> {
    /// The subcommand word.
    pub(crate) command: &'a str,
    /// Options before `--`.
    pub(crate) flags: Vec<&'a str>,
    /// Non-option arguments before `--`.
    pub(crate) operands: Vec<&'a [u8]>,
    /// Arguments after `--`.
    pub(crate) specs: Vec<&'a [u8]>,
}

/// Split the arguments: leading `-C <dir>` pairs move the start directory, `--no-literal-pathspecs` is Git's default.
pub(crate) fn parse<'a>(arguments: &'a [OsString], start: &mut PathBuf) -> Option<Parsed<'a>> {
    let mut index: usize = 0;
    while index < arguments.len() {
        let word: &OsStr = arguments[index].as_os_str();
        if word == "-C" {
            let target: &OsString = arguments.get(index + 1)?;
            if !target.is_empty() {
                *start = start.join(target);
            }
            // Git changes directory at each `-C` in turn and fails at the first one that is not a directory.
            if !start.is_dir() {
                return None;
            }
            index += 2;
        } else if word == "--no-literal-pathspecs" {
            index += 1;
        } else {
            break;
        }
    }
    let command: &str = arguments.get(index)?.to_str()?;
    let mut parsed: Parsed<'a> = Parsed { command, flags: Vec::new(), operands: Vec::new(), specs: Vec::new() };
    let mut after_separator: bool = false;
    for argument in &arguments[index + 1..] {
        let bytes: &[u8] = argument.as_encoded_bytes();
        if after_separator {
            parsed.specs.push(bytes);
        } else if bytes == b"--" {
            after_separator = true;
        } else if bytes.starts_with(b"-") {
            parsed.flags.push(argument.to_str()?);
        } else {
            parsed.operands.push(bytes);
        }
    }
    return Some(parsed);
}

/// Whether `flags` holds exactly the options of `allowed` (each at most once) and every one of `required`.
pub(crate) fn flags_are(flags: &[&str], required: &[&str], allowed: &[&str]) -> bool {
    for flag in required {
        if !flags.contains(flag) {
            return false;
        }
    }
    for (position, flag) in flags.iter().enumerate() {
        if !(allowed.contains(flag) || required.contains(flag)) {
            return false;
        }
        if flags[..position].contains(flag) {
            return false;
        }
    }
    return true;
}

/// Everything one query needs, opened lazily.
pub(crate) struct Context {
    /// The repository.
    pub(crate) layout: RepoLayout,
    /// The index file Git would read.
    pub(crate) index_path: PathBuf,
    /// The object directory override, if any.
    pub(crate) objects: Option<PathBuf>,
    /// Where the system and global configuration files are.
    pub(crate) scope: ScopeEnvironment,
    /// The environment Git would see, without the index and object directory variables.
    pub(crate) variables: Vec<(OsString, OsString)>,
}

impl Context {
    /// The object database.
    pub(crate) fn database(&self) -> Option<ObjectDatabase> {
        return ObjectDatabase::open(self.layout.common_dir.as_path(), self.objects.as_deref(), self.layout.object_format);
    }

    /// The index entries.
    pub(crate) fn index(&self) -> Option<Vec<IndexEntry>> {
        let parsed: ParsedIndex = self.index_with_tree()?;
        return Some(parsed.entries);
    }

    /// The index entries and the cache-tree extension.
    pub(crate) fn index_with_tree(&self) -> Option<ParsedIndex> {
        let bytes: Vec<u8> = crate::private_storage::read_regular_file(self.index_path.as_path(), 1 << 30).ok()?;
        return parse_index_with_tree(bytes.as_slice(), self.layout.object_format);
    }

    /// The index as flat entries and its cache-tree, refusing unmerged or intent-to-add entries.
    pub(crate) fn resolved_index_with_tree(&self) -> Option<(Vec<FlatEntry>, Option<CacheTree>)> {
        let parsed: ParsedIndex = self.index_with_tree()?;
        for entry in &parsed.entries {
            if entry.stage != 0 || entry.extended & INTENT_TO_ADD != 0 {
                return None;
            }
        }
        let mut flat: Vec<FlatEntry> = Vec::new();
        for entry in parsed.entries {
            flat.push(FlatEntry { path: entry.path, mode: entry.mode, id: entry.id });
        }
        return Some((flat, parsed.tree));
    }

    /// The root tree of a commit or tree named by `hex`.
    pub(crate) fn tree_of(&self, database: &ObjectDatabase, hex: &[u8]) -> Option<Vec<u8>> {
        let name: Vec<u8> = from_hex(hex, self.layout.object_format)?;
        let object = database.read(name.as_slice())?;
        return match object.kind {
            ObjectKind::Tree => Some(name),
            ObjectKind::Commit => Some(parse_commit(object.bytes.as_slice(), self.layout.object_format)?.tree),
            ObjectKind::Blob | ObjectKind::Tag => None,
        };
    }

    /// The zero object name spelled out for raw lines.
    pub(crate) fn zero(&self) -> String {
        return "0".repeat(self.layout.object_format.hex_length());
    }
}
