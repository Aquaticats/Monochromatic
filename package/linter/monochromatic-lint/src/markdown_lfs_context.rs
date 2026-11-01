//! What:
//!  Repository facts `markdown/lfs-image-url` needs,
//!  discovered once and resolved per file.
//! Why:
//!  The rule itself stays a pure function over a parsed document;
//!  every filesystem read
//! (`.lfsconfig`,
//!  root `.gitattributes`,
//!  image bytes) happens here,
//!  before the rule runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // discoverLfsImageRepo(startDirectory) -> repo | undefined; prepareLfsImageContext({ repo, filePath, document })
//! ```

/// Import configuration reads,
///  the tracked-path matcher,
///  object ids and destination classification.
/// Import the parsed document whose images and definitions name candidate paths.
use crate::{
    markdown_lfs_config::{
        LFS_CONFIG_FILENAME, LfsConfigError, read_lfs_object_base, read_optional_text,
    },
    markdown_lfs_oid::lfs_oid_of_bytes,
    markdown_lfs_patterns::{PathPatterns, lfs_tracked_patterns},
    markdown_lfs_target::{
        apply_segments, lexical_normal, object_url_parts, relative_target_path, repo_relative,
    },
    markdown_source::MarkdownSource,
};
/// Import node kinds and the typed image and definition payload decoders.
use satteri_ast::mdast::{MdastNodeType, decode_definition_data, decode_image_data};
/// Import an ordered map (deterministic iteration) and a lock for the shared per-run cache.
use std::{
    collections::BTreeMap,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

/// What:
///  The file name git reads for path attributes.
/// Why:
///  Only the repository root's file is consulted,
///  as in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT_ATTRIBUTES_FILENAME = '.gitattributes';
/// ```
const GIT_ATTRIBUTES_FILENAME: &str = ".gitattributes";

/// What:
///  What a repository-relative path resolves to for image rewriting.
/// Why:
///  The three states drive three different outcomes:
///  rewrite,
///  leave alone,
///  or report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LfsImageTarget = { kind: 'lfs'; oid: string } | { kind: 'plain' } | { kind: 'missing' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LfsImageTarget {
    /// The path exists and git-lfs tracks it;
    ///  `oid` is its current object id.
    Lfs {
        /// 64 lowercase hexadecimal characters.
        oid: String,
    },
    /// The path exists as an ordinary git blob.
    Plain,
    /// The path does not exist in the repository's working tree,
    ///  or was never referenced.
    Missing,
}

/// What:
///  Repository-wide facts shared by every file under one `.lfsconfig`.
/// Why:
///  The object base and tracked patterns are read once;
///  resolved targets are cached because a
/// fixing run checks the same file several times and many files reference the same images.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LfsImageRepo = { repoRoot: Path; objectBase: string; isLfsTracked(path): boolean };
/// ```
#[derive(Debug)]
pub struct LfsImageRepo {
    /// Absolute,
    ///  lexically normal directory holding `.lfsconfig`.
    pub repo_root: PathBuf,
    /// Credential-free object base;
    ///  objects are addressed as `<base>/<oid>/<path>`.
    pub object_base: String,
    /// Compiled `filter=lfs` patterns of the root `.gitattributes`.
    tracked: PathPatterns,
    /// Resolved targets by normalized repository path;
    ///  `Mutex` lets worker threads share one repository.
    resolved: Mutex<BTreeMap<String, LfsImageTarget>>,
}

/// What:
///  Per-file facts handed to the rule.
/// Why:
///  Every candidate path was resolved while this was prepared,
///  so the rule needs no I/O.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LfsImageContext = { filePath: Path; repoRoot: Path; objectBase: string; resolveTarget(path): LfsImageTarget };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LfsImageContext {
    /// Absolute,
    ///  lexically normal path of the Markdown file;
    ///  relative destinations resolve against its directory.
    pub file_path: PathBuf,
    /// Absolute,
    ///  lexically normal repository root.
    pub repo_root: PathBuf,
    /// Credential-free object base URL.
    pub object_base: String,
    /// Resolved targets keyed by the path exactly as the destination names it.
    pub targets: BTreeMap<String, LfsImageTarget>,
}

