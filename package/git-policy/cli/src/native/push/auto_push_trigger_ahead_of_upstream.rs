//! What:
//!  The checks that say whether the branch a command moved is a case the owner chose to hand over:
//!  the tip of the upstream tracking ref,
//!  the walk over the commit parents that says one tip is an ancestor of another,
//!  and the decision itself,
//!  with the step that declined when it is not that case.
//! Why:
//!  Ahead and not diverged is "the upstream tip is an ancestor of the branch tip",
//!  and a branch the command rewrote is a rebase,
//!  which the owner chose not to push.
//!  None of it reads a Git process:
//!  the refs and the objects are read from the files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function tipToPushOrWhy(watch: TipWatch): string | Declined;
//! function reaches(database, layout, from: string, target: string, budget: number): boolean | undefined;
//! ```

/// Import the watched branch and the decline the checks answer with.
use super::auto_push_trigger_watched_branch::{
    Declined,
    TipWatch,
    current_branch,
    declined,
};
/// Import the configuration listing the branch's upstream is read from,
/// with the scope a value is resolved in.
use crate::repo_read_config_list::{
    ListedEntry,
    list_config,
};
use crate::repo_read_config_scan::ScopeEnvironment;
/// Import the layout the objects and refs are found through,
/// the object kinds and object names the walk reads,
/// and the store it reads them from.
use crate::repo_read_layout::RepoLayout;
use crate::repo_read_object::{
    ObjectKind,
    from_hex,
};
use crate::repo_read_odb::ObjectDatabase;
/// Import the rule that turns a configured remote and merge into a tracking ref,
/// and the refs themselves.
use crate::repo_read_query_refs::tracking_ref;
use crate::repo_read_refs::Refs;
/// Import the reader of a commit object's parents,
/// which is what the walk follows.
use crate::repo_read_tree::parse_commit;
/// Import the queue of object names still to walk and the set of those already walked.
use std::collections::{
    HashSet,
    VecDeque,
};

/// The most commits the walk from a tip reads before it gives up and hands nothing over:
/// a branch far ahead,
/// or a history
/// that never reaches the upstream,
/// is not worth a long walk after every merge.
/// Measured:
/// the 439 most recent merges on `main` of
///  this repository brought in a median of 8 commits (90th percentile 50,
/// 99th percentile 317,
/// largest 6,542),
/// and only 1 exceeded
///  this budget,
/// so 438 of 439 are walked in full.
/// Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const WALK_BUDGET: usize = 4096;

/// Every readable value of `branch.<name>.<suffix>` in the configuration listing,
/// in order.
fn branch_values(
    listed: &[ListedEntry],
    name: &str,
    suffix: &str,
) -> Vec<String> {
    let key: String = format!("branch.{name}.{suffix}");
    let mut found: Vec<String> = Vec::new();
    for item in listed {
        if item.entry.key != key {
            continue;
        }
        if let Some(bytes) = item.entry.value.as_deref()
            && let Ok(text) = std::str::from_utf8(bytes)
        {
            found.push(text.to_owned());
        }
    }
    return found;
}

/// The tip the upstream tracking ref of the branch holds,
/// or the step that found no single plain upstream on a remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upstreamTip(watch: TipWatch, refs: Refs): string | Declined;
/// ```
fn upstream_tip(
    watch: &TipWatch,
    refs: &Refs,
) -> Result<String, Declined> {
    let Some(name) = watch.branch_ref().strip_prefix("refs/heads/") else {
        return Err(declined(
            "upstream-no-branch-name",
            watch.branch_ref().to_owned(),
        ));
    };
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(watch.variables());
    let Some(listed) = list_config(
        watch.layout().common_dir.as_path(),
        watch.layout().git_dir.as_path(),
        &scope,
        watch.variables(),
    ) else {
        return Err(declined(
            "upstream-config-unreadable",
            "the configuration listing was declined by the reader",
        ));
    };
    let remotes: Vec<String> = branch_values(listed.as_slice(), name, "remote");
    let merges: Vec<String> = branch_values(listed.as_slice(), name, "merge");
    let ([remote], [merge]) = (remotes.as_slice(), merges.as_slice()) else {
        return Err(declined(
            "upstream-not-single",
            format!(
                "branch.{name}.remote has {} values and branch.{name}.merge has {}",
                remotes.len(),
                merges.len()
            ),
        ));
    };
    if remote == "." {
        return Err(declined(
            "upstream-is-local",
            format!("branch.{name}.remote is the repository itself"),
        ));
    }
    let Some(tracking) = tracking_ref(listed.as_slice(), remote, merge) else {
        return Err(declined(
            "upstream-no-tracking-ref",
            format!("no remote.{remote}.fetch maps {merge}"),
        ));
    };
    let Some(tip) = refs.resolve(tracking.as_str()) else {
        return Err(declined("upstream-tracking-ref-missing", tracking));
    };
    return Ok(tip);
}

