//! What:
//!  The decision,
//! by content and without a Git process,
//! whether a tracked file that its index entry's stat data cannot
//!  vouch for (a racily clean entry,
//! a stale stat) still equals the committed blob:
//! hash the file as a blob and compare it
//!  with the entry's object name,
//! refusing where a content conversion could make the two differ.
//! Why:
//!  Git settles those entries by hashing the file;
//! stat data alone cannot tell a modified file from a copied or cloned one
//!  that kept the index's old inode and times.
//! The hash of the raw bytes is conclusive only when Git's own conversion on the
//!  way into the index (`filter`,
//! `ident`,
//! `working-tree-encoding`,
//! or a line-ending conversion that strips a carriage
//!  return) leaves the bytes alone,
//! so a path such an attribute applies to,
//! and a file with a carriage return,
//! are refused
//!  with that reason instead of guessed.
//! The file is read through a handle opened below the worktree root without following a
//!  link,
//! its type and mode are judged on that handle,
//! and its state is compared before and after the read,
//! so the bytes that
//!  were hashed are the bytes of the one file that was judged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new ContentJudge({ root, gitDir, commonDir, environment, format });
//! judge.agrees({ directory, parents, name, entry, id, indexSeconds }); // throws the reason the file cannot be proven equal
//! ```

use crate::fs_beneath::{entry_metadata_at, open_directory_beneath, open_file_at, read_link_at};
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_text::{ConfigEntry, last_entry};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
use crate::worktree_attributes::{may_apply, parse_attribute_lines};
pub use crate::worktree_clean_content_attribute_sources::{ConversionEnvironment, SystemAttributes, system_attributes_from};
use crate::worktree_clean_content_attribute_sources::{AttributeFile, FixedSources, converting_macros, first_converting_token, has_quoted_line};
use crate::worktree_clean_stat_verdicts::{StatEntry, StatVerdict, metadata_verdict};
use crate::worktree_path_failure_message::subject_failure;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// The largest file read into memory to be hashed;
/// a larger one is refused with that reason.
pub const CONTENT_LIMIT: u64 = 32 << 20;

/// The largest attributes file read;
/// Git ignores a larger one,
/// which here is a refusal.
pub const ATTRIBUTES_LIMIT: u64 = 4 << 20;

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
///  Everything `reader` yields,
/// or nothing when it yields more than `limit` bytes;
/// it is never asked for more than
///  `limit + 1`.
/// Why:
///   A file can grow after its size was looked at;
/// the bound is on the read,
/// not on an earlier look.
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

/// Whether two metadata describe one unchanged file:
/// same object,
/// size,
/// mode and times.
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

/// What:
///  The refusal for a file that changed while it was examined.
/// Why:
///   A file that moves under the check cannot be proven equal to the committed content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${shown} changed while it was examined, so it cannot be proven equal to the committed content`
/// ```
fn changed_while_examined(shown: &str) -> String {
    return format!("{shown} changed while it was examined, so it cannot be proven equal to the committed content");
}

/// What:
///  The refusal for a file larger than the content check reads.
/// Why:
///   A bound on the bytes read keeps one huge file from holding the add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${shown} is larger than ${CONTENT_LIMIT} bytes, ...`
/// ```
fn too_large_to_compare(shown: &str) -> String {
    return format!("{shown} is larger than {CONTENT_LIMIT} bytes, which is more than is read to compare it by content");
}

/// What:
///  The refusal for attributes Git takes from a tree and not from the files read here,
/// with the reason it does.
/// Why:
///   What such a tree sets cannot be ruled out by reading files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${reason}, so Git takes its attributes from a tree ...`
/// ```
fn unmodelled(reason: &str) -> String {
    return format!("{reason}, so Git takes its attributes from a tree and not from the files read here, and what it sets cannot be ruled out");
}

/// What:
///  Do nothing;
/// the seams of the content read run this unless a control puts work there.
/// Why:
///   The real read has nothing to do between its steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  Decides,
/// one entry at a time,
/// whether a file equals its committed blob by content.
/// Why:
///   The attributes files are read once,
/// on the first entry that needs them,
/// and the per-directory ones are cached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentJudge { agrees(args: Place & { entry: StatEntry; id: Uint8Array; indexSeconds: number }): void /* throws the reason */ }
/// ```
pub struct ContentJudge<'a> {
    /// The worktree root,
    /// held open.
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
    /// The `.gitattributes` of each directory,
    /// keyed by its path,
    /// read at most once;
    /// `None` when it has none.
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

