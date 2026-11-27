//! The ban on reading a file with a call that can wait:
//! no release file outside `foundation/plain_file.rs` reads a file with `std::fs::read` or
//! `std::fs::read_to_string`,
//! and every site that still does is on a list that only shrinks,
//! each entry naming the class whose reason keeps it.
//! Why:
//!  Opening a named pipe for reading waits for a writer that may never come,
//!  and a plain read of a path a person or another program can replace with a pipe holds a wrapped command
//!  for ever with nothing said
//!  (issue 6415 held `git status` at a pipe in a record folder,
//!  and the same shape held a commit at `--pathspec-from-file`).
//!  The reader that cannot wait exists (`foundation/plain_file.rs`):
//!  it opens with `O_NONBLOCK | O_NOFOLLOW`,
//!  takes the type from the open handle and reads to the caller's bound,
//!  so a name swapped for a pipe between a look and a read cannot redirect or hold it.
//!  A check that only named the folders moved so far (`records_read_without_blocking_tests.rs`) cannot see
//!  a new raw read of a Git directory,
//!  so this one names the calls themselves and holds today's sites in a list that may only shrink
//!  (item 2 of `doc/debt-ledger-2026-11-25.md`).

/// Path values.
use std::path::Path;

/// The words that decide which files the module tree compiles for release alone,
/// so a new binary or test target is covered through the manifest rather than a second list.
use super::release_sources_in;
/// The shared scan base: the crate root record, the roots `Cargo.toml` lists, the source root and a fixture writer.
use crate::source_scan_tests::{
    CrateRoot,
    real_roots,
    source_root,
    write_tree,
};

/// The two calls that open a file and read it whole,
/// as a line spells them,
/// and the whole of what this check refuses.
///  The first names the text reader and the second the byte reader;
/// `fs::read(` is the tail of `std::fs::read(` and of a bare `fs::read(`,
/// and it is not a prefix of the other, so counting both counts each call once.
const CALLS_THAT_CAN_WAIT: &[&str] = &["fs::read_to_string(", "fs::read("];

/// The one file that may make those calls,
/// by its path relative to the source root.
///  It is the reader this check exists to point at,
/// so a change that reads a file inside it is the reader's own business and not a bypass.
const THE_PLAIN_FILE_READER: &str = "foundation/plain_file.rs";

/// One class of raw read that stays,
/// and the reason the reader that cannot wait is not used there.
///  A list entry names one of these classes,
/// so no entry is left without a reason,
/// and a class no entry names is reported as stale.
const KEPT_CLASSES: &[(&str, &str)] = &[
    (
        "system",
        "the kernel or the operating system's own installation writes this name (a `/proc` or `/sys` fact, the mount \
         table, this boot's id, this machine's id, a cgroup file), so no person or program chooses to put a pipe there",
    ),
    (
        "own-record",
        "this wrapper wrote the file into a folder of its own, and the worst a pipe at that name does is lose a record \
         it owns; a later round moves these. A record a command's own start reads is never this class: a pipe there \
         holds every command, so such a read goes through `foundation/plain_file.rs` and \
         `records_read_without_blocking_tests` refuses a waiting read of those folders",
    ),
    (
        "git-directory",
        "Git wrote the file into a Git directory, or the wrapper wrote it beside one, so a pipe there breaks Git before \
         it breaks the wrapper, and native Git waits on it the same way; a later round moves these",
    ),
    (
        "person-path",
        "a path the person or the person's project holds (a configuration file, a file in a checkout, a program on the \
         machine), which the person's own command line or configuration names",
    ),
    (
        "test-stand-in",
        "a control's stand-in process writes this file to report to its control, and no command of a person's reaches \
         it",
    ),
];

