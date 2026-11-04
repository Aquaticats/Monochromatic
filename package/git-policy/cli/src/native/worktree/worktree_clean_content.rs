//! What:
//!  The decision, by content and without a Git process, whether a tracked file that its index entry's stat data cannot
//!  vouch for (a racily clean entry, a stale stat) still equals the committed blob: hash the file as a blob and compare it
//!  with the entry's object name, refusing where a content conversion could make the two differ.
//! Why:
//!  Git settles those entries by hashing the file; stat data alone cannot tell a modified file from a copied or cloned one
//!  that kept the index's old inode and times. The hash of the raw bytes is conclusive only when Git's own conversion on the
//!  way into the index (`filter`, `ident`, `working-tree-encoding`, or a line-ending conversion that strips a carriage
//!  return) leaves the bytes alone, so a path such an attribute applies to, and a file with a carriage return, are refused
//!  with that reason instead of guessed. The file is read through a handle opened below the worktree root without following a
//!  link, its type and mode are judged on that handle, and its state is compared before and after the read, so the bytes that
//!  were hashed are the bytes of the one file that was judged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new ContentJudge({ root, gitDir, commonDir, environment, format });
//! judge.agrees({ directory, parents, name, entry, id, indexSeconds }); // throws the reason the file cannot be proven equal
//! ```

use crate::fs_beneath::{entry_metadata_at, open_directory_beneath, open_file_at, read_link_at};
use crate::git_system_config::system_attributes_for;
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_config_text::{ConfigEntry, last_entry, parse_bool};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
use crate::worktree_attributes::{AttributeLine, CONVERTING_ATTRIBUTES, attribute_name, may_apply, parse_attribute_lines};
use crate::worktree_clean::{StatEntry, StatVerdict, metadata_verdict};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// The largest file read into memory to be hashed; a larger one is refused with that reason.
pub const CONTENT_LIMIT: u64 = 32 << 20;

/// The largest attributes file read; Git ignores a larger one, which here is a refusal.
pub const ATTRIBUTES_LIMIT: u64 = 4 << 20;

/// Where Git reads the system attributes file from, as far as it can be established.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemAttributes {
    /// `GIT_ATTR_NOSYSTEM` turns the file off.
    Disabled,
    /// The file the executable reads, at its own prefix.
    File(PathBuf),
    /// The location, or the variable's value, could not be established without running Git.
    Unknown,
}

/// What:
///  The environment facts the content decision depends on: the configuration files Git reads and the system attributes file.
/// Why:
///   A control injects both, so a decision never depends on the machine it runs on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConversionEnvironment = { scope: ScopeEnvironment; systemAttributes: 'disabled' | { file: string } | 'unknown'; alternateAttributeSource?: string };
/// ```
#[derive(Clone, Debug)]
pub struct ConversionEnvironment {
    /// Which system and global configuration files Git reads.
    pub scope: ScopeEnvironment,
    /// Where the system attributes file is.
    pub system_attributes: SystemAttributes,
    /// Why Git would take its attributes from a tree instead of the worktree files read here (`GIT_ATTR_SOURCE`, `--attr-source`,
    ///  `attr.tree`), when it would; the content shortcut is then refused.
    pub alternate_attribute_source: Option<String>,
    /// The variables the facts were read from, which the configuration listing reads its `GIT_CONFIG_COUNT` entries from too, so
    ///  one environment decides every fact; empty for a stand-in that names none.
    pub variables: Vec<(OsString, OsString)>,
}

impl ConversionEnvironment {
    /// The facts of an environment given as variables. The variables name the real Git (the forward-target variable) when the
    ///  system files are to be learned from it; without it the system files are unknown and the content decision refuses.
    pub fn from_variables(variables: &[(OsString, OsString)]) -> ConversionEnvironment {
        let selected: Option<String> = variables.iter().any(|(name, _value): &(OsString, OsString)| { return name == "GIT_ATTR_SOURCE"; }).then(|| { return String::from("GIT_ATTR_SOURCE is set"); });
        return ConversionEnvironment { scope: ScopeEnvironment::from_variables(variables), system_attributes: system_attributes_from(variables), alternate_attribute_source: selected, variables: variables.to_vec() };
    }
}

