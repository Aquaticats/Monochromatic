//! What:
//!  Which rules file the scanner loads,
//!  and which candidates it is given.
//! Why:
//!  The scanner library filters nothing;
//!  the standalone scanner and the TypeScript
//!      policy applied these choices around it.
//!  They are restated here once,
//!  as pure
//!      functions:
//!  the rules-file precedence (`FORBIDDEN_STRINGS_RULES`,
//!  else
//!      `forbidden-strings.local.txt` in the repository root) and the paths that hold
//!      the rules themselves,
//!  which would otherwise match their own content.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const rules = rulesSource(process.env.FORBIDDEN_STRINGS_RULES, repositoryRoot);
//! // const eligible = candidates.filter(candidate => isScannable(candidate, rulesCandidatePath(rules.path, repositoryRoot)));
//! ```

/// Import the candidate being judged and its change kinds.
use super::candidate_record::CandidateChange;
use super::candidate_version::Candidate;
/// Import the configured rules files.
use super::config_schema::RulesFileEntry;
/// Import the rules selection the adapter loads.
use super::scanner_adapter::RulesSource;
/// Import the scanner's rules-list entry and the one parser of the variable's list grammar.
use forbidden_strings::{RulesEntry, RulesListError, parse_rules_list};
/// `OsStr` is borrowed operating-system text of raw OS bytes (sibling `str` must be UTF-8).
use std::ffi::OsStr;
/// What:
///  `Component` is one piece of a path:
///  the root,
///  `.`,
///  `..`,
///  or a name.
///       `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
/// Why:
///   A configured rules path is resolved name by name,
///  without touching the filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolve, relative } from 'node:path';
/// ```
use std::path::{Component, Path, PathBuf};

/// What:
///  The environment variable naming the runtime rules file.
///       `&str` is a borrowed string compiled into the program.
/// Why:
///   It is the standalone scanner's own variable,
///  so one setting serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RULES_VARIABLE = 'FORBIDDEN_STRINGS_RULES';
/// ```
pub const RULES_VARIABLE: &str = "FORBIDDEN_STRINGS_RULES";

/// The rules file used when the variable is unset,
///  relative to the repository root.
pub const DEFAULT_RULES_FILE: &str = "forbidden-strings.local.txt";

/// What:
///  Repository paths never given to the scanner because they hold rule sources.
///       `&[&[u8]]` is a borrowed list of borrowed byte strings.
/// Why:
///   These files contain the patterns themselves and would match their own rules.
///       The list is the TypeScript policy's,
///  unchanged;
///  pathnames are compared as
///       bytes because candidate pathnames are bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCANNER_SELF_MATCH_PATHS = new Set(['package/forbidden-strings/cli/data/builtin-rules.txt', ...]);
/// ```
pub const SCANNER_SELF_MATCH_PATHS: &[&[u8]] = &[
    b"package/forbidden-strings/cli/data/betterleaks-default-config.toml",
    b"package/forbidden-strings/cli/data/builtin-rules.txt",
    b"package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts",
];

/// What:
///  Select the rules file:
///  the configured `rulesFile`,
///  else the variable's value,
///  else
///       the default file,
///  each relative to the repository root.
///  `Option<&str>` is "the
///       configured name or nothing";
///  `Option<&OsStr>` is "the variable's value or nothing".
/// Why:
///   The user decided on 2026-10-05 that configuration names the file first and the
///       variable second.
///  The spawned scanner ran with the repository root as its working
///       directory,
///  so a relative setting meant "relative to the root".
///  In-process there
///       is no such directory change,
///  so the root is joined explicitly;
///  `.join(..)` keeps
///       an absolute variable value as it is,
///  and a configured name is relative by
///       construction (`config_rules_file.rs`).
///  A configured name or a set variable makes
///       a missing file an error;
///  only the default file may be missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rulesSource(rulesFile: string | undefined, variable: string | undefined, repositoryRoot: string): RulesSource;
/// ```
pub fn rules_source(
    rules_files: Option<&[RulesFileEntry]>,
    configured: Option<&OsStr>,
    repository_root: &Path,
) -> Result<RulesSource, RulesListError> {
    // `if let Some(files) = rules_files` runs only when the configuration names files.
    if let Some(files) = rules_files {
        let mut entries: Vec<RulesEntry> = Vec::with_capacity(files.len());
        for file in files {
            entries.push(RulesEntry {
                path: repository_root.join(file.path.as_str()),
                optional: file.optional,
            });
        }
        return Ok(RulesSource { entries, explicit: true });
    }
    // `match` on the optional value: `Some(value)` is "present", `None` is "absent".
    match configured {
        Some(value) => {
            // The variable uses the scanner's rules-list grammar (comma-separated, a trailing `?` marks an
            // optional entry), so one parser serves both tools. A value that is not UTF-8 cannot be split,
            // so it stays one required path, as it was before lists existed.
            let Some(text) = value.to_str() else {
                return Ok(RulesSource {
                    entries: vec![RulesEntry { path: repository_root.join(value), optional: false }],
                    explicit: true,
                });
            };
            let mut entries: Vec<RulesEntry> = parse_rules_list(text)?;
            for entry in &mut entries {
                // `.join(..)` keeps an absolute entry as it is.
                entry.path = repository_root.join(entry.path.as_path());
            }
            return Ok(RulesSource { entries, explicit: true });
        }
        None => {
            return Ok(RulesSource {
                entries: vec![RulesEntry {
                    path: repository_root.join(DEFAULT_RULES_FILE),
                    optional: false,
                }],
                explicit: false,
            });
        }
    }
}