/// The sites that still make one of the two calls,
/// as `(path relative to the source root, how many calls, the class that keeps them)`.
///  A count is exact:
///  a file that gains one fails this check until the entry is raised,
///  which no class's reason allows,
///  and a file that loses one fails until the entry is lowered,
///  so the list can only shrink.
///  A file whose kept sites belong to two classes appears twice,
/// and the calls its entries give add up to the calls it makes.
const KEPT_SITES: &[(&str, usize, &str)] = &[
    ("account_companion/companion_settings.rs", 1, "system"),
    ("candidate/candidate_private_index.rs", 1, "own-record"),
    ("commit/conclusion_cleanup.rs", 2, "own-record"),
    ("dependent_version/policy_added_path.rs", 1, "person-path"),
    (
        "foundation/the_lines_of_a_log_compared_and_appended.rs",
        1,
        "own-record",
    ),
    ("foundation/runtime_limits.rs", 5, "system"),
    ("health/doctor_filesystem.rs", 1, "system"),
    ("invocation/real_git_candidate.rs", 2, "person-path"),
    ("invocation/started_as_git.rs", 2, "system"),
    ("landing/claim_time.rs", 2, "system"),
    ("landing/earlier_service_command_line_read.rs", 1, "system"),
    ("landing/held_commit_settle.rs", 2, "own-record"),
    ("landing/service_index_identity.rs", 2, "system"),
    ("landing/service_index_same.rs", 2, "git-directory"),
    ("landing/service_land_finish.rs", 1, "own-record"),
    ("landing/service_land_objects_of_batch.rs", 2, "own-record"),
    ("landing/service_message.rs", 1, "person-path"),
    ("landing/service_rules_precompile.rs", 2, "own-record"),
    (
        "landing/service_scratch_ref_marker_of_a_batch.rs",
        1,
        "own-record",
    ),
    ("lock/index_lock_evidence.rs", 1, "system"),
    ("lock/index_lock_holder_words.rs", 1, "system"),
    ("lock/proc_fd_holder_scan.rs", 1, "system"),
    ("lock/single_owner_claim.rs", 1, "system"),
    ("lock/stale_ref_lock_judge.rs", 1, "system"),
    ("merge/merge_isolation_editor.rs", 1, "own-record"),
    ("merge/merge_isolation_kept_copies.rs", 1, "own-record"),
    ("merge/merge_second_pass_finish.rs", 1, "git-directory"),
    (
        "merge/merge_second_pass_finish_stops.rs",
        1,
        "git-directory",
    ),
    ("policy/direct_fix_install.rs", 1, "git-directory"),
    ("policy/direct_fix_install_journal.rs", 2, "own-record"),
    ("policy/direct_fix_install_judging.rs", 1, "person-path"),
    ("process/process_start.rs", 2, "system"),
    ("process/process_tree.rs", 6, "system"),
    ("process/process_use.rs", 1, "system"),
    ("process/thread_quiet.rs", 2, "system"),
    ("push/push_progress_meter.rs", 3, "system"),
    ("repair/package_repair_cargo_lock.rs", 2, "person-path"),
    (
        "repair/package_repair_cargo_lock_never_replacing.rs",
        3,
        "person-path",
    ),
    (
        "repair/package_repair_prewarm_job_staged_state.rs",
        1,
        "git-directory",
    ),
    ("repair/package_repair_start.rs", 1, "test-stand-in"),
    ("repo_read/repo_read_odb.rs", 1, "git-directory"),
    ("repo_read/sequencer_marker.rs", 1, "git-directory"),
    (
        "repository_link_check/repository_move_freeze.rs",
        1,
        "own-record",
    ),
    (
        "repository_link_check/repository_link_probe.rs",
        1,
        "person-path",
    ),
    ("service/earlier_unix_sockets_that_listen.rs", 2, "system"),
    (
        "service/service_journal_records_read_from_the_journal.rs",
        4,
        "own-record",
    ),
    ("service/service_journal_record_age.rs", 1, "own-record"),
    ("service/service_provision.rs", 1, "own-record"),
    (
        "service/service_provision_what_is_already_in_place.rs",
        1,
        "own-record",
    ),
    (
        "service/service_provision_earlier_service_retirement.rs",
        1,
        "person-path",
    ),
    (
        "service/service_provision_unit_file_ownership_and_writes.rs",
        1,
        "own-record",
    ),
    ("shadow/shadow_pool_member.rs", 1, "person-path"),
    (
        "standards_review/standards_review_append.rs",
        2,
        "own-record",
    ),
    ("start_probe/probe_mount_calls.rs", 1, "system"),
    ("subvolume/fs_beneath_mount_facts.rs", 1, "system"),
    ("subvolume/subvolume_lock.rs", 1, "own-record"),
    ("transaction/recovery_files.rs", 1, "own-record"),
    ("worktree/worktree_clone_block_cloning.rs", 1, "system"),
    (
        "worktree/worktree_clone_conversion_fence.rs",
        2,
        "git-directory",
    ),
    ("worktree/worktree_dirty_restore.rs", 1, "git-directory"),
    (
        "worktree/worktree_ladder_target_conversion_agreement.rs",
        1,
        "person-path",
    ),
    ("worktree/worktree_process_inside.rs", 2, "system"),
    ("worktree/worktree_snapshot_source.rs", 1, "git-directory"),
    (
        "worktree/worktree_trash_exclude_line.rs",
        1,
        "git-directory",
    ),
    ("worktree_clone_add_route/add_preflight.rs", 1, "system"),
    (
        "worktree_clone_add_route/add_route_steps_snapshot_and_capture.rs",
        1,
        "git-directory",
    ),
    (
        "worktree_clone_binding_record/plan_folder_read.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_forced_removal/process_session_record.rs",
        1,
        "system",
    ),
    (
        "worktree_clone_mount/attached_mount_identity.rs",
        1,
        "system",
    ),
    (
        "worktree_clone_mount_handler/sweep_preconditions.rs",
        1,
        "system",
    ),
    (
        "worktree_clone_mount_handler/sweep_preconditions.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_origin_cleanup/origin_cleanup_reading.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_prune_pointer/pointer_restore.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_record_settling/settling_removals.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_record_settling/settling_slot.rs",
        1,
        "own-record",
    ),
    (
        "worktree_clone_removal_cancel/the_reversing_request.rs",
        1,
        "system",
    ),
    (
        "worktree_clone_removal_cancel/the_staged_state.rs",
        1,
        "system",
    ),
    (
        "worktree_clone_removal_proof_and_plan/unmount_request.rs",
        1,
        "system",
    ),
    ("worktree_clone_unmount/staging_calls.rs", 3, "system"),
    (
        "worktree_copying/worktree_copy_interrupted_add_settlement.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_interrupted_checkout.rs",
        2,
        "person-path",
    ),
    ("worktree_copying/worktree_copy_journal.rs", 1, "own-record"),
    (
        "worktree_copying/worktree_copy_owed_copy.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_owed_taking.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_source_still_being_filled.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_stage_claim.rs",
        2,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_stage_claim_owed_copy_elsewhere.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_stage_claim_reaping.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_stage_made_object.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_swapped_add_settlement.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_unfilled_record.rs",
        1,
        "own-record",
    ),
    (
        "worktree_copying/worktree_copy_unreadable_records.rs",
        2,
        "own-record",
    ),
];

