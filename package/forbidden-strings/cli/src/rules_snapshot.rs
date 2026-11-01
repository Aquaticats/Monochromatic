//! Reads the files a rules list names once and joins them into the one text the scanner compiles.
//!
//! The joined text is the only input of the compiled-rules cache key, because the compiled rules are a pure function
//! of it: two lists whose present files join to the same text compile to the same rules and may share an artifact.
//! The join is therefore the place that keeps the joined text equal in meaning to the files read separately:
//! each file is BOM-stripped, every file ends in a newline before the next begins, and files in different formats
//! (tail sections against legacy lines) are refused, because the later file's lines would otherwise become the body
//! of the earlier file's last section.
//! A compile failure of the joined text is re-diagnosed file by file, so the error names the file and a line
//! number local to it.

use crate::rule::frx::{parse_runtime_rules, source_format, strip_source_bom, SourceFormat};
use crate::rules_list::RulesEntry;
use crate::runtime_matcher::RuntimeRules;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A file that was present when read.
struct Present {
    /// The file's exact bytes, compared again before an artifact is published.
    bytes: Vec<u8>,
    /// The same bytes decoded as UTF-8.
    text: String,
}

/// What one entry held when it was read.
struct ReadEntry {
    /// The file the entry names.
    path: PathBuf,
    /// The file's content, or `None` when an optional entry was absent.
    present: Option<Present>,
}

/// The files of one rules list, read once, with their joined text.
pub(crate) struct RulesSnapshot {
    /// Every entry in list order, including the absent optional ones.
    entries: Vec<ReadEntry>,
    /// The text the scanner compiles: the present files joined in list order.
    text: String,
}

/// Reads and joins a rules list.
impl RulesSnapshot {
    /// Reads every entry in list order.
    ///
    /// A required entry that cannot be read, and any entry that is present but unreadable or not UTF-8, fails the
    /// read with an error naming that path.
    /// An optional entry whose file is absent is skipped; [`RulesSnapshot::skipped`] lists it.
    ///
    /// # Errors
    ///
    /// Fails on the first entry that cannot be read, and when the present files are in different formats.
    pub(crate) fn read(entries: &[RulesEntry]) -> Result<Self> {
        let mut read_entries: Vec<ReadEntry> = Vec::with_capacity(entries.len());
        for entry in entries {
            match std::fs::read(&entry.path) {
                Ok(bytes) => {
                    let text: String = String::from_utf8(bytes.clone()).map_err(|_| {
                        return anyhow!(
                            "read rules {}: stream did not contain valid UTF-8",
                            entry.path.display(),
                        );
                    })?;
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

    /// Whether no file was present, so there is no rules text at all.
    pub(crate) fn is_empty(&self) -> bool {
        return self.entries.iter().all(|entry| return entry.present.is_none());
    }

    /// The optional entries that were absent, in list order.
    pub(crate) fn skipped(&self) -> Vec<PathBuf> {
        return self
            .entries
            .iter()
            .filter(|entry| return entry.present.is_none())
            .map(|entry| return entry.path.clone())
            .collect();
    }

    /// Fails when no file was present, naming every listed path.
    ///
    /// Every entry being optional and absent leaves nothing to scan with, and scanning with no rules would pass
    /// every file, so the caller that has no other rule source refuses it.
    ///
    /// # Errors
    ///
    /// Fails with `rules <paths>: no rules loaded ...` when [`RulesSnapshot::is_empty`] holds.
    pub(crate) fn require_present(&self) -> Result<()> {
        if !self.is_empty() {
            return Ok(());
        }
        let listed: Vec<String> =
            self.entries.iter().map(|entry| return entry.path.display().to_string()).collect();
        return Err(anyhow!(
            "rules {}: no rules loaded, because every entry is optional and none of those files exists",
            listed.join(", "),
        ));
    }

    /// The present files' paths as one label for an error: one path, or the paths separated by `, `.
    pub(crate) fn label(&self) -> String {
        let names: Vec<String> = self
            .entries
            .iter()
            .filter(|entry| return entry.present.is_some())
            .map(|entry| return entry.path.display().to_string())
            .collect();
        return names.join(", ");
    }

    /// Whether every entry still holds what was read: same bytes for a present file, still absent for a skipped one.
    ///
    /// A cache artifact is published only while this holds, so a file edited, removed, or created while the rules
    /// compiled never publishes an artifact for text that no longer exists.
    pub(crate) fn still_current(&self) -> bool {
        return self.entries.iter().all(|entry| match (&entry.present, std::fs::read(&entry.path)) {
            (Some(expected), Ok(now)) => return expected.bytes == now,
            (None, Err(error)) => return error.kind() == std::io::ErrorKind::NotFound,
            _ => return false,
        });
    }

    /// Compiles the joined text, attributing a failure to the file that caused it.
    ///
    /// # Errors
    ///
    /// Fails with `rules <path>: <reason>`, the reason carrying no rule text.
    pub(crate) fn compile(&self) -> Result<RuntimeRules> {
        match RuntimeRules::compile(&self.text) {
            Ok(compiled) => return Ok(compiled),
            Err(joined_error) => {
                let attributed: Option<anyhow::Error> = self.attribute_failure();
                return Err(attributed.unwrap_or_else(|| {
                    return anyhow!("rules {}: {}", self.label(), joined_error);
                }));
            }
        }
    }

    /// Re-diagnoses a failed join file by file.
    ///
    /// Returns the first file that fails alone, or the file that repeats a section name another file declared,
    /// or `None` when fewer than two files are present, or every file compiles alone and no name repeats,
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
    return entries
        .iter()
        .filter_map(|entry| {
            return entry.present.as_ref().map(|file| return (entry.path.as_path(), file.text.as_str()));
        })
        .collect();
}

/// Joins the present files in list order.
///
/// One file joins to its exact text, so a single-file list keeps the cache key it had before lists existed.
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

/// Registers the snapshot, join, and attribution tests (sidecar, lint-exempt).
#[cfg(test)]
#[path = "rules_snapshot_tests.rs"]
mod tests;