/// Where a tracked file is:
/// the held directory that contains it,
/// the names above it and its own name.
pub struct Place<'a> {
    /// The directory holding the file,
    /// opened below the root without following a link.
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

    /// Read an attributes file at an absolute path the way Git does,
    /// following a link;
    /// absent is `None`.
    fn read_outside(path: &Path, label: String) -> Result<Option<AttributeFile>, String> {
        // Without blocking, so a named pipe is found out by its type instead of waiting for a writer.
        let file = match std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC).open(path) {
            Ok(opened) => opened,
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => return Ok(None),
            Err(error) => return Err(format!("the attributes file {} cannot be read ({error})", path.display())),
        };
        return Self::parse_open(file, label, Vec::new());
    }

    /// Read the `.gitattributes` of the held directory `directory`;
    /// absent is `None`,
    /// a link or anything but a file is a refusal.
    fn read_in(directory: &std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let file = match open_file_at(directory, OsStr::new(".gitattributes")) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{label} cannot be read without following a link ({error}), so the attributes it sets cannot be ruled out")),
        };
        return Self::parse_open(file, label, base);
    }

    /// Parse an attributes file that is open,
    /// refusing one that is not a plain file or is larger than the limit.
    fn parse_open(file: std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let metadata: std::fs::Metadata = subject_failure(file.metadata(), &label, "cannot be examined")?;
        if !metadata.file_type().is_file() {
            return Err(format!("{label} is not a plain file (a named pipe, a device or a directory), so the attributes it sets cannot be ruled out"));
        }
        let Some(bytes) = subject_failure(read_at_most(&file, ATTRIBUTES_LIMIT), &label, "cannot be read")? else {
            return Err(format!("{label} is larger than {ATTRIBUTES_LIMIT} bytes, so the attributes it sets cannot be ruled out"));
        };
        return Ok(Some(AttributeFile { label, base, lines: parse_attribute_lines(bytes.as_slice()) }));
    }

    /// The configuration Git would list,
    /// read in process.
    fn config_entries(&self) -> Result<Vec<ConfigEntry>, String> {
        let Some(listed) = list_config(self.common_dir, self.git_dir, &self.environment.scope, self.environment.variables.as_slice()) else {
            return Err(String::from("the Git configuration cannot be read without running Git, so a global attributes file cannot be ruled out"));
        };
        let mut entries: Vec<ConfigEntry> = Vec::new();
        for item in listed {
            entries.push(item.entry);
        }
        return Ok(entries);
    }

    /// The attributes file `core.attributesFile` names,
    /// or the default one,
    /// from the configuration entries.
    fn global_attributes_path(&self, entries: &[ConfigEntry]) -> Result<Option<PathBuf>, String> {
        if let Some(entry) = last_entry(entries, "core.attributesfile") {
            let Some(value) = entry.value.as_deref() else {
                return Err(String::from("core.attributesFile has no value"));
            };
            if let Some(rest) = value.strip_prefix(b"~/") {
                let Some(home) = self.environment.scope.home.as_deref() else {
                    return Err(String::from("core.attributesFile starts with ~/ but HOME is not set"));
                };
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

    /// The attributes files that apply whatever the path is,
    /// with the macros among them that set a converting attribute.
    fn load_fixed(&self) -> Result<FixedSources, String> {
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

    /// The `.gitattributes` of the directory below the root named by `parents`,
    /// read once.
    fn directory_file(&mut self, parents: &[OsString]) -> Result<Option<AttributeFile>, String> {
        let mut names: Vec<Vec<u8>> = Vec::new();
        for name in parents {
            names.push(name.as_bytes().to_vec());
        }
        let key: Vec<u8> = names.join(&b'/');
        if !self.directories.contains_key(&key) {
            let directory = match open_directory_beneath(self.root, parents) {
                Ok(opened) => opened,
                Err(error) => return Err(format!("{} cannot be opened below the worktree ({error})", String::from_utf8_lossy(key.as_slice()))),
            };
            let label: String = format!("{}/.gitattributes", String::from_utf8_lossy(key.as_slice()));
            let file: Option<AttributeFile> = Self::read_in(&directory, label, key.clone())?;
            self.directories.insert(key.clone(), file);
        }
        return Ok(self.directories.get(&key).cloned().flatten());
    }

    /// The first converting attribute that may apply to the file at `path` and where it is set,
    /// or nothing.
    fn converting_attribute(&mut self, path: &[u8], parents: &[OsString]) -> Result<Option<(String, String)>, String> {
        if self.fixed.is_none() {
            self.fixed = Some(self.load_fixed());
        }
        let loaded: Result<FixedSources, String> = match self.fixed.clone() {
            Some(found) => found,
            None => return Err(String::from("the attributes were not loaded")),
        };
        let fixed: FixedSources = loaded?;
        let mut files: Vec<AttributeFile> = fixed.files;
        for depth in 1..=parents.len() {
            if let Some(file) = self.directory_file(&parents[..depth])? {
                files.push(file);
            }
        }
        for file in &files {
            let below_base: bool = match path.strip_prefix(file.base.as_slice()) {
                Some(rest) => rest.starts_with(b"/"),
                None => false,
            };
            let below: bool = file.base.is_empty() || below_base;
            if below && has_quoted_line(file.lines.as_slice()) {
                return Ok(Some((String::from("a quoted line"), file.label.clone())));
            }
            for line in &file.lines {
                if line.is_macro {
                    continue;
                }
                let Some(token) = first_converting_token(line.tokens.as_slice(), &fixed.macros) else { continue };
                if may_apply(line.subject.as_slice(), file.base.as_slice(), path) {
                    return Ok(Some((String::from_utf8_lossy(token.as_slice()).into_owned(), file.label.clone())));
                }
            }
        }
        return Ok(None);
    }

    /// What:
    ///  The bytes of the file at `place`,
    /// judged to be the one file its entry describes:
    /// opened below the held directory
    ///  without following a link,
    /// its type and mode judged on the open handle,
    /// and its state the same before and after.
    /// Why:
    ///   A swap,
    /// a `chmod` or a write while the file is read must refuse instead of passing,
    /// and the read is bounded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readStable(place: Place, entry: StatEntry, indexSeconds: number): Buffer; // throws the reason
    /// ```
    fn read_stable(place: &Place<'_>, entry: &StatEntry, index_seconds: u32, shown: &str, seams: &mut ReadSeams<'_>) -> Result<Vec<u8>, String> {
        let named: std::fs::Metadata = subject_failure(entry_metadata_at(place.directory, place.name), &shown, "cannot be examined")?;
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &named, index_seconds) {
            return Err(reason);
        }
        if named.file_type().is_symlink() {
            let target: PathBuf = subject_failure(read_link_at(place.directory, place.name), &shown, "cannot be read")?;
            let Ok(again) = entry_metadata_at(place.directory, place.name) else {
                return Err(changed_while_examined(shown));
            };
            if !same_state(&named, &again) {
                return Err(changed_while_examined(shown));
            }
            return Ok(target.into_os_string().into_vec());
        }
        (seams.before_open)();
        let file: std::fs::File = subject_failure(open_file_at(place.directory, place.name), &shown, "cannot be opened without following a link")?;
        let held: std::fs::Metadata = subject_failure(file.metadata(), &shown, "cannot be examined")?;
        if held.dev() != named.dev() || held.ino() != named.ino() {
            return Err(changed_while_examined(shown));
        }
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &held, index_seconds) {
            return Err(reason);
        }
        if held.len() > CONTENT_LIMIT {
            return Err(too_large_to_compare(shown));
        }
        (seams.before_read)();
        let Some(data) = subject_failure(read_at_most(&file, CONTENT_LIMIT), &shown, "cannot be read")? else {
            return Err(too_large_to_compare(shown));
        };
        (seams.after_read)();
        let Ok(after) = file.metadata() else {
            return Err(changed_while_examined(shown));
        };
        let Ok(by_name) = entry_metadata_at(place.directory, place.name) else {
            return Err(changed_while_examined(shown));
        };
        if !same_state(&held, &after) || !same_state(&held, &by_name) {
            return Err(changed_while_examined(shown));
        }
        return Ok(data);
    }

    /// What:
    ///  Whether the file at `place` equals the blob named `id`;
    /// the reason it cannot be proven so when it does not.
    /// Why:
    ///   Where a conversion applies,
    /// or the file has a carriage return,
    /// the bytes alone prove nothing and the answer is a
    ///  refusal that says so;
    /// a file that really differs says it differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.agrees(place, entry, id, indexSeconds); // throws the reason
    /// ```
    pub fn agrees(&mut self, place: &Place<'_>, entry: &StatEntry, id: &[u8], index_seconds: u32) -> Result<(), String> {
        return self.agrees_seamed(place, entry, id, index_seconds, &mut ReadSeams { before_open: &mut do_nothing, before_read: &mut do_nothing, after_read: &mut do_nothing });
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

/// Controls for the findings of the review of the content decision,
/// on real Git repositories.
#[cfg(test)]
#[path = "worktree_clean_content_safety_tests.rs"]
mod safety_tests;
