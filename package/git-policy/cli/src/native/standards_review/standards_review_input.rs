//! What:
//!  The pure part of the standards-review shadow run:
//! which landed commits get an entry,
//! and the capped model input
//!  built from the commit's patch and the excerpts of its changed code files.
//! Why:
//!  This is the root `mise.toml` task `shadow-review` after its Git reads:
//! the no-entry,
//! no-code-files and review
//!  cases,
//! the 2000-line patch cap,
//! the eight-file and 4000-line excerpt caps,
//! and the 24000-unit input cap.
//! Taking
//!  the patch and the file contents as plain values lets the caps be tested without Git,
//! a network or a clock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = classify(names); const input = buildReviewInput({ commit });
//! ```

use crate::standards_review_patch::{NewSpan, is_line_feed_byte, new_spans};
use crate::standards_review_text::{utf16_length, utf16_prefix};
use std::collections::BTreeMap;

/// What:
///  One changed path exactly as Git reported it:
/// its bytes,
/// never text decoded from them.
/// Why:
///  A path on Linux is a byte string,
/// and a name that does not decode must not be read as one that does:
/// Git would then
///  answer with another file's object,
/// whose content would leave the machine as this file's excerpt.
/// Text appears only
///  where a person or the model reads a label,
/// through its `Display`,
/// never where an object is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChangedPath { constructor(readonly bytes: Uint8Array) {} toString() { return decodeLossily(this.bytes) } }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChangedPath {
    /// The path's bytes,
    /// as `git diff-tree -z` prints them.
    bytes: Vec<u8>,
}

impl ChangedPath {
    /// A path from the bytes Git printed for it.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        return Self { bytes };
    }

    /// The bytes Git printed for it,
    /// which name its object and nothing else.
    pub fn as_bytes(&self) -> &[u8] {
        return self.bytes.as_slice();
    }

    /// Whether the path ends with `suffix`,
    /// as the code-name test asks.
    pub fn ends_with(&self, suffix: &str) -> bool {
        return self.bytes.ends_with(suffix.as_bytes());
    }

    /// What:
    ///  Whether the path names exactly `other`,
    /// as the tracked log's own name is compared.
    /// Why:
    ///   `==` on bytes,
    /// never on text,
    /// so two names that decode alike stay apart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// equals(other: ChangedPath) { return bytesEqual(this.bytes, other.bytes) }
    /// ```
    pub fn is(&self, other: &str) -> bool {
        return self.bytes.as_slice() == other.as_bytes();
    }
}

/// What:
///  The path as text for a label,
/// with a name that is not text replaced.
/// Why:
///   A label is read by a person;
/// the object it labels was already read by the path's own bytes,
/// so the replacement here
///  cannot reach another file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder().decode(this.bytes) // lossy
/// ```
impl std::fmt::Display for ChangedPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(String::from_utf8_lossy(self.bytes.as_slice()).as_ref());
    }
}

/// The tracked log's repository-relative path:
/// the repository's switch for the review,
/// and the file a commit changes
/// without getting an entry.
/// It is frozen history that the service never writes.
pub const TRACKED_LOG_RELATIVE: &str = "doc/audit/standards-review-gate-shadow.md";

/// The live log's path under the common Git directory:
/// untracked,
/// so a landing never makes a checkout show a change.
pub const SHARED_LOG_RELATIVE: &str = "monochromatic-git-policy-cli/standards-review-shadow.md";

/// What:
///  The live log of a repository:
/// the one file every worktree's service instance appends its entries to.
/// Why:
///   The log sits in the common Git directory,
/// which `git status` never lists and no worktree copy reaches,
/// and one
///   file serves all worktrees.
/// The service already knows the directory from the work item,
/// so no Git start finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'monochromatic-git-policy-cli/standards-review-shadow.md')
/// ```
pub fn shared_log_path(common_dir: &std::path::Path) -> std::path::PathBuf {
    return common_dir.join(SHARED_LOG_RELATIVE);
}

/// What:
///  Whether a worktree has the tracked shadow log:
/// the review exists only where the repository keeps one.
/// Why:
///   The tracked file's presence is the repository's own switch;
/// no configuration names a program,
/// the wrapper leaves a
///   review item only where it is true,
/// and the service never writes that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(repository, trackedLogRelative))
/// ```
pub fn has_shadow_log(repository: &std::path::Path) -> bool {
    return repository.join(TRACKED_LOG_RELATIVE).is_file();
}

