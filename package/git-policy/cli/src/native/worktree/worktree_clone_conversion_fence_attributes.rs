//! What:
//!  The attributes a conversion fence reads:
//! which attributes file above a tracked path is untracked,
//! what Git answers for each tracked path,
//! the conversion rules the answer is judged by,
//! and the half of the fence those answers decide before anything is copied.
//! Why:
//!  Git resolves which attributes file applies,
//! in which order,
//! and what a macro sets,
//! so the fence never matches a pattern itself and every source Git reads is answered by Git.
//! A tracked attributes file is the tree's and is read by both worktrees;
//! an untracked one is the source's alone,
//! and no answer Git gives today can prove what it said when the source's files were added.
//! This half is what a declining source pays for:
//! one `git check-attr` and the count over its answer,
//! before a blob is read and before anything is copied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = fenceTheAttributes(source, context); // throws the reason the rung cannot be proven
//! refuseUntrackedAttributes(source); // the same refusal, decided before anything is read
//! ```

use crate::transaction_git::{
    GitContext,
    GitOutput,
};
/// The change a rung cannot serve,
/// and why,
/// re-exported so every caller of this fence names it here.
pub use crate::worktree_change_a_rung_cannot_serve::{
    ChangeNotServed,
    WhyNotServed,
};
use crate::worktree_clean::{
    StatEntry,
    stored_stats,
};
use crate::worktree_clone_conversion_decline::{
    MODE_SYMLINK,
    MODE_TYPE_MASK,
    converting_paths,
    declines,
};
use crate::worktree_clone_conversion_git_reads::{
    run_in,
    run_reason,
    words,
};
use crate::worktree_clone_conversion_rules::{
    ConversionRules,
    PathAttributes,
    apply_attribute,
    checkout_converts,
    conversion_rules_from_config,
    needs_blob_size,
};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{
    HashMap,
    HashSet,
};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// The name of the attributes file whose untracked presence refuses the rung.
const ATTRIBUTES_NAME: &str = ".gitattributes";

/// What:
///  The first directory above a tracked path that holds an attributes file the index does not track,
/// as the reason the rung ends.
/// Why:
///   Git reads such a file for the paths beneath it,
/// the new worktree will not have it,
/// and the source's files may have been made under a rule that is gone
///  (the gate review's finding L3).
/// Only the ancestors of tracked paths are looked at,
/// so the answer costs no walk of the worktree's ignored directories;
/// the root is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstUntrackedAttributes(entries, root): string | undefined;
/// ```
pub(crate) fn first_untracked_attributes(
    entries: &[StatEntry],
    root: &Path,
) -> Option<String> {
    let mut tracked: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        if is_attributes_path(entry.path.as_slice()) {
            tracked.insert(entry.path.clone());
        }
    }
    let mut looked: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        let mut directory: Vec<u8> = Vec::new();
        let mut boundary: usize = 0;
        loop {
            if looked.insert(directory.clone())
                && let Some(reason) = attributes_file_at(root, directory.as_slice(), &tracked)
            {
                return Some(reason);
            }
            let Some(next) = entry.path[boundary..]
                .iter()
                .position(|byte| return *byte == b'/')
            else {
                break;
            };
            boundary += next + 1;
            directory = entry.path[..boundary - 1].to_vec();
        }
    }
    return None;
}

/// Whether a path names an attributes file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAttributes = (path) => path === '.gitattributes' || path.endsWith('/.gitattributes');
/// ```
pub(crate) fn is_attributes_path(path: &[u8]) -> bool {
    return path == ATTRIBUTES_NAME.as_bytes()
        || path.ends_with(format!("/{ATTRIBUTES_NAME}").as_bytes());
}

