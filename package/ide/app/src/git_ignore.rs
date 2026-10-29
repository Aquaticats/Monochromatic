//! Which listed entries git would ignore,
//!  decided from the rule files alone:
//!  no git process and no read of the index.

/// Entries are the dirents of one directory snapshot.
use crate::workspace::DirectoryEntry;
/// What:
///  `Gitignore` is a compiled rule file;
///  `GitignoreBuilder` reads one;
///  `gitconfig_excludes_path` names the global excludes file from the user's git configuration;
///  `Match` is the three-way answer of one rule file (`None`,
///  `Ignore`,
///  `Whitelist`).
/// Why:
///  ripgrep's own crate parses the file format,
///  so negation,
///  directory-only patterns,
///  anchoring,
///  and `**` follow one maintained implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Gitignore, GitignoreBuilder, gitconfigExcludesPath, Match } from 'ignore';
/// ```
use ignore::{
    Match,
    gitignore::{Gitignore, GitignoreBuilder, gitconfig_excludes_path},
};
/// Native names stay as bytes;
///  paths are never turned into text.
use std::{
    collections::{BTreeSet, HashMap},
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Find the work tree above a directory.
mod repository;
/// Fingerprint a rule file by its metadata.
mod stamp;

/// Stamps of rule files travel with the answer.
pub use stamp::Stamp;

/// The most entries of one directory that are matched.
/// A directory with more entries shows nothing dimmed and logs why:
///  matching costs about a microsecond per entry and rule file,
///  and this bound keeps one huge directory from holding the single reader.
pub const MAX_ENTRIES: usize = 20_000;

/// The most rule files kept compiled at once;
///  past it the cache starts again,
///  so a long session cannot grow it without bound.
const MAX_CACHED_FILES: usize = 4_096;

/// How long the global excludes file's path is trusted before git's configuration is read again.
const GLOBAL_PATH_TTL: Duration = Duration::from_secs(1);

/// What:
///  Where the global excludes file (`core.excludesFile`) comes from.
///  `Option<PathBuf>` inside `File` is a path that may be absent.
/// Why:
///  The application reads git's configuration;
///  tests name a file,
///  so they never read the real home folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GlobalSource = { kind: 'gitConfig' } | { kind: 'file'; path: string | undefined };
/// ```
#[derive(Clone, Debug)]
pub enum GlobalSource {
    /// `core.excludesFile` of the global git configuration,
    ///  or `$XDG_CONFIG_HOME/git/ignore`.
    GitConfig,
    /// A fixed file,
    ///  or none.
    File(
        /// The path of the file,
        ///  or nothing for no global rules.
        Option<PathBuf>,
    ),
}

/// What:
///  Stamps of the rule files that decided one directory's answer.
///  `own` is the directory's own `.gitignore`;
///  `shared` is everything above and beside it.
/// Why:
///  A change of `own` can change the answer of every directory below,
///  and a change of `shared` the answer of every directory of the work tree,
///  so the tree rereads those too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleStamps = { own: Stamp; shared: { top?: string; exclude: Stamp; global?: [string, Stamp] } };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleStamps {
    /// The listed directory's own `.gitignore`.
    pub own: Stamp,
    /// The top of the work tree,
    ///  or `None` outside one.
    pub top: Option<PathBuf>,
    /// The work tree's `info/exclude`.
    pub exclude: Stamp,
    /// The global excludes file and its stamp.
    pub global: Option<(PathBuf, Stamp)>,
}

/// What:
///  One directory's answer:
///  the names of its entries that git ignores,
///  and the stamps of the rules that said so.
///  `BTreeSet<OsString>` is an ordered set of native names.
/// Why:
///  The answer travels with the listing through the same single-use request,
///  so an outdated answer is dropped with its outdated listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IgnoreAnswer = { names: Set<string>; stamps: RuleStamps };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IgnoreAnswer {
    /// Names of the entries that are ignored;
    ///  empty when nothing is,
    ///  and also when no answer could be given.
    pub names: BTreeSet<OsString>,
    /// Stamps of the rules behind `names`.
    pub stamps: RuleStamps,
}

/// One compiled rule file and what it was compiled from.
struct Compiled {
    /// Directory the file's patterns are relative to.
    root: PathBuf,
    /// The file's metadata when it was compiled.
    stamp: Stamp,
    /// The compiled patterns;
    ///  empty for a missing file.
    rules: Gitignore,
}

