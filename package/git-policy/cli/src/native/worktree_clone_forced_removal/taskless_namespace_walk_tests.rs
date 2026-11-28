//! Controls for the walk that finds a mount namespace no task holds:
//!  a kernel without the call is refused with the design's own word,
//!  every other refusal is blind,
//!  the difference between the listed ids and the held ones is the taskless
//!  set,
//!  a task's namespace answers its id and a task that has gone is not blind,
//!  and this machine's own listing holds this process's own namespace.
//! Why:
//!  The guest arm measures the kernel's answers on the template's own release,
//!  and this file is what fails when the walk's decision around them is wrong
//!  or when a kernel that cannot answer is not refused by name.
//!  Nothing here changes a namespace:
//!  the reads are `listns` and `NS_GET_ID` on this process's own `/proc`,
//!  and both are reads.

use super::*;
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::the_namespace_listing::TheCallsRefusal;

/// A kernel that answers `ENOSYS` is refused,
/// with the word the design gives a namespace this build cannot reach.
#[test]
fn a_kernel_without_listns_is_refused_with_the_designs_word() {
    let refusal: TasklessWalkRefusal = the_walks_refusal_of_the_call(TheCallsRefusal {
        calling: "listns",
        errno: libc::ENOSYS,
    });
    assert_eq!(refusal.word(), RefusalWord::HolderUnendable, "{refusal}");
    assert!(
        refusal.to_string().contains("has no such call"),
        "{refusal}"
    );
}

/// A listing the kernel refused is blind and not unendable,
/// because the call is there and answered something else.
#[test]
fn a_refused_listing_is_a_blind_read() {
    let refusal: TasklessWalkRefusal = the_walks_refusal_of_the_call(TheCallsRefusal {
        calling: "listns",
        errno: libc::EPERM,
    });
    assert_eq!(refusal.word(), RefusalWord::HolderBlind, "{refusal}");
    assert!(
        refusal.to_string().contains("refused the listing"),
        "{refusal}"
    );
}

/// The ids one task holds are not taskless,
/// and every listed id no task holds is.
#[test]
fn the_difference_between_the_listing_and_the_tasks_is_the_taskless_set() {
    let listed: Vec<u64> = vec![8, 10, 18, 23];
    // What: the ids the control says this machine's tasks hold.
    // Why: the walk's decision is this set taken out of the listing.
    let held: BTreeSet<u64> = BTreeSet::from([8, 18, 23]);
    assert_eq!(the_ids_no_task_holds(listed.as_slice(), &held), vec![10]);
    // A positive control: with the held set empty every listed id is taskless, so the comparison can move.
    let none_held: BTreeSet<u64> = BTreeSet::new();
    assert_eq!(the_ids_no_task_holds(listed.as_slice(), &none_held), listed);
    // An id a task holds that the listing never gave is not a namespace the walk may act on.
    let extra_held: BTreeSet<u64> = BTreeSet::from([8, 18, 23, 99]);
    assert_eq!(
        the_ids_no_task_holds(listed.as_slice(), &extra_held),
        vec![10]
    );
}

/// Every refusal that is not a kernel without the call is blind.
#[test]
fn every_other_refusal_is_a_blind_read() {
    let refusals: [TasklessWalkRefusal; 4] = [
        TasklessWalkRefusal::TheListingWasRefused {
            errno: libc::EPERM,
        },
        TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
            task: PathBuf::from("/proc/1"),
            errno: libc::EACCES,
        },
        TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
            id: 166,
            errno: libc::ESTALE,
        },
        TasklessWalkRefusal::ANamespacesTableCouldNotBeRead {
            identity: String::from("mnt:[4026532397]"),
            said: String::from("the child never answered"),
        },
    ];
    for refusal in refusals {
        assert_eq!(refusal.word(), RefusalWord::HolderBlind, "{refusal}");
    }
}

