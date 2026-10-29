//! Corpus drawn from this repo's own non-gitignored lines:
//!  realistic scanner input.
//!
//! What:
//!  [`build_corpus`] walks the repo with the same `ignore::WalkBuilder` config the
//! real forbidden-strings scanner uses,
//!  reads each text file,
//!  and collects its lines up
//! to a cap.
//!  Why:
//!  a secret scanner runs over exactly this file set,
//!  so the honest
//! throughput is the one measured on real source lines (mixed case,
//!  punctuation,
//! identifiers,
//!  prose) rather than synthetic noise.

/// Imports the path types for repo file enumeration.
use std::path::PathBuf;

/// Imports the gitignore-aware walker the scanner uses.
use ignore::WalkBuilder;

/// Longest line admitted into the corpus.
///
/// What:
///  a per-line byte ceiling.
///  Why:
///  minified or generated megabyte lines would
/// skew the average length and the scan,
///  so they are skipped.
const MAX_LINE_LEN: usize = 4_096;

/// Returns the repository root directory.
///
/// What:
///  walks up from the working dir to the first ancestor holding a `.git` entry.
/// Why:
///  the walk and reads resolve against the repo root regardless of the bench's
/// working dir,
///  without shelling out to the repo's `cli-git` wrapper.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("working dir is readable");
    loop {
        if dir.join(".git").exists() {
            return dir;
        }
        if !dir.pop() {
            panic!("no .git directory found above the working dir");
        }
    }
}

/// Lists every non-gitignored file under the repo root.
///
/// What:
///  the same walk the scanner uses,
///  `WalkBuilder::new(root).hidden(false)
/// .ignore(false)` skipping the `.git`/`.jj` subtrees,
///  keeping only files.
///  Why:
/// `hidden(false)` includes tracked dotfiles,
///  `ignore(false)` leaves `.ignore`
/// re-exclusions out so `.gitignore` alone decides the set,
///  exactly as the scanner does.
fn non_ignored_files(root: &PathBuf) -> Vec<PathBuf> {
    return WalkBuilder::new(root)
        .hidden(false)
        .ignore(false)
        .filter_entry(|entry| return entry.file_name() != ".git" && entry.file_name() != ".jj")
        .build()
        .filter_map(Result::ok)
        .filter(|entry| return entry.file_type().is_some_and(|kind| return kind.is_file()))
        .map(|entry| return entry.into_path())
        .collect()
}

/// Appends one file's usable text lines to `lines`.
///
/// What:
///  skips binary files (any NUL byte) and over-long or empty lines,
///  trimming a
/// trailing carriage return.
///  Why:
///  keeps the corpus to real,
///  scannable text lines.
fn gather_file(path: &PathBuf, lines: &mut Vec<Vec<u8>>) {
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    if bytes.contains(&0) {
        return;
    }
    for raw in bytes.split(|&b| return b == b'\n') {
        let line = raw.strip_suffix(b"\r").unwrap_or(raw);
        if line.is_empty() || line.len() > MAX_LINE_LEN {
            continue;
        }
        lines.push(line.to_vec());
    }
}

/// Builds the corpus from every non-gitignored text line in the repo.
///
/// What:
///  enumerates non-ignored files and gathers all their lines.
///  Why:
///  a faithful,
/// reproducible stand-in for the real files a secret scanner processes;
///  whatever
/// credentials the repo genuinely contains are matched by both engines,
///  so the parity
/// check stays meaningful.
pub fn build_corpus() -> Vec<Vec<u8>> {
    let root = repo_root();
    let mut files = non_ignored_files(&root);
    files.sort();
    let mut lines: Vec<Vec<u8>> = Vec::new();
    for file in files {
        gather_file(&file, &mut lines);
    }
    if lines.is_empty() {
        panic!("gathered no corpus lines from the repo");
    }
    return lines
}