/// What:
///  Decides ignored entries for the one reader thread,
///  and keeps compiled rule files until their stamp changes.
///  `HashMap<PathBuf, Compiled>` is an unordered map from a rule file's path.
/// Why:
///  Compiling a `.gitignore` costs far more than matching against it,
///  and the reader lists the same directories every second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class IgnoreMatcher { evaluate(directory: string, entries: DirectoryEntry[]): IgnoreAnswer }
/// ```
pub struct IgnoreMatcher {
    /// Where the global excludes file comes from.
    source: GlobalSource,
    /// The global excludes path last read from git's configuration,
    ///  and when.
    global_path: Option<(Instant, Option<PathBuf>)>,
    /// Compiled rule files by path.
    files: HashMap<PathBuf, Compiled>,
    /// The last reason a directory showed nothing dimmed,
    ///  so a repeated reason is logged once.
    reasons: HashMap<PathBuf, String>,
}

/// Ask one compiled rule file;
///  the first file that has an opinion decides,
///  a negation included.
fn decide<'a>(rules: impl Iterator<Item = &'a Gitignore>, path: &Path, is_directory: bool) -> bool {
    for file in rules {
        match file.matched(path, is_directory) {
            Match::Ignore(_) => return true,
            Match::Whitelist(_) => return false,
            Match::None => {}
        }
    }
    return false;
}

/// What:
///  The compiled rules in decision order for a path inside the level at `depth`:
///  deeper `.gitignore` files first,
///  then `info/exclude`,
///  then the global file.
///  `&'a` ties the returned rules to the borrowed table of compiled files.
/// Why:
///  The first file with an opinion wins,
///  so the order is the decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rulesFor(depth: number, compiled: Map<string, Compiled>, files: string[], excludeFile: string, global?: [string, Stamp]): Gitignore[];
/// ```
fn rules_for<'a>(
    depth: usize,
    compiled_files: &'a HashMap<PathBuf, Compiled>,
    files: &[PathBuf],
    exclude_file: &Path,
    global: &Option<(PathBuf, Stamp)>,
) -> Vec<&'a Gitignore> {
    let mut found: Vec<&Gitignore> = Vec::new();
    for file in files[..depth].iter().rev() {
        if let Some(compiled) = compiled_files.get(file) {
            found.push(&compiled.rules);
        }
    }
    if let Some(compiled) = compiled_files.get(exclude_file) {
        found.push(&compiled.rules);
    }
    if let Some((path, _stamp)) = global
        && let Some(compiled) = compiled_files.get(path)
    {
        found.push(&compiled.rules);
    }
    return found;
}

/// Decisions read rule files only.
impl IgnoreMatcher {
    /// A matcher with nothing compiled yet.
    pub fn new(source: GlobalSource) -> Self {
        return Self {
            source,
            global_path: None,
            files: HashMap::new(),
            reasons: HashMap::new(),
        };
    }

    /// The global excludes path,
    ///  read from git's configuration at most once per `GLOBAL_PATH_TTL`.
    fn global(&mut self, now: Instant) -> Option<PathBuf> {
        match &self.source {
            GlobalSource::File(path) => return path.clone(),
            GlobalSource::GitConfig => {}
        }
        if let Some((read, path)) = &self.global_path
            && now.saturating_duration_since(*read) < GLOBAL_PATH_TTL
        {
            return path.clone();
        }
        let path = gitconfig_excludes_path();
        self.global_path = Some((now, path.clone()));
        return path;
    }

    /// Compile `file` with patterns relative to `root`,
    ///  unless the compiled copy still has the file's stamp.
    /// A missing file is an empty rule set;
    ///  a file with invalid lines keeps its valid ones,
    ///  and the invalid ones are logged once per stamp.
    fn compile(&mut self, file: &Path, root: &Path) -> Stamp {
        let stamp = Stamp::of(file);
        if let Some(found) = self.files.get(file)
            && found.stamp == stamp
            && found.root == root
        {
            return stamp;
        }
        if self.files.len() >= MAX_CACHED_FILES {
            tracing::info!(
                files = self.files.len(),
                "dropping every compiled ignore rule file; the cache is full"
            );
            self.files.clear();
        }
        let rules = if stamp.exists() {
            let mut builder = GitignoreBuilder::new(root);
            if let Some(error) = builder.add(file) {
                tracing::warn!(path = %file.display(), %error, "an ignore rule file has lines git would reject or could not be fully read");
            }
            match builder.build() {
                Ok(rules) => rules,
                Err(error) => {
                    tracing::warn!(path = %file.display(), %error, "cannot compile an ignore rule file; treating it as empty");
                    Gitignore::empty()
                }
            }
        } else {
            Gitignore::empty()
        };
        tracing::debug!(path = %file.display(), root = %root.display(), exists = stamp.exists(), patterns = rules.len(), "compiled an ignore rule file");
        self.files.insert(
            file.to_path_buf(),
            Compiled {
                root: root.to_path_buf(),
                stamp,
                rules,
            },
        );
        return stamp;
    }