/// What:
///  Where the system attributes file is for a Git started with `variables`.
/// Why:
///   `GIT_ATTR_NOSYSTEM` is read as Git reads a boolean environment variable (empty is false); the file is where the
///  executable says it is, never derived from the system configuration path, which `GIT_CONFIG_SYSTEM` moves alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemAttributesFrom(variables: [string, string][]): SystemAttributes;
/// ```
pub fn system_attributes_from(variables: &[(OsString, OsString)]) -> SystemAttributes {
    let switch: Option<&OsString> = variables.iter().rev().find(|(name, _value): &&(OsString, OsString)| { return name == "GIT_ATTR_NOSYSTEM"; }).map(|(_name, value): &(OsString, OsString)| { return value; });
    let disabled: Option<bool> = match switch {
        None => Some(false),
        Some(value) if value.is_empty() => Some(false),
        Some(value) => parse_bool(Some(value.as_bytes())),
    };
    return match disabled {
        Some(true) => SystemAttributes::Disabled,
        Some(false) => system_attributes_for(variables).map_or(SystemAttributes::Unknown, SystemAttributes::File),
        None => SystemAttributes::Unknown,
    };
}

/// One attributes file as it was read.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributeFile {
    /// How the reason names the file.
    label: String,
    /// The directory of the file relative to the worktree root, `/` separated, empty for the root.
    base: Vec<u8>,
    /// Its lines.
    lines: Vec<AttributeLine>,
}

/// The attributes files that apply whatever the path is, and the macros among them that set a converting attribute.
#[derive(Clone, Debug)]
struct FixedSources {
    /// The files, in no particular order: any of them may apply.
    files: Vec<AttributeFile>,
    /// Macro names that set a converting attribute, directly or through another macro.
    macros: HashSet<Vec<u8>>,
}

/// The object name of `data` stored as a blob.
fn blob_name(data: &[u8], format: ObjectFormat) -> Vec<u8> {
    let mut framed: Vec<u8> = format!("blob {}\0", data.len()).into_bytes();
    framed.extend_from_slice(data);
    return match format {
        ObjectFormat::Sha1 => sha1(framed.as_slice()).to_vec(),
        ObjectFormat::Sha256 => sha256(framed.as_slice()).to_vec(),
    };
}

/// What:
///  Everything `reader` yields, or nothing when it yields more than `limit` bytes; it is never asked for more than
///  `limit + 1`.
/// Why:
///   A file can grow after its size was looked at; the bound is on the read, not on an earlier look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAtMost(reader: Reader, limit: number): Buffer | undefined;
/// ```
pub fn read_at_most(reader: impl Read, limit: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut data: Vec<u8> = Vec::new();
    reader.take(limit + 1).read_to_end(&mut data)?;
    return Ok(if data.len() as u64 > limit { None } else { Some(data) });
}

/// Whether two metadata describe one unchanged file: same object, size, mode and times.
fn same_state(first: &std::fs::Metadata, second: &std::fs::Metadata) -> bool {
    return first.dev() == second.dev()
        && first.ino() == second.ino()
        && first.size() == second.size()
        && first.mode() == second.mode()
        && first.mtime() == second.mtime()
        && first.mtime_nsec() == second.mtime_nsec()
        && first.ctime() == second.ctime()
        && first.ctime_nsec() == second.ctime_nsec();
}

/// Whether a token or a macro name can change a file's bytes on the way into the index.
fn converts(token: &[u8], macros: &HashSet<Vec<u8>>) -> bool {
    let name: &[u8] = attribute_name(token);
    return CONVERTING_ATTRIBUTES.contains(&name) || macros.contains(name);
}