/// What:
///  Whether the directory below `root` holds an attributes file the index does not track,
/// named as the reason the rung ends.
/// Why:
///   A tracked one is the tree's and is read by both worktrees;
/// an untracked one is the source's alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesFileAt(root, directory, tracked): string | undefined;
/// ```
fn attributes_file_at(
    root: &Path,
    directory: &[u8],
    tracked: &HashSet<Vec<u8>>,
) -> Option<String> {
    let mut path: Vec<u8> = directory.to_vec();
    if !path.is_empty() {
        path.push(b'/');
    }
    path.extend_from_slice(ATTRIBUTES_NAME.as_bytes());
    if tracked.contains(path.as_slice()) {
        return None;
    }
    let on_disk: PathBuf = root.join(OsStr::from_bytes(path.as_slice()));
    if std::fs::symlink_metadata(on_disk.as_path()).is_err() {
        return None;
    }
    return Some(format!(
        "{} is in the source and not in its index, so the rules Git read when its tracked files were added cannot be known",
        String::from_utf8_lossy(path.as_slice())
    ));
}

/// What:
///  One `git check-attr --all -z --stdin` over every tracked path,
/// as the attributes of each path.
/// Why:
///   Git resolves which attributes file applies,
/// in which order,
/// and what a macro sets,
/// so the fence never matches a pattern itself;
/// a path Git does not answer for keeps the defaults,
/// which the configuration then decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesOf(context, root, entries): Map<string, PathAttributes>;
/// ```
fn attributes_of(
    context: &GitContext,
    root: &Path,
    entries: &[StatEntry],
) -> Result<HashMap<Vec<u8>, PathAttributes>, String> {
    let mut input: Vec<u8> = Vec::new();
    for entry in entries {
        input.extend_from_slice(entry.path.as_slice());
        input.push(0);
    }
    let output: GitOutput = run_in(
        context,
        root,
        &words(&["check-attr", "--all", "-z", "--stdin"]),
        Some(input),
        Vec::new(),
    )?;
    if output.code != Some(0) {
        return Err(format!(
            "the attributes of the source cannot be read: {}",
            run_reason("`git check-attr`", &output)
        ));
    }
    let mut attributes: HashMap<Vec<u8>, PathAttributes> = HashMap::new();
    let mut fields = output.stdout.split(|byte| return *byte == 0);
    loop {
        let (Some(path), Some(name), Some(value)) = (fields.next(), fields.next(), fields.next())
        else {
            break;
        };
        if path.is_empty() {
            break;
        }
        let held: &mut PathAttributes = attributes.entry(path.to_vec()).or_default();
        apply_attribute(held, name, value);
    }
    return Ok(attributes);
}

/// What:
///  Refuse a source with an untracked or ignored `.gitattributes` above a tracked path,
///  with the reason the rung cannot be proven.
/// Why:
///   Git read that file when the source's files were added and the new worktree will not have it,
/// so no answer Git gives today
///  can prove what it said then.
/// The walk costs no Git run and no walk of the ignored directories,
/// so the rung takes it before it copies anything,
///  beside the other cheap refusals;
/// the fence's own reads then run where the shape is,
///  before or beside the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseUntrackedAttributes(source); // throws the reason
/// ```
pub fn refuse_untracked_attributes(source: &SnapshotSource) -> Result<(), String> {
    let Some(entries) = stored_stats(
        source.clean.index_bytes.as_slice(),
        source.clean.hash_length,
    ) else {
        return Err(String::from("the source's index changed shape"));
    };
    return match first_untracked_attributes(entries.as_slice(), source.root.as_path()) {
        Some(reason) => Err(reason),
        None => Ok(()),
    };
}

/// What:
///  The half of the fence the attributes alone decide:
/// the tracked entries,
/// the attributes Git answers for each of them,
/// the conversion rules,
///  and nothing that needs a blob.
/// Why:
///   This is the half that a decline needs,
/// and the decline must be decided before the first copy
///  (`doc/planning/cli-git-clone-rung-conversion-fence.md`,
/// option B's first step):
/// its two reads are `git check-attr` and the count over its answer,
///  so a source above the quarter costs the add one Git checkout and this query and no copy at all.
/// The rest of the fence needs `git ls-tree`,
/// which is the expensive read,
///  so it stays where the copies are (`decide_the_paths`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeFence = { entries: StatEntry[]; attributes: Map<string, PathAttributes>; rules: ConversionRules };
/// ```
#[derive(Clone)]
pub struct AttributeFence {
    /// The tracked entries of the source's frozen index,
    /// in its order.
    pub(crate) entries: Vec<StatEntry>,
    /// The attributes Git answered for each path.
    pub(crate) attributes: HashMap<Vec<u8>, PathAttributes>,
    /// The conversion rules the answer is judged by.
    pub(crate) rules: ConversionRules,
    /// Whether any changed path could change this fence's answer:
    /// a path a rule of its own converts,
    ///  or a link whose copy no index entry can vouch for.
    pub(crate) a_changed_path_could_matter: bool,
}

