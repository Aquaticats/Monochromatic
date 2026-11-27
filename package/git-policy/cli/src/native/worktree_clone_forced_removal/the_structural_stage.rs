//! What:
//!  The act's structural stage, in both halves:
//!  the half the caller runs, which enters the holder's namespace through one
//!  single-threaded child and reads the child's bounded report,
//!  and the half the child runs there,
//!  which proves the copy and the checkout's own mount and,
//!  for way 7,
//!  stops the parent propagating and makes the directory.
//! Why:
//!  No kernel change may be made before the act knows which mount it is about,
//!  and way 7's slave change and directory must both happen before any signal,
//!  so a refusal here ends nothing
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Way 7,
//!  the parent a slave").
//!  The two halves are one file because the report the child hands back is the
//!  caller's own precondition read after the stage.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ready = makeTheWayReady(route, copy);   // after the preflight, before any signal
//! ```

use super::the_copy_taken_out_of_a_namespace::{
    CopyRefusal,
    DESTINATION_DIRECTORY_MODE,
    NamespaceCopy,
    Ready,
    refused_from,
};
use super::the_mount_table_the_act_reads::{
    read_the_table,
    row_of,
    rows_of,
    topmost_at,
};
use crate::worktree_clone_forced_removal::descriptor_calls::{
    make_directory_at,
    open_directory,
};
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::mount_namespace_child::{
    ActReport,
    run_in,
};
use crate::worktree_clone_forced_removal::receiver_closure::MountRow;
use crate::worktree_clone_forced_removal::recorded_handle::{
    mount_id_of,
    open_from_the_handle,
};
use std::ffi::CString;
use std::os::fd::AsRawFd;

/// What:
///  The structural stage:
///  prove the copy and the checkout's own mount,
///  and for way 7 stop the parent propagating and make the directory.
/// Why:
///  A kernel change must not be made before the act knows which mount it is
///  about,
///  and way 7's directory and slave change must both happen before any signal
///  so a refusal here ends nothing.
///
/// # Errors
/// The namespace could not be entered or read,
/// the topmost mount at the slot is not the clone's,
/// the copy's parent is not the checkout's own mount,
/// the slave change or the directory was refused.
pub fn make_the_way_ready(
    route: Route,
    copy: &NamespaceCopy<'_>,
) -> Result<Ready, CopyRefusal> {
    let wanted: Route = route;
    let report: ActReport =
        run_in(copy.namespace, || return ready_in_here(wanted, copy)).map_err(|refusal| {
            return CopyRefusal {
                word: RefusalWord::HolderUnendable,
                calling: refusal.calling,
                errno: refusal.errno,
                said: refusal.said,
            };
        })?;
    let mount_id: u64 = match report.outcome {
        crate::worktree_clone_forced_removal::mount_namespace_child::ActOutcome::Done => report
            .detail
            .split('|')
            .next()
            .and_then(|text| return text.parse::<u64>().ok())
            .ok_or_else(|| {
                return CopyRefusal {
                    word: RefusalWord::HolderUnendable,
                    calling: "report",
                    errno: None,
                    said: String::from("the child's report names no copy mount"),
                };
            })?,
        _ => return Err(refused_from(&report)),
    };
    let rows: Vec<MountRow> = rows_of(report.table.as_str());
    let Some(copy_row) = row_of(&rows, mount_id) else {
        return Err(CopyRefusal {
            word: RefusalWord::HolderUnendable,
            calling: "mountinfo",
            errno: None,
            said: format!(
                "the copy's mount {mount_id} is not in the table the child read after the stage"
            ),
        });
    };
    let directory: Option<String> = match route {
        Route::InPlaceByTheHandle => None,
        Route::SlaveMoveAndUnmount => Some(String::from(copy.directory)),
    };
    return Ok(Ready {
        route,
        mount_id,
        parent_id: u64::from(copy_row.parent_id),
        directory,
    });
}

