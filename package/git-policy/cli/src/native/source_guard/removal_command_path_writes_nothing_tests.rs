//! What:
//!  The check that the removal's command path runs the read-only reading and no
//!  entry that writes or destroys.
//! Why:
//!  This release builds the snapshot batch,
//!  the head destroy,
//!  the origin cleanup and the Git settle,
//!  and runs none of them,
//!  so a removal that filed the intent,
//!  unmounted the clone,
//!  wrote the destruction plan and sent the batch would leave a person
//!  with an unmounted clone,
//!  snapshots it destroyed,
//!  and a plan no command of this release could finish
//!  or cancel
//!  (plan steps 14 to 17;
//!  `worktree_clone_removal_proof_and_plan::removal_reading`).
//!  The command's own controls drive the reading and the scripted pool,
//!  so none of them would notice the wiring handed over to the writing entry or to the
//!  destruction step:
//!  that is a fact about which functions the release names,
//!  and this check reads the release source for it.
//!  The destruction step is step 15's checks and the batch together,
//!  and the completion is the whole removal from that batch to the report,
//!  so the same cut holds all three:
//!  no release file outside the removal's own folder names any of them,
//!  nor the plan folder's own type,
//!  whose every writer is a method on it,
//!  nor the unmount request the entry sends,
//!  nor the snapshot batch's request.
//!  The round that switches the command path on is the change that names them,
//!  and it removes this check in the same commit,
//!  listing it in its round's report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/removal_command_path_writes_nothing_tests.ts: one name check over the release tree.
//! ```

use std::path::Path;

use super::blocking_code_off_the_runtime_tests::read_the_code_of;
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    source_root,
};

/// The folder the removal's own steps live in,
/// whose files are the ones allowed to name the writing functions.
const THE_REMOVAL_FOLDER: &str = "worktree_clone_removal_proof_and_plan/";

/// One name a release file outside the removal's folder may not hold,
/// the file that must still declare it,
/// and the text that proves the declaration is there.
struct WritingName {
    /// The text a release file outside the folder may not hold:
    /// a function's name as a call or an import spells it,
    /// or the one type every writer of the plan folder is a method on.
    banned: &'static str,
    /// The file that declares it,
    /// and which the check reads to prove the name is a name in the tree.
    declaring_file: &'static str,
    /// The declaration the declaring file must still hold.
    declaration: &'static str,
    /// The files whose job is to publish the name besides its declaration:
    /// a module that re-exports it names it without calling it.
    publishing_files: &'static [&'static str],
    /// What the name does,
    /// in the sentence the check fails with.
    what: &'static str,
}

/// The names that wait for the steps which can finish or cancel what they start:
/// the removal's three writing entries,
/// the plan folder's own type,
/// whose every writer is a method on it,
/// the unmount request,
/// the snapshot batch's request,
/// the finish pass that settles a removal whose head is gone,
/// the cancel command,
/// and the settle command.
/// The names whose own module re-exports them
/// are the batch's request,
/// the finish pass's entry
/// and the settle command,
/// and a re-export is no call,
/// so those modules are named beside their declarations.
const THE_WRITING_NAMES: &[WritingName] = &[
    WritingName {
        banned: "enter_the_removal",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_entry.rs",
        declaration: "fn enter_the_removal(",
        publishing_files: &[],
        what: "files the intent, unmounts the clone and writes the destruction plan",
    },
    WritingName {
        banned: "destroy_the_removal",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_destruction.rs",
        declaration: "fn destroy_the_removal(",
        publishing_files: &[],
        what: "sends the snapshot batch and destroys what it planned",
    },
    WritingName {
        banned: "complete_the_removal",
        declaring_file: "worktree_clone_removal_proof_and_plan/removal_completion.rs",
        declaration: "fn complete_the_removal(",
        publishing_files: &[],
        what: "destroys the snapshots and the head, attempts the origin cleanup, settles the registration and removes the records",
    },
    WritingName {
        banned: "PlanFolder",
        declaring_file: "worktree_clone_removal_proof_and_plan/plan_publication.rs",
        declaration: "pub struct PlanFolder",
        publishing_files: &[],
        what: "makes the destruction plan's folder and writes a plan into it",
    },
    WritingName {
        banned: "request_the_unmount",
        declaring_file: "worktree_clone_removal_proof_and_plan/unmount_request.rs",
        declaration: "fn request_the_unmount(",
        publishing_files: &[],
        what: "sends the unmount request to the handler",
    },
    WritingName {
        banned: "send_request",
        declaring_file: "worktree_clone_snapshot_batch/snapshot_batch_send.rs",
        declaration: "fn send_request(",
        publishing_files: &["worktree_clone_snapshot_batch/mod.rs"],
        what: "sends one verified snapshot request to the pool",
    },
    WritingName {
        banned: "finish_the_removal_plans",
        declaring_file: "worktree_clone_removal_finish_pass/pass_over_the_plans.rs",
        declaration: "pub fn finish_the_removal_plans(",
        publishing_files: &["worktree_clone_removal_finish_pass/mod.rs"],
        what: "reads the plan folder and settles the plans of a removal whose head is gone",
    },
    WritingName {
        banned: "cancel_the_removal",
        declaring_file: "worktree_clone_removal_cancel/the_cancel_command.rs",
        declaration: "pub fn cancel_the_removal(",
        publishing_files: &[],
        what: "reverses a removal under the slot's claim and rewrites its records",
    },
    WritingName {
        banned: "settle_the_records_of_a_slot",
        declaring_file: "worktree_clone_record_settling/settling_command.rs",
        declaration: "pub fn settle_the_records_of_a_slot(",
        publishing_files: &["worktree_clone_record_settling/mod.rs"],
        what: "settles the records a person typed the flags of, under the slot's claim",
    },
];

