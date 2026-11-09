//! What:
//!  The attribute files a content decision reads:
//! where Git takes them from,
//! how a `.gitattributes` file is parsed,
//! and which of its lines can
//!  convert a file's bytes on the way into the index.
//! Why:
//!  `worktree_clean_content.rs` decides whether a file equals its committed bytes,
//! and that decision may only trust the
//!  bytes when no attribute Git applies converts them;
//! the reading of the files lives here so the decision stays readable,
//! and every byte
//!  read is charged to the caller's bound when it set one (`parse_open`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = ContentJudge.new(...); judge.chargeAttributesTo(spent); const applies = judge.convertingAttribute(path, parents);
//! ```

use crate::worktree_clean_content::ContentJudge;
use crate::worktree_clean_content::unmodelled;
use crate::fs_beneath::{open_directory_beneath, open_file_at};
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_text::{ConfigEntry, last_entry};
use crate::worktree_attributes::{may_apply, parse_attribute_lines};
use crate::worktree_clean_content::{ATTRIBUTES_LIMIT, CONTENT_LIMIT};
use crate::worktree_clean_content_attribute_sources::{AttributeFile, FixedSources, SystemAttributes, converting_macros, first_converting_token, has_quoted_line};
use crate::worktree_path_failure_message::subject_failure;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// The bytes one attribute read asks for at most.
const ATTRIBUTES_CHUNK: usize = 64 * 1024;