/// What:
///  Look up a prepared target.
/// Why:
///  A path the preparation never saw reads as missing,
///  never as an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolveTarget(repoRelativePath: string): LfsImageTarget
/// ```
impl LfsImageContext {
    /// Return an owned copy so the caller is not tied to this context's borrow.
    pub fn resolve_target(&self, repo_relative_path: &str) -> LfsImageTarget {
        if let Some(target) = self.targets.get(repo_relative_path) {
            return target.clone();
        }
        return LfsImageTarget::Missing;
    }
}

/// What:
///  The nearest directory,
///  starting at `start` and walking toward the filesystem root,
///  that
/// holds a regular `.lfsconfig` file.
/// Why:
///  That directory is the repository root object paths are relative to.
/// A directory or other non-file entry with that name is not a configuration and is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findLfsRepoRoot(start: Path): Path | undefined;
/// ```
pub(crate) fn find_lfs_repo_root(start: &Path) -> Result<Option<PathBuf>, LfsConfigError> {
    for directory in start.ancestors() {
        let candidate: PathBuf = directory.join(LFS_CONFIG_FILENAME);
        match std::fs::metadata(&candidate) {
            Ok(metadata) => {
                if metadata.is_file() {
                    return Ok(Some(directory.to_path_buf()));
                }
            }
            Err(error) => {
                if error.kind() != ErrorKind::NotFound && error.kind() != ErrorKind::NotADirectory {
                    return Err(LfsConfigError {
                        message: format!("Cannot inspect {}: {error}.", candidate.display()),
                    });
                }
            }
        }
    }
    return Ok(None);
}

/// What:
///  Discover the repository a directory belongs to,
///  or `None` when no ancestor declares an LFS endpoint.
/// Why:
///  Without a declared server there is no object URL to write,
///  so the rule is inert there.
/// A found `.lfsconfig` without an endpoint ends the search;
///  farther ancestors are not consulted.
/// `start` is an absolute directory;
///  `.` and `..` components are resolved lexically first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function discoverLfsImageRepo(start: Path): LfsImageRepo | undefined;
/// ```
pub fn discover_lfs_image_repo(start: &Path) -> Result<Option<LfsImageRepo>, LfsConfigError> {
    let normal: PathBuf = lexical_normal(start);
    let Some(repo_root): Option<PathBuf> = find_lfs_repo_root(&normal)? else {
        return Ok(None);
    };
    let Some(object_base): Option<String> = read_lfs_object_base(&repo_root)? else {
        return Ok(None);
    };
    let attributes: PathBuf = repo_root.join(GIT_ATTRIBUTES_FILENAME);
    let text: String = read_optional_text(&attributes)?.unwrap_or_default();
    let patterns: Vec<String> = lfs_tracked_patterns(text.as_str());
    let tracked: PathPatterns = match PathPatterns::new(patterns.as_slice()) {
        Ok(compiled) => compiled,
        Err(error) => {
            return Err(LfsConfigError {
                message: format!("{}: {error}", attributes.display()),
            });
        }
    };
    return Ok(Some(LfsImageRepo {
        repo_root,
        object_base,
        tracked,
        resolved: Mutex::new(BTreeMap::<String, LfsImageTarget>::new()),
    }));
}

/// What:
///  Whether the rule's `exclude` patterns name a file under lint.
/// Why:
///  Patterns are relative to the repository root;
///  a file outside the repository is never
/// excluded by them.
///  Only the root is needed,
///  so exclusion is decided before the repository's
/// endpoint is read and an excluded file is unaffected by an unusable endpoint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExcluded(repoRoot, filePath, exclude): boolean;
/// ```
pub fn is_excluded(repo_root: &Path, file_path: &Path, exclude: &PathPatterns) -> bool {
    let normal: PathBuf = lexical_normal(file_path);
    let Some(relative): Option<String> = repo_relative(repo_root, &normal) else {
        return false;
    };
    return exclude.matches(relative.as_str());
}

