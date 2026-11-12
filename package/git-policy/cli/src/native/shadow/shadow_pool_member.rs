//! What:
//!  What a finished shadow in the pool must be for this invocation to take it:
//!  the tree a creation would have made of what a commit reads,
//! or a refusal that names what differs.
//! Why:
//!  A commit takes a shadow another process made,
//! so the one new risk is trusting a tree made earlier.
//!  That risk has one answer a test can hold:
//! a member is either the tree a creation makes of what a commit reads,
//! or it is refused
//! (`doc/handover/questions-for-the-owner.md`,
//! "May a commit take its private repository from a pool made ahead?").
//!  What a commit reads is the named set of shared entries
//! (`shadow_links.rs`,
//! `SHARED_ENTRY_NAMES`),
//! so a member is judged on those and never on the rest of what a common Git directory collects:
//!  a file another command wrote since the fill
//!  (`FETCH_HEAD`,
//! a log generation,
//! a worktree)
//!  is state no commit reads,
//! so it neither makes a member wrong nor refuses one.
//! Everything here is a read:
//! the comparison costs no directory-entry creation on the volume a commit already waits on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = await refusalOfMember(template, member); // undefined when the member is what a creation makes
//! ```

/// The creation's own `HEAD`,
/// before any commit names its branch.
use crate::shadow_skeleton::PLACEHOLDER_HEAD;
/// The creation's skeleton,
/// which a member must hold whole.
use crate::shadow_skeleton::SKELETON_DIRECTORIES;
/// The facts a creation is made from.
use crate::shadow_create::ShadowTemplate;
/// The entries a creation links,
/// the names a member may hold as links,
/// and the per-worktree file it copies.
use crate::shadow_links::{has_worktree_config, is_shared_shadow_entry, shared_entries_of};
/// The directories of a shadow that stay the creation's own,
/// and the path each check names.
use std::path::{Path, PathBuf};

