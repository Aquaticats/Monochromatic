//! Reads the files a rules list names once and joins them into the one text the scanner compiles.
//!
//! The joined text is the only input of the compiled-rules cache key,
//! because the compiled rules are a pure function
//! of it:
//! two lists whose present files join to the same text compile to the same rules and may share an artifact.
//! The join is therefore the place that keeps the joined text equal in meaning to the files read separately:
//! each file is BOM-stripped,
//! every file ends in a newline before the next begins,
//! and files in different formats
//! (tail sections against legacy lines) are refused,
//! because the later file's lines would otherwise become the body
//! of the earlier file's last section.
//! A compile failure of the joined text is re-diagnosed file by file,
//! so the error names the file and a line
//! number local to it.

/// Reads the format of each file's text,
/// strips its byte order mark,
/// and parses it,
/// so the joined text keeps one format
/// and a compile failure can be re-diagnosed file by file.
use crate::rule::frx::{parse_runtime_rules, source_format, strip_source_bom, SourceFormat};
/// One entry of the rules list,
/// a path and whether the file may be absent.
use crate::rules_list::RulesEntry;
/// The compiled rules that the joined text becomes.
use crate::runtime_matcher::RuntimeRules;
/// Builds the errors that name a file,
/// and the `Result` every step returns.
use anyhow::{anyhow, Result};
/// Remembers which file declared each rule,
/// so a duplicate names both files.
use std::collections::HashMap;
/// Borrowed and owned paths of the rules files read.
use std::path::{Path, PathBuf};

/// A file that was present when read.
struct Present {
    /// The file's exact bytes,
    /// compared again before an artifact is published.
    bytes: Vec<u8>,
    /// The same bytes decoded as UTF-8.
    text: String,
}

/// What one entry held when it was read.
struct ReadEntry {
    /// The file the entry names.
    path: PathBuf,
    /// The file's content,
    /// or `None` when an optional entry was absent.
    present: Option<Present>,
}

/// The files of one rules list,
/// read once,
/// with their joined text.
pub(crate) struct RulesSnapshot {
    /// Every entry in list order,
    /// including the absent optional ones.
    entries: Vec<ReadEntry>,
    /// The text the scanner compiles:
    /// the present files joined in list order.
    text: String,
}

/// Whether an optional entry was absent when the list was read.
fn is_absent(entry: &ReadEntry) -> bool {
    return entry.present.is_none();
}

/// The path of an entry as display text.
fn path_text_of(entry: &ReadEntry) -> String {
    return entry.path.display().to_string();
}

/// Whether the file system still agrees with what one entry held:
/// the same bytes for a present file,
/// still absent for a skipped one.
///
/// The file is read by `read_rules_file`,
/// so a name that became a named pipe is answered at once
/// and is never taken for the text that was read.
fn holds_what_was_read(entry: &ReadEntry) -> bool {
    return match (&entry.present, read_rules_file(&entry.path)) {
        (Some(expected), Ok(now)) => expected.bytes == now,
        (None, Err(error)) => error.kind() == std::io::ErrorKind::NotFound,
        _ => false,
    };
}

/// What:
///  The bytes of the rules file at `path`,
///  opened without waiting for a writer.
/// Why:
///  A named pipe at a rules name blocks a plain read for ever,
/// so `git add`,
/// `git commit` and `check --all`
///  never ended when one was planted
/// (`doc/troubleshooting/a-fifo-where-a-file-is-expected-ends-no-command.md`).
///  `O_NONBLOCK` answers at once whatever the name holds,
/// the descriptor's own type then decides whether its
///  bytes may be read at all,
/// and a link is followed as the configuration means it.
///  A directory,
/// a pipe,
/// a socket or a device is refused with the words `refused_for_not_being_plain`,
/// so the caller
///  fails naming the path instead of scanning with no rule from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRulesFile(path) { const file = openSync(path, 'r', 0o666, O_NONBLOCK); if (!fstatSync(file).isFile()) throw notPlain(); return readFileSync(file); }
/// ```
#[cfg(unix)]
fn read_rules_file(path: &Path) -> std::io::Result<Vec<u8>> {
    /// `OpenOptionsExt` adds `custom_flags`, the open flags the standard library has no setting for.
    use std::os::unix::fs::OpenOptionsExt as _;
    /// `Read` adds `read_to_end` to the opened file.
    use std::io::Read as _;
    let mut opened: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC).open(path)?;
    if !opened.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, refused_for_not_being_plain()));
    }
    let mut bytes: Vec<u8> = Vec::new();
    opened.read_to_end(&mut bytes)?;
    return Ok(bytes);
}

