//! What:
//! The words of the repository's current sources that can name a container image or a volume:
//! every run of file-name-like characters in the build files,
//! scripts and source of the checkout.
//! Why:
//! The owner asked to keep every image "no current image tag of this repository names"
//! and (by the same rule) every volume a current task names.
//! A tag the sources spell out (`localhost/monochromatic/ide:latest`) is current;
//! a tag a script builds from a template (`localhost/git-policy-native-test:${tag}`) names no tag.
//! The scan reads text only,
//! skips dependency and build directories,
//! and stops at its deadline.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function collectReferences(root: string, deadline: Deadline): Set<string>
//! ```

/// Files and sets.
use std::{collections::HashSet, fs, path::{Path, PathBuf}};

use crate::lean_identity::Deadline;

/// Directory names the scan never enters.
const SKIPPED_DIRECTORIES: [&str; 9] = ["node_modules", "target", ".git", "dist", ".claude", ".cache", "doc", ".pnpm-store", "LICENSES"];

/// File extensions the scan reads.
const READ_EXTENSIONS: [&str; 6] = ["toml", "ts", "rs", "sh", "yml", "yaml"];

/// The largest file the scan reads.
const LARGEST_FILE_BYTES: u64 = 512 * 1024;

/// What:
/// `fn reads(path: &Path) -> bool` says whether a file is one the scan reads:
/// a Containerfile or a file with a source extension.
/// Why:
/// Documentation and lockfiles name old tags and are skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reads = (path: string): boolean
/// ```
fn reads(path: &Path) -> bool {
    let name: String = path.file_name().map(|text| { return text.to_string_lossy().to_string(); }).unwrap_or_default();
    if name.starts_with("Containerfile") || name.starts_with("Dockerfile") {
        return true;
    }
    return path.extension().is_some_and(|extension| { return READ_EXTENSIONS.iter().any(|known| { return extension == *known; }); });
}

/// What:
/// `pub fn words_of(text: &str, words: &mut HashSet<String>)` adds every run of `[A-Za-z0-9._/:@-]` in the text to the set,
/// and the parts of each run split at colons.
/// Why:
/// `localhost/x:tag` is one word,
/// and `ide-cargo:/cargo` yields `ide-cargo` for volume names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function wordsOf(text: string, words: Set<string>): void
/// ```
pub fn words_of(text: &str, words: &mut HashSet<String>) {
    let mut current: String = String::new();
    let flush = |run: &mut String, words: &mut HashSet<String>| {
        // A run that ends at a colon is a template (`repo:${tag}`): it names no tag, so it adds nothing.
        let templated: bool = run.ends_with(':');
        let trimmed: &str = run.trim_matches(['.', ':', '-', '/']);
        if trimmed.len() > 2 && !templated {
            words.insert(trimmed.to_string());
            for part in trimmed.split(':') {
                if part.len() > 2 {
                    words.insert(part.trim_matches(['.', '-', '/']).to_string());
                }
            }
        }
        run.clear();
    };
    for character in text.chars() {
        if character.is_ascii_alphanumeric() || "._/:@-".contains(character) {
            current.push(character);
        } else {
            flush(&mut current, words);
        }
    }
    flush(&mut current, words);
}

/// What:
/// `pub fn collect_references(root: &Path, deadline: &Deadline) -> Option<HashSet<String>>` reads the words of every source file
/// under the root.
/// Why:
/// `None` means the deadline passed before the whole checkout was read,
/// and then nothing may be judged "not named".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function collectReferences(root: string, deadline: Deadline): Set<string> | undefined
/// ```
pub fn collect_references(root: &Path, deadline: &Deadline) -> Option<HashSet<String>> {
    let mut words: HashSet<String> = HashSet::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        if deadline.passed() {
            return None;
        }
        let Ok(entries) = fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else { continue };
            let path: PathBuf = entry.path();
            if kind.is_dir() {
                let name: String = entry.file_name().to_string_lossy().to_string();
                if !SKIPPED_DIRECTORIES.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if kind.is_file() && reads(&path) && entry.metadata().is_ok_and(|metadata| { return metadata.len() <= LARGEST_FILE_BYTES; }) {
                if let Ok(text) = fs::read_to_string(&path) {
                    words_of(&text, &mut words);
                }
            }
        }
    }
    return Some(words);
}