impl AttributeFence {
    /// What:
    ///  Whether a changed path could change this fence's answer at all,
    ///  so one read of the target tree could end the rung.
    /// Why:
    ///   The skip's read is one Git run,
    /// and in a repository no rule converts and no link is tracked no changed path can be one the rung cannot serve,
    /// so the read buys nothing and is not paid.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const matters = (read) => read.converting > 0 || read.hasALink;
    /// ```
    pub fn a_changed_path_could_matter(&self) -> bool {
        return self.a_changed_path_could_matter;
    }

    /// What:
    ///  Whether any tracked path's decision needs a blob size at all.
    /// Why:
    ///   The sizes are one `git ls-tree`,
    /// the dear read of this fence,
    ///  and only a link or a path an attribute rule hands to Git is decided without one:
    /// a path with no rule still needs its blob's size,
    ///  because its file may hold the form of a rule that is gone today
    ///  (`needs_blob_size`).
    /// So an ordinary source pays this read too,
    /// and the answer is nothing only for a source of links and converting paths,
    ///  where a read started earlier is not waited for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const needsSizes = (read) => read.entries.some((entry) => !isLink(entry) && needsBlobSize(read.attributes.get(entry.path), read.rules));
    /// ```
    pub fn needs_sizes(&self) -> bool {
        return self.entries.iter().any(|entry| {
            let held: PathAttributes = self
                .attributes
                .get(entry.path.as_slice())
                .cloned()
                .unwrap_or_default();
            return entry.mode & MODE_TYPE_MASK != MODE_SYMLINK
                && needs_blob_size(&held, &self.rules);
        });
    }

    /// What:
    ///  The bytes the clones copy:
    /// the recorded sizes of the tracked files whose attributes leave them to the size proof,
    /// and of every link.
    /// Why:
    ///   A clone copies the working files the fence may carry and no other,
    /// so a limit on the bytes a clone copies must leave out every path an attribute rule hands to Git:
    /// a filter such as `filter=lfs` makes the index record the file's own bytes while the blob holds a pointer,
    /// and this machine's own checkout records 244 MB of such bytes out of 770 MB
    /// (the release pass over alpha.45's finding M2).
    /// A path whose file changed or whose times are stale is counted here as its index records it,
    /// which the partial clone then writes with Git rather than copying,
    /// so the sum is the carried bytes as the attributes and the index alone say them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bytes = (read) => read.entries.filter((entry) => isLink(entry) || needsBlobSize(read.attributes.get(entry.path), read.rules)).reduce((sum, entry) => sum + entry.stored.size, 0);
    /// ```
    pub fn bytes_the_clones_carry(&self) -> u64 {
        let mut total: usize = 0;
        for entry in &self.entries {
            let held: PathAttributes = self
                .attributes
                .get(entry.path.as_slice())
                .cloned()
                .unwrap_or_default();
            if entry.mode & MODE_TYPE_MASK == MODE_SYMLINK || needs_blob_size(&held, &self.rules) {
                total += entry.stored.size as usize;
            }
        }
        return total as u64;
    }

