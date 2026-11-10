//! What:
//!  The two questions the landing asks about a commit that Git built (a signed commit),
//! answered from the object store without
//!  starting Git:
//! which parents the commit has,
//! and whether every object it adds to the branch is present.
//! Why:
//!  Each question was a `git rev-list` start (about five milliseconds warm,
//! more when the host has evicted the Git executable)
//!  on the wait of every signed commit.
//! A commit that has the branch tip as its only parent adds only the objects that differ
//!  from the tip's tree,
//! which a walk of the two trees finds by reading the directories that changed.
//! Only a proof is taken from
//!  here:
//! whatever the walk cannot show (a repository with replacement objects,
//! grafts or a shallow history,
//! a commit with
//!  another header,
//! an object that looks missing) is left to the `rev-list` the landing always ran,
//! so a refusal is Git's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parents = parentsOf(commitName); // undefined: ask Git
//! const intact = addsOnlyPresentObjects(commitName, tipName); // false: ask Git
//! ```

use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{ObjectKind, from_hex, name_length, to_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{CommitFields, MODE_GITLINK, TreeEntry, is_tree_mode, parse_commit, parse_tree};
use crate::config_snapshot::ConfigSnapshot;
use crate::transaction_git::GitContext;
use std::collections::HashMap;
use std::path::Path;

/// How many tree entries one proof may examine before it gives up and leaves the question to Git.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a walk past this bound makes the landing run the `rev-list`
///  it always ran,
/// so a Git start is added and no result changes.
const ENTRY_BUDGET: usize = 1_000_000;

/// The deepest tree nesting one proof follows,
/// as the readers of `repo_read_tree.rs` follow it.
const TREE_DEPTH_LIMIT: usize = 256;

/// Counts one name comparison of this landing's proofs,
/// which a control reads to show the work grows with the directory and not with its square;
/// the release executable counts nothing.
#[cfg(test)]
fn note_name_step() {
    crate::landing_state_carried_by_its_task::with_the_landing_state(|state| state.name_steps.set(state.name_steps.get() + 1));
}

/// The release executable does not count name comparisons.
#[cfg(not(test))]
fn note_name_step() {}

/// The name comparisons the proofs of this landing have made so far.
#[cfg(test)]
pub(crate) fn name_steps() -> u64 {
    return crate::landing_state_carried_by_its_task::with_the_landing_state(|state| return state.name_steps.get());
}

/// The commit's header facts the landing needs.
#[derive(Debug, Eq, PartialEq)]
pub struct BuiltCommit {
    /// The commit's own name,
    /// as a binary name,
    /// which the proof looks up again when it runs.
    name: Vec<u8>,
    /// The parents,
    /// in order,
    /// as hexadecimal names.
    pub parents: Vec<String>,
    /// The root tree,
    /// as a binary name.
    tree: Vec<u8>,
}

/// The format of the names,
/// from their length.
fn format_of(hex: &str) -> ObjectFormat {
    return if hex.len() == ObjectFormat::Sha256.hex_length() { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
}

/// The environment variables that change which history,
/// which replacements or which object store Git reads.
const HISTORY_VARIABLES: [&str; 6] = [
    "GIT_SHALLOW_FILE",
    "GIT_GRAFT_FILE",
    "GIT_REPLACE_REF_BASE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// Whether the repository has anything that makes `rev-list` show another history than the commit objects say,
/// at the places
///  Git looks by default.
fn history_is_rewritten(common_dir: &Path) -> bool {
    return common_dir.join("shallow").exists() || common_dir.join("info").join("grafts").exists();
}

/// What:
///  Whether the overlay of the landing's Git context sets a variable of this name.
/// Why:
///  `history_is_plain` counts a variable the overlay sets the same as one the environment sets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git.overlay.some(([key]) => key === name)
/// ```
fn overlay_sets(git: &GitContext, name: &str) -> bool {
    for (key, _value) in git.overlay.iter() {
        if key.to_string_lossy() == name {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a global option of the invocation switches replacement objects off or changes where they are read.
/// Why:
///  `history_is_plain` refuses a prefix with `--no-replace-objects`,
/// `--replace-refs` or `core.useReplaceRefs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changesReplacements = (word: string) => { const t = word.toLowerCase(); return t === '--no-replace-objects' || t.startsWith('--replace-refs') || t.includes('core.usereplacerefs'); };
/// ```
fn changes_replacements(word: &std::ffi::OsString) -> bool {
    let text = word.to_string_lossy().to_ascii_lowercase();
    return text == "--no-replace-objects" || text.starts_with("--replace-refs") || text.contains("core.usereplacerefs");
}

/// What:
///  The parents of a commit as hexadecimal names.
/// Why:
///  The built commit carries its parents the way Git prints them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parents.map((parent) => toHex(parent))
/// ```
fn hex_of_parents(parents: &[Vec<u8>]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for parent in parents {
        names.push(to_hex(parent.as_slice()));
    }
    return names;
}

/// What:
///  Whether Git,
/// started the way the landing starts it,
/// shows the history the commit objects say.
/// Why:
///   The checks of a commit Git built are answered from the object store only then.
/// Git reads its history from the files at the
///  default places unless a variable names others (`GIT_SHALLOW_FILE`,
/// `GIT_GRAFT_FILE`),
/// shows replacements from a namespace a
///  variable names (`GIT_REPLACE_REF_BASE`) unless they are switched off (`GIT_NO_REPLACE_OBJECTS`,
/// `--no-replace-objects`,
///  `core.useReplaceRefs`),
/// and reads another object store when told (`GIT_OBJECT_DIRECTORY`,
/// `GIT_ALTERNATE_OBJECT_DIRECTORIES`).
/// The
///  environment is the service's own and the overlay of the landing's Git context;
/// the configuration is the snapshot the landing
///  read.
/// A configuration that could not be read,
/// a setting of any of these and a file at a default place all say "not plain".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot !== undefined && !HISTORY_VARIABLES.some(isSet) && !prefix.some(changesReplacements) && !hasKey('core.usereplacerefs') && !filesAtDefaultPlaces()
/// ```
pub fn history_is_plain(common_dir: &Path, git: &GitContext, snapshot: Option<&ConfigSnapshot>) -> bool {
    let Some(read) = snapshot else { return false };
    if history_is_rewritten(common_dir) {
        return false;
    }
    // A variable of the environment or of the overlay that names another place for Git's history is "not plain".
    for name in HISTORY_VARIABLES {
        if std::env::var_os(name).is_some() || overlay_sets(git, name) {
            return false;
        }
    }
    let prefix_changes_replacements: bool = git.global_prefix.iter().any(changes_replacements);
    if prefix_changes_replacements {
        return false;
    }
    for entry in read.entries().iter() {
        if entry.key == "core.usereplacerefs" {
            return false;
        }
    }
    return true;
}

/// What:
///  The commit's own facts a landing reads from the object store:
/// its parents,
/// its root tree and the message it carries,
/// which the reflog row repeats.
/// Why:
///   The reflog message is the first line of the commit's message,
/// and a commit Git built carries it only in the object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitContent = { parents: string[]; tree: Uint8Array; message: Uint8Array };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct CommitContent {
    /// The parents,
    /// in order,
    /// as hexadecimal names.
    pub parents: Vec<String>,
    /// The root tree,
    /// as a binary name.
    pub tree: Vec<u8>,
    /// The message:
    /// everything after the blank line that ends the header.
    pub message: Vec<u8>,
}

/// What:
///  The parents,
/// tree and message of one commit object,
/// read without starting Git;
/// `None` when the object is absent,
/// not a plain commit,
/// or written with an encoding Git would convert.
/// Why:
///   One read answers both questions the landing asks of a commit it built or landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readCommitContent = (commonDir, commit) => parseCommit(readObject(commonDir, commit));
/// ```
pub fn commit_content(common_dir: &Path, commit: &str) -> Option<CommitContent> {
    let format: ObjectFormat = format_of(commit);
    let database: ObjectDatabase = ObjectDatabase::open(common_dir, None, format)?;
    let object = database.read(from_hex(commit.as_bytes(), format)?.as_slice())?;
    if object.kind != ObjectKind::Commit {
        return None;
    }
    let fields = parse_commit(object.bytes.as_slice(), format)?;
    return Some(CommitContent { parents: hex_of_parents(&fields.parents), tree: fields.tree, message: fields.message });
}

/// What:
///  The parents of the commit `built`;
/// `None` when Git is to say (the object is not a plain commit,
/// or the repository's
///  history is rewritten by grafts,
/// a shallow file or replacement objects).
/// Why:
///   `git rev-list --parents --max-count=1 <commit>` prints the commit and these parents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readCommit(commonDir, built)?.parents
/// ```
pub fn read_commit(common_dir: &Path, built: &str) -> Option<BuiltCommit> {
    if history_is_rewritten(common_dir) {
        return None;
    }
    let content: CommitContent = commit_content(common_dir, built)?;
    return Some(BuiltCommit { name: from_hex(built.as_bytes(), format_of(built))?, parents: content.parents, tree: content.tree });
}

/// What the walk found.
#[derive(Debug, Eq, PartialEq)]
enum Found {
    /// Every object is present.
    Present,
    /// Something is absent or cannot be read here;
    /// Git decides.
    Unproven,
}

/// One walk:
/// where objects are read,
/// and how much it may still examine.
struct Proof<'a> {
    /// The object store.
    database: &'a ObjectDatabase,
    /// The object format.
    format: ObjectFormat,
    /// Entries left to examine.
    budget: usize,
}

impl Proof<'_> {
    /// The entries of the tree `name`,
    /// or `None` when it is absent,
    /// not a tree or unreadable.
    fn entries(&self, name: &[u8]) -> Option<Vec<TreeEntry>> {
        let object = self.database.read(name)?;
        if object.kind != ObjectKind::Tree {
            return None;
        }
        return parse_tree(object.bytes.as_slice(), name_length(self.format));
    }

    /// Check the tree `new` against `old` (the tree the branch tip has at the same place,
    /// when it has one):
    /// an entry equal on both
    ///  sides is reachable from the tip,
    /// every other blob must exist,
    /// and every other tree is read and checked the same way.
    fn tree(&mut self, new: &[u8], old: Option<&[u8]>, depth: usize) -> Found {
        if old == Some(new) {
            return Found::Present;
        }
        if depth > TREE_DEPTH_LIMIT {
            return Found::Unproven;
        }
        let Some(wanted) = self.entries(new) else { return Found::Unproven };
        // A directory that is not named, or cannot be read, has no entries to compare with.
        let before: Vec<TreeEntry> = match old {
            Some(name) => self.entries(name).unwrap_or_default(),
            None => Vec::new(),
        };
        // Both directories are paid for from the budget: a wide one is bounded by its width, whatever the other holds.
        let cost: usize = wanted.len().saturating_add(before.len());
        if cost > self.budget {
            return Found::Unproven;
        }
        self.budget -= cost;
        // The tip's entries are indexed by name once, so each new entry is matched with one lookup and the work grows with the
        //  two directories and not with their product.
        let mut by_name: HashMap<&[u8], &TreeEntry> = HashMap::with_capacity(before.len());
        for candidate in &before {
            note_name_step();
            by_name.insert(candidate.name.as_slice(), candidate);
        }
        for entry in wanted {
            note_name_step();
            let same_name: Option<&&TreeEntry> = by_name.get(entry.name.as_slice());
            if matches!(same_name, Some(candidate) if candidate.id == entry.id && candidate.mode == entry.mode) || entry.mode == MODE_GITLINK {
                continue;
            }
            if is_tree_mode(entry.mode) {
                let below: Option<&[u8]> = match same_name {
                    Some(candidate) if is_tree_mode(candidate.mode) => Some(candidate.id.as_slice()),
                    _ => None,
                };
                if self.tree(entry.id.as_slice(), below, depth + 1) == Found::Unproven {
                    return Found::Unproven;
                }
            } else if !self.database.contains(entry.id.as_slice()) {
                return Found::Unproven;
            }
        }
        return Found::Present;
    }
}

/// What:
///  Whether every object that the commit `built` adds to a branch at `tip` is present:
/// the commit's tree and every tree and blob
///  below it that the tip's tree does not have the same.
/// Why:
///   `git rev-list --objects <built> --not <tip>` fails when an object it would list is missing,
/// and Git moves a branch to a
///  commit without looking,
/// so a prune that took an object after the commit was made must be seen before the branch moves.
/// This
///  answers `true` only for a commit whose one parent is the tip,
/// where the objects to check are those that differ from the tip's
///  tree;
/// any other commit,
/// and any object that looks absent,
/// answers `false` so that Git decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// onlyParentIs(built, tip) && everyChangedObjectExists(built.tree, tip.tree)
/// ```
pub fn adds_only_present_objects(common_dir: &Path, built: &BuiltCommit, tip: &str) -> bool {
    if history_is_rewritten(common_dir) || built.parents.len() != 1 || built.parents[0] != tip {
        return false;
    }
    let format: ObjectFormat = format_of(tip);
    let Some(database) = ObjectDatabase::open(common_dir, None, format) else { return false };
    let Some(tip_bytes) = from_hex(tip.as_bytes(), format) else { return false };
    let Some(tip_object) = database.read(tip_bytes.as_slice()) else { return false };
    // Only a commit has fields to read.
    let tip_fields_read: Option<CommitFields> = if tip_object.kind == ObjectKind::Commit { parse_commit(tip_object.bytes.as_slice(), format) } else { None };
    let Some(tip_fields) = tip_fields_read else { return false };
    // The commit was read before the objects were checked, and an object can go in between: it is looked up again here.
    if !database.contains(built.name.as_slice()) {
        return false;
    }
    let mut proof: Proof<'_> = Proof { database: &database, format, budget: ENTRY_BUDGET };
    return proof.tree(built.tree.as_slice(), Some(tip_fields.tree.as_slice()), 0) == Found::Present;
}