/// The call this release's removal command makes instead.
/// The opening parenthesis is part of it,
/// because an import names the reading too:
/// the check is about the call the command path makes,
/// not the name in a `use` line.
const THE_READING_CALL: &str = "read_the_removal(&";

/// The removal's command path.
const THE_COMMAND_PATH: &str = "worktree_copying/worktree_removal_of_a_bound_clone.rs";

/// What:
///  One release file's code,
///  comments and literal characters masked,
///  so a doc line about the entry is not a call to it.
/// Why:
///  Both the command path's doc and the module's doc name the entry in prose to
///  say where it belongs,
///  and a check that read prose could only be satisfied by deleting the
///  explanation.
///
/// # Throws
/// The file whose literals or comments do not close.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = mask(readFileSync(path, 'utf8'));
/// ```
fn code_of(
    root: &Path,
    path: &str,
) -> Result<String, String> {
    let text: String = std::fs::read_to_string(root.join(path))
        .map_err(|error| return format!("{path} cannot be read: {error}"))?;
    let read = read_the_code_of(text.as_str())
        .map_err(|error| return format!("{path} cannot be read as code: {error}"))?;
    return Ok(read.masked.join("\n"));
}

/// What:
///  What one scan of the release tree answers:
///  every file outside the removal's folder that names a writing function,
///  whether the command path names the reading instead,
///  and which writing names are still declared where the check expects them.
/// Why:
///  The scan answers the questions the check asks,
///  so the test reads as the rule does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = scanTheReleaseTree(root, roots);
/// ```
struct Scan {
    /// The release files that hold a banned name from outside the folder.
    callers: Vec<String>,
    /// Whether the removal's command path calls the read-only reading.
    the_command_path_reads: bool,
    /// The banned names whose declaring file still holds their declaration.
    declared: Vec<&'static str>,
}

/// One scan over the crate's release sources.
fn scan_the_release_tree(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Scan, String> {
    let mut callers: Vec<String> = Vec::new();
    let mut the_command_path_reads: bool = false;
    let mut declared: Vec<&'static str> = Vec::new();
    for file in classified_sources(root, roots)? {
        if file.test_only {
            continue;
        }
        let code: String = code_of(root, file.path.as_str())?;
        for writing in THE_WRITING_NAMES {
            if file.path == writing.declaring_file && code.contains(writing.declaration) {
                declared.push(writing.banned);
            }
            // A declaration is no call, and neither is the re-export a batch's own module publishes.
            let publishes: bool = file.path == writing.declaring_file
                || writing.publishing_files.contains(&file.path.as_str());
            if !publishes
                && !file.path.starts_with(THE_REMOVAL_FOLDER)
                && code.contains(writing.banned)
            {
                callers.push(format!("{} names {}", file.path, writing.banned));
            }
        }
        if file.path == THE_COMMAND_PATH && code.contains(THE_READING_CALL) {
            the_command_path_reads = true;
        }
    }
    return Ok(Scan {
        callers,
        the_command_path_reads,
        declared,
    });
}

/// What:
///  No release file outside the removal's folder names a function that writes or destroys,
///  the command path names the reading,
///  and every such function is still declared where the check expects it.
/// Why:
///  The first two are the rule:
///  the writing entry and the destruction step wait for the steps that can finish or cancel what
///  they start.
///  The third keeps the check from passing on a tree where a function was simply
///  deleted,
///  which would leave plan steps 15 to 17 nothing to wire.
#[test]
fn the_removal_command_path_runs_the_reading_and_no_writing_entry() {
    let root = source_root();
    let scan: Scan = scan_the_release_tree(root.as_path(), real_roots().as_slice())
        .expect("the crate's release source tree reads");
    for writing in THE_WRITING_NAMES {
        assert!(
            scan.declared.contains(&writing.banned),
            "positive control: `{}` still declares `{}`, so the name this check bans is a name in the tree",
            writing.declaring_file,
            writing.declaration
        );
    }
    assert!(
        scan.the_command_path_reads,
        "`{THE_COMMAND_PATH}` must call `{THE_READING_CALL}…)`: this release's removal proves its lineage read-only and writes nothing"
    );
    assert!(
        scan.callers.is_empty(),
        "no release file outside `{THE_REMOVAL_FOLDER}` may name a function that writes or destroys, nor the plan folder's own type, until the round that switches the command path on, and these do: {:?}",
        scan.callers
    );
}

/// What:
///  That the banned names are the ones this file means:
///  each declares what it does in the failure sentence,
///  so a reader of a failing run knows which release change reached the command path.
/// Why:
///  The one test above is a rule over a list,
///  and a list whose entries lose their words would leave a failure unreadable.
#[test]
fn every_banned_name_says_what_it_does() {
    for writing in THE_WRITING_NAMES {
        assert!(
            !writing.what.is_empty(),
            "`{}` names what it does for a reader of a failure",
            writing.banned
        );
    }
}
