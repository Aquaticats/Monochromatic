//! What:
//!  Split the drift of a run into committed outputs and local outputs,
//! and keep only the part a command asked for.
//! Why:
//!  A fresh checkout holds every committed output but none of the gitignored ones,
//! so CI must check the committed outputs before it
//!  regenerates anything and the local outputs only afterwards.
//! The split is derived from the repository's own `.gitignore` files,
//! so no rule declares it and it cannot go stale.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OutputScope = 'all' | 'committed' | 'local';
//! function selectDrifts(view: View, drifts: Drift[], scope: OutputScope): Drift[]
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Drift`,
/// `DriftReason` from `crate::drift`.
use crate::drift::{Drift, DriftReason};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `Match` from `ignore`.
use ignore::Match;
/// Imports `Gitignore`,
/// `GitignoreBuilder` from `ignore::gitignore`.
use ignore::gitignore::{Gitignore, GitignoreBuilder};
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// What:
///  Which outputs a `check` or `write` covers.
/// Why:
///  `Committed` outputs are the paths no `.gitignore` rule matches;
/// `Local` outputs are the ones a rule matches.
///  `All` is the default and changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OutputScope = 'all' | 'committed' | 'local';
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputScope {
    /// Every planned output.
    #[default]
    All,
    /// Planned outputs that no `.gitignore` rule matches.
    Committed,
    /// Planned outputs that a `.gitignore` rule matches.
    Local,
}

/// What:
///  The text form of a scope and its inverse.
/// Why:
///  The command line and the server protocol both spell a scope as one word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name(): string; static parse(text: string): OutputScope | undefined
/// ```
impl OutputScope {
    /// What:
    ///  The word for this scope:
    /// `all`,
    /// `committed` or `local`.
    /// Why:
    ///  Messages,
    /// the `outputs` member of result events and the protocol use it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// name(): 'all' | 'committed' | 'local'
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            OutputScope::All => return "all",
            OutputScope::Committed => return "committed",
            OutputScope::Local => return "local",
        }
    }

    /// What:
    ///  Read a scope from its word.
    /// Why:
    ///  The server decodes the `outputs` request member with it;
    /// an unknown word is the caller's mistake.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static parse(text: string): OutputScope | undefined
    /// ```
    pub fn parse(text: &str) -> Option<OutputScope> {
        match text {
            "all" => return Some(OutputScope::All),
            "committed" => return Some(OutputScope::Committed),
            "local" => return Some(OutputScope::Local),
            _ => return None,
        }
    }
}

/// What:
///  The `.gitignore` matcher of each directory read so far,
/// keyed by the directory's repo-relative path (`""` for the root);
/// `None` records a directory without one.
/// Why:
///  Many drifted paths share ancestors,
/// and each ancestor's file is read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Matchers = Map<string, Gitignore | undefined>;
/// ```
type Matchers = BTreeMap<String, Option<Gitignore>>;

/// What:
///  The repo-relative path of the `.gitignore` file in a directory.
/// Why:
///  The root directory is the empty string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoreFileOf(dir: string): string
/// ```
fn ignore_file_of(dir: &str) -> String {
    if dir.is_empty() {
        return ".gitignore".to_string();
    }
    return format!("{dir}/.gitignore");
}

/// What:
///  Read and compile the `.gitignore` of one directory through the view.
///  `Ok(None)` means the directory has none.
/// Why:
///  Reading through the view records the file,
/// so `write` revalidates it and `watch` watches it,
/// and a rule that plans a `.gitignore` is classified by the planned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadMatcher(view: View, dir: string): Gitignore | undefined
/// ```
fn load_matcher(view: &mut View, dir: &str) -> Result<Option<Gitignore>, Diagnostic> {
    let file = ignore_file_of(dir);
    let Some(text) = view.read_text(&file)? else { return Ok(None) };
    let base: PathBuf = if dir.is_empty() { view.root().to_path_buf() } else { view.root().join(dir) };
    let mut builder = GitignoreBuilder::new(base);
    for line in text.lines() {
        if let Err(error) = builder.add_line(None, line) {
            let message = format!("{file} holds a pattern that cannot be compiled, so outputs cannot be split into committed and local: {error}");
            return Err(Diagnostic::new("gitignore-invalid", &message).with_path(&file).with_hint("fix the pattern, or run without --outputs"));
        }
    }
    match builder.build() {
        Ok(matcher) => return Ok(Some(matcher)),
        Err(error) => {
            let message = format!("{file} cannot be compiled, so outputs cannot be split into committed and local: {error}");
            return Err(Diagnostic::new("gitignore-invalid", &message).with_path(&file).with_hint("fix the file, or run without --outputs"));
        }
    }
}

