//! What:
//!  Expand native literal inputs and explicit path globs into supported source files.
//! Why:
//!  Glob syntax must not reinterpret an existing filename,
//!  and unmatched inputs must remain distinguishable from an empty directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Prefer an existing literal path; otherwise compile the requested glob and walk only its literal directory prefix.
//! ```

/// Import the existing walker and its typed failure.
use crate::file_discovery::{DiscoveryOptions, FileDiscoveryError, discover_literal_path};
/// Import the same glob grammar used for configuration selectors.
use globset::{GlobBuilder, GlobMatcher};
/// Import native path collection and I/O error classification.
use std::collections::BTreeSet;
/// Import the I/O error kind so a missing path can be told apart from other failures.
use std::io::ErrorKind;
/// Import the path component and path types used to normalise caller-supplied paths.
use std::path::{Component, Path, PathBuf};

/// Determine whether a path token contains glob punctuation;
///  existing files are tested before calling this.
fn has_glob(text: &str) -> bool {
    for character in text.chars() {
        if character == '*' || character == '?' || character == '[' || character == '{' {
            return true;
        }
    }
    return false;
}

/// Retain complete literal path components before the first pattern component.
fn walk_root(pattern: &Path, cwd: &Path) -> PathBuf {
    let mut prefix: PathBuf = PathBuf::new();
    for component in pattern.components() {
        if let Component::Normal(part) = component {
            let text: &str = part.to_str().expect("pattern was checked as UTF-8");
            if has_glob(text) {
                break;
            }
        }
        prefix.push(component.as_os_str());
    }
    if prefix.is_absolute() {
        return prefix;
    }
    return cwd.join(prefix);
}

/// Expand a glob without converting discovered file names to UTF-8 or losing their operating-system bytes.
fn expand_glob(
    pattern: &Path,
    text: &str,
    options: &DiscoveryOptions,
) -> Result<Vec<PathBuf>, FileDiscoveryError> {
    // Path components remove a leading './' without changing pattern punctuation or native literal lookup.
    let mut normalized: PathBuf = PathBuf::new();
    for component in pattern.components() {
        if component != Component::CurDir {
            normalized.push(component.as_os_str());
        }
    }
    let normalized_text: &str = normalized.to_str().expect("UTF-8 pattern components");
    let mut builder: GlobBuilder<'_> = GlobBuilder::new(normalized_text);
    builder.literal_separator(true);
    let matcher: GlobMatcher = match builder.build() {
        Ok(glob) => glob.compile_matcher(),
        Err(error) => {
            return Err(FileDiscoveryError {
                message: format!("Invalid lint input pattern {text:?}: {error}."),
            });
        }
    };
    let root: PathBuf = walk_root(pattern, &options.cwd);
    match std::fs::metadata(&root) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Ok(Vec::<PathBuf>::new());
            }
        }
        Err(error) => {
            if error.kind() == ErrorKind::NotFound || error.kind() == ErrorKind::NotADirectory {
                return Ok(Vec::<PathBuf>::new());
            }
            return Err(FileDiscoveryError {
                message: format!(
                    "Cannot inspect root {} of input pattern {text:?}: {error}.",
                    root.display()
                ),
            });
        }
    }
    let candidates: Vec<PathBuf> = discover_literal_path(&root, options)?;
    let mut matched: Vec<PathBuf> = Vec::<PathBuf>::new();
    for candidate in candidates {
        let target: &Path = if pattern.is_absolute() {
            candidate.as_path()
        } else {
            match candidate.strip_prefix(&options.cwd) {
                Ok(relative) => relative,
                Err(error) => {
                    return Err(FileDiscoveryError {
                        message: format!(
                            "Cannot match {} relative to {}: {error}.",
                            candidate.display(),
                            options.cwd.display()
                        ),
                    });
                }
            }
        };
        if matcher.is_match(target) {
            matched.push(candidate);
        }
    }
    return Ok(matched);
}

/// Collect explicit input tokens,
///  defaulting to cwd only when the caller supplies no paths.
pub fn collect_inputs(
    inputs: &[PathBuf],
    options: &DiscoveryOptions,
    allow_unmatched: bool,
) -> Result<Vec<PathBuf>, FileDiscoveryError> {
    if inputs.is_empty() {
        return discover_literal_path(&options.cwd, options);
    }
    let mut files: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for input in inputs {
        let absolute: PathBuf = if input.is_absolute() {
            input.clone()
        } else {
            options.cwd.join(input)
        };
        match std::fs::metadata(&absolute) {
            Ok(_) => {
                files.extend(discover_literal_path(&absolute, options)?);
                continue;
            }
            Err(error) => {
                if error.kind() != ErrorKind::NotFound && error.kind() != ErrorKind::NotADirectory {
                    return Err(FileDiscoveryError {
                        message: format!(
                            "Cannot inspect lint input {}: {error}.",
                            absolute.display()
                        ),
                    });
                }
            }
        }
        let mut matched: Vec<PathBuf> = Vec::<PathBuf>::new();
        if let Some(text) = input.to_str()
            && has_glob(text)
        {
            matched = expand_glob(input, text, options)?;
        }
        if matched.is_empty() && !allow_unmatched {
            return Err(FileDiscoveryError {
                message: format!(
                    "Lint input {} matched no supported source files. Correct the path/pattern or use --no-error-on-unmatched-pattern.",
                    input.display()
                ),
            });
        }
        files.extend(matched);
    }
    return Ok(files.into_iter().collect::<Vec<PathBuf>>());
}