/// Patch lines kept for the model.
pub const CAP_LINES: usize = 2000;

/// Code files that get an excerpt.
pub const MAX_EXCERPT_FILES: usize = 8;

/// Lines of context kept around the changed span of an excerpted file.
pub const EXCERPT_CONTEXT: i64 = 30;

/// Excerpt lines after which no further file gets an excerpt.
pub const EXCERPT_LINE_CAP: i64 = 4000;

/// Lines of the head of a file that has no changed line to anchor an excerpt on:
/// the context on both sides and one.
pub const HEAD_WINDOW: i64 = 2 * EXCERPT_CONTEXT + 1;

/// UTF-16 units of model input.
pub const INPUT_UNIT_CAP: usize = 24000;

/// Name endings that count as code.
pub const CODE_SUFFIXES: [&str; 6] = [".ts", ".tsx", ".rs", ".js", ".mjs", ".cjs"];

/// What:
///  What a landed commit's changed names say before anything is read from Git.
/// Why:
///   A commit with no names (a root commit,
/// a merge) or only the log gets no entry,
/// and a commit with no code gets a
///   three-line skip entry;
/// neither needs the patch or any file,
/// so neither costs a Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { kind: 'silent' } | { kind: 'no-code' } | { kind: 'review'; codeFiles: ChangedPath[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Plan {
    /// No entry at all.
    Silent,
    /// The skip entry.
    NoCode,
    /// A review of these code files,
    /// in name order.
    Review(Vec<ChangedPath>),
}

/// What:
///  The plan for the names `git diff-tree --name-only -r -z` prints for the landed commit.
/// Why:
///   It is the script's first three branches,
/// in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (names.every((name) => name === logRelative)) return silent; if (codeFiles.length === 0) return noCode;
/// ```
pub fn plan_for(names: &[ChangedPath]) -> Plan {
    let mut only_the_log: bool = true;
    for name in names {
        if !name.is(TRACKED_LOG_RELATIVE) {
            only_the_log = false;
            break;
        }
    }
    if only_the_log {
        return Plan::Silent;
    }
    let mut code_files: Vec<ChangedPath> = Vec::new();
    for name in names {
        let mut is_code: bool = false;
        for suffix in CODE_SUFFIXES {
            if name.ends_with(suffix) {
                is_code = true;
                break;
            }
        }
        if is_code {
            code_files.push(name.clone());
        }
    }
    if code_files.is_empty() {
        return Plan::NoCode;
    }
    return Plan::Review(code_files);
}

/// What:
///  Everything the review reads about one landed commit:
/// the patch,
/// and the new-side text of each code file that
///  will be excerpted.
/// A path missing from `contents` has no new-side file (it was deleted).
/// Why:
///   One value is all the pure builder needs,
/// whichever way the reads were made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedCommit = { sha: string; subject: string; patch: Uint8Array; contents: Map<ChangedPath, string> };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandedCommit {
    /// Full commit name.
    pub sha: String,
    /// Subject line,
    /// trimmed.
    pub subject: String,
    /// `git show --format= --patch` bytes,
    /// as Git wrote them.
    pub patch: Vec<u8>,
    /// New-side text by the path's own bytes.
    pub contents: BTreeMap<ChangedPath, String>,
}

/// What:
///  The sizes the entry reports and the text the model receives.
/// Why:
///   `model_input` is cut at the unit cap,
/// while `model_input_units` is the length the script prints:
/// the cut length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewInput = { codeFileCount: number; diffLineCount: number; truncated: boolean; modelInput: string; modelInputUnits: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewInput {
    /// Code files in the commit.
    pub code_file_count: usize,
    /// Lines of the whole patch (a final newline counts as one more,
    /// empty,
    /// line).
    pub diff_line_count: usize,
    /// Whether the patch is over `CAP_LINES`.
    pub truncated: bool,
    /// The text sent to the model.
    pub model_input: String,
    /// The reported length of `model_input` in UTF-16 units.
    pub model_input_units: usize,
}

