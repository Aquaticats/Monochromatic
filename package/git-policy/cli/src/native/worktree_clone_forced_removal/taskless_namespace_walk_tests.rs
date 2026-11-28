//! Controls for the walk that finds a mount namespace no task holds:
//!  the paging stops at the kernel's end of the list and never loops,
//!  a kernel without the call is refused with the design's own word,
//!  the difference between the listed ids and the held ones is the taskless
//!  set,
//!  the ioctl number is the one the C library's own macro builds,
//!  and this machine's own listing holds this process's own namespace.
//! Why:
//!  The guest arm measures the kernel's answers on the template's own release,
//!  and this file is what fails when the walk's own logic around them is wrong
//!  or when a kernel that cannot answer is not refused by name.
//!  Nothing here changes a namespace:
//!  the two reads are `listns` and `NS_GET_ID` on this process's own `/proc`,
//!  and both are reads.

use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::taskless_namespace_walk::*;
use std::collections::BTreeSet;

/// What:
///  A page the walk is handed in place of the kernel's answer.
/// Why:
///  The paging rule is the walk's own decision,
///  and a control that needed a kernel to answer twice could not run on a
///  machine whose kernel has no `listns` at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pages = [[1, 2], [3]]; let at = 0; const ask = () => ({ ids: pages[at++] ?? [], errno: at > pages.length ? 21 : undefined });
/// ```
fn the_pages(
    pages: &[&[u64]],
    answer: Option<i32>,
) -> impl FnMut(Option<u64>) -> PageAnswer {
    // What: the pages this control was handed, copied so the returned closure owns them.
    // Why: a closure that borrowed them could not outlive this call, and the walk asks it from outside.
    let pages: Vec<Vec<u64>> = pages.iter().map(|page| return page.to_vec()).collect();
    // What: how many calls this control has answered.
    // Why: the fabricated pages are handed out in order and the last call gets the errno.
    let mut called: usize = 0;
    return move |_cursor: Option<u64>| {
        let page: &[u64] = match pages.get(called) {
            Some(ids) => ids.as_slice(),
            None => &[],
        };
        called += 1;
        // What: nothing to call when the pages run out, which is the walk's own end of the list.
        // Why: the control states the end rather than a kernel answering it.
        if called > pages.len() {
            return PageAnswer {
                ids: Vec::new(),
                errno: answer,
            };
        }
        return PageAnswer {
            ids: page.to_vec(),
            errno: None,
        };
    };
}

/// Two pages and the kernel's end of the list answer every id the pages held.
#[test]
fn the_pages_are_walked_until_the_kernel_says_the_list_ended() {
    let mut ask = the_pages(&[&[8, 10, 18], &[23, 25]], Some(libc::ENOENT));
    let listed: Vec<u64> = pages_until_the_end(|cursor| return ask(cursor))
        .expect("the end of the list is not a refusal");
    assert_eq!(listed, vec![8, 10, 18, 23, 25]);
}

/// A kernel that answers `ENOSYS` is refused,
/// with the word the design gives a namespace this build cannot reach.
#[test]
fn a_kernel_without_listns_is_refused_with_the_designs_word() {
    let mut ask = the_pages(&[], Some(libc::ENOSYS));
    let refusal: TasklessWalkRefusal = pages_until_the_end(|cursor| return ask(cursor))
        .expect_err("a kernel without the call is a refusal");
    assert_eq!(refusal.word(), RefusalWord::HolderUnendable, "{refusal}");
    assert!(
        refusal.to_string().contains("has no such call"),
        "{refusal}"
    );
}

/// A refused listing is blind and not unendable,
/// because the call is there and answered something else.
#[test]
fn a_refused_listing_is_a_blind_read() {
    let mut ask = the_pages(&[], Some(libc::EPERM));
    let refusal: TasklessWalkRefusal = pages_until_the_end(|cursor| return ask(cursor))
        .expect_err("a refused listing is a refusal");
    assert_eq!(refusal.word(), RefusalWord::HolderBlind, "{refusal}");
    assert!(
        refusal.to_string().contains("refused the listing"),
        "{refusal}"
    );
}

