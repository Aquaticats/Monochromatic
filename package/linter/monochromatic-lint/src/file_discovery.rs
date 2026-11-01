//! What:
//!  Discover supported source files from explicit native file/directory paths.
//! Why:
//!  Ignore sources,
//!  hidden directories and I/O failures must be handled before language/configuration dispatch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Walk one literal input root; glob expansion and per-file configuration matching are separate boundaries.
//! ```

/// Import the language choice that decides which files are inputs.
use crate::run_paths::language_of;
/// Import the glob override builder that layers the crate's exclusions over ignore files.
use ignore::overrides::{Override, OverrideBuilder};
/// Import gitignore-aware traversal and its override-pattern compiler.
use ignore::{DirEntry, WalkBuilder};
/// Import deterministic path deduplication and native filesystem types.
use std::collections::BTreeSet;
/// Import the OS string type for file names that may not be valid UTF-8.
use std::ffi::OsStr;
/// Import the path types used to walk and compare filesystem locations.
use std::path::{Path, PathBuf};

/// Owned discovery failure rather than an unreadable path silently disappearing from the result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileDiscoveryError {
    /// Affected path/pattern and failed operation.
    pub message: String,
}

/// Render the discovery failure without losing its context.
impl std::fmt::Display for FileDiscoveryError {
    /// Borrow the formatter while writing owned text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Integrate typed discovery failures with application error handling.
impl std::error::Error for FileDiscoveryError {}

/// Explicit controls for a literal-path walk,
///  with no executable or callback configuration.
pub struct DiscoveryOptions {
    /// Invocation's absolute directory,
    ///  used for relative ignore patterns.
    pub cwd: PathBuf,
    /// Disable user/repository ignores,
    ///  but never walk Git's metadata directory.
    pub no_ignore: bool,
    /// Extra exclusion globs supplied on the command line.
    pub ignore_patterns: Vec<String>,
    /// Extra ignore files supplied on the command line.
    pub ignore_paths: Vec<PathBuf>,
}

/// Check extension bytes through native OsStr equality,
///  without lossy path conversion.
pub fn supported_source(path: &Path) -> bool {
    // One definition of a supported input serves discovery and planning, so the two cannot drift apart.
    return language_of(path).is_some();
}

/// Git metadata is never source,
///  even when user ignore files are disabled.
fn not_git_metadata(entry: &DirEntry) -> bool {
    return entry.file_name() != OsStr::new(".git");
}

/// Add one exclusion using the walker's inverse-polarity override grammar.
fn add_exclusion(builder: &mut OverrideBuilder, pattern: &str) -> Result<(), FileDiscoveryError> {
    if let Err(error) = builder.add(format!("!{pattern}").as_str()) {
        return Err(FileDiscoveryError {
            message: format!("Cannot compile file-discovery exclusion {pattern:?}: {error}."),
        });
    }
    return Ok(());
}

/// Compile all exclusions together instead of replacing the previous override on each loop iteration.
fn overrides(options: &DiscoveryOptions) -> Result<Override, FileDiscoveryError> {
    let mut builder: OverrideBuilder = OverrideBuilder::new(&options.cwd);
    add_exclusion(&mut builder, "**/node_modules/**")?;
    for pattern in &options.ignore_patterns {
        add_exclusion(&mut builder, pattern.as_str())?;
    }
    match builder.build() {
        Ok(value) => return Ok(value),
        Err(error) => {
            return Err(FileDiscoveryError {
                message: format!("Cannot build file-discovery exclusions: {error}."),
            });
        }
    }
}

/// Read supported files from an explicit file or directory,
///  preserving native path bytes and stable ordering.
pub fn discover_literal_path(
    path: &Path,
    options: &DiscoveryOptions,
) -> Result<Vec<PathBuf>, FileDiscoveryError> {
    if !path.is_absolute() || !options.cwd.is_absolute() {
        return Err(FileDiscoveryError {
            message: String::from(
                "File discovery requires an absolute input path and invocation directory.",
            ),
        });
    }
    let metadata: std::fs::Metadata = match std::fs::metadata(path) {
        Ok(value) => value,
        Err(error) => {
            return Err(FileDiscoveryError {
                message: format!("Cannot inspect lint input {}: {error}.", path.display()),
            });
        }
    };
    if metadata.is_file() {
        // Preserve the incumbent's explicit-file behavior: naming a file bypasses traversal ignores.
        if supported_source(path) {
            return Ok(vec![path.to_path_buf()]);
        }
        return Ok(Vec::<PathBuf>::new());
    }
    if !metadata.is_dir() {
        return Err(FileDiscoveryError {
            message: format!(
                "Lint input {} is neither a regular file nor a directory.",
                path.display()
            ),
        });
    }
    let mut builder: WalkBuilder = WalkBuilder::new(path);
    builder.current_dir(options.cwd.clone());
    builder.hidden(false);
    builder.follow_links(false);
    builder.filter_entry(not_git_metadata);
    if options.no_ignore {
        builder.standard_filters(false);
    } else {
        builder.overrides(overrides(options)?);
        for ignore in &options.ignore_paths {
            if let Some(error) = builder.add_ignore(ignore) {
                return Err(FileDiscoveryError {
                    message: format!(
                        "Cannot read file-discovery ignore file {}: {error}.",
                        ignore.display()
                    ),
                });
            }
        }
    }
    let mut selected: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for result in builder.build() {
        let entry: DirEntry = match result {
            Ok(value) => value,
            Err(error) => {
                return Err(FileDiscoveryError {
                    message: format!("Cannot walk lint input {}: {error}.", path.display()),
                });
            }
        };
        if let Some(error) = entry.error() {
            return Err(FileDiscoveryError {
                message: format!(
                    "Cannot apply ignore rules while walking {}: {error}.",
                    entry.path().display()
                ),
            });
        }
        let Some(kind): Option<std::fs::FileType> = entry.file_type() else {
            continue;
        };
        if kind.is_file() && supported_source(entry.path()) {
            selected.insert(entry.path().to_path_buf());
        }
    }
    return Ok(selected.into_iter().collect::<Vec<PathBuf>>());
}

/// Keep actual filesystem controls outside release artifacts.
#[cfg(test)]
#[path = "file_discovery_tests.rs"]
mod tests;