    /// What:
    ///  The first of `changed` a clone of this source cannot serve,
    ///  in the source's index order,
    ///  and why,
    ///  or nothing when every changed path is one the clones make and vouch for.
    /// Why:
    ///   This is the half of the fence that is decided before the copies,
    /// so it is what lets the ladder see a target tree its tree difference would refuse before it claims anything:
    /// a path here whose entry the fence installs without stat data,
    ///  or whose link the copies make again,
    ///  is a path `git read-tree -m -u` refuses.
    /// A path the source's index does not hold has no entry to refuse,
    ///  so the search runs over the entries and asks which of them changed,
    ///  not over the changed paths and their attributes,
    ///  because a path Git answers nothing for has no attribute entry at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const firstNotServed = (read, changed) => read.entries.find((entry) => changed.has(entry.path) && (isLink(entry) || checkoutConverts(read.attributes.get(entry.path), read.rules)));
    /// ```
    pub fn first_change_it_cannot_serve(
        &self,
        changed: &[Vec<u8>],
    ) -> Option<ChangeNotServed> {
        if changed.is_empty() {
            return None;
        }
        let changed_paths: HashSet<&[u8]> =
            changed.iter().map(|path| return path.as_slice()).collect();
        for entry in &self.entries {
            if !changed_paths.contains(entry.path.as_slice()) {
                continue;
            }
            if entry.mode & MODE_TYPE_MASK == MODE_SYMLINK {
                return Some(ChangeNotServed {
                    path: entry.path.clone(),
                    why: WhyNotServed::ALinkIsCopied,
                });
            }
            let held: PathAttributes = self
                .attributes
                .get(entry.path.as_slice())
                .cloned()
                .unwrap_or_default();
            if checkout_converts(&held, &self.rules) {
                return Some(ChangeNotServed {
                    path: entry.path.clone(),
                    why: WhyNotServed::TheFenceWritesIt,
                });
            }
        }
        return None;
    }
}

/// What:
///  Read the attributes of every tracked path and decide the decline,
/// before anything is copied.
/// Why:
///   The cheap refusal comes first (no Git run),
/// then one `git check-attr` and the count of the paths a rule of
///  their own would convert;
/// a source above the quarter ends the rung here,
/// which is what makes a declined add one Git checkout plus this query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceTheAttributes(source, context): AttributeFence; // throws the reason the rung cannot be proven
/// ```
pub fn fence_the_attributes(
    source: &SnapshotSource,
    context: &GitContext,
) -> Result<AttributeFence, String> {
    // The index is read once here and both the untracked-attributes refusal and the attributes walk judge those entries;
    // a second read of this repository's 24,648 entries costs about 2 ms on every add.
    let Some(entries) = stored_stats(
        source.clean.index_bytes.as_slice(),
        source.clean.hash_length,
    ) else {
        return Err(String::from("the source's index changed shape"));
    };
    if let Some(reason) = first_untracked_attributes(entries.as_slice(), source.root.as_path()) {
        return Err(reason);
    }
    let rules: ConversionRules = conversion_rules_from_config(source.config.entries());
    let attributes: HashMap<Vec<u8>, PathAttributes> =
        attributes_of(context, source.root.as_path(), entries.as_slice())?;
    // What:
    //  The decline rule: how many paths a rule of their own would convert is known from the attributes alone, before any
    //  blob is read and before anything is cloned.
    // Why:
    //  Writing many paths through Git makes the fence slower than `main`'s own checkout (the converting source measured
    //  1.50 times `main` without the rule, `measure-r5-crlf-*`),
    //  and the rule is decided here, before the add's record is claimed and before real Git runs, so a declining source
    //  pays only these reads and Git's own single checkout (measured at 1.67 to 1.75 of `main` when the rule was decided
    //  after the copies, and at 1.06 to 1.09 when it was decided before them).
    #[cfg(feature = "measurement-routes")]
    let measure_decline: bool =
        std::env::var("MONOCHROMATIC_CLONE_FENCE").unwrap_or_default() != "nodecline";
    #[cfg(not(feature = "measurement-routes"))]
    let measure_decline: bool = true;
    let converting: usize = converting_paths(entries.as_slice(), &attributes, &rules);
    if measure_decline && declines(converting, entries.len()) {
        return Err(format!(
            "a line-ending rule, a filter, an ident or an encoding can change {converting} of {} paths, more than a quarter of them, so this rung declines and Git's checkout writes the worktree",
            entries.len()
        ));
    }
    // A link is never converted by its path's rules, so the two questions are counted separately: a changed link is a path
    //  the copies cannot make Git's own entry, and one converting path is what the fence hands Git.
    let a_changed_path_could_matter: bool = converting > 0
        || entries
            .iter()
            .any(|entry| return entry.mode & MODE_TYPE_MASK == MODE_SYMLINK);
    return Ok(AttributeFence {
        entries,
        attributes,
        rules,
        a_changed_path_could_matter,
    });
}