/// What:
///  The excerpt of one file:
/// its header and the text of the window around the changed span.
///  The window runs from `EXCERPT_CONTEXT` lines before the first changed new-side line to `EXCERPT_CONTEXT` lines
///  after the last,
/// cut from the new-side file.
/// A file with no changed line to anchor on (a binary file,
/// a mode change,
///  a pure rename) gets the head of the file instead.
/// Why:
///   Returns the window's line count too,
/// which the 4000-line cap adds up.
/// The script cut the window from old-side
///   numbers,
/// which sit in the wrong place after an earlier insertion,
/// and gave a file without hunks a nonsensical
///   negative window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excerptParts.push(`// candidate excerpt of ${name}, lines ${from} to ${to} of ${fileLines.length}`);
/// ```
fn excerpt(name: &ChangedPath, content: &str, spans: &[NewSpan]) -> (Vec<String>, i64) {
    let file_lines: Vec<&str> = content.split('\n').collect();
    let total: i64 = i64::try_from(file_lines.len()).unwrap_or(i64::MAX);
    let mut first_changed: Option<i64> = None;
    let mut last_changed: Option<i64> = None;
    for span in spans {
        let first: i64 = i64::try_from(span.first).unwrap_or(i64::MAX);
        let last: i64 = i64::try_from(span.last).unwrap_or(i64::MAX);
        first_changed = match first_changed {
            Some(known) => Some(known.min(first)),
            None => Some(first),
        };
        last_changed = match last_changed {
            Some(known) => Some(known.max(last)),
            None => Some(last),
        };
    }
    let (from, to) = match (first_changed, last_changed) {
        (Some(first), Some(last)) => (1.max(first.saturating_sub(EXCERPT_CONTEXT)), total.min(last.saturating_add(EXCERPT_CONTEXT))),
        _ => (1, total.min(HEAD_WINDOW)),
    };
    let header: String = format!("// candidate excerpt of {name}, lines {from} to {to} of {total}");
    let begin: usize = usize::try_from(from - 1).unwrap_or(0);
    let end: usize = usize::try_from(to).unwrap_or(0);
    let body: String = if begin < end { file_lines[begin..end].join("\n") } else { String::new() };
    return (vec![header, body], to - from + 1);
}

/// What:
///  Build the capped model input for a reviewed commit.
/// Why:
///   The input is the first `CAP_LINES` patch lines,
/// a blank line,
/// then each excerpt separated by blank lines,
/// cut to
///   `INPUT_UNIT_CAP` UTF-16 units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lunaInput = (lines.slice(0, capLines).join('\n') + '\n\n' + excerptParts.join('\n\n')).slice(0, 24000);
/// ```
pub fn build_input(commit: &LandedCommit, code_files: &[ChangedPath]) -> ReviewInput {
    let lines: Vec<&[u8]> = commit.patch.split(is_line_feed_byte).collect();
    let kept: &[&[u8]] = &lines[..lines.len().min(CAP_LINES)];
    let mut parts: Vec<String> = Vec::new();
    let mut excerpt_lines: i64 = 0;
    for name in code_files.iter().take(MAX_EXCERPT_FILES) {
        if excerpt_lines >= EXCERPT_LINE_CAP {
            break;
        }
        let Some(content) = commit.contents.get(name) else {
            continue;
        };
        let spans: Vec<NewSpan> = new_spans(lines.as_slice(), name.as_bytes());
        let (pieces, covered) = excerpt(name, content, spans.as_slice());
        parts.extend(pieces);
        excerpt_lines += covered;
    }
    // The patch enters the model input as text; the section match above used its bytes.
    let mut patch_bytes: Vec<u8> = Vec::new();
    for (index, line) in kept.iter().enumerate() {
        if index > 0 {
            patch_bytes.push(b'\n');
        }
        patch_bytes.extend_from_slice(line);
    }
    let patch_text: String = String::from_utf8_lossy(patch_bytes.as_slice()).into_owned();
    let full: String = format!("{patch_text}\n\n{}", parts.join("\n\n"));
    let units: usize = utf16_length(full.as_str()).min(INPUT_UNIT_CAP);
    return ReviewInput {
        code_file_count: code_files.len(),
        diff_line_count: lines.len(),
        truncated: lines.len() > CAP_LINES,
        model_input: utf16_prefix(full.as_str(), INPUT_UNIT_CAP),
        model_input_units: units,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_input_tests.rs"]
mod tests;