/// What:
///  Whether `target` is `from` or one of its ancestors,
/// within the walk budget;
/// nothing when the walk could not tell.
/// Why:
///   Ahead and not diverged is "the upstream tip is an ancestor of the branch tip";
/// the walk goes from the branch tip,
/// which
///  reaches the upstream within the commits the branch is ahead by,
/// and stops at the budget on a history that never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reaches(database, from: string, target: string, budget: number): boolean | undefined;
/// ```
pub fn reaches(
    database: &ObjectDatabase,
    layout: &RepoLayout,
    from: &str,
    target: &str,
    budget: usize,
) -> Option<bool> {
    let start: Vec<u8> = from_hex(from.as_bytes(), layout.object_format)?;
    let wanted: Vec<u8> = from_hex(target.as_bytes(), layout.object_format)?;
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut queue: VecDeque<Vec<u8>> = VecDeque::new();
    seen.insert(start.clone());
    queue.push_back(start);
    let mut read: usize = 0;
    while let Some(name) = queue.pop_front() {
        if name == wanted {
            return Some(true);
        }
        if read >= budget {
            return None;
        }
        read += 1;
        let object = database.read(name.as_slice())?;
        if object.kind != ObjectKind::Commit {
            return None;
        }
        for parent in parse_commit(object.bytes.as_slice(), layout.object_format)?.parents {
            if seen.insert(parent.clone()) {
                queue.push_back(parent);
            }
        }
    }
    return Some(false);
}

/// What:
///  The tip to hand over after the command,
/// or the step that found the branch not a case the owner chose.
/// Why:
///   The branch must be the same one,
/// its tip must have moved forward from the old tip (a rewritten branch is a rebase),
///  and the new tip must be ahead of its upstream and not diverged:
/// the upstream tip is an ancestor of it and not it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tipToPushOrWhy(watch: TipWatch): string | Declined;
/// ```
pub fn tip_to_push_or_why(watch: &TipWatch) -> Result<String, Declined> {
    let (branch_ref, after): (String, String) = current_branch(watch.layout())?;
    if branch_ref != watch.branch_ref() {
        return Err(declined(
            "branch-changed",
            format!(
                "the command started on {} and ended on {branch_ref}",
                watch.branch_ref()
            ),
        ));
    }
    if after == watch.tip_before() {
        return Err(declined(
            "tip-unchanged",
            format!("{branch_ref} stayed at {after}"),
        ));
    }
    let refs: Refs = Refs::new(
        watch.layout().common_dir.as_path(),
        watch.layout().object_format.hex_length(),
    );
    let upstream: String = upstream_tip(watch, &refs)?;
    if upstream == after {
        return Err(declined(
            "upstream-equals-tip",
            format!("{after} is already the upstream tip"),
        ));
    }
    let Some(database) =
        ObjectDatabase::open_where_the_layout_proved_the_store(watch.layout(), None)
    else {
        return Err(declined(
            "object-database-unopened",
            watch.layout().common_dir.display().to_string(),
        ));
    };
    let Some(grew) = reaches(
        &database,
        watch.layout(),
        after.as_str(),
        watch.tip_before(),
        WALK_BUDGET,
    ) else {
        return Err(declined(
            "walk-undecided-growth",
            format!("from {after} to the old tip {}", watch.tip_before()),
        ));
    };
    if !grew {
        return Err(declined(
            "not-grown",
            format!(
                "the old tip {} is not an ancestor of {after}: a rewritten branch",
                watch.tip_before()
            ),
        ));
    }
    let Some(ahead) = reaches(
        &database,
        watch.layout(),
        after.as_str(),
        upstream.as_str(),
        WALK_BUDGET,
    ) else {
        return Err(declined(
            "walk-undecided-ahead",
            format!("from {after} to the upstream tip {upstream}"),
        ));
    };
    if !ahead {
        return Err(declined(
            "not-ahead",
            format!("the upstream tip {upstream} is not an ancestor of {after}: diverged"),
        ));
    }
    return Ok(after);
}

/// `tip_to_push_or_why` without the reason.
pub fn tip_to_push(watch: &TipWatch) -> Option<String> {
    return tip_to_push_or_why(watch).ok();
}