/// What:
///  The names of the macros that set a converting attribute, directly or through other macros.
/// Why:
///   Reverse edges (attribute name to the macros that mention it) and a queue visit every definition once, whatever the order of
///  the lines, so a long chain costs work in proportion to its length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertingMacros(files: AttributeFile[]): Set<string>;
/// ```
fn converting_macros(files: &[AttributeFile]) -> HashSet<Vec<u8>> {
    let mut mentioned_by: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();
    for line in files.iter().flat_map(|file: &AttributeFile| { return file.lines.iter(); }).filter(|line: &&AttributeLine| { return line.is_macro; }) {
        for token in &line.tokens {
            mentioned_by.entry(attribute_name(token.as_slice()).to_vec()).or_default().push(line.subject.clone());
        }
    }
    let mut known: HashSet<Vec<u8>> = HashSet::new();
    let mut queue: VecDeque<Vec<u8>> = CONVERTING_ATTRIBUTES.iter().map(|name: &&[u8]| { return name.to_vec(); }).collect();
    while let Some(name) = queue.pop_front() {
        for macro_name in mentioned_by.get(&name).into_iter().flatten() {
            if known.insert(macro_name.clone()) {
                queue.push_back(macro_name.clone());
            }
        }
    }
    return known;
}

/// What:
///  Decides, one entry at a time, whether a file equals its committed blob by content.
/// Why:
///   The attributes files are read once, on the first entry that needs them, and the per-directory ones are cached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentJudge { agrees(args: Place & { entry: StatEntry; id: Uint8Array; indexSeconds: number }): void /* throws the reason */ }
/// ```
pub struct ContentJudge<'a> {
    /// The worktree root, held open.
    root: &'a std::fs::File,
    /// The worktree's Git directory.
    git_dir: &'a Path,
    /// The shared Git directory.
    common_dir: &'a Path,
    /// Which files Git reads in this environment.
    environment: &'a ConversionEnvironment,
    /// The object format of the repository.
    format: ObjectFormat,
    /// The attributes files that do not depend on the path.
    fixed: Option<Result<FixedSources, String>>,
    /// The `.gitattributes` of each directory, keyed by its path, read at most once; `None` when it has none.
    directories: HashMap<Vec<u8>, Option<AttributeFile>>,
}

/// The moments of a judgment at which a control changes the file under it.
pub struct ReadSeams<'a> {
    /// Called after the file's state was first read and before it is opened.
    pub before_open: &'a mut dyn FnMut(),
    /// Called after the open handle was judged and before the bytes are read.
    pub before_read: &'a mut dyn FnMut(),
    /// Called after the bytes were read and before the state is read again.
    pub after_read: &'a mut dyn FnMut(),
}

/// Where a tracked file is: the held directory that contains it, the names above it and its own name.
pub struct Place<'a> {
    /// The directory holding the file, opened below the root without following a link.
    pub directory: &'a std::fs::File,
    /// The names of the directories between the root and the file.
    pub parents: &'a [OsString],
    /// The file's own name.
    pub name: &'a OsStr,
}