/// What:
///  Repository operations that read the working tree.
/// Why:
///  Target resolution owns the cache lock;
///  callers see only resolved values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repo.resolveTarget(repoRelativePath)
/// ```
impl LfsImageRepo {
    /// What:
    ///  Read one normalized repository path from disk without consulting the cache.
    /// Why:
    ///  Existence,
    ///  tracking and the current object id are three separate facts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readTarget(relative: string, absolute: Path): LfsImageTarget;
    /// ```
    fn read_target(
        &self,
        relative: &str,
        absolute: &Path,
    ) -> Result<LfsImageTarget, LfsConfigError> {
        match std::fs::metadata(absolute) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Ok(LfsImageTarget::Missing);
                }
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound || error.kind() == ErrorKind::NotADirectory {
                    return Ok(LfsImageTarget::Missing);
                }
                return Err(LfsConfigError {
                    message: format!(
                        "Cannot inspect image target {}: {error}.",
                        absolute.display()
                    ),
                });
            }
        }
        if !self.tracked.matches(relative) {
            return Ok(LfsImageTarget::Plain);
        }
        match std::fs::read(absolute) {
            Ok(bytes) => {
                return Ok(LfsImageTarget::Lfs {
                    oid: lfs_oid_of_bytes(bytes.as_slice()),
                });
            }
            Err(error) => {
                return Err(LfsConfigError {
                    message: format!("Cannot read image target {}: {error}.", absolute.display()),
                });
            }
        }
    }

    /// What:
    ///  Resolve a forward-slash path as a destination names it.
    /// Why:
    ///  `.` and `..` segments are resolved lexically first;
    ///  a path that leaves the repository
    /// is missing from it and is never read,
    ///  so a Markdown file cannot make the linter open files
    /// outside the repository.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function resolveTarget(repoRelativePath: string): LfsImageTarget;
    /// ```
    pub fn resolve_target(
        &self,
        repo_relative_path: &str,
    ) -> Result<LfsImageTarget, LfsConfigError> {
        let absolute: PathBuf = apply_segments(&self.repo_root, repo_relative_path);
        let Some(relative): Option<String> = repo_relative(&self.repo_root, &absolute) else {
            return Ok(LfsImageTarget::Missing);
        };
        {
            // The guard is dropped at the end of this block so the lock is not held during file reads.
            let cache: MutexGuard<'_, BTreeMap<String, LfsImageTarget>> = self
                .resolved
                .lock()
                .expect("target cache lock is not poisoned");
            if let Some(target) = cache.get(relative.as_str()) {
                return Ok(target.clone());
            }
        }
        let target: LfsImageTarget = self.read_target(relative.as_str(), &absolute)?;
        let mut cache: MutexGuard<'_, BTreeMap<String, LfsImageTarget>> = self
            .resolved
            .lock()
            .expect("target cache lock is not poisoned");
        cache.insert(relative, target.clone());
        return Ok(target);
    }
}

/// What:
///  Prepare one file's context by resolving every path an image or image definition may name.
/// Why:
///  The rule then decides fixes synchronously;
///  a fix only swaps a destination between the
/// relative and object forms of the same path,
///  so the candidate set is stable across fix passes.
/// `file_path` is absolute;
///  `.` and `..` components are resolved lexically first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareLfsImageContext({ repo, filePath, document }): LfsImageContext;
/// ```
pub fn prepare_lfs_image_context(
    repo: &LfsImageRepo,
    file_path: &Path,
    document: &MarkdownSource,
) -> Result<LfsImageContext, LfsConfigError> {
    let normal: PathBuf = lexical_normal(file_path);
    let mut targets: BTreeMap<String, LfsImageTarget> = BTreeMap::<String, LfsImageTarget>::new();
    for id in document.visible_nodes() {
        let kind: MdastNodeType = document.kind(*id);
        let url: &str = if kind == MdastNodeType::Image {
            document.text(decode_image_data(document.data(*id)).url)
        } else if kind == MdastNodeType::Definition {
            document.text(decode_definition_data(document.data(*id)).url)
        } else {
            continue;
        };
        if let Some(parts) = object_url_parts(url, repo.object_base.as_str()) {
            let target: LfsImageTarget = repo.resolve_target(parts.repo_relative_path.as_str())?;
            targets.insert(parts.repo_relative_path, target);
        }
        if let Some(path) = relative_target_path(url, &normal, &repo.repo_root) {
            let target: LfsImageTarget = repo.resolve_target(path.as_str())?;
            targets.insert(path, target);
        }
    }
    return Ok(LfsImageContext {
        file_path: normal,
        repo_root: repo.repo_root.clone(),
        object_base: repo.object_base.clone(),
        targets,
    });
}

/// Disposable-repository controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_context_tests.rs"]
mod tests;
