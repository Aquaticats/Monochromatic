//! What:
//!  The two questions the landing asks about a commit that Git built (a signed commit), answered from the object store without
//!  starting Git: which parents the commit has, and whether every object it adds to the branch is present.
//! Why:
//!  Each question was a `git rev-list` start (about five milliseconds warm, more when the host has evicted the Git executable)
//!  on the wait of every signed commit. A commit that has the branch tip as its only parent adds only the objects that differ
//!  from the tip's tree, which a walk of the two trees finds by reading the directories that changed. Only a proof is taken from
//!  here: whatever the walk cannot show (a repository with replacement objects, grafts or a shallow history, a commit with
//!  another header, an object that looks missing) is left to the `rev-list` the landing always ran, so a refusal is Git's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parents = parentsOf(commitName); // undefined: ask Git
//! const intact = addsOnlyPresentObjects(commitName, tipName); // false: ask Git
//! ```

use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{ObjectKind, from_hex, name_length, to_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{MODE_GITLINK, TreeEntry, is_tree_mode, parse_commit, parse_tree};
use std::path::Path;

/// How many tree entries one proof may examine before it gives up and leaves the question to Git.
///  Exempt from a citation under `LQM` (class: reader fallback): a walk past this bound makes the landing run the `rev-list`
///  it always ran, so a Git start is added and no result changes.
const ENTRY_BUDGET: usize = 1_000_000;

/// The deepest tree nesting one proof follows, as the readers of `repo_read_tree.rs` follow it.
const TREE_DEPTH_LIMIT: usize = 256;

/// The commit's header facts the landing needs.
#[derive(Debug, Eq, PartialEq)]
pub struct BuiltCommit {
    /// The parents, in order, as hexadecimal names.
    pub parents: Vec<String>,
    /// The root tree, as a binary name.
    tree: Vec<u8>,
}

/// The format of the names, from their length.
fn format_of(hex: &str) -> ObjectFormat {
    return if hex.len() == ObjectFormat::Sha256.hex_length() { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
}

/// Whether the repository has anything that makes `rev-list` show another history than the commit objects say.
fn history_is_rewritten(common_dir: &Path) -> bool {
    return common_dir.join("shallow").exists() || common_dir.join("info").join("grafts").exists();
}

/// What:
///  The parents of the commit `built`; `None` when Git is to say (the object is not a plain commit, or the repository's
///  history is rewritten by grafts, a shallow file or replacement objects).
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
    let format: ObjectFormat = format_of(built);
    let database: ObjectDatabase = ObjectDatabase::open(common_dir, None, format)?;
    let object = database.read(from_hex(built.as_bytes(), format)?.as_slice())?;
    if object.kind != ObjectKind::Commit {
        return None;
    }
    let fields = parse_commit(object.bytes.as_slice(), format)?;
    return Some(BuiltCommit { parents: fields.parents.iter().map(|parent| { return to_hex(parent.as_slice()); }).collect(), tree: fields.tree });
}

/// What the walk found.
#[derive(Debug, Eq, PartialEq)]
enum Found {
    /// Every object is present.
    Present,
    /// Something is absent or cannot be read here; Git decides.
    Unproven,
}

/// One walk: where objects are read, and how much it may still examine.
struct Proof<'a> {
    /// The object store.
    database: &'a ObjectDatabase,
    /// The object format.
    format: ObjectFormat,
    /// Entries left to examine.
    budget: usize,
}

impl Proof<'_> {
    /// The entries of the tree `name`, or `None` when it is absent, not a tree or unreadable.
    fn entries(&self, name: &[u8]) -> Option<Vec<TreeEntry>> {
        let object = self.database.read(name)?;
        if object.kind != ObjectKind::Tree {
            return None;
        }
        return parse_tree(object.bytes.as_slice(), name_length(self.format));
    }

    /// Check the tree `new` against `old` (the tree the branch tip has at the same place, when it has one): an entry equal on both
    ///  sides is reachable from the tip, every other blob must exist, and every other tree is read and checked the same way.
    fn tree(&mut self, new: &[u8], old: Option<&[u8]>, depth: usize) -> Found {
        if old == Some(new) {
            return Found::Present;
        }
        if depth > TREE_DEPTH_LIMIT {
            return Found::Unproven;
        }
        let Some(wanted) = self.entries(new) else { return Found::Unproven };
        let before: Vec<TreeEntry> = old.and_then(|name| { return self.entries(name); }).unwrap_or_default();
        if wanted.len() > self.budget {
            return Found::Unproven;
        }
        self.budget -= wanted.len();
        for entry in wanted {
            let same_name = before.iter().find(|candidate| { return candidate.name == entry.name; });
            if same_name.is_some_and(|candidate| { return candidate.id == entry.id && candidate.mode == entry.mode; }) || entry.mode == MODE_GITLINK {
                continue;
            }
            if is_tree_mode(entry.mode) {
                let below: Option<&[u8]> = same_name.filter(|candidate| { return is_tree_mode(candidate.mode); }).map(|candidate| { return candidate.id.as_slice(); });
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
///  Whether every object that the commit `built` adds to a branch at `tip` is present: the commit's tree and every tree and blob
///  below it that the tip's tree does not have the same.
/// Why:
///   `git rev-list --objects <built> --not <tip>` fails when an object it would list is missing, and Git moves a branch to a
///  commit without looking, so a prune that took an object after the commit was made must be seen before the branch moves. This
///  answers `true` only for a commit whose one parent is the tip, where the objects to check are those that differ from the tip's
///  tree; any other commit, and any object that looks absent, answers `false` so that Git decides.
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
    let Some(tip_fields) = (tip_object.kind == ObjectKind::Commit).then(|| { return parse_commit(tip_object.bytes.as_slice(), format); }).flatten() else { return false };
    let mut proof: Proof<'_> = Proof { database: &database, format, budget: ENTRY_BUDGET };
    return proof.tree(built.tree.as_slice(), Some(tip_fields.tree.as_slice()), 0) == Found::Present;
}
