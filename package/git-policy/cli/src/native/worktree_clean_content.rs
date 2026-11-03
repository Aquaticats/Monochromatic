//! What:
//!  The decision, by content and without a Git process, whether a tracked file that its index entry's stat data cannot
//!  vouch for (a racily clean entry, a stale stat) still equals the committed blob: hash the file as a blob and compare it
//!  with the entry's object name, refusing where a content conversion could make the two differ.
//! Why:
//!  Git settles those entries by hashing the file; stat data alone cannot tell a modified file from a copied or cloned one
//!  that kept the index's old inode and times. The hash of the raw bytes is conclusive only when Git's own conversion on the
//!  way into the index (`filter`, `ident`, `working-tree-encoding`, or a line-ending conversion that strips a carriage
//!  return) leaves the bytes alone, so a path such an attribute applies to, and a file with a carriage return, are refused
//!  with that reason instead of guessed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new ContentJudge({ root, gitDir, commonDir, scope, format });
//! judge.agrees(entry, blobId); // throws the reason the file cannot be proven equal to the blob
//! ```

use super::repo_read_config_list::list_config;
use super::repo_read_config_scan::ScopeEnvironment;
use super::repo_read_config_text::last_entry;
use super::repo_read_layout::ObjectFormat;
use super::subvolume_sha::{sha1, sha256};
use super::worktree_clean::StatEntry;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The largest file read into memory to be hashed; a larger one is refused with that reason.
pub const CONTENT_LIMIT: u64 = 32 << 20;

/// The attributes that change a file's bytes on the way into the index, besides the line-ending ones.
const CONVERTING_ATTRIBUTES: &[&str] = &["filter", "ident", "working-tree-encoding"];

/// One line of an attributes file: a pattern with its attribute tokens, or a macro definition.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributeLine {
    /// The pattern, or the macro name when `is_macro`.
    subject: String,
    /// Whether the line defines a macro (`[attr]name ...`).
    is_macro: bool,
    /// The attribute tokens as written (`filter=lfs`, `-text`, `ident`).
    tokens: Vec<String>,
}

/// One attributes file as it was read.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributeFile {
    /// How the reason names the file.
    label: String,
    /// The directory of the file relative to the worktree root, `/` separated, empty for the root.
    base: String,
    /// Its lines.
    lines: Vec<AttributeLine>,
}

/// What:
///  The lines of an attributes file's text: patterns with tokens, and macro definitions.
/// Why:
///   A comment or blank line applies nothing; a quoted pattern keeps its quotes so the matcher can decline it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAttributeLines(text: string): AttributeLine[];
/// ```
fn parse_attribute_lines(text: &str) -> Vec<AttributeLine> {
    let mut lines: Vec<AttributeLine> = Vec::new();
    for raw in text.lines() {
        let line: &str = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (subject, rest): (&str, &str) = if line.starts_with('"') {
            match line[1..].find('"') {
                Some(end) => (&line[..end + 2], &line[end + 2..]),
                None => (line, ""),
            }
        } else {
            line.split_once(char::is_whitespace).unwrap_or((line, ""))
        };
        let tokens: Vec<String> = rest.split_whitespace().map(String::from).collect();
        match subject.strip_prefix("[attr]") {
            Some(name) => lines.push(AttributeLine { subject: String::from(name), is_macro: true, tokens }),
            None => lines.push(AttributeLine { subject: String::from(subject), is_macro: false, tokens }),
        }
    }
    return lines;
}

/// The attribute a token sets, unset or leaves unspecified, without the sign and the value.
fn attribute_name(token: &str) -> &str {
    return token.trim_start_matches(['-', '!']).split('=').next().unwrap_or("");
}

/// What:
///  Whether `text` matches the shell pattern `pattern` of an attributes line, where `*` and `?` stop at a slash.
/// Why:
///   Both are compared in lower case, so a case-insensitive file system can only make a match more likely, which refuses more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wildcard(pattern: string, text: string): boolean;
/// ```
fn wildcard(pattern: &[u8], text: &[u8]) -> bool {
    // `reach[j]` says the pattern consumed so far can match the first `j` bytes of the text.
    let mut reach: Vec<bool> = vec![false; text.len() + 1];
    reach[0] = true;
    for &symbol in pattern {
        let mut next: Vec<bool> = vec![false; text.len() + 1];
        if symbol == b'*' {
            next[0] = reach[0];
            for at in 1..=text.len() {
                next[at] = reach[at] || (next[at - 1] && text[at - 1] != b'/');
            }
        } else {
            for at in 1..=text.len() {
                let same: bool = if symbol == b'?' { text[at - 1] != b'/' } else { symbol == text[at - 1] };
                next[at] = reach[at - 1] && same;
            }
        }
        reach = next;
    }
    return reach[text.len()];
}

