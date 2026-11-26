//! What:
//!  The act's unmount stage, in both halves:
//!  the half the caller runs, which enters the holder's namespace through one
//!  single-threaded child and reads the child's bounded report,
//!  and the half the child runs there,
//!  which unmounts the copy in place by the handle or moves it onto way 7's
//!  directory and unmounts it there,
//!  then removes that directory.
//! Why:
//!  This is the only stage that drops the copy,
//!  and it runs after every signal,
//!  so the act's last change is the one that releases the dataset's hold
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Way 8,
//!  in place by the handle").
//!  Every call here names the copy by a descriptor and never by a path,
//!  so no name resolved after the proof can steer it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = takeTheCopyOut(ready, copy);     // after the signals
//! ```

use super::the_copy_taken_out_of_a_namespace::{
    CopyRefusal,
    NamespaceCopy,
    Ready,
    WayOut,
    refused_from,
};
use super::the_mount_table_the_act_reads::{
    read_the_table,
    row_of,
    rows_of,
};
use crate::worktree_clone_forced_removal::descriptor_calls::{
    open_directory,
    open_directory_below,
    remove_directory_at,
};
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::mount_namespace_child::{
    ActReport,
    run_in,
};
use crate::worktree_clone_forced_removal::receiver_closure::MountRow;
use crate::worktree_clone_forced_removal::recorded_handle::open_from_the_handle;
use std::ffi::CString;
use std::os::fd::AsRawFd;

/// What:
///  The unmount stage:
///  unmount the copy in place by the handle,
///  or move it onto the directory and unmount it there,
///  then remove the directory.
/// Why:
///  This is the only stage that drops the copy,
///  and it runs after every signal,
///  so the act's last change is the one that releases the dataset's hold.
///
/// # Errors
/// The namespace could not be entered,
/// the copy is not where the structural stage left it,
/// the unmount, the move or the directory's removal was refused.
pub fn take_the_copy_out(
    ready: &Ready,
    copy: &NamespaceCopy<'_>,
) -> Result<WayOut, CopyRefusal> {
    let planned: Ready = ready.clone();
    let report: ActReport = run_in(copy.namespace, || return unmount_in_here(&planned, copy))
        .map_err(|refusal| {
            return CopyRefusal {
                word: RefusalWord::HolderUnendable,
                calling: refusal.calling,
                errno: refusal.errno,
                said: refusal.said,
            };
        })?;
    if report.outcome
        != crate::worktree_clone_forced_removal::mount_namespace_child::ActOutcome::Done
    {
        return Err(refused_from(&report));
    }
    let rows: Vec<MountRow> = rows_of(report.table.as_str());
    if row_of(&rows, ready.mount_id).is_some() {
        return Err(CopyRefusal {
            word: RefusalWord::HolderUnendable,
            calling: "mountinfo",
            errno: None,
            said: format!(
                "the copy's mount {} is still in the table the child read after the unmount",
                ready.mount_id
            ),
        });
    }
    return Ok(WayOut {
        mount_id: ready.mount_id,
        directory: ready.directory.clone(),
    });
}