/// What:
///  `fn calls_on_a_line(line: &str) -> usize` counts the calls one line makes,
///  of both words in `CALLS_THAT_CAN_WAIT` together.
/// Why:
///  Two calls on one line are two reads,
///  and a count per line would let a second one in beside the first for free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function callsOnALine(line: string): number { let n = 0; for (const call of CALLS) { for (const at of line.matchAll(call)) n++ } return n }
/// ```
fn calls_on_a_line(line: &str) -> usize {
    let mut counted: usize = 0;
    for call in CALLS_THAT_CAN_WAIT {
        // The part of the line the search has not read, so one call is counted once and the text before it is not read again.
        let mut remaining: &str = line;
        loop {
            // Where the next call of this word starts, or nothing when the rest of the line holds none.
            let Some(place) = remaining.find(call) else {
                break;
            };
            counted += 1;
            remaining = &remaining[place + call.len()..];
        }
    }
    return counted;
}

/// What:
///  `fn calls_in(text: &str) -> usize` counts the calls a file's text makes,
///  leaving out any line that begins as a comment.
/// Why:
///  A comment or a doc comment names a call without making one,
///  and a line-at-a-time cut at a leading `//` is exact for a file whose comments stand on their own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls = text.split('\n').filter((line) => !line.trimStart().startsWith('//')).reduce((n, line) => n + callsOnALine(line), 0)
/// ```
fn calls_in(text: &str) -> usize {
    let mut counted: usize = 0;
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        counted += calls_on_a_line(line);
    }
    return counted;
}