/// What:
///  Whether the pattern of an attributes line in the directory `base` may apply to the file `path`; a spelling the matcher
///  does not model (a quote, an escape, a bracket class, `**`, a trailing slash) answers yes.
/// Why:
///   Answering yes refuses a file that Git might not have matched; answering no for a file it matches would accept one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayApply(pattern: string, base: string, path: string): boolean;
/// ```
fn may_apply(pattern: &str, base: &str, path: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if pattern.starts_with('"') || pattern.contains(['\\', '[']) || pattern.contains("**") || pattern.ends_with('/') {
        return true;
    }
    let relative: &str = if base.is_empty() {
        path
    } else {
        match path.strip_prefix(base).and_then(|rest: &str| { return rest.strip_prefix('/'); }) {
            Some(rest) => rest,
            None => return false,
        }
    };
    let lowered_path: String = relative.to_lowercase();
    let lowered_pattern: String = pattern.to_lowercase();
    if lowered_pattern.contains('/') {
        return wildcard(lowered_pattern.trim_start_matches('/').as_bytes(), lowered_path.as_bytes());
    }
    let name: &str = lowered_path.rsplit('/').next().unwrap_or("");
    return wildcard(lowered_pattern.as_bytes(), name.as_bytes());
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
///  Decides, one entry at a time, whether a file equals its committed blob by content.
/// Why:
///   The attributes files are read once, on the first entry that needs them, and the per-directory ones are cached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentJudge { agrees(entry: StatEntry, id: Uint8Array): void /* throws the reason */ }
/// ```
pub struct ContentJudge<'a> {
    /// The worktree root.
    root: &'a Path,
    /// The worktree's Git directory.
    git_dir: &'a Path,
    /// The shared Git directory.
    common_dir: &'a Path,
    /// Which system and global files Git reads in this environment.
    scope: &'a ScopeEnvironment,
    /// The object format of the repository.
    format: ObjectFormat,
    /// The attributes files that do not depend on the path, and the macros that set a converting attribute.
    fixed: Option<Result<(Vec<AttributeFile>, Vec<String>), String>>,
    /// The `.gitattributes` of each directory, read at most once; `None` when the directory has none.
    directories: HashMap<String, Option<AttributeFile>>,
}

