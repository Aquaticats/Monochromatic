//! What:
//!  Controls for the journal lines:
//!  each line carries its event's own fields,
//!  every value is quoted so a space cannot split it,
//!  the record names the blocker and its end,
//!  the structural refusal says no copy moved,
//!  and the closing line counts what the design asks.
//! Why:
//!  The journal is the only record of who a forced removal ended,
//!  so a line that lost a field or split an event loses the record itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(quoted("a b'c")).toBe(`'a b'\\''c'`);
//! ```

use super::{
    ABlockersEnd, AClosingLine, ADestinationDirectory, AHolderNotEnded, APropagationChange, ARefusedNamespace, AStoppedUnit,
    AStructuralRefusal, TheActsJournal, as_a_block, dataset_word, quoted,
};
use crate::worktree_clone_forced_removal::blocker_end::EndResult;
use crate::worktree_clone_forced_removal::forced_removal_words::Stage;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The nonce every line of these controls repeats.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a canonical nonce");
}

/// The act's own two fields.
fn a_journal<'a>(nonce: &'a RequestNonce) -> TheActsJournal<'a> {
    return TheActsJournal { dataset: "tank/clones/one", nonce };
}

#[test]
fn a_value_with_a_space_or_a_quote_stays_one_field() {
    assert_eq!(quoted("a b"), "'a b'");
    assert_eq!(quoted("a'b"), "'a'\\''b'");
    assert_eq!(quoted(""), "''");
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).a_holder_not_ended(&AHolderNotEnded {
        namespace: "mnt:[1]",
        mount_id: Some(42),
        holder: "a command with spaces",
        failed_proof: "the read answered EACCES",
    });
    assert!(line.contains("holder='a command with spaces'"), "the holder stays one value, and the line reads {line}");
    assert!(line.contains("failed-proof='the read answered EACCES'"), "the proof stays one value, and the line reads {line}");
}

#[test]
fn a_blocker_s_line_names_what_was_found_sent_and_become_of_it() {
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).a_blocker_ended(&ABlockersEnd {
        namespace: "mnt:[4026532001]",
        mount_id: Some(77),
        pid: 4321,
        started: 987654,
        session: 4300,
        found_by: "cwd",
        signals: &["SIGTERM", "SIGKILL"],
        result: EndResult::Killed,
    });
    assert!(line.starts_with("forced-removal blocker-ended "), "the line names its event first, and it reads {line}");
    for wanted in ["dataset='tank/clones/one'", "nonce='00112233445566778899aabbccddeeff'", "namespace='mnt:[4026532001]'", "mount=77", "pid=4321", "started=987654", "session=4300", "found-by='cwd'", "signals='SIGTERM,SIGKILL'", "result=killed"] {
        assert!(line.contains(wanted), "the line carries `{wanted}`, and it reads {line}");
    }
    assert!(!line.contains('\n'), "a journal line is one line");
}

#[test]
fn a_holder_that_held_the_mount_without_a_copy_names_no_mount() {
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).a_blocker_ended(&ABlockersEnd {
        namespace: "mnt:[7]",
        mount_id: None,
        pid: 1,
        started: 2,
        session: 1,
        found_by: "map_files",
        signals: &[],
        result: EndResult::Gone,
    });
    assert!(line.contains("mount=none"), "a path holder names no copy's mount, and it reads {line}");
    assert!(line.contains("signals=''"), "nothing was sent to a task that had gone, and it reads {line}");
    assert!(line.contains("result=gone"));
}

#[test]
fn the_propagation_line_names_both_tag_words_and_whether_the_removal_finished() {
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).a_propagation_change(&APropagationChange {
        namespace: "mnt:[8]",
        mount_id: 31,
        was: "shared:12",
        now: "master:12",
        change: "slave",
        finished: false,
    });
    assert!(line.contains("was='shared:12'"), "and it reads {line}");
    assert!(line.contains("now='master:12'"));
    assert!(line.contains("change='slave'"));
    assert!(line.contains("removal=failed"), "a change left after a failed removal says so, and it reads {line}");
}

#[test]
fn the_structural_refusal_says_no_copy_moved_and_what_was_changed() {
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).a_structural_refusal(&AStructuralRefusal {
        namespace: "mnt:[9]",
        changed: &["mnt:[8]"],
        directory: "removed",
        said: "mount MS_SLAVE answered EINVAL",
    });
    assert!(line.contains("copy-moved=no"), "the third state's own fact, and it reads {line}");
    assert!(line.contains("changed='mnt:[8]'"));
    assert!(line.contains("directory='removed'"));
}

#[test]
fn the_other_lines_carry_their_own_facts() {
    let nonce: RequestNonce = a_nonce();
    let journal: TheActsJournal<'_> = a_journal(&nonce);
    let directory: String = journal.a_destination_directory(&ADestinationDirectory { namespace: "mnt:[3]", name: ".git-worktree-clone-removal-9", removed: true });
    assert!(directory.contains("name='.git-worktree-clone-removal-9'"));
    assert!(directory.contains("left=no"), "the directory's life is the line's own field, and it reads {directory}");
    let unit: String = journal.a_stopped_unit(&AStoppedUnit { unit: "watcher.service", cgroup: "/user.slice/watcher.service", namespace: "mnt:[4]" });
    assert!(unit.contains("unit='watcher.service'"), "and it reads {unit}");
    assert!(unit.contains("again=run-the-removal-and-start-the-unit"), "the person is told the unit may be started again");
    let refused: String = journal.a_refused_namespace(&ARefusedNamespace { namespace: "mnt:[5]", word: "holder-blind", said: "EACCES" });
    assert!(refused.contains("word='holder-blind'"), "and it reads {refused}");
}

#[test]
fn the_closing_line_counts_what_the_design_asks() {
    let nonce: RequestNonce = a_nonce();
    let line: String = a_journal(&nonce).closed(&AClosingLine { ended: 3, unproven: 1, scan_clear: true, propagation_changes: 2, stage: Stage::Done });
    for wanted in ["ended=3", "unproven=1", "scan=clear", "changed=2", "stage='done'"] {
        assert!(line.contains(wanted), "the closing line carries `{wanted}`, and it reads {line}");
    }
    let not_clear: String = a_journal(&nonce).closed(&AClosingLine { ended: 0, unproven: 0, scan_clear: false, propagation_changes: 0, stage: Stage::Unmount });
    assert!(not_clear.contains("scan=not-clear"));
}

#[test]
fn a_block_of_lines_puts_one_event_on_each_line_and_names_the_dataset_canonically() {
    let nonce: RequestNonce = a_nonce();
    let lines: Vec<String> = vec![a_journal(&nonce).closed(&AClosingLine { ended: 0, unproven: 0, scan_clear: true, propagation_changes: 0, stage: Stage::Done })];
    let text: String = as_a_block(lines.as_slice());
    assert_eq!(text.lines().count(), 1);
    assert!(text.ends_with('\n'));
    let guid: DatasetGuid = DatasetGuid::parse("18446744073709551615").expect("a canonical GUID");
    assert_eq!(dataset_word(guid), guid.file_name());
}