#[cfg(test)]
thread_local! {
    /// How many link targets this thread's claims have asked for,
    /// counted by the controls only.
    /// It is per thread,
    /// so that a control which shows a claim asks one question per shared entry
    ///  cannot be disturbed by another test's claim.
    static LINK_QUESTIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What:
///  The entries of a member that the skeleton and the creation always make,
/// which no link and no copy stands in for.
/// Why:
///  A member holds these plus the shared entries and,
/// for a linked worktree,
/// one copied file,
/// and anything else is a leftover of a use that would be trusted by the next commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWN_ENTRIES = ['HEAD', 'config', 'objects', 'refs'];
/// ```
const OWN_ENTRIES: [&str; 4] = ["HEAD", "config", "objects", "refs"];

/// What:
///  The per-worktree file a creation copies into a shadow.
/// Why:
///   It is the one entry that is neither the skeleton's nor a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORKTREE_CONFIG = 'config.worktree';
/// ```
const WORKTREE_CONFIG: &str = "config.worktree";

/// What:
///  Why this member is not the tree a creation would have made,
/// or nothing when it is.
/// Why:
///   The caller refuses a named member and removes it,
/// so a member made for another worktree,
/// another object directory or another release never reaches a commit.
///  A reftable repository is never pooled at all,
/// so the entries Git's own `init` adds there are not considered.
///  An entry a commit reads that the repository has gained since the fill is refused,
/// because the member would give the commit the view of the moment it was made;
///  an entry it has lost is not,
/// since a link to a name that is no longer there resolves to nothing,
/// which is the view a creation of this moment gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await refusalOfMember(template, member); // 'its config is not the one this invocation would write' | undefined
/// ```
pub fn refusal(template: &ShadowTemplate, member: &Path) -> std::io::Result<Option<String>> {
    // The member itself first, and asked of the name and never of what a link there names:
    // listing it below follows a link, so a link at a member's name would be judged, and then written into, as the tree
    // it points at rather than as the tree a creation made.
    // The name carries no separator, which `join("")` would add and which makes `lstat` resolve the link.
    let own_directory: bool = match std::fs::symlink_metadata(member) {
        Ok(metadata) => metadata.is_dir(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    if !own_directory {
        return Ok(Some(String::from("it is not the directory a creation makes")));
    }
    if read_bytes(member.join("config").as_path())? != template.config_bytes() {
        return Ok(Some(String::from("its config is not the one this invocation would write")));
    }
    if read_bytes(member.join("objects").join("info").join("alternates").as_path())? != template.alternates_bytes() {
        return Ok(Some(String::from("its alternates do not name this invocation's object directory")));
    }
    if read_bytes(member.join("HEAD").as_path())? != PLACEHOLDER_HEAD.as_bytes() {
        return Ok(Some(String::from("its HEAD is not the one a creation leaves before a commit names its branch")));
    }
    // The skeleton's directories are asked about by name, and the member itself is the question above.
    for relative in SKELETON_DIRECTORIES {
        if relative.is_empty() {
            continue;
        }
        let path: PathBuf = member.join(relative);
        let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else {
            return Ok(Some(format!("the skeleton directory {relative} is not a directory")));
        };
        if !metadata.is_dir() {
            return Ok(Some(format!("the skeleton directory {relative} is not a directory")));
        }
    }
    let (shared, keeps_its_own_info) = shared_entries_of(template.common_dir.as_path(), template.git_dir.as_path())?;
    if keeps_its_own_info {
        return Ok(Some(String::from("this worktree keeps its own info directory, so its shadows are made where the commit is")));
    }
    let copies_worktree_config: bool = has_worktree_config(template.git_dir.as_path())?;
    // One listing of the member answers both questions its entries are asked: that each is one this invocation's
    // creation makes, or one a creation could have linked, and that every entry at a shared name is the link it made.
    // The type of each entry comes with the listing (`d_type`), so a shared entry costs the one call that reads its target:
    // `read_link` alone refuses an entry that is missing and one that is not a link.
    let listed: Vec<(std::ffi::OsString, bool)> = listing_of(member)?;
    for (name, is_link) in &listed {
        let spelled: String = name.to_string_lossy().into_owned();
        let own: bool = OWN_ENTRIES.contains(&spelled.as_str()) || (copies_worktree_config && spelled == WORKTREE_CONFIG);
        if own {
            // A creation writes these entries itself, so a link at one of them is a leftover of a use,
            // and a write through it reaches the file it names: `HEAD` is written with `fs::write`.
            if *is_link {
                return Ok(Some(format!("it holds {spelled} as a link, which this invocation's creation does not make")));
            }
            continue;
        }
        if !is_shared_shadow_entry(spelled.as_str()) {
            return Ok(Some(format!("it holds {spelled}, which this invocation's creation does not make")));
        }
        // A shared name the repository no longer holds is the view a creation gives too,
        // but only as the link a creation made: a real directory or file there is a leftover of a use.
        // The type of a shared entry is settled here, which leaves the loop below the presence and the target of one.
        if !is_link {
            return Ok(Some(format!("the shared entry {spelled} is not a link in the member")));
        }
    }
    // Every entry the creation shares is here, and links to the same real entry.
    for entry in &shared {
        let shared_named: String = entry.name.to_string_lossy().into_owned();
        let link: PathBuf = member.join(&entry.name);
        let mut present: bool = false;
        for (name, _is_link) in &listed {
            if *name == entry.name {
                present = true;
                break;
            }
        }
        if !present {
            return Ok(Some(format!("the shared entry {shared_named} is not in the member")));
        }
        if link_target(link.as_path())? != entry.source {
            return Ok(Some(format!("the shared entry {shared_named} links somewhere other than the repository's own entry")));
        }
    }
    if copies_worktree_config
        && read_bytes(member.join(WORKTREE_CONFIG).as_path())?
            != read_bytes(template.git_dir.join(WORKTREE_CONFIG).as_path())?
    {
        return Ok(Some(String::from("its copy of the worktree's configuration is not the one a creation would copy")));
    }
    // Nothing of an earlier commit's work survives in a member.
    if !is_empty(member.join("objects").join("pack").as_path())? {
        return Ok(Some(String::from("it holds a pack, which no creation leaves")));
    }
    if !is_empty(member.join("refs").join("heads").as_path())?
        || !is_empty(member.join("refs").join("tags").as_path())?
    {
        return Ok(Some(String::from("it holds a ref, which no creation leaves")));
    }
    return Ok(None);
}

/// What:
///  The entries of a directory,
/// each with the name and whether it is a link.
/// Why:
///   The name and the type come from the one listing,
/// so a caller that asks both of every entry costs one read of the directory and no question per entry.
///  The type is the entry's own,
/// never what a link points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listingOf(directory) // [{ name, isLink }]
/// ```
fn listing_of(directory: &Path) -> std::io::Result<Vec<(std::ffi::OsString, bool)>> {
    let mut listed: Vec<(std::ffi::OsString, bool)> = Vec::new();
    for listed_entry in std::fs::read_dir(directory)? {
        let entry: std::fs::DirEntry = listed_entry?;
        listed.push((entry.file_name(), entry.file_type()?.is_symlink()));
    }
    return Ok(listed);
}

/// What:
///  Where a link points,
///  and the refusal of one that is not there or is not a link.
/// Why:
///   This is the one question a claim asks of an entry the creation shares,
/// and the answer it needs is the target.
/// A missing entry and an entry that is not a link are both failures of this call,
/// so no second question stands before it.
///  Every answer is counted for the controls,
/// which hold that a claim asks nothing else of a shared entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkTarget(path) // the target, or ENOENT/EINVAL
/// ```
fn link_target(link: &Path) -> std::io::Result<PathBuf> {
    note_link_question();
    return std::fs::read_link(link);
}

/// What:
///  One link question asked by a claim,
/// counted for the controls only.
/// Why:
///   A claim that asks a shared entry's target once and nothing else of it is the shape this file holds to,
/// and a count is the only way a control can see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteLinkQuestion()
/// ```
fn note_link_question() {
    #[cfg(test)]
    LINK_QUESTIONS.with(|counter| { counter.set(counter.get() + 1); });
}

/// What:
///  How many link questions this thread's claims have asked since they were last reset.
/// Why:
///   The controls read it to see the shape of a claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkQuestions()
/// ```
#[cfg(test)]
fn link_questions() -> u64 {
    return LINK_QUESTIONS.with(std::cell::Cell::get);
}

/// What:
///  Forgets the link questions asked so far on this thread.
/// Why:
///   A control starts counting at the claim it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetLinkQuestions()
/// ```
#[cfg(test)]
fn reset_link_questions() {
    LINK_QUESTIONS.with(|counter| { counter.set(0); });
}

/// What:
///  A file's exact bytes,
///  and nothing for one that is not there.
/// Why:
///   Every file of a member is compared byte for byte:
/// its config and alternates name paths,
///  and the worktree's configuration can hold one too,
/// so two files that differ only in bytes that are not UTF-8
///  decode to the same text and would be judged equal.
/// A member that kept the older one would hand a commit the editor,
/// the hook
///  or the command the person has since changed.
/// A missing entry is a difference the caller refuses,
/// not an error it has to tell from a
///  real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readBytes(path) // the empty buffer when absent
/// ```
fn read_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    return match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    };
}

/// What:
///  Whether a directory holds no entry.
/// Why:
///  A pack and a ref are what a used shadow holds;
/// a directory of them is what a member never has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await isEmpty(path)
/// ```
fn is_empty(path: &Path) -> std::io::Result<bool> {
    return match std::fs::read_dir(path) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    };
}

/// Member checks stay out of the release executable.
#[cfg(test)]
#[path = "shadow_pool_member_tests.rs"]
mod tests;
