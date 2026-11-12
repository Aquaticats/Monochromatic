//! What:
//!  The files ref storage read in process:
//! one ref by exact name,
//! the symbolic chain behind it,
//! and the list of every ref.
//! Why:
//!  A loose file wins over `packed-refs`;
//! a symbolic file names another ref;
//! both are plain text.
//!  Every spelling the reader is not sure of (a link,
//! a name Git itself would refuse,
//! a damaged value,
//! a linked
//!  worktree's split of per-worktree and shared refs) is `Unsure`,
//! which sends the caller to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refs = new Refs(commonDir, hashHexLength); refs.lookup('refs/heads/main'); refs.list();
//! ```

#![cfg(unix)]

use crate::private_storage::read_regular_file;
use std::path::{Path, PathBuf};

/// The longest loose ref file or `packed-refs` line this reader looks at.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const LOOSE_LIMIT: u64 = 4096;

/// The largest `packed-refs` file this reader looks at.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const PACKED_LIMIT: u64 = 256 << 20;

/// How many symbolic ref hops are followed.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const SYMBOLIC_DEPTH_LIMIT: usize = 5;

/// What a ref holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefValue {
    /// An object name as lower-case hexadecimal.
    Direct(String),
    /// The name of another ref.
    Symbolic(String),
}

/// The answer to a lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lookup {
    /// The ref exists with this value.
    Found(RefValue),
    /// Neither a loose file nor a packed line names it.
    Missing,
    /// Something here is outside the plain layouts.
    Unsure,
}

/// The shared refs of one repository whose worktree owns its Git directory.
pub struct Refs {
    /// The common directory,
    /// which holds `refs/` and `packed-refs`.
    directory: PathBuf,
    /// The number of hexadecimal digits of an object name.
    hex_length: usize,
}

/// Whether `name` is a ref name Git lists and this reader can spell:
/// `refs/` followed by components Git's own rule accepts.
/// Why:
///   Git's rule is its `refname_disposition` table (`refs.c`,
/// `check_refname_component`):
/// every printable byte but space and `*` `:` `?` `[` `\` `^` `~`,
/// with `.` refused directly after another `.`,
/// `{` refused directly after `@`,
/// a component refused when it is empty,
/// starts with `.` or ends with `.lock`,
/// and the whole name refused when it ends with `.`.
///   This reader takes that rule and keeps declining on the bytes it cannot prove Git lists:
/// a byte above ASCII,
/// a control byte and DEL.
///  A name that is refused here ends the whole listing or the lookup with `Unsure`,
/// which is the safe direction,
/// because a ref name the reader answered with the wrong spelling would be a wrong answer where a
///  decline only costs a Git start.
///   `@`,
/// `+`,
/// `{` and `}` are accepted,
/// which is what the scoped tags of this repository need (`refs/tags/@monochromatic-dev/...`).
pub fn is_listable_ref_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("refs/") else {
        return false;
    };
    if rest.is_empty() || rest.ends_with('.') || rest.contains("..") || rest.contains("@{") {
        return false;
    }
    for component in rest.split('/') {
        let plain: bool = !component.is_empty()
            && !component.starts_with('.')
            && !component.ends_with(".lock")
            && component.bytes().all(is_reference_name_byte);
        if !plain {
            return false;
        }
    }
    return true;
}

/// Whether `byte` may appear in a component of a reference name this reader takes:
/// every printable ASCII byte but the ones Git forbids in a stored ref name
/// (`*`,
/// `:`,
/// `?`,
/// `[`,
/// `\`,
/// `^` and `~`).
///   `.` and `{` are handled by their own rules in `is_listable_ref_name`,
/// and the bytes outside printable ASCII are left to Git.
fn is_reference_name_byte(byte: u8) -> bool {
    return byte.is_ascii_graphic() && !matches!(byte, b'*' | b':' | b'?' | b'[' | b'\\' | b'^' | b'~');
}

/// Whether `byte` is a lower-case hexadecimal digit.
fn is_lower_hex_digit(byte: u8) -> bool {
    return matches!(byte, b'0'..=b'9' | b'a'..=b'f');
}

/// Whether `byte` is the digit zero.
fn is_zero_digit(byte: u8) -> bool {
    return byte == b'0';
}

/// Whether `byte` is a control byte (below space,
/// or DEL),
/// which a packed ref name cannot hold.
fn is_control_byte(byte: u8) -> bool {
    return byte < 0x20 || byte == 0x7f;
}

impl Refs {
    /// Readers for the refs under `directory`,
    /// with object names of `hex_length` digits.
    pub fn new(directory: &Path, hex_length: usize) -> Refs {
        return Refs { directory: directory.to_path_buf(), hex_length };
    }