/// What:
///  The structural stage as the child runs it,
///  inside the holder's namespace.
/// Why:
///  Every descriptor and path here must be the ones the holder's namespace
///  has,
///  and the report's first field is the copy's mount id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkout = open(copy.checkout); const rows = table();
/// const row = topmostAt(rows, copy.slot); require(row.source === dataset && row.fs === 'zfs');
/// require(mountIdOf(checkout) === row.parentId);
/// if (way === 'in-place-by-the-handle') { const covered = openByHandle(checkout, handle); require(mountIdOf(covered) === mountIdOf(checkout)); }
/// else { mount(null, fdPath(checkout), null, MS_SLAVE, null); require(!rowOf(table(), checkoutId).shared.length); mkdirat(checkout, directory, 0700); }
/// return done(`${row.mountId}|…`);
/// ```
fn ready_in_here(
    route: Route,
    copy: &NamespaceCopy<'_>,
) -> ActReport {
    let step = |calling: &'static str, error: std::io::Error, said: &str| -> ActReport {
        return ActReport::failed(
            format!("{calling}: {said}"),
            error.raw_os_error().unwrap_or(libc::EINVAL),
            String::new(),
        );
    };
    let checkout: std::fs::File = match open_directory(copy.checkout) {
        Ok(folder) => folder,
        Err(error) => return step("open", error, "the checkout's own directory"),
    };
    let checkout_id: u64 = match mount_id_of(&checkout) {
        Ok(id) => id,
        Err(refusal) => {
            return ActReport::failed(
                format!("fdinfo: {}", refusal.said),
                refusal.errno,
                String::new(),
            );
        }
    };
    let table: String = match read_the_table() {
        Ok(text) => text,
        Err(error) => return step("mountinfo", error, "the holder namespace's own table"),
    };
    let rows: Vec<MountRow> = rows_of(table.as_str());
    let Some(topmost) = topmost_at(&rows, copy.slot) else {
        return ActReport::refused(
            format!("no mount sits at {} in this namespace", copy.slot.display()),
            table,
        );
    };
    if topmost.filesystem != "zfs" || topmost.source != copy.dataset {
        return ActReport::refused(
            format!(
                "the topmost mount at {} is {} of `{}` and not a mount of `{}`",
                copy.slot.display(),
                topmost.mount_id,
                topmost.source,
                copy.dataset
            ),
            table,
        );
    }
    if u64::from(topmost.parent_id) != checkout_id {
        return ActReport::refused(
            format!(
                "the copy at {} sits on mount {} and the checkout's own mount is {checkout_id}",
                copy.slot.display(),
                topmost.parent_id
            ),
            table,
        );
    }
    match route {
        Route::InPlaceByTheHandle => match open_from_the_handle(&checkout, copy.handle) {
            Ok(covered) => match mount_id_of(&covered) {
                Ok(id) if id == checkout_id => {}
                Ok(id) => {
                    return ActReport::refused(
                        format!(
                            "the handle reopened a descriptor on mount {id} and the checkout's own mount is {checkout_id}"
                        ),
                        table,
                    );
                }
                Err(refusal) => {
                    return ActReport::failed(
                        format!("fdinfo: {}", refusal.said),
                        refusal.errno,
                        String::new(),
                    );
                }
            },
            Err(refusal) => {
                let errno: i32 = refusal.errno;
                let said: String = format!("open_by_handle_at: {}", refusal.said);
                return if errno == libc::EINVAL {
                    ActReport::refused(said, table)
                } else {
                    ActReport::failed(said, errno, String::new())
                };
            }
        },
        Route::SlaveMoveAndUnmount => {
            if let Err(error) = make_the_parent_a_slave(&checkout) {
                return step("mount MS_SLAVE", error, "the checkout's own mount");
            }
            let after: String = match read_the_table() {
                Ok(text) => text,
                Err(error) => return step("mountinfo", error, "the table after the slave change"),
            };
            let after_rows: Vec<MountRow> = rows_of(after.as_str());
            match row_of(&after_rows, checkout_id) {
                Some(row) if row.shared.is_empty() => {}
                Some(row) => {
                    return ActReport::refused(
                        format!(
                            "the checkout's mount {checkout_id} still shares with group {:?}",
                            row.shared
                        ),
                        after,
                    );
                }
                None => {
                    return ActReport::refused(
                        format!(
                            "the checkout's mount {checkout_id} is not in the table after the slave change"
                        ),
                        after,
                    );
                }
            }
            if let Err(error) =
                make_directory_at(&checkout, copy.directory, DESTINATION_DIRECTORY_MODE)
            {
                return step("mkdirat", error, "way 7's directory in the checkout");
            }
            return ActReport::done(format!("{}|slave-and-directory", topmost.mount_id), after);
        }
    }
    return ActReport::done(format!("{}|handle-proved", topmost.mount_id), table);
}

/// What:
///  Make one mount stop propagating,
///  naming it by its descriptor's `/proc` path.
/// Why:
///  `MS_SLAVE` on a descriptor's path acts on the mount the descriptor names
///  because a propagation change requires the path to be a mount root,
///  and way 7 needs the parent to stop propagating before the move.
fn make_the_parent_a_slave(mount: &std::fs::File) -> Result<(), std::io::Error> {
    let named: CString = CString::new(format!("/proc/self/fd/{}", mount.as_raw_fd()))
        .expect("no NUL in a descriptor path");
    // SAFETY: no source, file system or data is named, the path outlives the call, and the flag asks only for the
    // slave change.
    let changed: libc::c_int = unsafe {
        libc::mount(
            std::ptr::null(),
            named.as_ptr(),
            std::ptr::null(),
            libc::MS_SLAVE,
            std::ptr::null(),
        )
    };
    if changed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}