/// What:
///  `fn counts_in(root, roots) -> Result<Vec<(String, usize)>, String>` is every release file that makes a
///  call that can wait, with how many it makes.
/// Why:
///  Both halves of the check read one measurement,
///  so a file it cannot read stops it rather than passing as a file with no calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const counts = releaseSources(root).map((p) => [p, callsIn(readFileSync(p))]).filter(([, n]) => n > 0)
/// ```
fn counts_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<(String, usize)>, String> {
    let mut found: Vec<(String, usize)> = Vec::new();
    for path in release_sources_in(root, roots)? {
        if path == THE_PLAIN_FILE_READER {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(path.as_str()))
            .map_err(|error| return format!("cannot read {path}: {error}"))?;
        let counted: usize = calls_in(text.as_str());
        if counted > 0 {
            found.push((path, counted));
        }
    }
    return Ok(found);
}

/// What:
///  `fn listed() -> Vec<(String, usize)>` is `KEPT_SITES` keyed by file,
///  the calls of a file's entries added up.
/// Why:
///  A file whose sites belong to two classes appears twice in the list,
///  and the question the check asks of a file is about all of its calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = new Map<string, number>(); for (const [p, n] of KEPT_SITES) listed.set(p, (listed.get(p) ?? 0) + n)
/// ```
fn listed() -> Vec<(String, usize)> {
    let mut added: Vec<(String, usize)> = Vec::new();
    for &(path, count, _class) in KEPT_SITES {
        // Whether an earlier entry of the same file took this one's calls.
        let mut took_it: bool = false;
        for (named, total) in added.iter_mut() {
            if named == path {
                *total += count;
                took_it = true;
                break;
            }
        }
        if !took_it {
            added.push((String::from(path), count));
        }
    }
    added.sort();
    return added;
}

/// What:
///  `fn findings_in(root, roots, allowed) -> Result<Vec<String>, String>` is every release file whose calls
///  differ from what `allowed` gives it,
///  and every entry of `allowed` that no file answers.
/// Why:
///  One body answers both directions a drift can take,
///  and the check's own fixtures call it with a list they write,
///  so the counting and the comparison are shown to work on a tree that is not this crate's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = countsIn(root); const allowed = new Map(entries); return [...drift, ...entries.filter(([p]) => !found.has(p))]
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
    allowed: &[(String, usize)],
) -> Result<Vec<String>, String> {
    let counted: Vec<(String, usize)> = counts_in(root, roots)?;
    let mut findings: Vec<String> = Vec::new();
    for (path, calls) in counted.iter() {
        let allowed_here: usize = allowed
            .iter()
            .find(|(named, _total)| return named == path)
            .map(|(_named, total)| return *total)
            .unwrap_or(0);
        if *calls == allowed_here {
            continue;
        }
        if allowed_here == 0 {
            findings.push(format!(
                "{path}: makes {calls} call(s) that can wait and is not on the list"
            ));
        } else {
            findings.push(format!(
                "{path}: the list gives {allowed_here} call(s) and the file makes {calls}"
            ));
        }
    }
    for (path, allowed_here) in allowed {
        if counted.iter().any(|(named, _calls)| return named == path) {
            continue;
        }
        findings.push(format!(
            "{path}: the list gives {allowed_here} call(s) and the file makes none"
        ));
    }
    return Ok(findings);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots(), &listed()).expect("the crate's sources");
}

/// Nothing outside the plain-file reader reads a file with a call that can wait.
#[test]
fn no_release_file_outside_the_plain_file_reader_makes_a_call_that_can_wait() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "opening a named pipe for reading waits for a writer that may never come, so a file is read through \
         `crate::plain_file` (`read_plain_file`, `read_plain_text`, `read_plain_text_at`), which opens without blocking, \
         takes the type from the open handle and reads to a bound; a site that must stay goes on `KEPT_SITES` under the \
         class whose reason keeps it, and the list only shrinks (item 2 of `doc/debt-ledger-2026-11-25.md`):\n{}",
        found.join("\n")
    );
}

/// Every class an entry names carries a reason,
/// and a class no entry names is stale.
#[test]
fn every_listed_class_carries_a_reason_and_keeps_a_site() {
    let unused: Vec<String> = KEPT_CLASSES
        .iter()
        .filter(|(class, _why)| {
            return !KEPT_SITES
                .iter()
                .any(|(_path, _count, named)| return named == class);
        })
        .map(|(class, _why)| return String::from(*class))
        .collect();
    assert!(
        unused.is_empty(),
        "these classes keep no site, so remove them from `KEPT_CLASSES`:\n{}",
        unused.join("\n")
    );
    let unreasoned: Vec<String> = KEPT_SITES
        .iter()
        .filter(|(_path, _count, class)| {
            return !KEPT_CLASSES
                .iter()
                .any(|(named, _why)| return named == class);
        })
        .map(|(path, _count, class)| return format!("{path}: `{class}`"))
        .collect();
    assert!(
        unreasoned.is_empty(),
        "these entries name a class `KEPT_CLASSES` gives no reason for:\n{}",
        unreasoned.join("\n")
    );
}