/// The words a rules file that is not a plain file is refused with,
/// naming the shapes a person may have left there.
fn refused_for_not_being_plain() -> &'static str {
    return "it is not a plain file (a named pipe, a device or a directory)";
}

/// What:
///  The bytes of the rules file at `path` where the open carries no `O_NONBLOCK`.
/// Why:
///  No target without `unix` has a named pipe a person can plant at a path,
/// so a directory is the one shape
///  refused before the read.
/// UNTESTED ON WINDOWS: no gate runs this branch (`README`, Requirements).
#[cfg(not(unix))]
fn read_rules_file(path: &Path) -> std::io::Result<Vec<u8>> {
    if !std::fs::metadata(path)?.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, refused_for_not_being_plain()));
    }
    return std::fs::read(path);
}

/// The `(path, text)` pair of an entry that was present,
/// or `None` for a skipped one.
fn present_pair(entry: &ReadEntry) -> Option<(&Path, &str)> {
    let file = entry.present.as_ref()?;
    return Some((entry.path.as_path(), file.text.as_str()));
}

/// Reads and joins a rules list.
impl RulesSnapshot {
    /// Reads every entry in list order.
    ///
    /// A required entry that cannot be read,
    /// and any entry that is present but unreadable or not UTF-8,
    /// fails the
    /// read with an error naming that path.
    /// An optional entry whose file is absent is skipped;
    /// [`RulesSnapshot::skipped`] lists it.
    ///
    /// Each file is read by `read_rules_file`,
    /// so a named pipe at a rules name ends this read at once naming the path,
    /// and an optional
    ///  entry that holds one fails like any other unreadable file instead of counting as absent.
    ///
    /// # Errors
    ///
    /// Fails on the first entry that cannot be read,
    /// and when the present files are in different formats.
    pub(crate) fn read(entries: &[RulesEntry]) -> Result<Self> {
        let mut read_entries: Vec<ReadEntry> = Vec::with_capacity(entries.len());
        for entry in entries {
            match read_rules_file(&entry.path) {
                Ok(bytes) => {
                    let text: String = match String::from_utf8(bytes.clone()) {
                        Ok(text) => text,
                        Err(_) => {
                            return Err(anyhow!(
                                "read rules {}: stream did not contain valid UTF-8",
                                entry.path.display(),
                            ));
                        }
                    };
                    read_entries.push(ReadEntry {
                        path: entry.path.clone(),
                        present: Some(Present { bytes, text }),
                    });
                }
                Err(error) if entry.optional && error.kind() == std::io::ErrorKind::NotFound => {
                    read_entries.push(ReadEntry { path: entry.path.clone(), present: None });
                }
                Err(error) => return Err(anyhow!("read rules {}: {}", entry.path.display(), error)),
            }
        }
        let text: String = join(&present_files(&read_entries))?;
        return Ok(Self { entries: read_entries, text });
    }

    /// The joined text the scanner compiles and the cache key covers.
    pub(crate) fn text(&self) -> &str {
        return &self.text;
    }

    /// Whether no file was present,
    /// so there is no rules text at all.
    pub(crate) fn is_empty(&self) -> bool {
        return self.entries.iter().all(is_absent);
    }

    /// The optional entries that were absent,
    /// in list order.
    pub(crate) fn skipped(&self) -> Vec<PathBuf> {
        let mut skipped: Vec<PathBuf> = Vec::new();
        for entry in &self.entries {
            if is_absent(entry) {
                skipped.push(entry.path.clone());
            }
        }
        return skipped;
    }

    /// Fails when no file was present,
    /// naming every listed path.
    ///
    /// Every entry being optional and absent leaves nothing to scan with,
    /// and scanning with no rules would pass
    /// every file,
    /// so the caller that has no other rule source refuses it.
    ///
    /// # Errors
    ///
    /// Fails with `rules <paths>: no rules loaded ...` when [`RulesSnapshot::is_empty`] holds.
    pub(crate) fn require_present(&self) -> Result<()> {
        if !self.is_empty() {
            return Ok(());
        }
        let listed: Vec<String> =
            self.entries.iter().map(path_text_of).collect();
        return Err(anyhow!(
            "rules {}: no rules loaded, because every entry is optional and none of those files exists",
            listed.join(", "),
        ));
    }

