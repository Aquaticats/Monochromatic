//! What:
//!  The pure part of the standards-review shadow run: which landed commits get an entry, and the capped model input
//!  built from the commit's patch and the excerpts of its changed code files.
//! Why:
//!  This is the root `mise.toml` task `shadow-review` after its Git reads: the no-entry, no-code-files and review
//!  cases, the 2000-line patch cap, the eight-file and 4000-line excerpt caps, and the 24000-unit input cap. Taking
//!  the patch and the file contents as plain values lets the caps be tested without Git, a network or a clock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = classify(names); const input = buildReviewInput({ commit });
//! ```

use crate::standards_review_patch::{NewSpan, new_spans};
use crate::standards_review_text::{utf16_length, utf16_prefix};
use std::collections::BTreeMap;

/// The tracked log's repository-relative path: the repository's switch for the review, and the file a commit changes
/// without getting an entry. It is frozen history that the service never writes.
pub const TRACKED_LOG_RELATIVE: &str = "doc/audit/standards-review-gate-shadow.md";

/// The live log's path under the common Git directory: untracked, so a landing never makes a checkout show a change.
pub const SHARED_LOG_RELATIVE: &str = "cli-git/standards-review-shadow.md";

/// What:
///  The live log of a repository: the one file every worktree's service instance appends its entries to.
/// Why:
///   The log sits in the common Git directory, which `git status` never lists and no worktree copy reaches, and one
///   file serves all worktrees. The service already knows the directory from the work item, so no Git start finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git/standards-review-shadow.md')
/// ```
pub fn shared_log_path(common_dir: &std::path::Path) -> std::path::PathBuf {
    return common_dir.join(SHARED_LOG_RELATIVE);
}

/// What:
///  Whether a worktree has the tracked shadow log: the review exists only where the repository keeps one.
/// Why:
///   The tracked file's presence is the repository's own switch; no configuration names a program, the wrapper leaves a
///   review item only where it is true, and the service never writes that file.
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

/// Lines of the head of a file that has no changed line to anchor an excerpt on: the context on both sides and one.
pub const HEAD_WINDOW: i64 = 2 * EXCERPT_CONTEXT + 1;

/// UTF-16 units of model input.
pub const INPUT_UNIT_CAP: usize = 24000;

/// Name endings that count as code.
pub const CODE_SUFFIXES: [&str; 6] = [".ts", ".tsx", ".rs", ".js", ".mjs", ".cjs"];

/// What:
///  What a landed commit's changed names say before anything is read from Git.
/// Why:
///   A commit with no names (a root commit, a merge) or only the log gets no entry, and a commit with no code gets a
///   three-line skip entry; neither needs the patch or any file, so neither costs a Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { kind: 'silent' } | { kind: 'no-code' } | { kind: 'review'; codeFiles: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Plan {
    /// No entry at all.
    Silent,
    /// The skip entry.
    NoCode,
    /// A review of these code files, in name order.
    Review(Vec<String>),
}

/// What:
///  The plan for the names `git diff-tree --name-only -r` prints for the landed commit.
/// Why:
///   It is the script's first three branches, in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (names.every((name) => name === logRelative)) return silent; if (codeFiles.length === 0) return noCode;
/// ```
pub fn plan_for(names: &[String]) -> Plan {
    if names.iter().all(|name| return name == TRACKED_LOG_RELATIVE) {
        return Plan::Silent;
    }
    let code_files: Vec<String> = names
        .iter()
        .filter(|name| return CODE_SUFFIXES.iter().any(|suffix| return name.ends_with(suffix)))
        .cloned()
        .collect();
    if code_files.is_empty() {
        return Plan::NoCode;
    }
    return Plan::Review(code_files);
}

/// What:
///  Everything the review reads about one landed commit: the patch, and the new-side text of each code file that
///  will be excerpted. A path missing from `contents` has no new-side file (it was deleted).
/// Why:
///   One value is all the pure builder needs, whichever way the reads were made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedCommit = { sha: string; subject: string; patch: string; contents: Map<string, string> };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandedCommit {
    /// Full commit name.
    pub sha: String,
    /// Subject line, trimmed.
    pub subject: String,
    /// `git show --format= --patch` text.
    pub patch: String,
    /// New-side text by path.
    pub contents: BTreeMap<String, String>,
}

/// What:
///  The sizes the entry reports and the text the model receives.
/// Why:
///   `model_input` is cut at the unit cap, while `model_input_units` is the length the script prints: the cut length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewInput = { codeFileCount: number; diffLineCount: number; truncated: boolean; modelInput: string; modelInputUnits: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewInput {
    /// Code files in the commit.
    pub code_file_count: usize,
    /// Lines of the whole patch (a final newline counts as one more, empty, line).
    pub diff_line_count: usize,
    /// Whether the patch is over `CAP_LINES`.
    pub truncated: bool,
    /// The text sent to the model.
    pub model_input: String,
    /// The reported length of `model_input` in UTF-16 units.
    pub model_input_units: usize,
}

/// What:
///  The excerpt of one file: its header and the text of the window around the changed span.
///  The window runs from `EXCERPT_CONTEXT` lines before the first changed new-side line to `EXCERPT_CONTEXT` lines
///  after the last, cut from the new-side file. A file with no changed line to anchor on (a binary file, a mode change,
///  a pure rename) gets the head of the file instead.
/// Why:
///   Returns the window's line count too, which the 4000-line cap adds up. The script cut the window from old-side
///   numbers, which sit in the wrong place after an earlier insertion, and gave a file without hunks a nonsensical
///   negative window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excerptParts.push(`// candidate excerpt of ${name}, lines ${from} to ${to} of ${fileLines.length}`);
/// ```
fn excerpt(name: &str, content: &str, spans: &[NewSpan]) -> (Vec<String>, i64) {
    let file_lines: Vec<&str> = content.split('\n').collect();
    let total: i64 = i64::try_from(file_lines.len()).unwrap_or(i64::MAX);
    let first_changed: Option<i64> = spans.iter().map(|span| return i64::try_from(span.first).unwrap_or(i64::MAX)).min();
    let last_changed: Option<i64> = spans.iter().map(|span| return i64::try_from(span.last).unwrap_or(i64::MAX)).max();
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
///   The input is the first `CAP_LINES` patch lines, a blank line, then each excerpt separated by blank lines, cut to
///   `INPUT_UNIT_CAP` UTF-16 units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lunaInput = (lines.slice(0, capLines).join('\n') + '\n\n' + excerptParts.join('\n\n')).slice(0, 24000);
/// ```
pub fn build_input(commit: &LandedCommit, code_files: &[String]) -> ReviewInput {
    let lines: Vec<&str> = commit.patch.split('\n').collect();
    let kept: &[&str] = &lines[..lines.len().min(CAP_LINES)];
    let mut parts: Vec<String> = Vec::new();
    let mut excerpt_lines: i64 = 0;
    for name in code_files.iter().take(MAX_EXCERPT_FILES) {
        if excerpt_lines >= EXCERPT_LINE_CAP {
            break;
        }
        let Some(content) = commit.contents.get(name) else {
            continue;
        };
        let spans: Vec<NewSpan> = new_spans(lines.as_slice(), name);
        let (pieces, covered) = excerpt(name, content, spans.as_slice());
        parts.extend(pieces);
        excerpt_lines += covered;
    }
    let full: String = format!("{}\n\n{}", kept.join("\n"), parts.join("\n\n"));
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