/// What:
///  The unmount stage as the child runs it,
///  inside the holder's namespace.
/// Why:
///  The unmount names the copy by a descriptor,
///  never by a path,
///  and way 7's move names its source and destination by descriptors too,
///  so no name resolved after the proof can steer the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (way === 'in-place-by-the-handle') { umount2(fdPath(covered), 0); }
/// else { move_mount(copyFd, '', destinationFd, '', F_EMPTY_PATH|T_EMPTY_PATH); umount2(fdPath(destination), 0); unlinkat(checkout, directory, AT_REMOVEDIR); }
/// return done(...);
/// ```
fn unmount_in_here(
    ready: &Ready,
    copy: &NamespaceCopy<'_>,
) -> ActReport {
    let step = |calling: &'static str, error: std::io::Error, said: &str| -> ActReport {
        return ActReport::failed(
            format!("{calling}: {said}"),
            error.raw_os_error().unwrap_or(libc::EINVAL),
            String::new(),
        );
    };
    match ready.route {
        Route::InPlaceByTheHandle => {
            let checkout: std::fs::File = match open_directory(copy.checkout) {
                Ok(folder) => folder,
                Err(error) => return step("open", error, "the checkout's own directory"),
            };
            let covered: std::fs::File = match open_from_the_handle(&checkout, copy.handle) {
                Ok(covered) => covered,
                Err(refusal) => {
                    return ActReport::failed(
                        format!("open_by_handle_at: {}", refusal.said),
                        refusal.errno,
                        String::new(),
                    );
                }
            };
            if let Err(error) = unmount_the_descriptor(&covered) {
                return step("umount2", error, "the copy at the handle's dentry");
            }
            return match read_the_table() {
                Ok(text) => ActReport::done(format!("{}|unmounted-in-place", ready.mount_id), text),
                Err(error) => step("mountinfo", error, "the table after the unmount"),
            };
        }
        Route::SlaveMoveAndUnmount => {
            let checkout: std::fs::File = match open_directory(copy.checkout) {
                Ok(folder) => folder,
                Err(error) => return step("open", error, "the checkout's own directory"),
            };
            let source: std::fs::File = match open_directory(copy.slot) {
                Ok(folder) => folder,
                Err(error) => return step("open", error, "the copy at the slot"),
            };
            let Some(directory) = ready.directory.as_deref() else {
                return ActReport::refused(
                    String::from("way 7 names no directory to move onto"),
                    String::new(),
                );
            };
            let destination: std::fs::File = match open_directory_below(&checkout, directory) {
                Ok(folder) => folder,
                Err(error) => return step("openat", error, "way 7's directory"),
            };
            if let Err(error) = move_the_copy(&source, &destination) {
                return step("move_mount", error, "the copy onto way 7's directory");
            }
            // The descriptor the move took the copy from is one reference to the moved mount, and a plain unmount
            // refuses `EBUSY` while any task's reference remains, so it is closed before the unmount, as the design's
            // own step says.
            drop(source);
            if let Err(error) = unmount_the_descriptor(&destination) {
                return step("umount2", error, "the copy at way 7's directory");
            }
            if let Err(error) = remove_directory_at(&checkout, directory) {
                return step("unlinkat", error, "way 7's directory");
            }
            return match read_the_table() {
                Ok(text) => {
                    ActReport::done(format!("{}|moved-and-unmounted", ready.mount_id), text)
                }
                Err(error) => step("mountinfo", error, "the table after the unmount"),
            };
        }
    }
}

/// What:
///  Move one mount onto a directory of this namespace.
/// Why:
///  The move names both ends by descriptor,
///  and a non-shared destination takes it without propagating the move.
fn move_the_copy(
    source: &std::fs::File,
    destination: &std::fs::File,
) -> Result<(), std::io::Error> {
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: both descriptors are this child's own, the empty paths outlive the call, and the flag word names both
    // ends as descriptors.
    let moved: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            source.as_raw_fd(),
            empty.as_ptr(),
            destination.as_raw_fd(),
            empty.as_ptr(),
            libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH,
        ) as libc::c_int
    };
    if moved != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Unmount the topmost mount at a descriptor's dentry,
///  never lazily.
/// Why:
///  A lazy detach leaves the copy alive while a task holds it,
///  so it would not release the dataset,
///  and `umount2` on a descriptor's `/proc` path is the one call the design
///  allows inside another namespace.
fn unmount_the_descriptor(mount: &std::fs::File) -> Result<(), std::io::Error> {
    let named: CString = CString::new(format!("/proc/self/fd/{}", mount.as_raw_fd()))
        .expect("no NUL in a descriptor path");
    // SAFETY: the path outlives the call and no flag asks for force or lazy detach.
    let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    if taken != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}