    /// The present files' paths as one label for an error:
    /// one path,
    /// or the paths separated by `, `.
    pub(crate) fn label(&self) -> String {
        let mut names: Vec<String> = Vec::new();
        for entry in &self.entries {
            if !is_absent(entry) {
                names.push(path_text_of(entry));
            }
        }
        return names.join(", ");
    }

    /// Whether every entry still holds what was read:
    /// same bytes for a present file,
    /// still absent for a skipped one.
    ///
    /// A cache artifact is published only while this holds,
    /// so a file edited,
    /// removed,
    /// or created while the rules
    /// compiled never publishes an artifact for text that no longer exists.
    pub(crate) fn still_current(&self) -> bool {
        return self.entries.iter().all(holds_what_was_read);
    }

    /// Compiles the joined text,
    /// attributing a failure to the file that caused it.
    ///
    /// # Errors
    ///
    /// Fails with `rules <path>: <reason>`,
    /// the reason carrying no rule text.
    pub(crate) fn compile(&self) -> Result<RuntimeRules> {
        match RuntimeRules::compile(&self.text) {
            Ok(compiled) => return Ok(compiled),
            Err(joined_error) => {
                let attributed: Option<anyhow::Error> = self.attribute_failure();
                match attributed {
                    Some(error) => return Err(error),
                    None => return Err(anyhow!("rules {}: {}", self.label(), joined_error)),
                }
            }
        }
    }

    /// Re-diagnoses a failed join file by file.
    ///
    /// Returns the first file that fails alone,
    /// or the file that repeats a section name another file declared,
    /// or `None` when fewer than two files are present,
    /// or every file compiles alone and no name repeats,
    /// leaving the caller to report the joined failure.
    fn attribute_failure(&self) -> Option<anyhow::Error> {
        let present: Vec<(&Path, &str)> = present_files(&self.entries);
        if present.len() < 2 {
            return None;
        }
        let mut declared_by: HashMap<String, &Path> = HashMap::new();
        for (path, text) in &present {
            if source_format(text) == SourceFormat::Empty {
                continue;
            }
            if let Err(error) = RuntimeRules::compile(text) {
                return Some(anyhow!("rules {}: {}", path.display(), error));
            }
            let Ok(inputs) = parse_runtime_rules(strip_source_bom(text)) else {
                continue;
            };
            for input in inputs {
                let Some(name) = input.name else {
                    continue;
                };
                if let Some(first) = declared_by.get(&name) {
                    return Some(anyhow!(
                        "rules {}: section name '{}' is already declared by {}",
                        path.display(),
                        name,
                        first.display(),
                    ));
                }
                declared_by.insert(name, path);
            }
        }
        return None;
    }
}

/// The present files as `(path, text)` pairs in list order.
fn present_files(entries: &[ReadEntry]) -> Vec<(&Path, &str)> {
    return entries.iter().filter_map(present_pair).collect();
}

/// Joins the present files in list order.
///
/// One file joins to its exact text,
/// so a single-file list keeps the cache key it had before lists existed.
fn join(present: &[(&Path, &str)]) -> Result<String> {
    if let [(_, only)] = present {
        return Ok((*only).to_string());
    }
    let mut first_format: Option<(&Path, SourceFormat)> = None;
    for (path, text) in present {
        let format: SourceFormat = source_format(text);
        if format == SourceFormat::Empty {
            continue;
        }
        match first_format {
            None => first_format = Some((path, format)),
            Some((first_path, expected)) if expected != format => {
                return Err(anyhow!(
                    "rules {}: this file is in the {} format but {} is in the {} format; every file of a rules \
                     list must use one format, because joining them would reinterpret the later file's lines",
                    path.display(),
                    format_name(format),
                    first_path.display(),
                    format_name(expected),
                ));
            }
            Some(_) => {}
        }
    }
    let mut joined: String = String::new();
    for (_, text) in present {
        joined.push_str(strip_source_bom(text));
        if !joined.ends_with('\n') {
            joined.push('\n');
        }
    }
    return Ok(joined);
}

/// The words for a format in an error.
fn format_name(format: SourceFormat) -> &'static str {
    match format {
        SourceFormat::Tail => return "tail ('==> name <==' sections)",
        SourceFormat::Legacy => return "legacy (one rule per line)",
        SourceFormat::Empty => return "empty",
    }
}

/// Registers the snapshot,
/// join,
/// and attribution tests (sidecar,
/// lint-exempt).
#[cfg(test)]
#[path = "rules_snapshot_tests.rs"]
mod tests;