/// What:
///  Make sure the matcher of a directory is in the cache.
/// Why:
///  The cache holds `None` for a directory without a `.gitignore`,
/// so a missing file is not looked up twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureMatcher(view: View, matchers: Matchers, dir: string): void
/// ```
fn ensure_matcher(view: &mut View, matchers: &mut Matchers, dir: &str) -> Result<(), Diagnostic> {
    if matchers.contains_key(dir) {
        return Ok(());
    }
    let matcher = load_matcher(view, dir)?;
    matchers.insert(dir.to_string(), matcher);
    return Ok(());
}

/// What:
///  Decide whether git ignores one path,
/// looking at the path and then at its parents.
/// Why:
///  Git settles each path component from the root down:
/// the deepest `.gitignore` with a matching line wins (`!` re-includes),
/// and a path below an ignored directory is ignored for good,
/// which is how `/.claude/` covers `.claude/skills/x/SKILL.md`.
///  Only `.gitignore` files in the tree count;
/// `.git/info/exclude` and the user's global excludes file are left out,
/// so every clone and CI agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isIgnored(view: View, matchers: Matchers, rel: string, isDir: boolean): boolean
/// ```
fn is_ignored(view: &mut View, matchers: &mut Matchers, rel: &str, is_dir: bool) -> Result<bool, Diagnostic> {
    let root = view.root().to_path_buf();
    let segments: Vec<&str> = rel.split('/').collect();
    let mut dirs: Vec<String> = vec![String::new()];
    let mut prefix = String::new();
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            prefix.push('/');
        }
        prefix.push_str(segment);
        let last = index + 1 == segments.len();
        let ignored = prefix_is_ignored(view, matchers, &dirs, &root, &prefix, !last || is_dir)?;
        if ignored {
            return Ok(true);
        }
        dirs.push(prefix.clone());
    }
    return Ok(false);
}

/// What:
///  Ask the `.gitignore` files of the enclosing directories,
/// deepest first,
/// whether one path prefix is ignored.
/// Why:
///  The first file with a matching line decides,
/// as in git;
/// a path no file mentions is not ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixIsIgnored(view: View, matchers: Matchers, dirs: string[], root: string, prefix: string, isDir: boolean): boolean
/// ```
fn prefix_is_ignored(view: &mut View, matchers: &mut Matchers, dirs: &[String], root: &Path, prefix: &str, is_dir: bool) -> Result<bool, Diagnostic> {
    let target = root.join(prefix);
    for dir in dirs.iter().rev() {
        ensure_matcher(view, matchers, dir)?;
        let Some(Some(matcher)) = matchers.get(dir) else { continue };
        match matcher.matched(&target, is_dir) {
            Match::Ignore(_) => return Ok(true),
            Match::Whitelist(_) => return Ok(false),
            Match::None => {}
        }
    }
    return Ok(false);
}

/// What:
///  Keep only the drift in the chosen scope.
/// Why:
///  Drift outside the scope is neither reported nor repaired and does not change the exit code.
///  A path that is tracked although a pattern matches it counts as local,
/// because the index is not consulted;
/// a plain `check` after the local outputs exist still sees it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectDrifts(view: View, drifts: Drift[], scope: OutputScope): Drift[]
/// ```
pub fn select_drifts(view: &mut View, drifts: Vec<Drift>, scope: OutputScope) -> Result<Vec<Drift>, Diagnostic> {
    if scope == OutputScope::All {
        return Ok(drifts);
    }
    let mut matchers: Matchers = BTreeMap::new();
    let mut kept: Vec<Drift> = Vec::new();
    for drift in drifts {
        let is_dir = drift.reason == DriftReason::ExtraDirectory;
        let local = is_ignored(view, &mut matchers, &drift.path, is_dir)?;
        if local == (scope == OutputScope::Local) {
            kept.push(drift);
        }
    }
    return Ok(kept);
}