/// A page that adds no id and answers no errno ends the walk,
/// which is what stops it asking forever on a kernel that never says
/// `ENOENT`.
#[test]
fn a_page_that_adds_nothing_ends_the_walk() {
    let mut ask = the_pages(&[&[8, 10]], None);
    let listed: Vec<u64> = pages_until_the_end(|cursor| return ask(cursor))
        .expect("a page that adds nothing ends the list");
    assert_eq!(listed, vec![8, 10]);
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

/// The ioctl number this build builds is the one the C library builds for
/// the same macro,
/// which is the control that the arithmetic in this file is the kernel's
/// `_IOR`.
#[test]
fn the_namespace_ioctl_number_is_the_one_the_c_library_builds() {
    // `NS_GET_USERNS` is `_IO(NSIO, 0x1)`: the same group, the same shift, and no width.
    assert_eq!(libc::NS_GET_USERNS, (NSIO << 8) | 0x1);
    assert_eq!(
        NS_GET_ID,
        (IOCTL_READING << 30) | (NS_GET_ID_WIDTH << 16) | (NSIO << 8) | NS_GET_ID_NUMBER
    );
    assert_ne!(NS_GET_ID, libc::NS_GET_USERNS);
}

/// The handle this build writes is the width and the type the template's
/// kernel decodes,
/// with the id where the kernel reads it.
#[test]
fn the_handle_carries_the_id_where_the_kernel_reads_it() {
    // What: the id a control turns into a handle.
    // Why: the bytes are the only thing the kernel reads, and this states where.
    let id: u64 = 0x0102_0304_0506_0708;
    let mut handle: TheHandle = TheHandle {
        handle_bytes: NSFS_FILE_HANDLE_BYTES as u32,
        handle_type: FILEID_NSFS,
        f_handle: [0; NSFS_FILE_HANDLE_BYTES],
    };
    handle.f_handle[0..8].copy_from_slice(&id.to_ne_bytes());
    assert_eq!(handle.handle_bytes as usize, NSFS_FILE_HANDLE_BYTES);
    assert_eq!(handle.handle_type, 0xf1);
    assert_eq!(&handle.f_handle[0..8], &id.to_ne_bytes());
    assert!(handle.f_handle[8..].iter().all(|byte| return *byte == 0));
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
fn a_process_table_holding_this_process() -> std::path::PathBuf {
    // What: one folder of this call's own, which the test thread removes however the control ends.
    // Why: a fixed path outside the fixture root would be shared with every other run.
    let root: std::path::PathBuf = crate::testing::test_support::fixture("walked-namespaces");
    let table: std::path::PathBuf = root.join("proc");
    // What: this process's number, which is the entry's name in the shaped table.
    // Why: the walk skips any entry whose name is not a process number.
    let pid: String = std::process::id().to_string();
    let links: std::path::PathBuf = table.join(pid).join("ns");
    std::fs::create_dir_all(links.as_path()).expect("the shaped table's own folders");
    std::os::unix::fs::symlink(OUR_MOUNT_NAMESPACE, links.join("mnt"))
        .expect("a link to this process's own namespace");
    return table;
}

/// The ids a table of one entry answers are that entry's namespace,
/// and a table with no entry answers none.
#[test]
fn a_tasks_namespace_answers_its_id_and_a_table_with_no_task_answers_none() {
    let table: std::path::PathBuf = a_process_table_holding_this_process();
    let held: BTreeSet<u64> = the_ids_this_machines_tasks_hold(table.as_path())
        .expect("this process's own namespace link is readable");
    assert_eq!(held.len(), 1, "one entry answers one id: {held:?}");
    // What: one more table, which holds no process at all.
    // Why: the empty table is the control that the set is the table's and not a constant.
    let empty: std::path::PathBuf =
        crate::testing::test_support::fixture("walked-namespaces-empty");
    let none: std::path::PathBuf = empty.join("proc");
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
    let empty: std::path::PathBuf = crate::testing::test_support::fixture("walked-namespaces-gone");
    let table: std::path::PathBuf = empty.join("proc");
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

/// Every refusal that is not a kernel without the call is blind.
#[test]
fn every_other_refusal_is_a_blind_read() {
    let refusals: [TasklessWalkRefusal; 4] = [
        TasklessWalkRefusal::TheListingWasRefused {
            errno: libc::EPERM,
        },
        TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
            task: std::path::PathBuf::from("/proc/1"),
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
///  This machine's answer to the listing,
///  and the reason the two branches of the control below are the whole of it.
/// Why:
///  A kernel of `6.19` or newer lists,
///  and every guest of `6.8` the arms ran on answers `ENOSYS`,
///  so the control must pass on both and say which it ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = theMountNamespacesOnThisMachine();   // ok, or the ENOSYS refusal
/// ```
fn the_listing_here() -> Result<Vec<u64>, TasklessWalkRefusal> {
    return the_mount_namespaces_on_this_machine();
}

/// What:
///  This machine's own answer:
///  either the listing holds this process's own mount namespace,
///  or the kernel has no such call and the walk refuses with the design's
///  word.
/// Why:
///  The first half is the positive control the guest arm's own answer stands
///  on:
///  a listing that answered no id would pass every assertion about a
///  difference between two sets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours = theIdOf('/proc/self/ns/mnt'); const listed = theMountNamespacesOnThisMachine();
/// ```
#[test]
fn this_machines_listing_holds_its_own_mount_namespace_or_refuses_the_call() {
    let listed: Vec<u64> = match the_listing_here() {
        Ok(ids) => ids,
        Err(refusal) => {
            assert_eq!(refusal.word(), RefusalWord::HolderUnendable, "{refusal}");
            assert!(
                matches!(refusal, TasklessWalkRefusal::TheKernelHasNoListns { .. }),
                "{refusal}"
            );
            return;
        }
    };
    let table: std::path::PathBuf = a_process_table_holding_this_process();
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