    /// Log a reason once per directory,
    ///  and again when it changes.
    fn nothing_dimmed(&mut self, directory: &Path, reason: String) {
        if self.reasons.get(directory) == Some(&reason) {
            return;
        }
        if self.reasons.len() >= MAX_CACHED_FILES {
            self.reasons.clear();
        }
        tracing::info!(directory = %directory.display(), %reason, "no tree entry is dimmed in this directory");
        self.reasons.insert(directory.to_path_buf(), reason);
    }

    /// Which of `entries`,
    ///  listed from `directory`,
    ///  git would ignore.
    /// Outside a work tree,
    ///  or past `MAX_ENTRIES`,
    ///  the answer names nothing and the reason is logged.
    /// Matching follows git's order:
    ///  an ignored directory ignores everything inside it whatever the deeper rules say;
    ///  otherwise the deepest `.gitignore` decides first,
    ///  then `info/exclude`,
    ///  then the global excludes file,
    ///  and the first file with an opinion wins.
    /// Git's index is not read,
    ///  so a tracked file that matches a pattern is reported ignored,
    ///  where git would not.
    pub fn evaluate(&mut self, directory: &Path, entries: &[DirectoryEntry]) -> IgnoreAnswer {
        let Some(repository) = repository::find(directory) else {
            self.nothing_dimmed(directory, "no .git above this directory".to_string());
            return IgnoreAnswer::default();
        };
        // Top-first levels, from the work tree's top down to the listed directory.
        let mut levels: Vec<&Path> = Vec::new();
        for candidate in directory.ancestors() {
            levels.push(candidate);
            if candidate == repository.top {
                break;
            }
        }
        levels.reverse();
        let mut own = Stamp::default();
        let mut files: Vec<PathBuf> = Vec::new();
        for level in &levels {
            let file = level.join(".gitignore");
            own = self.compile(&file, level);
            files.push(file);
        }
        let exclude_file = repository.common_dir.join("info").join("exclude");
        let exclude = self.compile(&exclude_file, &repository.top);
        let global_file = self.global(Instant::now());
        let global = match global_file {
            Some(path) => {
                let stamp = self.compile(&path, &repository.top);
                Some((path, stamp))
            }
            None => None,
        };
        let stamps = RuleStamps {
            own,
            top: Some(repository.top.clone()),
            exclude,
            global: global.clone(),
        };
        if entries.len() > MAX_ENTRIES {
            self.nothing_dimmed(
                directory,
                format!(
                    "{} entries exceed the {MAX_ENTRIES} entry limit of ignore matching",
                    entries.len()
                ),
            );
            return IgnoreAnswer {
                names: BTreeSet::new(),
                stamps,
            };
        }
        self.reasons.remove(directory);
        // A directory is ignored when its parent is,
        // or when the rules above it say so;
        // git never re-includes a file below an ignored directory.
        let mut inside_ignored = false;
        for (depth, level) in levels.iter().enumerate().skip(1) {
            let rules = rules_for(depth, &self.files, &files, &exclude_file, &global);
            if decide(rules.into_iter(), level, true) {
                inside_ignored = true;
                break;
            }
        }
        let mut names = BTreeSet::new();
        if inside_ignored {
            for entry in entries {
                names.insert(entry.name.clone());
            }
        } else {
            let rules = rules_for(levels.len(), &self.files, &files, &exclude_file, &global);
            for entry in entries {
                if decide(rules.iter().copied(), &entry.path, entry.is_directory) {
                    names.insert(entry.name.clone());
                }
            }
        }
        tracing::debug!(
            directory = %directory.display(),
            entries = entries.len(),
            ignored = names.len(),
            inside_ignored,
            top = %repository.top.display(),
            "decided which entries git ignores"
        );
        return IgnoreAnswer { names, stamps };
    }
}

/// Decisions are checked against real rule files in disposable folders.
#[cfg(test)]
#[path = "git_ignore_tests.rs"]
mod tests;