/// Every listed file is still a release source,
/// so a file that moved or became test-only does not go on excusing a count.
#[test]
fn every_listed_file_is_a_release_source() {
    let sources: Vec<String> =
        release_sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = listed()
        .into_iter()
        .filter(|(path, _total)| return !sources.iter().any(|file| return file == path))
        .map(|(path, total)| return format!("{path}: {total}"))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name no release source of this crate, so remove them:\n{}",
        stale.join("\n")
    );
}

/// The scan fires on a planted read and stays quiet for the same module reading through `plain_file`,
/// so a check that stopped reading or a list that stopped being compared fails here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_read_and_passes_a_plain_one() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-raw-file-reads");
    write_tree(
        root.as_path(),
        &[
            (
                "raw.rs",
                "pub fn read(path: &std::path::Path) -> Vec<u8> {\n    return std::fs::read(path).unwrap_or_default();\n}\n",
            ),
            (
                "plain.rs",
                "pub fn read(path: &std::path::Path) -> Vec<u8> {\n    return crate::plain_file::read_plain_file(path, \
                 crate::plain_file::SMALL_FILE_LIMIT).unwrap_or_default();\n}\n",
            ),
            (
                "commented.rs",
                "// A comment that names std::fs::read(path) is not a call.\npub fn read() -> usize {\n    return 0;\n}\n",
            ),
            (
                "lib.rs",
                "pub mod commented;\npub mod plain;\npub mod raw;\n",
            ),
        ],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("lib.rs")],
        &[],
    )
    .expect("the fixture");
    assert_eq!(
        found,
        vec![String::from(
            "raw.rs: makes 1 call(s) that can wait and is not on the list"
        )],
        "one read is planted and one reads a plain file"
    );
    crate::test_support::remove(root.as_path());
}

/// A file that makes one more call than the list gives it is a finding,
/// so a second read cannot ride along beside a listed one.
#[test]
fn a_file_that_makes_more_calls_than_the_list_gives_is_a_finding() {
    let root: std::path::PathBuf =
        crate::test_support::fixture("source-guard-raw-file-reads-count");
    write_tree(
        root.as_path(),
        &[(
            "twice.rs",
            "pub fn read(path: &std::path::Path) -> usize {\n    let first: usize = std::fs::read(path).unwrap_or_default().len();\n    \
             let second: usize = std::fs::read(path).unwrap_or_default().len();\n    return first + second;\n}\n",
        )],
    );
    let allowed: Vec<(String, usize)> = vec![(String::from("twice.rs"), 1)];
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("twice.rs")],
        &allowed,
    )
    .expect("the fixture");
    assert_eq!(
        found,
        vec![String::from(
            "twice.rs: the list gives 1 call(s) and the file makes 2"
        )],
        "both calls of one file are counted"
    );
    crate::test_support::remove(root.as_path());
}

/// A listed file that makes no call any more is a finding,
/// so the list shrinks when a site is moved rather than keeping a count that excuses nothing.
#[test]
fn a_listed_site_that_is_gone_is_a_finding() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-raw-file-reads-gone");
    write_tree(
        root.as_path(),
        &[(
            "moved.rs",
            "pub fn read(path: &std::path::Path) -> Vec<u8> {\n    return crate::plain_file::read_plain_file(path, \
             crate::plain_file::SMALL_FILE_LIMIT).unwrap_or_default();\n}\n",
        )],
    );
    let allowed: Vec<(String, usize)> = vec![(String::from("moved.rs"), 1)];
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("moved.rs")],
        &allowed,
    )
    .expect("the fixture");
    assert_eq!(
        found,
        vec![String::from(
            "moved.rs: the list gives 1 call(s) and the file makes none"
        )],
        "a moved site lowers the list"
    );
    crate::test_support::remove(root.as_path());
}