impl<'a> ContentJudge<'a> {
    /// A judge for the worktree held open as `root`.
    pub fn new(root: &'a std::fs::File, git_dir: &'a Path, common_dir: &'a Path, environment: &'a ConversionEnvironment, format: ObjectFormat) -> ContentJudge<'a> {
        return ContentJudge { root, git_dir, common_dir, environment, format, fixed: None, directories: HashMap::new() };
    }

    /// Read an attributes file at an absolute path the way Git does, following a link; absent is `None`.
    fn read_outside(path: &Path, label: String) -> Result<Option<AttributeFile>, String> {
        // Without blocking, so a named pipe is found out by its type instead of waiting for a writer.
        let file = match std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC).open(path) {
            Ok(opened) => opened,
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => return Ok(None),
            Err(error) => return Err(format!("the attributes file {} cannot be read ({error})", path.display())),
        };
        return Self::parse_open(file, label, Vec::new());
    }

    /// Read the `.gitattributes` of the held directory `directory`; absent is `None`, a link or anything but a file is a refusal.
    fn read_in(directory: &std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let file = match open_file_at(directory, OsStr::new(".gitattributes")) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{label} cannot be read without following a link ({error}), so the attributes it sets cannot be ruled out")),
        };
        return Self::parse_open(file, label, base);
    }

    /// Parse an attributes file that is open, refusing one that is not a plain file or is larger than the limit.
    fn parse_open(file: std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let metadata: std::fs::Metadata = file.metadata().map_err(|error: std::io::Error| { return format!("{label} cannot be examined ({error})"); })?;
        if !metadata.file_type().is_file() {
            return Err(format!("{label} is not a plain file (a named pipe, a device or a directory), so the attributes it sets cannot be ruled out"));
        }
        let bytes: Vec<u8> = read_at_most(&file, ATTRIBUTES_LIMIT)
            .map_err(|error: std::io::Error| { return format!("{label} cannot be read ({error})"); })?
            .ok_or_else(|| { return format!("{label} is larger than {ATTRIBUTES_LIMIT} bytes, so the attributes it sets cannot be ruled out"); })?;
        return Ok(Some(AttributeFile { label, base, lines: parse_attribute_lines(bytes.as_slice()) }));
    }

    /// The configuration Git would list, read in process.
    fn config_entries(&self) -> Result<Vec<ConfigEntry>, String> {
        let listed = list_config(self.common_dir, self.git_dir, &self.environment.scope, self.environment.variables.as_slice())
            .ok_or_else(|| { return String::from("the Git configuration cannot be read without running Git, so a global attributes file cannot be ruled out"); })?;
        return Ok(listed.into_iter().map(|item| { return item.entry; }).collect());
    }

    /// The attributes file `core.attributesFile` names, or the default one, from the configuration entries.
    fn global_attributes_path(&self, entries: &[ConfigEntry]) -> Result<Option<PathBuf>, String> {
        if let Some(entry) = last_entry(entries, "core.attributesfile") {
            let value: &[u8] = entry.value.as_deref().ok_or_else(|| { return String::from("core.attributesFile has no value"); })?;
            if let Some(rest) = value.strip_prefix(b"~/") {
                let home: &Path = self.environment.scope.home.as_deref().ok_or_else(|| { return String::from("core.attributesFile starts with ~/ but HOME is not set"); })?;
                return Ok(Some(home.join(OsStr::from_bytes(rest))));
            }
            let path: PathBuf = PathBuf::from(OsStr::from_bytes(value));
            if !path.is_absolute() {
                return Err(format!("core.attributesFile is {:?}, which is not an absolute path", String::from_utf8_lossy(value)));
            }
            return Ok(Some(path));
        }
        return Ok(match (&self.environment.scope.xdg, &self.environment.scope.home) {
            (Some(directory), _) => Some(directory.join("git/attributes")),
            (None, Some(home)) => Some(home.join(".config/git/attributes")),
            (None, None) => None,
        });
    }

    /// The attributes files that apply whatever the path is, with the macros among them that set a converting attribute.
    fn load_fixed(&self) -> Result<FixedSources, String> {
        let unmodelled = |reason: &str| -> String { return format!("{reason}, so Git takes its attributes from a tree and not from the files read here, and what it sets cannot be ruled out"); };
        if let Some(reason) = self.environment.alternate_attribute_source.as_deref() {
            return Err(unmodelled(reason));
        }
        let entries: Vec<ConfigEntry> = self.config_entries()?;
        if last_entry(entries.as_slice(), "attr.tree").is_some() {
            return Err(unmodelled("attr.tree is set"));
        }
        let mut files: Vec<AttributeFile> = Vec::new();
        if let Some(root_file) = Self::read_in(self.root, String::from(".gitattributes"), Vec::new())? {
            files.push(root_file);
        }
        let mut outside: Vec<(PathBuf, String)> = vec![(self.common_dir.join("info/attributes"), String::from("info/attributes"))];
        match &self.environment.system_attributes {
            SystemAttributes::Disabled => {}
            SystemAttributes::File(path) => outside.push((path.clone(), path.display().to_string())),
            SystemAttributes::Unknown => return Err(String::from("the location of the system attributes file cannot be established without running Git, so what it sets cannot be ruled out")),
        }
        if let Some(path) = self.global_attributes_path(entries.as_slice())? {
            outside.push((path.clone(), path.display().to_string()));
        }
        for (path, label) in outside {
            if let Some(file) = Self::read_outside(path.as_path(), label)? {
                files.push(file);
            }
        }
        let macros: HashSet<Vec<u8>> = converting_macros(files.as_slice());
        return Ok(FixedSources { files, macros });
    }

    /// The `.gitattributes` of the directory below the root named by `parents`, read once.
    fn directory_file(&mut self, parents: &[OsString]) -> Result<Option<AttributeFile>, String> {
        let key: Vec<u8> = parents.iter().map(|name: &OsString| { return name.as_bytes().to_vec(); }).collect::<Vec<Vec<u8>>>().join(&b'/');
        if !self.directories.contains_key(&key) {
            let directory = open_directory_beneath(self.root, parents).map_err(|error: std::io::Error| { return format!("{} cannot be opened below the worktree ({error})", String::from_utf8_lossy(key.as_slice())); })?;
            let label: String = format!("{}/.gitattributes", String::from_utf8_lossy(key.as_slice()));
            let file: Option<AttributeFile> = Self::read_in(&directory, label, key.clone())?;
            self.directories.insert(key.clone(), file);
        }
        return Ok(self.directories.get(&key).cloned().flatten());
    }

    /// The first converting attribute that may apply to the file at `path` and where it is set, or nothing.
    fn converting_attribute(&mut self, path: &[u8], parents: &[OsString]) -> Result<Option<(String, String)>, String> {
        if self.fixed.is_none() {
            self.fixed = Some(self.load_fixed());
        }
        let fixed: FixedSources = self.fixed.clone().ok_or_else(|| { return String::from("the attributes were not loaded"); })??;
        let mut files: Vec<AttributeFile> = fixed.files;
        for depth in 1..=parents.len() {
            if let Some(file) = self.directory_file(&parents[..depth])? {
                files.push(file);
            }
        }
        for file in &files {
            let below: bool = file.base.is_empty() || path.strip_prefix(file.base.as_slice()).is_some_and(|rest: &[u8]| { return rest.starts_with(b"/"); });
            if below && file.lines.iter().any(|line: &AttributeLine| { return line.quoted; }) {
                return Ok(Some((String::from("a quoted line"), file.label.clone())));
            }
            for line in file.lines.iter().filter(|line: &&AttributeLine| { return !line.is_macro; }) {
                let Some(token) = line.tokens.iter().find(|token: &&Vec<u8>| { return converts(token.as_slice(), &fixed.macros); }) else { continue };
                if may_apply(line.subject.as_slice(), file.base.as_slice(), path) {
                    return Ok(Some((String::from_utf8_lossy(token.as_slice()).into_owned(), file.label.clone())));
                }
            }
        }
        return Ok(None);
    }

    /// What:
    ///  The bytes of the file at `place`, judged to be the one file its entry describes: opened below the held directory
    ///  without following a link, its type and mode judged on the open handle, and its state the same before and after.
    /// Why:
    ///   A swap, a `chmod` or a write while the file is read must refuse instead of passing, and the read is bounded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readStable(place: Place, entry: StatEntry, indexSeconds: number): Buffer; // throws the reason
    /// ```
    fn read_stable(place: &Place<'_>, entry: &StatEntry, index_seconds: u32, shown: &str, seams: &mut ReadSeams<'_>) -> Result<Vec<u8>, String> {
        let changed = || -> String { return format!("{shown} changed while it was examined, so it cannot be proven equal to the committed content"); };
        let named: std::fs::Metadata = entry_metadata_at(place.directory, place.name).map_err(|error: std::io::Error| { return format!("{shown} cannot be examined ({error})"); })?;
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &named, index_seconds) {
            return Err(reason);
        }
        if named.file_type().is_symlink() {
            let target: PathBuf = read_link_at(place.directory, place.name).map_err(|error: std::io::Error| { return format!("{shown} cannot be read ({error})"); })?;
            let again: std::fs::Metadata = entry_metadata_at(place.directory, place.name).map_err(|_error: std::io::Error| { return changed(); })?;
            if !same_state(&named, &again) {
                return Err(changed());
            }
            return Ok(target.into_os_string().into_vec());
        }
        (seams.before_open)();
        let file: std::fs::File = open_file_at(place.directory, place.name).map_err(|error: std::io::Error| { return format!("{shown} cannot be opened without following a link ({error})"); })?;
        let held: std::fs::Metadata = file.metadata().map_err(|error: std::io::Error| { return format!("{shown} cannot be examined ({error})"); })?;
        if held.dev() != named.dev() || held.ino() != named.ino() {
            return Err(changed());
        }
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &held, index_seconds) {
            return Err(reason);
        }
        let too_large = || -> String { return format!("{shown} is larger than {CONTENT_LIMIT} bytes, which is more than is read to compare it by content"); };
        if held.len() > CONTENT_LIMIT {
            return Err(too_large());
        }
        (seams.before_read)();
        let data: Vec<u8> = read_at_most(&file, CONTENT_LIMIT).map_err(|error: std::io::Error| { return format!("{shown} cannot be read ({error})"); })?.ok_or_else(too_large)?;
        (seams.after_read)();
        let after: std::fs::Metadata = file.metadata().map_err(|_error: std::io::Error| { return changed(); })?;
        let by_name: std::fs::Metadata = entry_metadata_at(place.directory, place.name).map_err(|_error: std::io::Error| { return changed(); })?;
        if !same_state(&held, &after) || !same_state(&held, &by_name) {
            return Err(changed());
        }
        return Ok(data);
    }

    /// What:
    ///  Whether the file at `place` equals the blob named `id`; the reason it cannot be proven so when it does not.
    /// Why:
    ///   Where a conversion applies, or the file has a carriage return, the bytes alone prove nothing and the answer is a
    ///  refusal that says so; a file that really differs says it differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.agrees(place, entry, id, indexSeconds); // throws the reason
    /// ```
    pub fn agrees(&mut self, place: &Place<'_>, entry: &StatEntry, id: &[u8], index_seconds: u32) -> Result<(), String> {
        return self.agrees_seamed(place, entry, id, index_seconds, &mut ReadSeams { before_open: &mut || {}, before_read: &mut || {}, after_read: &mut || {} });
    }

    /// `agrees` with the moments at which a control may change the file.
    pub fn agrees_seamed(&mut self, place: &Place<'_>, entry: &StatEntry, id: &[u8], index_seconds: u32, seams: &mut ReadSeams<'_>) -> Result<(), String> {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        if let Some((token, label)) = self.converting_attribute(entry.path.as_slice(), place.parents)? {
            return Err(if token == "a quoted line" {
                format!("{shown} cannot be compared by content: {label} has a quoted line, whose pattern and attributes are not modelled, so what it sets cannot be ruled out")
            } else {
                format!("{shown} cannot be compared by content: the attribute {token} from {label} may change its bytes on the way into the index")
            });
        }
        let data: Vec<u8> = Self::read_stable(place, entry, index_seconds, shown.as_str(), seams)?;
        if blob_name(data.as_slice(), self.format) != id {
            return Err(format!("{shown} differs from the committed content (its recorded size and times could not vouch for it, so its content was compared)"));
        }
        if data.contains(&b'\r') {
            return Err(format!("{shown} cannot be compared by content: it contains a carriage return, so a line-ending conversion may change its bytes on the way into the index"));
        }
        return Ok(());
    }
}

#[cfg(test)]
#[path = "worktree_clean_content_tests.rs"]
mod tests;

/// Controls for the findings of the review of the content decision, on real Git repositories.
#[cfg(test)]
#[path = "worktree_clean_content_safety_tests.rs"]
mod safety_tests;
