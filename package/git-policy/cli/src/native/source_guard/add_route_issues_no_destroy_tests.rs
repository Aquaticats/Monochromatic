//! What:
//!  The check that no release file of the add route names an entry that
//!  destroys a clone,
//! an origin snapshot or a snapshot set.
//! Why:
//!  The add's preflight refuses on the six permissions its own steps use and
//!  warns on the three only a removal uses,
//!  and that split is true only while no step of the add destroys
//!  (`worktree_clone_add_route::add_grants`,
//!  `doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 3C,
//!  "The split moves at plan step 11,
//!  when the add's own failure path starts using both `destroy` grants").
//!  Step 11's build kept that failure path out,
//!  because nothing of this release settles a failed restore at a slot
//!  (`worktree_clone_add_route::add_source_the_restore_cannot_prove`),
//!  so a failed add keeps the clone and the origin snapshot.
//!  The round that gives the add a destroying failure path,
//!  or the settle it hands over to,
//!  is the change that names one of these entries in the add's folder,
//!  and it moves the two `destroy` grants into the add's own table in the same
//!  commit,
//!  removing this check with them and saying so in its round's report.
//!  A doc line may name an entry without calling it,
//!  so the check reads masked code:
//!  comments and literals are not what the release runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/add_route_issues_no_destroy_tests.ts: no release file under worktree_clone_add_route/ names a destroy entry.
//! ```

use std::path::Path;

use super::blocking_code_off_the_runtime_tests::read_the_code_of;
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    source_root,
};

/// The folder whose release files may not name a destroying entry,
/// which is the add's own.
const THE_ADD_FOLDER: &str = "worktree_clone_add_route/";

/// One entry that destroys a clone,
/// an origin snapshot or a snapshot set,
/// with the file that declares it and the declaration the check reads to prove
/// the name still exists in the tree.
struct DestroyingEntry {
    /// The name as a call or an import spells it.
    name: &'static str,
    /// The file that declares it.
    declaring_file: &'static str,
    /// The declaration that file must still hold.
    declaration: &'static str,
    /// What it destroys,
    /// in the sentence the check fails with.
    what: &'static str,
}

/// Every entry of this release that destroys a route object.
/// The three the removal's own steps call are here with the origin cleanup's
/// attempt and the snapshot batch's request,
/// because the add must reach none of them.
const THE_DESTROYING_ENTRIES: &[DestroyingEntry] = &[
    DestroyingEntry {
        name: "destroy_the_head",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_head_destroy.rs",
        declaration: "pub fn destroy_the_head(",
        what: "destroys the verified clone",
    },
    DestroyingEntry {
        name: "send_request",
        declaring_file: "worktree_clone_snapshot_batch/snapshot_batch_send.rs",
        declaration: "pub fn send_request(",
        what: "sends a snapshot set's destroy request to the pool",
    },
    DestroyingEntry {
        name: "attempt_the_cleanup",
        declaring_file: "worktree_clone_origin_cleanup/origin_cleanup_attempt.rs",
        declaration: "pub fn attempt_the_cleanup(",
        what: "destroys an origin snapshot",
    },
    DestroyingEntry {
        name: "destroy_the_removal",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_destruction.rs",
        declaration: "pub fn destroy_the_removal(",
        what: "sends the removal's snapshot batch and destroys what it planned",
    },
    DestroyingEntry {
        name: "complete_the_removal",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_completion.rs",
        declaration: "pub fn complete_the_removal(",
        what: "destroys the removal's snapshots and the clone",
    },
];

/// What one scan of the release tree answers:
/// the destroying entries the add's folder names,
/// and the ones still declared where the check expects them.
struct Scan {
    /// `<file> names <entry>` for every hit in the add's folder.
    names: Vec<String>,
    /// The entries whose declaring file still holds their declaration.
    declared: Vec<&'static str>,
}

/// One scan over the crate's release sources.
fn scan_the_release_tree(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Scan, String> {
    let mut names: Vec<String> = Vec::new();
    let mut declared: Vec<&'static str> = Vec::new();
    for file in classified_sources(root, roots)? {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("{} cannot be read: {error}", file.path))?;
        let read = read_the_code_of(text.as_str())
            .map_err(|error| return format!("{} cannot be read as code: {error}", file.path))?;
        let code: String = read.masked.join("\n");
        for entry in THE_DESTROYING_ENTRIES {
            if file.path == entry.declaring_file && code.contains(entry.declaration) {
                declared.push(entry.name);
            }
            if file.path.starts_with(THE_ADD_FOLDER) && code.contains(entry.name) {
                names.push(format!("{} names {}", file.path, entry.name));
            }
        }
    }
    return Ok(Scan {
        names,
        declared,
    });
}

/// What:
///  No release file of the add route names a destroying entry,
///  and every entry the check bans is still declared where it expects.
/// Why:
///  The first is the rule:
///  the add's grant split refuses six and warns three because the add never
///  destroys.
///  The second keeps the check from passing on a tree where an entry was
///  renamed or deleted,
///  which would leave the rule true by accident and the ban naming nothing.
#[test]
fn the_add_route_issues_no_destroy() {
    let root = source_root();
    let scan: Scan = scan_the_release_tree(root.as_path(), real_roots().as_slice())
        .expect("the crate's release source tree reads");
    for entry in THE_DESTROYING_ENTRIES {
        assert!(
            scan.declared.contains(&entry.name),
            "positive control: `{}` still declares `{}`, so the name this check bans is a name in the tree",
            entry.declaring_file,
            entry.declaration
        );
    }
    assert!(
        scan.names.is_empty(),
        "no release file under `{THE_ADD_FOLDER}` may name an entry that destroys a clone, an origin snapshot or a snapshot set while the add's own grant table refuses no `destroy` grant, and these do: {:?}; the round that gives the add a destroying failure path moves both `destroy` grants into `THE_ADDS_OWN_STEPS` and removes this check in the same commit",
        scan.names
    );
}

/// What:
///  That every banned entry says what it destroys,
///  so a failing run tells a reader what release change reached the add.
#[test]
fn every_banned_entry_says_what_it_destroys() {
    for entry in THE_DESTROYING_ENTRIES {
        assert!(
            !entry.what.is_empty(),
            "`{}` says what it destroys for a reader of a failure",
            entry.name
        );
    }
}