/// What:
///  The pathnames the rules files would carry as candidates,
///  for each listed file that lies inside the repository.
///       `Vec<Vec<u8>>` is an owned list of owned pathname byte strings.
/// Why:
///   Any tracked rules file would match its own patterns,
///  so every one is left out of the scan,
///       not only the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rulesCandidatePaths = (rules: RulesSource, root: string) => rules.entries.flatMap(e => rulesCandidatePath(e.path, root) ?? []);
/// ```
pub fn rules_candidate_paths(rules: &RulesSource, repository_root: &Path) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = Vec::with_capacity(rules.entries.len());
    for entry in &rules.entries {
        if let Some(found) = rules_candidate_path(entry.path.as_path(), repository_root) {
            paths.push(found);
        }
    }
    return paths;
}

/// What:
///  Resolve `.` and `..` in a path by its names alone.
///       `PathBuf::new()` is an empty owned path;
///  `.push(..)` appends one piece and
///       `.pop()` removes the last one.
/// Why:
///   The rules file need not exist,
///  so the filesystem cannot be asked;
///  this is the
///       same textual resolution the TypeScript policy applied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const normalized = path.resolve(configured);
/// ```
fn lexically_normalized(path: &Path) -> PathBuf {
    let mut normalized: PathBuf = PathBuf::new();
    // `.components()` yields the path's pieces from left to right.
    for component in path.components() {
        // What: `match` on the piece's kind; `Component::Normal(_)` is an ordinary name.
        // Why:  `.` adds nothing and `..` removes the name before it; everything else is kept.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (part === '.') continue; if (part === '..') parts.pop(); else parts.push(part);
        // ```
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                // `.as_os_str()` lends the piece's own text.
                normalized.push(component.as_os_str());
            }
        }
    }
    return normalized;
}

/// What:
///  Turn a repository-relative native path into Git's pathname bytes:
///  its names joined by `/`.
///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
/// Why:
///   Git stores `/` between names on every system.
///  Joining the names here,
///  instead
///       of copying the path's own text,
///  gives the same bytes whatever separator the
///       operating system uses,
///  with one implementation for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitPath = relativePath.split(path.sep).join('/');
/// ```
fn git_path_bytes(relative: &Path) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for component in relative.components() {
        if !bytes.is_empty() {
            bytes.push(b'/');
        }
        // `.as_encoded_bytes()` views one name as bytes: the raw bytes on Unix, UTF-8 elsewhere.
        bytes.extend_from_slice(component.as_os_str().as_encoded_bytes());
    }
    return bytes;
}

/// What:
///  The rules file's pathname as a candidate would carry it,
///  when it lies inside the repository.
///       `Option<Vec<u8>>` is "pathname bytes or nothing".
/// Why:
///   A rules file tracked in the repository would match its own patterns,
///  so its
///       candidate is not scanned.
///  A rules file outside the repository can never be a
///       candidate,
///  which is "nothing".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rulesCandidatePath(rulesPath: string, repositoryRoot: string): string | undefined;
/// ```
pub fn rules_candidate_path(rules_path: &Path, repository_root: &Path) -> Option<Vec<u8>> {
    let normalized: PathBuf = lexically_normalized(rules_path);
    // What: `.strip_prefix(root)` is `Ok(rest)` when the path starts with the root's
    //       names; `.ok()?` turns a failure into `None` and returns it.
    // Why:  Only a path below the root has a repository-relative pathname.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relative = path.relative(root, normalized); if (relative.startsWith('..')) return undefined;
    // ```
    let relative: &Path = normalized.strip_prefix(repository_root).ok()?;
    if relative.as_os_str().is_empty() {
        // `None` is the "absent" variant: the root itself is not a file.
        return None;
    }
    // `Some(...)` is the "present" variant.
    return Some(git_path_bytes(relative));
}

/// What:
///  Whether a candidate is given to the scanner.
///       `Option<&[u8]>` is "the rules file's pathname bytes or nothing".
/// Why:
///   A deleted path has no content and leaves the tree;
///  the rule-source files and
///       the rules file itself would match their own patterns.
///  Every other candidate
///       is scanned,
///  whatever its mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isScannable(candidate: Candidate, rulesPath?: string): boolean;
/// ```
pub fn is_scannable(candidate: &Candidate, rules_paths: &[Vec<u8>]) -> bool {
    if candidate.change == CandidateChange::Deleted {
        return false;
    }
    // `.iter().any(..)` asks whether any rules file's pathname equals the candidate's.
    if rules_paths.iter().any(|rules_path| return rules_path.as_slice() == candidate.path.as_slice()) {
        return false;
    }
    // `.contains(&..)` asks whether the list holds this exact byte string.
    return !SCANNER_SELF_MATCH_PATHS.contains(&candidate.path.as_slice());
}

/// Selection controls stay out of the release executable.
#[cfg(test)]
#[path = "scanner_selection_tests.rs"]
mod tests;