impl<'a> ContentJudge<'a> {
    /// A judge for the worktree at `root`.
    pub fn new(root: &'a Path, git_dir: &'a Path, common_dir: &'a Path, scope: &'a ScopeEnvironment, format: ObjectFormat) -> ContentJudge<'a> {
        return ContentJudge { root, git_dir, common_dir, scope, format, fixed: None, directories: HashMap::new() };
    }

    /// Read an attributes file; absent is `None`, unreadable is the reason.
    fn read_file(path: &Path, label: String, base: String) -> Result<Option<AttributeFile>, String> {
        return match std::fs::read(path) {
            Ok(bytes) => Ok(Some(AttributeFile { label, base, lines: parse_attribute_lines(String::from_utf8_lossy(bytes.as_slice()).as_ref()) })),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => Ok(None),
            Err(error) => Err(format!("the attributes file {} cannot be read ({error})", path.display())),
        };
    }

    /// The attributes file `core.attributesFile` names, or the default one, from the configuration read in process.
    fn global_attributes_path(&self) -> Result<Option<PathBuf>, String> {
        let variables: Vec<(OsString, OsString)> = std::env::vars_os().collect();
        let listed = list_config(self.common_dir, self.git_dir, self.scope, variables.as_slice())
            .ok_or_else(|| { return String::from("the Git configuration cannot be read without running Git, so a global attributes file cannot be ruled out"); })?;
        let entries: Vec<super::repo_read_config_text::ConfigEntry> = listed.into_iter().map(|item| { return item.entry; }).collect();
        if let Some(entry) = last_entry(entries.as_slice(), "core.attributesfile") {
            let value: &[u8] = entry.value.as_deref().ok_or_else(|| { return String::from("core.attributesFile has no value"); })?;
            let text: String = String::from_utf8_lossy(value).into_owned();
            if let Some(rest) = text.strip_prefix("~/") {
                let home: &Path = self.scope.home.as_deref().ok_or_else(|| { return String::from("core.attributesFile starts with ~/ but HOME is not set"); })?;
                return Ok(Some(home.join(rest)));
            }
            let path: PathBuf = PathBuf::from(OsStr::from_bytes(value));
            if !path.is_absolute() {
                return Err(format!("core.attributesFile is {text:?}, which is not an absolute path"));
            }
            return Ok(Some(path));
        }
        return Ok(match (&self.scope.xdg, &self.scope.home) {
            (Some(directory), _) => Some(directory.join("git/attributes")),
            (None, Some(home)) => Some(home.join(".config/git/attributes")),
            (None, None) => None,
        });
    }

    /// The attributes files that apply whatever the path is, with the macros among them that set a converting attribute.
    fn load_fixed(&self) -> Result<(Vec<AttributeFile>, Vec<String>), String> {
        let mut files: Vec<AttributeFile> = Vec::new();
        let system: PathBuf = self.scope.system.clone().or_else(|| { return self.scope.system_default.clone(); }).and_then(|config: PathBuf| { return config.parent().map(|directory: &Path| { return directory.join("gitattributes"); }); }).unwrap_or_else(|| { return PathBuf::from("/etc/gitattributes"); });
        let mut sources: Vec<(PathBuf, String)> = vec![(self.root.join(".gitattributes"), String::from(".gitattributes")), (self.common_dir.join("info/attributes"), String::from("info/attributes")), (system.clone(), system.display().to_string())];
        if let Some(path) = self.global_attributes_path()? {
            sources.push((path.clone(), path.display().to_string()));
        }
        for (path, label) in sources {
            if let Some(file) = Self::read_file(path.as_path(), label, String::new())? {
                files.push(file);
            }
        }
        let mut macros: Vec<String> = Vec::new();
        loop {
            let before: usize = macros.len();
            for line in files.iter().flat_map(|file: &AttributeFile| { return file.lines.iter(); }) {
                let converts: bool = line.tokens.iter().any(|token: &String| { return CONVERTING_ATTRIBUTES.contains(&attribute_name(token.as_str())) || macros.iter().any(|name: &String| { return name == attribute_name(token.as_str()); }); });
                if line.is_macro && converts && !macros.contains(&line.subject) {
                    macros.push(line.subject.clone());
                }
            }
            if macros.len() == before {
                break;
            }
        }
        return Ok((files, macros));
    }

    /// The first converting attribute that may apply to `path` and where it is set, or nothing.
    fn converting_attribute(&mut self, path: &str) -> Result<Option<(String, String)>, String> {
        if self.fixed.is_none() {
            self.fixed = Some(self.load_fixed());
        }
        let (fixed, macros): (Vec<AttributeFile>, Vec<String>) = self.fixed.clone().ok_or_else(|| { return String::from("the attributes were not loaded"); })??;
        let mut files: Vec<AttributeFile> = fixed;
        let mut base: String = String::new();
        let mut parents: Vec<String> = vec![String::new()];
        for part in path.split('/').take(path.matches('/').count()) {
            base = if base.is_empty() { String::from(part) } else { format!("{base}/{part}") };
            parents.push(base.clone());
        }
        for directory in parents.into_iter().filter(|name: &String| { return !name.is_empty(); }) {
            if !self.directories.contains_key(&directory) {
                let file: Option<AttributeFile> = Self::read_file(self.root.join(&directory).join(".gitattributes").as_path(), format!("{directory}/.gitattributes"), directory.clone())?;
                self.directories.insert(directory.clone(), file);
            }
            if let Some(Some(file)) = self.directories.get(&directory) {
                files.push(file.clone());
            }
        }
        for file in &files {
            for line in file.lines.iter().filter(|line: &&AttributeLine| { return !line.is_macro; }) {
                let Some(token) = line.tokens.iter().find(|token: &&String| { return CONVERTING_ATTRIBUTES.contains(&attribute_name(token.as_str())) || macros.iter().any(|name: &String| { return name == attribute_name(token.as_str()); }); }) else { continue };
                if may_apply(line.subject.as_str(), file.base.as_str(), path) {
                    return Ok(Some((token.clone(), file.label.clone())));
                }
            }
        }
        return Ok(None);
    }

    /// What:
    ///  Whether the file of `entry` equals the blob named `id`; the reason it cannot be proven so when it does not.
    /// Why:
    ///   Where a conversion applies, or the file has a carriage return, the bytes alone prove nothing and the answer is a
    ///  refusal that says so; a file that really differs says it differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.agrees(entry, id); // throws the reason
    /// ```
    pub fn agrees(&mut self, entry: &StatEntry, id: &[u8]) -> Result<(), String> {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        if let Some((token, label)) = self.converting_attribute(shown.as_str())? {
            return Err(format!("{shown} cannot be compared by content: the attribute {token} from {label} may change its bytes on the way into the index"));
        }
        let location: PathBuf = self.root.join(OsStr::from_bytes(entry.path.as_slice()));
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(&location).map_err(|error: std::io::Error| { return format!("{shown} cannot be examined ({error})"); })?;
        let data: Vec<u8> = if metadata.file_type().is_symlink() {
            std::fs::read_link(&location).map_err(|error: std::io::Error| { return format!("{shown} cannot be read ({error})"); })?.as_os_str().as_bytes().to_vec()
        } else if metadata.file_type().is_file() {
            if metadata.len() > CONTENT_LIMIT {
                return Err(format!("{shown} is larger than {} bytes, which is more than is read to compare it by content", CONTENT_LIMIT));
            }
            std::fs::read(&location).map_err(|error: std::io::Error| { return format!("{shown} cannot be read ({error})"); })?
        } else {
            return Err(format!("{shown} is neither a file nor a symbolic link"));
        };
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
