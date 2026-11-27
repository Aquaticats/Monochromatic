//! What:
//!  One scan's reads of the objects a push introduces:
//! the commits it has parsed and when they were committed,
//! the tree comparisons it has made,
//! and the blobs a comparison found new.
//! Why:
//!  The walk in `lfs_pointer_scan.rs` asks a scan for a commit,
//! its committer time and one comparison,
//! and the scan reads each object once and remembers the answer,
//! so a merge's shared history is read a constant number of times.
//! The two files pass the package's length limit only apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = Scan.of(database, format, budget); const commit = scan.commit(name); scan.compare(tree, parents);
//! ```

/// The object format the names and the entries are read in.
use crate::repo_read_layout::ObjectFormat;
/// The kind of an object,
/// and the length of a name in this format.
use crate::repo_read_object::{
    ObjectKind,
    name_length,
};
/// The objects themselves.
use crate::repo_read_odb::ObjectDatabase;
/// The parsed commit and tree,
/// and the modes an entry can have.
use crate::repo_read_tree::{
    CommitFields,
    MODE_GITLINK,
    is_tree_mode,
    parse_commit,
    parse_tree,
};
/// The commits by name,
/// the committer times,
/// the comparisons made,
/// and the blobs found new.
use std::collections::{
    BTreeSet,
    HashMap,
    HashSet,
};
/// When the scan gives up.
use std::time::{
    Duration,
    Instant,
};
/// What one scan reads and remembers.
pub(crate) struct Scan<'a> {
    /// The objects.
    database: &'a ObjectDatabase,
    /// The object format.
    format: ObjectFormat,
    /// Parsed commits by name.
    commits: HashMap<Vec<u8>, CommitFields>,
    /// Committer times of the parsed commits,
    /// in seconds.
    times: HashMap<Vec<u8>, i64>,
    /// Comparisons already made:
    /// a tree followed by the parent trees it was compared with.
    /// The same tree compared with
    ///  other parents (a root commit holding what a later commit also holds) is a different comparison.
    seen_trees: HashSet<Vec<Vec<u8>>>,
    /// Blobs that are new relative to some parent,
    /// by name.
    candidates: BTreeSet<Vec<u8>>,
    /// When the scan gives up.
    deadline: Instant,
}