    /// Whether `text` is a plain,
    /// non-null,
    /// lower-case object name.
    fn is_name(&self, text: &str) -> bool {
        return text.len() == self.hex_length
            && text.bytes().all(is_lower_hex_digit)
            && !text.bytes().all(is_zero_digit);
    }

    /// The value of a loose file's text.
    fn parse_loose(&self, text: &str) -> Option<RefValue> {
        let line: &str = text.strip_suffix('\n')?;
        if let Some(target) = line.strip_prefix("ref: ") {
            if is_listable_ref_name(target) {
                return Some(RefValue::Symbolic(target.to_string()));
            }
            return None;
        }
        if self.is_name(line) {
            return Some(RefValue::Direct(line.to_string()));
        }
        return None;
    }

    /// The packed lines as `(name, object name)`,
    /// or `None` when the file is not a plain `packed-refs`.
    fn packed(&self) -> Option<Vec<(String, String)>> {
        return self.packed_named(false);
    }

    /// What:
    ///  How many refs `packed-refs` names,
    /// without their values,
    /// so a caller can tell how long a listing would be before it reads one.
    /// Why:
    ///  The count is read before the listing is built,
    /// and a file holding more refs than the caller answers in process must make it decline without
    ///  walking `refs/` or checking one object.
    /// A line with no space names no ref and makes the file unsure;
    /// a comment is allowed only first,
    /// and a peeled line belongs to the ref above it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// countPackedRefs(directory): number | undefined;
    /// ```
    pub fn packed_ref_count(&self) -> Option<usize> {
        let bytes: Vec<u8> = match read_regular_file(self.directory.join("packed-refs").as_path(), PACKED_LIMIT) {
            Ok(bytes) => bytes,
            Err(crate::private_storage::ReadRefusal::Missing) => return Some(0),
            Err(_) => return None,
        };
        let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
        let mut count: usize = 0;
        for (position, line) in text.lines().enumerate() {
            if line.starts_with('#') {
                if position != 0 {
                    return None;
                }
                continue;
            }
            if line.starts_with('^') {
                continue;
            }
            line.split_once(' ')?;
            count += 1;
        }
        return Some(count);
    }

    /// The packed lines as `(name, object name)`.
    /// With `skip_unplain` set,
    /// a well-formed line whose name Git itself would refuse,
    /// a name holding a space for one,
    /// is left out instead of making the whole file unreadable.
    /// Why:
    /// a lookup of one name Git accepts cannot be answered by a line Git refuses,
    /// and a Git that lists refs words a broken packed name on standard error
    /// and prints the other refs without it (`ref-filter.c`,
    /// "ignoring ref with broken name"),
    /// and the reader leaves the reading of such a listing to Git.
    /// A line with a damaged object name still ends the read.
    fn packed_named(&self, skip_unplain: bool) -> Option<Vec<(String, String)>> {
        let bytes: Vec<u8> = match read_regular_file(self.directory.join("packed-refs").as_path(), PACKED_LIMIT) {
            Ok(bytes) => bytes,
            Err(crate::private_storage::ReadRefusal::Missing) => return Some(Vec::new()),
            Err(_) => return None,
        };
        let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
        let mut packed: Vec<(String, String)> = Vec::new();
        // A peeled line (`^<name>`) is valid only directly after a ref line.
        let mut after_ref: bool = false;
        for (position, line) in text.lines().enumerate() {
            if line.starts_with('#') {
                if position != 0 {
                    return None;
                }
                continue;
            }
            if let Some(peeled) = line.strip_prefix('^') {
                if !after_ref || !self.is_name(peeled) {
                    return None;
                }
                after_ref = false;
                continue;
            }
            let (object, name) = line.split_once(' ')?;
            if !self.is_name(object) || name.is_empty() || name.bytes().any(is_control_byte) {
                return None;
            }
            after_ref = true;
            if !is_listable_ref_name(name) {
                if skip_unplain {
                    continue;
                }
                return None;
            }
            packed.push((name.to_string(), object.to_string()));
        }
        return Some(packed);
    }

    /// What the `HEAD` file of `git_dir` holds,
    /// when it is a plain symbolic or direct spelling.
    pub fn lookup_head(&self, git_dir: &Path) -> Option<RefValue> {
        let bytes: Vec<u8> = read_regular_file(git_dir.join("HEAD").as_path(), LOOSE_LIMIT).ok()?;
        return self.parse_loose(std::str::from_utf8(bytes.as_slice()).ok()?);
    }

