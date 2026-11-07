//! What:
//!  The intent of a batch:
//! the record of what a landing is about to do,
//! written as `key=value` lines and read back,
//! with the
//!  parts a reader needs (request identities,
//! commits,
//! paths,
//! index,
//! Git directory,
//! cleanup).
//! Why:
//!  The journal (`service_journal.rs`) writes,
//! lists and prunes files of intents;
//! the text form needs no file,
//! so it is a file the
//!  journal uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = intentText(intent); const intent2 = parseIntent(text); // undefined when damaged
//! ```

use crate::repo_read_layout::dot_git_target;
use crate::service_protocol::Cleanup;
use std::path::{Path, PathBuf};
/// What:
///  What the service decided to do for a batch,
/// written before the branch moves.
/// Why:
///   Enough to finish the batch after a kill:
/// where the branch was,
/// where it goes,
/// which requests,
/// which index paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Intent = { seq; repository; commonDir; reference; old; new; ids; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    /// Number of the batch,
    /// unique and increasing within one journal.
    pub seq: u64,
    /// Worktree root whose index the batch updates.
    pub repository: PathBuf,
    /// Common git directory.
    pub common_dir: PathBuf,
    /// The branch.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The branch after the batch.
    pub new: String,
    /// Identities of the requests in the batch.
    pub ids: Vec<String>,
    /// The commit each request became,
    /// in the same order as the identities.
    pub commits: Vec<String>,
    /// Paths whose index entries the batch updates.
    pub paths: Vec<String>,
    /// Whether the index entries to update are the paths that differ between `old` and `new`,
    /// read from the repository
    ///  when they are needed (a commit Git built has no list of changes to carry,
    /// and its names are bytes).
    pub derived: bool,
    /// The conclusion state the landing removes after the branch moved;
    /// nothing for any other batch.
    pub cleanup: Option<Cleanup>,
    /// The real index whose lock the landing held,
    /// so a restart can remove its own stale lock and finish the index.
    pub index: PathBuf,
    /// The worktree's own Git directory,
    /// which names the instance that serves the batch;
    /// an intent written before it was
    ///  recorded reads as the Git directory of its repository's worktree.
    pub git_dir: PathBuf,
}

/// The text of an intent:
/// `key=value` lines,
/// tabs between list items.
pub fn intent_text(intent: &Intent) -> String {
    let cleanup: String = match intent.cleanup.as_ref() {
        Some(found) => format!("{}\t{}\t{}", found.git_dir.display(), found.scratch.display(), u8::from(found.reftable)),
        None => String::new(),
    };
    return format!(
        "seq={}\nrepository={}\ncommon={}\nref={}\nold={}\nnew={}\nids={}\ncommits={}\npaths={}\nderived={}\ncleanup={}\nindex={}\ngitdir={}\n",
        intent.seq,
        intent.repository.display(),
        intent.common_dir.display(),
        intent.reference,
        intent.old,
        intent.new,
        intent.ids.join("\t"),
        intent.commits.join("\t"),
        intent.paths.join("\t"),
        u8::from(intent.derived),
        cleanup,
        intent.index.display(),
        intent.git_dir.display(),
    );
}

/// What:
///  The worktree's own Git directory,
/// read from the worktree's `.git` entry:
/// the entry itself when it is a directory,
/// the
///  directory a `gitdir:` file names for a linked worktree,
/// and `.git` under the worktree when it cannot be read.
/// Why:
///   An intent written before it recorded its Git directory has only the repository to name the worktree by;
/// the worktree's
///  instance is the hash of this directory,
/// so the recovery that finds the intent must resolve it the way a commit does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDirOfWorktree('/work/repo') // '/work/repo/.git', or '/work/main/.git/worktrees/repo' for a linked worktree
/// ```
pub(crate) fn git_dir_of_worktree(repository: &Path) -> PathBuf {
    return match dot_git_target(repository) {
        Some(Some((git_dir, _))) => git_dir,
        Some(None) | None => repository.join(".git"),
    };
}

/// What:
///  The value of the first `key=value` line of an intent text whose key is `key`.
/// Why:
///  `parse_intent` reads every field of the intent this way;
/// a line with the key but no equals sign is not the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').map((line) => line.startsWith(key + '=') ? line.slice(key.length + 1) : undefined).find((value) => value !== undefined)
/// ```
pub(crate) fn intent_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(key)
            && let Some(found) = rest.strip_prefix('=')
        {
            return Some(found);
        }
    }
    return None;
}

/// What:
///  The tab-separated items of an intent field,
/// empty items left out.
/// Why:
///  The request identities,
/// commits and paths of an intent are lists in one line each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value(key)?.split('\t').filter((item) => item !== '')
/// ```
pub(crate) fn intent_list(text: &str, key: &str) -> Option<Vec<String>> {
    let found: &str = intent_value(text, key)?;
    let mut items: Vec<String> = Vec::new();
    for item in found.split('\t') {
        if !item.is_empty() {
            items.push(item.to_string());
        }
    }
    return Some(items);
}

/// What:
///  The cleanup of an intent from its field:
/// the Git directory,
/// the scratch directory and the reftable flag,
/// tab-separated.
/// Why:
///  `parse_intent` hands it to `Option::and_then`;
/// a field with fewer than three parts is no cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [gitDir, scratch, reftable] = entry.split('\t'); return reftable === undefined ? undefined : { gitDir, scratch, reftable: reftable === '1' };
/// ```
pub(crate) fn parse_cleanup(entry: &str) -> Option<Cleanup> {
    let mut parts = entry.split('\t');
    let (git_dir, scratch, reftable) = (parts.next()?, parts.next()?, parts.next()?);
    return Some(Cleanup { git_dir: PathBuf::from(git_dir), scratch: PathBuf::from(scratch), reftable: reftable == "1" });
}

/// What:
///  Reads an intent's text.
/// Why:
///   A damaged file is no intent:
/// recovery drops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIntent(text) // undefined when damaged
/// ```
pub fn parse_intent(text: &str) -> Option<Intent> {
    // An intent written before the index was recorded names it as the repository's own.
    let index: PathBuf = match intent_value(text, "index") {
        Some(named) => PathBuf::from(named),
        None => PathBuf::from(intent_value(text, "repository").unwrap_or("")).join(".git").join("index"),
    };
    let repository: PathBuf = PathBuf::from(intent_value(text, "repository")?);
    // An intent written before the Git directory was recorded names its worktree by the repository it was made for, never
    //  by the directory its index sits in, which `GIT_INDEX_FILE` can put anywhere.
    let worktree_git_dir: PathBuf = match intent_value(text, "gitdir") {
        Some(named) => PathBuf::from(named),
        None => git_dir_of_worktree(repository.as_path()),
    };
    return Some(Intent {
        seq: intent_value(text, "seq")?.parse().ok()?,
        repository,
        common_dir: PathBuf::from(intent_value(text, "common")?),
        reference: intent_value(text, "ref")?.to_string(),
        old: intent_value(text, "old")?.to_string(),
        new: intent_value(text, "new")?.to_string(),
        ids: intent_list(text, "ids")?,
        commits: intent_list(text, "commits")?,
        paths: intent_list(text, "paths")?,
        // Intents written before these fields existed name neither.
        derived: intent_value(text, "derived") == Some("1"),
        index,
        git_dir: worktree_git_dir,
        cleanup: intent_value(text, "cleanup").and_then(parse_cleanup),
    });
}
