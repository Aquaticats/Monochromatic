//! What:
//!  The controls of the list a removal prints:
//!  the slot,
//!  the dataset,
//!  every snapshot name,
//!  the bookmarks that go with the head,
//!  and the sentence that nothing comes back.
//! Why:
//!  The project decided the printed report is the only trace that outlives the removal beyond its
//!  records,
//!  so a list that omitted a name would leave a person with no way to ask about it
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "What trace of a destruction outlives the removal").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(lost.lines()).toContain('  snapshot tank/wt/x@own');
//! ```

use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;

/// The list one control prints.
fn a_list() -> NamesToBeLost {
    return NamesToBeLost {
        slot: String::from("fix-flaky-test"),
        dataset: String::from("tank/wt/fix-flaky-test"),
        snapshots: vec![
            String::from("tank/wt/fix-flaky-test@sanoid-hourly"),
            String::from("tank/wt/fix-flaky-test@worktree-own"),
        ],
        bookmarks: vec![String::from("tank/wt/fix-flaky-test#from-own")],
    };
}

#[test]
fn the_list_names_the_slot_the_dataset_and_every_snapshot_in_the_plans_order() {
    let lines: Vec<String> = a_list().lines();
    assert!(
        lines[0].contains("fix-flaky-test") && lines[0].contains("tank/wt/fix-flaky-test"),
        "the first line names the slot and the dataset: {}",
        lines[0]
    );
    assert_eq!(lines[1], "  snapshot tank/wt/fix-flaky-test@sanoid-hourly");
    assert_eq!(lines[2], "  snapshot tank/wt/fix-flaky-test@worktree-own");
    assert!(
        lines[3].contains("bookmark tank/wt/fix-flaky-test#from-own"),
        "the bookmark is named: {}",
        lines[3]
    );
    assert!(
        lines[3].contains("cannot be restored"),
        "a bookmark that goes with the head says so: {}",
        lines[3]
    );
    let last: &String = lines.last().expect("the list has a last line");
    assert!(
        last.contains("2 snapshot(s)") && last.contains("1 bookmark(s)"),
        "the count says what is lost: {last}"
    );
    assert!(
        last.contains("cannot be restored by this wrapper"),
        "the list says the loss cannot be undone: {last}"
    );
    assert!(
        last.contains("the main checkout keeps every snapshot it holds"),
        "the list says what stands: {last}"
    );
}

#[test]
fn a_clone_with_no_bookmark_says_zero_and_a_clone_with_none_lost_is_empty() {
    let mut list: NamesToBeLost = a_list();
    list.bookmarks = Vec::new();
    assert!(
        list.text().contains("0 bookmark(s)"),
        "zero bookmarks still reads as a count"
    );
    list.snapshots = Vec::new();
    assert!(
        list.text().contains("0 snapshot(s)"),
        "a removal with no own snapshot prints its count as zero"
    );
}
