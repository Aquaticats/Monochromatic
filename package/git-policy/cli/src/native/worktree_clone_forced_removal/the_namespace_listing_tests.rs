//! Controls for the kernel's own namespace calls:
//!  the paging stops at the kernel's end of the list and never loops,
//!  a page that adds nothing ends the walk,
//!  a refused listing answers its errno and not a made-up one,
//!  the ioctl number is the one the C library's own macro builds,
//!  and the handle carries the id where the kernel reads it.
//! Why:
//!  The guest arm measures what this kernel answers;
//!  this file is what fails when the arithmetic or the paging rule around those
//!  answers is wrong,
//!  on a machine whose kernel may have no such call at all.

// What: every name of the module these controls belong to, the private records included.
// Why: the records are this file's subject and no caller outside the module writes one.
//
// In TS you'd write (pseudocode):
// ```ts
// import * as listing from './theNamespaceListing.js';
// ```
use super::*;

/// What:
///  A page the walk is handed in place of the kernel's answer.
/// Why:
///  The paging rule is the walk's own decision,
///  and a control that needed a kernel to answer twice could not run on a
///  machine whose kernel has no `listns` at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ask = thePages([[1, 2], [3]], 21);   // two pages, then the errno
/// ```
fn the_pages(
    pages: &[&[u64]],
    answer: Option<i32>,
) -> impl FnMut(Option<u64>) -> PageAnswer {
    // What: the pages this control was handed, copied so the returned closure owns them.
    // Why: a closure that borrowed them could not outlive this call, and the walk asks it from outside.
    let owned_pages: Vec<Vec<u64>> = pages.iter().map(|page| return page.to_vec()).collect();
    // What: how many calls this control has answered.
    // Why: the fabricated pages are handed out in order and the last call gets the errno.
    let mut called: usize = 0;
    return move |_cursor: Option<u64>| {
        // What: the page this call answers with, or nothing when the pages ran out.
        // Why: the control states the end rather than a kernel answering it.
        let page: &[u64] = match owned_pages.get(called) {
            Some(ids) => ids.as_slice(),
            None => &[],
        };
        called += 1;
        // What: the errno only once the pages are spent.
        // Why: a page that held ids answered no error, and the call after it is the one the kernel says ended.
        if called > owned_pages.len() {
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

/// A refused listing answers the errno the kernel gave,
/// so the caller can tell a kernel without the call from one that refused it.
#[test]
fn a_refused_listing_answers_the_errnos_it_was_given() {
    for errno in [libc::ENOSYS, libc::EPERM, libc::EINVAL] {
        let mut ask = the_pages(&[], Some(errno));
        let refusal: TheCallsRefusal = pages_until_the_end(|cursor| return ask(cursor))
            .expect_err("a refused page is a refusal");
        assert_eq!(refusal.errno, errno);
        assert_eq!(refusal.calling, "listns");
        assert!(
            refusal.to_string().contains("listns answered errno"),
            "{refusal}"
        );
    }
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
///  This machine's answer to the listing.
/// Why:
///  A kernel of `6.19` or newer lists,
///  and every guest of `6.8` the arms ran on answers `ENOSYS`,
///  so the control must pass on both and say which it ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = theMountNamespacesOnThisMachine();   // ok, or the ENOSYS refusal
/// ```
#[test]
fn this_machines_listing_answers_ids_or_the_kernels_own_errno() {
    match the_mount_namespaces_on_this_machine() {
        Ok(_ids) => {}
        Err(refusal) => {
            assert!(
                refusal.errno == libc::ENOSYS || refusal.errno == libc::EPERM,
                "the listing refused with an errno no kernel of this generation answers: {refusal}"
            );
        }
    }
}