impl<'a> ContentJudge<'a> {
    /// Read an attributes file at an absolute path the way Git does,
    /// following a link;
    /// absent is `None`.
    pub(super) fn read_outside(&self, path: &Path, label: String) -> Result<Option<AttributeFile>, String> {
        // Without blocking, so a named pipe is found out by its type instead of waiting for a writer.
        let file = match std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC).open(path) {
            Ok(opened) => opened,
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => return Ok(None),
            Err(error) => return Err(format!("the attributes file {} cannot be read ({error})", path.display())),
        };
        return self.parse_open(file, label, Vec::new());
    }

    /// Read the `.gitattributes` of the held directory `directory`;
    /// absent is `None`,
    /// a link or anything but a file is a refusal.
    pub(super) fn read_in(&self, directory: &std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let file = match open_file_at(directory, OsStr::new(".gitattributes")) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{label} cannot be read without following a link ({error}), so the attributes it sets cannot be ruled out")),
        };
        return self.parse_open(file, label, base);
    }

    /// Parse an attributes file that is open,
    /// refusing one that is not a plain file or is larger than the limit.
    /// What:
    ///  The bytes of an attributes file this reader still has room for,
    /// with what was read charged to the caller's bound.
    /// Why:
    ///   An attributes file is read under the same lock as the files it describes,
    /// so its bytes are part of what the caller bounds;
    /// a file that does not fit is refused,
    /// which costs a refresh and never a failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function attributeRoom(metadata) { const room = budget ? CONTENT_LIMIT - budget.bytes : ATTRIBUTES_LIMIT; return Math.min(room, ATTRIBUTES_LIMIT); }
    /// ```
    pub(super) fn attribute_room(&self) -> u64 {
        return match self.budget {
            Some(spent) => CONTENT_LIMIT.saturating_sub(spent.get()).min(ATTRIBUTES_LIMIT),
            None => ATTRIBUTES_LIMIT,
        };
    }

    /// Parse an attributes file that is open,
    /// refusing one that is not a plain file or is larger than the room it has,
    /// and charging the bytes it took.
    pub(super) fn parse_open(&self, file: std::fs::File, label: String, base: Vec<u8>) -> Result<Option<AttributeFile>, String> {
        let metadata: std::fs::Metadata = subject_failure(file.metadata(), &label, "cannot be examined")?;
        if !metadata.file_type().is_file() {
            return Err(format!("{label} is not a plain file (a named pipe, a device or a directory), so the attributes it sets cannot be ruled out"));
        }
        let room: u64 = self.attribute_room();
        if metadata.len() > room {
            return Err(format!("{label} is larger than the {room} bytes this caller may still read, so the attributes it sets cannot be ruled out"));
        }
        let Some(bytes) = self.attribute_bytes(&file, room, &label)? else {
            return Err(format!("{label} changed while it was read, so the attributes it sets cannot be ruled out"));
        };
        return Ok(Some(AttributeFile { label, base, lines: parse_attribute_lines(bytes.as_slice()) }));
    }

    /// What:
    ///  Take attributes files with `read`,
    /// for a control that hands over what a file grown under the read would.
    /// Why:
    ///   The overshoot the review found needs a file that grows between the size check and the read,
    /// and a control cannot make that race happen without a seam.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.readAttributesWith(read);
    /// ```
    #[cfg(test)]
    pub fn read_attributes_with(&mut self, read: &'a dyn Fn(&std::fs::File, &mut [u8]) -> std::io::Result<usize>) {
        self.attribute_read = Some(read);
    }

    /// What:
    ///  The bytes of an attributes file,
    /// up to `room`,
    /// taken with the read a control sets or the operating system's own,
    /// each piece charged as it arrives.
    /// Why:
    ///   The bound is on the reading,
    /// not on the size an earlier look reported:
    /// a file that grows after that look may not hand over a byte the caller
    ///  never counted,
    /// so the read is capped at the room and the file is measured again after it,
    /// and a file whose size moved under the read is refused rather than parsed
    ///  short.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function attributeBytes(file, room) {
    ///   const data = []; while (data.length < room) { const taken = read(file, new Uint8Array(Math.min(64 << 10, room - data.length)));
    ///     if (taken === 0) break; data.push(...taken); spent += taken; }
    ///   return stat(file).size === data.length ? data : undefined;
    /// }
    /// ```
    fn attribute_bytes(&self, file: &std::fs::File, room: u64, label: &dyn std::fmt::Display) -> Result<Option<Vec<u8>>, String> {
        let mut data: Vec<u8> = Vec::new();
        let mut buffer: Vec<u8> = vec![0; usize::try_from(room.min(ATTRIBUTES_CHUNK as u64)).unwrap_or(ATTRIBUTES_CHUNK)];
        while (data.len() as u64) < room {
            let want: usize = usize::try_from(room - data.len() as u64).unwrap_or(usize::MAX).min(ATTRIBUTES_CHUNK);
            let taken: usize = match self.attribute_read {
                Some(read) => subject_failure(read(file, &mut buffer[..want]), label, "cannot be read")?,
                None => {
                    use std::io::Read as _;
                    subject_failure((&*file).take(want as u64).read(&mut buffer[..want]), label, "cannot be read")?
                }
            };
            if taken == 0 {
                break;
            }
            let took: usize = taken.min(want);
            data.extend_from_slice(&buffer[..took]);
            self.charge(took as u64);
        }
        let after: std::fs::Metadata = subject_failure(file.metadata(), label, "cannot be examined")?;
        if after.len() != data.len() as u64 {
            return Ok(None);
        }
        return Ok(Some(data));
    }

    /// Add `bytes` to the caller's bound,
    /// when it set one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function charge(bytes) { if (budget) budget.bytes += bytes; }
    /// ```
    pub(super) fn charge(&self, bytes: u64) {
        if let Some(spent) = self.budget {
            spent.set(spent.get() + bytes);
        }
    }

    /// The configuration Git would list,
    /// read in process.
    pub(super) fn config_entries(&self) -> Result<Vec<ConfigEntry>, String> {
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
    pub(super) fn global_attributes_path(&self, entries: &[ConfigEntry]) -> Result<Option<PathBuf>, String> {
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
    pub(super) fn load_fixed(&self) -> Result<FixedSources, String> {
        if let Some(reason) = self.environment.alternate_attribute_source.as_deref() {
            return Err(unmodelled(reason));
        }
        let entries: Vec<ConfigEntry> = self.config_entries()?;
        if last_entry(entries.as_slice(), "attr.tree").is_some() {
            return Err(unmodelled("attr.tree is set"));
        }
        let mut files: Vec<AttributeFile> = Vec::new();
        if let Some(root_file) = self.read_in(self.root, String::from(".gitattributes"), Vec::new())? {
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
            if let Some(file) = self.read_outside(path.as_path(), label)? {
                files.push(file);
            }
        }
        let macros: HashSet<Vec<u8>> = converting_macros(files.as_slice());
        return Ok(FixedSources { files, macros });
    }

    /// The `.gitattributes` of the directory below the root named by `parents`,
    /// read once.
    pub(super) fn directory_file(&mut self, parents: &[OsString]) -> Result<Option<AttributeFile>, String> {
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
            let file: Option<AttributeFile> = self.read_in(&directory, label, key.clone())?;
            self.directories.insert(key.clone(), file);
        }
        return Ok(self.directories.get(&key).cloned().flatten());
    }

    /// The first converting attribute that may apply to the file at `path` and where it is set,
    /// or nothing.
    pub(super) fn converting_attribute(&mut self, path: &[u8], parents: &[OsString]) -> Result<Option<(String, String)>, String> {
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
}

/// Controls for the attribute reading stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clean_content_attributes_tests.rs"]
mod tests;