/// What:
///  A folder shaped like `/proc` that holds one task entry,
///  whose `ns/mnt` names this process's own mount namespace.
/// Why:
///  The walk reads each task's namespace through its own link,
///  and a control that read this machine's whole `/proc` would need every
///  other account's link,
///  which an unprivileged control may not open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = mkdir(`${fixture}/proc/${pid}/ns`); symlink('/proc/self/ns/mnt', `${table}/mnt`);
/// ```
fn a_process_table_holding_this_process() -> PathBuf {
    // What: one folder of this call's own, which the test thread removes however the control ends.
    // Why: a fixed path outside the fixture root would be shared with every other run.
    let root: PathBuf = crate::testing::test_support::fixture("walked-namespaces");
    let table: PathBuf = root.join("proc");
    // What: this process's number, which is the entry's name in the shaped table.
    // Why: the walk skips any entry whose name is not a process number.
    let pid: String = std::process::id().to_string();
    let links: PathBuf = table.join(pid).join("ns");
    std::fs::create_dir_all(links.as_path()).expect("the shaped table's own folders");
    std::os::unix::fs::symlink(
        crate::worktree_clone_forced_removal::the_namespace_listing::OUR_MOUNT_NAMESPACE,
        links.join("mnt"),
    )
    .expect("a link to this process's own namespace");
    return table;
}

/// The ids a table of one entry answers are that entry's namespace,
/// and a table with no entry answers none.
#[test]
fn a_tasks_namespace_answers_its_id_and_a_table_with_no_task_answers_none() {
    let table: PathBuf = a_process_table_holding_this_process();
    let held: BTreeSet<u64> = the_ids_this_machines_tasks_hold(table.as_path())
        .expect("this process's own namespace link is readable");
    assert_eq!(held.len(), 1, "one entry answers one id: {held:?}");
    // What: one more table, which holds no process at all.
    // Why: the empty table is the control that the set is the table's and not a constant.
    let empty: PathBuf = crate::testing::test_support::fixture("walked-namespaces-empty");
    let none: PathBuf = empty.join("proc");
    std::fs::create_dir_all(none.as_path()).expect("an empty table");
    let answered: BTreeSet<u64> =
        the_ids_this_machines_tasks_hold(none.as_path()).expect("an empty table is no refusal");
    assert!(
        answered.is_empty(),
        "an empty table answers no id: {answered:?}"
    );
}

/// A task whose namespace link is not there is gone rather than blind,
/// which is the rule the scan's own reads keep.
#[test]
fn a_task_that_ended_is_gone_and_not_a_blind_read() {
    let empty: PathBuf = crate::testing::test_support::fixture("walked-namespaces-gone");
    let table: PathBuf = empty.join("proc");
    // What: the entry of a task whose `ns/mnt` link no run made.
    // Why: the open of a link that is not there answers `ENOENT`, which is a task that is gone.
    std::fs::create_dir_all(table.join("999999").as_path()).expect("a task's own folder");
    let answered: BTreeSet<u64> =
        the_ids_this_machines_tasks_hold(table.as_path()).expect("a gone task is no refusal");
    assert!(
        answered.is_empty(),
        "a gone task holds nothing: {answered:?}"
    );
}

/// This machine's listing holds this process's own mount namespace,
/// or the kernel has no such call and the walk refuses with the design's word.
/// Why:
///  The first half is the positive control the guest arm's own answer stands on:
///  a listing that answered no id would pass every assertion about a difference
///  between two sets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours = theIdOf('/proc/self/ns/mnt'); const listed = theMountNamespacesOnThisMachine();
/// ```
#[test]
fn this_machines_listing_holds_its_own_mount_namespace_or_refuses_the_call() {
    let listed: Vec<u64> =
        match crate::worktree_clone_forced_removal::the_namespace_listing::the_mount_namespaces_on_this_machine()
        {
            Ok(ids) => ids,
            Err(refusal) => {
                let walk: TasklessWalkRefusal = the_walks_refusal_of_the_call(refusal);
                // A kernel with no such call is the owner's own country; a refusal of any other errno means this
                // control could not read the machine and says which.
                assert!(
                    walk.word() == RefusalWord::HolderUnendable
                        || walk.word() == RefusalWord::HolderBlind,
                    "{walk}"
                );
                return;
            }
        };
    let table: PathBuf = a_process_table_holding_this_process();
    // What: the one id this process's own namespace answered.
    // Why: our own namespace is the row a listing of this machine must hold, and no other test can remove it.
    let held: BTreeSet<u64> = the_ids_this_machines_tasks_hold(table.as_path())
        .expect("this process's own namespace is readable");
    let ours: u64 = *held
        .iter()
        .next()
        .expect("our own namespace answered an id");
    assert!(
        listed.contains(&ours),
        "the listing does not hold this process's own namespace {ours}: listed {listed:?}"
    );
    // The walk's own decision over the two sets is what the guest arm's claim rests on: our own namespace is not
    // taskless, because this process is in it.
    assert!(
        !the_ids_no_task_holds(listed.as_slice(), &held).contains(&ours),
        "our own namespace read as taskless"
    );
}