impl<'a> Scan<'a> {
    /// What:
    ///  A scan of `database` in `format` that gives up `budget` from now.
    /// Why:
    ///   The maps a scan fills are its own,
    /// and the walk that asks for the answers only reads them,
    /// so the one place that makes a scan is in the file that holds its fields.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Scan.of(database, format, budget); // { commits: new Map(), times: new Map(), ... }
    /// ```
    pub(crate) fn of(
        database: &'a ObjectDatabase,
        format: ObjectFormat,
        budget: Duration,
    ) -> Scan<'a> {
        // What:     `Scan { ... }` builds one scan with empty maps and a deadline `budget` from now.
        // Why:      The walk fills them; nothing about one upload is decided here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { database, format, commits: new Map(), times: new Map(), seenTrees: new Set(), candidates: new Set(), deadline: Date.now() + budgetMs };
        // ```
        return Scan {
            database,
            format,
            commits: HashMap::new(),
            times: HashMap::new(),
            seen_trees: HashSet::new(),
            candidates: BTreeSet::new(),
            deadline: Instant::now() + budget,
        };
    }

    /// The parsed commit `name`;
    /// `None` when it is missing,
    /// not a commit,
    /// or in a spelling the reader declines.
    pub(crate) fn commit(
        &mut self,
        name: &[u8],
    ) -> Option<CommitFields> {
        if let Some(known) = self.commits.get(name) {
            return Some(known.clone());
        }
        let object = self.database.read(name)?;
        if object.kind != ObjectKind::Commit {
            return None;
        }
        let parsed: CommitFields = parse_commit(object.bytes.as_slice(), self.format)?;
        self.times
            .insert(name.to_vec(), committer_time(object.bytes.as_slice())?);
        self.commits.insert(name.to_vec(), parsed.clone());
        return Some(parsed);
    }

    /// The committer time of a commit already read.
    pub(crate) fn time(
        &self,
        name: &[u8],
    ) -> i64 {
        return self.times.get(name).copied().unwrap_or_default();
    }

    /// The entries of the tree `name`.
    fn tree(
        &self,
        name: &[u8],
    ) -> Option<Vec<crate::repo_read_tree::TreeEntry>> {
        let object = self.database.read(name)?;
        if object.kind != ObjectKind::Tree {
            return None;
        }
        return parse_tree(object.bytes.as_slice(), name_length(self.format));
    }

    /// The blobs a comparison found new,
    /// by name.
    pub(crate) fn candidates(&self) -> &BTreeSet<Vec<u8>> {
        return &self.candidates;
    }

    /// Whether the scan has run out of its budget.
    pub(crate) fn past_its_deadline(&self) -> bool {
        return Instant::now() >= self.deadline;
    }

    /// Record the blobs of `tree` that none of the `parents` (trees at the same place) holds under the same name.
    pub(crate) fn compare(
        &mut self,
        tree: &[u8],
        parents: &[Vec<u8>],
    ) -> Option<()> {
        if Instant::now() >= self.deadline {
            return None;
        }
        let mut parent_is_tree: bool = false;
        for parent in parents {
            if parent.as_slice() == tree {
                parent_is_tree = true;
                break;
            }
        }
        // The pair is recorded as seen only when no parent is the tree itself, as `||` stopped at the first true side.
        if parent_is_tree
            || !self
                .seen_trees
                .insert([vec![tree.to_vec()], parents.to_vec()].concat())
        {
            return Some(());
        }
        let entries = self.tree(tree)?;
        let mut parent_entries = Vec::with_capacity(parents.len());
        for parent in parents {
            parent_entries.push(self.tree(parent)?);
        }
        for entry in &entries {
            if entry.mode == MODE_GITLINK {
                continue;
            }
            // The entry of the same name in each parent that has one, in parent order.
            let mut counterparts: Vec<&crate::repo_read_tree::TreeEntry> = Vec::new();
            for list in &parent_entries {
                for candidate in list {
                    if candidate.name == entry.name {
                        counterparts.push(candidate);
                        break;
                    }
                }
            }
            if is_tree_mode(entry.mode) {
                let mut below: Vec<Vec<u8>> = Vec::new();
                for other in &counterparts {
                    if is_tree_mode(other.mode) {
                        below.push(other.id.clone());
                    }
                }
                self.compare(entry.id.as_slice(), below.as_slice())?;
            } else {
                let mut held_by_a_parent: bool = false;
                for other in &counterparts {
                    if other.id == entry.id && !is_tree_mode(other.mode) {
                        held_by_a_parent = true;
                        break;
                    }
                }
                if !held_by_a_parent {
                    self.candidates.insert(entry.id.clone());
                }
            }
        }
        return Some(());
    }
}

/// Whether one byte is the line feed that ends a line of a commit.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether one byte is the `>` that closes the email address of a committer line.
/// `&u8` borrows the byte,
/// as
///  `rposition` hands it over.
fn is_closing_angle_byte(byte: &u8) -> bool {
    return *byte == b'>';
}

/// The committer time in seconds of a commit object's content.
fn committer_time(bytes: &[u8]) -> Option<i64> {
    let mut header_end_found: Option<usize> = None;
    for (position, pair) in bytes.windows(2).enumerate() {
        if pair == b"\n\n" {
            header_end_found = Some(position);
            break;
        }
    }
    let header_end: usize = header_end_found?;
    let mut committer_line: Option<&[u8]> = None;
    for line in bytes[..header_end].split(is_line_feed_byte) {
        if line.starts_with(b"committer ") {
            committer_line = Some(line);
            break;
        }
    }
    let line: &[u8] = committer_line?;
    let after_email: &[u8] = &line[line.iter().rposition(is_closing_angle_byte)? + 1..];
    let text: &str = std::str::from_utf8(after_email).ok()?;
    return text.split_whitespace().next()?.parse::<i64>().ok();
}