    /// What ref `name` holds,
    /// loose first.
    pub fn lookup(&self, name: &str) -> Lookup {
        if !is_listable_ref_name(name) {
            return Lookup::Unsure;
        }
        let path: PathBuf = self.directory.join(name);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => {
                let Ok(bytes) = read_regular_file(path.as_path(), LOOSE_LIMIT) else {
                    return Lookup::Unsure;
                };
                let Ok(text) = std::str::from_utf8(bytes.as_slice()) else {
                    return Lookup::Unsure;
                };
                let Some(value) = self.parse_loose(text) else {
                    return Lookup::Unsure;
                };
                return Lookup::Found(value);
            }
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Lookup::Unsure,
            // A file where a directory is expected (`refs/heads/a` as a file, asked for `refs/heads/a/b`) is absence.
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => {}
            Err(_) => return Lookup::Unsure,
        }
        let Some(packed) = self.packed_named(true) else {
            return Lookup::Unsure;
        };
        for (candidate, object) in packed {
            if candidate == name {
                return Lookup::Found(RefValue::Direct(object));
            }
        }
        return Lookup::Missing;
    }

    /// The object name `name` resolves to through symbolic refs;
    /// `None` when it is missing or unsure.
    pub fn resolve(&self, name: &str) -> Option<String> {
        let mut current: String = name.to_string();
        for _ in 0..SYMBOLIC_DEPTH_LIMIT {
            match self.lookup(current.as_str()) {
                Lookup::Found(RefValue::Direct(object)) => return Some(object),
                Lookup::Found(RefValue::Symbolic(target)) => current = target,
                Lookup::Missing | Lookup::Unsure => return None,
            }
        }
        return None;
    }

    /// Every ref under `refs/`,
    /// sorted by name,
    /// loose over packed;
    /// `None` when a name is one Git refuses or one this reader keeps declining on.
    pub fn list(&self) -> Option<Vec<(String, RefValue)>> {
        let mut found: std::collections::BTreeMap<String, RefValue> = std::collections::BTreeMap::new();
        for (name, object) in self.packed()? {
            found.insert(name, RefValue::Direct(object));
        }
        self.walk("refs", &mut found)?;
        return Some(found.into_iter().collect());
    }

    /// What:
    ///  Every ref stored loose under one namespace of this directory's `refs` tree,
    /// sorted by name;
    ///  `None` when a name is one Git refuses or one this reader keeps declining on.
    /// Why:
    ///  Git keeps three namespaces in a linked worktree's own Git directory rather than in the shared one
    ///  (`refs/bisect`,
    /// `refs/worktree` and `refs/rewritten`,
    /// `refs.c` `parse_worktree_ref`),
    ///  and a listing that has to be what Git prints from that worktree must include them.
    ///  They are never packed: `git pack-refs` leaves them out,
    /// so no `packed-refs` is read here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// looseNamespace('refs/bisect'): [name, value][] | undefined;
    /// ```
    pub fn loose_namespace(&self, namespace: &str) -> Option<Vec<(String, RefValue)>> {
        if !namespace.starts_with("refs/") {
            return None;
        }
        let mut found: std::collections::BTreeMap<String, RefValue> = std::collections::BTreeMap::new();
        self.walk(namespace, &mut found)?;
        return Some(found.into_iter().collect());
    }

    /// What:
    ///  Adds every loose ref under `start` to `found`,
    ///  descending into directories,
    ///  or `None` when something under it is not a layout this reader can prove.
    /// Why:
    ///   The walk of a whole repository and the walk of one of its namespaces are the same walk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// walk(start, found): boolean;
    /// ```
    fn walk(&self, start: &str, found: &mut std::collections::BTreeMap<String, RefValue>) -> Option<()> {
        let mut pending: Vec<String> = vec![start.to_string()];
        while let Some(directory) = pending.pop() {
            let listing = match std::fs::read_dir(self.directory.join(&directory)) {
                Ok(listing) => listing,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return None,
            };
            for candidate in listing {
                let entry = candidate.ok()?;
                let name: String = format!("{directory}/{}", entry.file_name().to_str()?);
                let kind = entry.file_type().ok()?;
                if kind.is_dir() {
                    pending.push(name);
                } else if kind.is_file() && !name.ends_with(".lock") {
                    match self.lookup(name.as_str()) {
                        Lookup::Found(value) => {
                            found.insert(name, value);
                        }
                        Lookup::Missing | Lookup::Unsure => return None,
                    }
                } else {
                    return None;
                }
            }
        }
        return Some(());
    }
}

#[cfg(test)]
#[path = "repo_read_refs_tests.rs"]
mod tests;
