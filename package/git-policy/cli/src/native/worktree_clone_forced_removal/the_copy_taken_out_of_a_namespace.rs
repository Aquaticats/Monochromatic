//! What:
//!  The act on one holder namespace's copy:
//!  the structural stage that proves the copy and makes way 7's directory,
//!  and the unmount stage that takes the copy out by the handle or by the move.
//! Why:
//!  This is the act the design's two ways are made of
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Way 8,
//!  in place by the handle",
//!  and "Way 7,
//!  the parent a slave"),
//!  and every call in it is made inside the holder's namespace,
//!  by a single-threaded child,
//!  because that is the only place the kernel lets a mount be changed there.
//!  Each stage re-reads the kernel's own table before and after what it
//!  changes,
//!  so an act that fails names the call and the mount it was about rather than
//!  reporting success on a mount it never saw.
//!  The two stages are apart because the order is the design's:
//!  the structural stage runs over every namespace before any signal,
//!  and the move sits after the signals so a blocker's write during the wait
//!  lands in the copy and not in the checkout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ready = makeTheWayReady(route, copy);   // after the preflight, before any signal
//! const done = takeTheCopyOut(ready, copy);     // after the signals
//! ```

use crate::fs_mount_options::{MountFacts, parse_mount_table};
use crate::worktree_clone_forced_removal::descriptor_calls::{
    make_directory_at, open_directory, open_directory_below, remove_directory_at,
};
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::mount_namespace_child::{ActReport, run_in};
use crate::worktree_clone_forced_removal::receiver_closure::MountRow;
use crate::worktree_clone_forced_removal::recorded_handle::{
    RecordedHandle, mount_id_of, open_from_the_handle,
};
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::path::Path;

/// The mode way 7's directory carries,
/// which the act asks for when it makes it.
const DESTINATION_DIRECTORY_MODE: u32 = DESTINATION_MODE;

/// The mode way 7's directory carries:
/// its owner may enter it and nobody else may read or write it.
pub const DESTINATION_MODE: u32 = 0o700;

/// Where the kernel lists the mounts of the namespace a child entered.
const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";

/// What:
///  One holder namespace's copy and everything the act names it by.
/// Why:
///  The act reads the checkout's own mount,
///  the slot's own dentry and the add's recorded handle inside the holder's
///  namespace,
///  so all four travel together and no caller can hand one namespace's path to
///  another's descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceCopy = { namespace; identity; checkout; slot; dataset; handle; directory };
/// ```
pub struct NamespaceCopy<'a> {
    /// The mount namespace's descriptor,
    /// which the child enters.
    pub namespace: &'a std::fs::File,
    /// The namespace's kernel identity,
    /// for the journal and the report.
    pub identity: &'a str,
    /// The checkout's own path as that namespace spells it.
    pub checkout: &'a Path,
    /// The slot's path as that namespace spells it.
    pub slot: &'a Path,
    /// The clone's dataset name,
    /// which a mount of the clone matches exactly.
    pub dataset: &'a str,
    /// The handle the add recorded of the slot's underlay.
    pub handle: &'a RecordedHandle,
    /// The reserved name way 7's directory carries in the checkout.
    pub directory: &'a str,
}

/// What:
///  What the structural stage left,
///  in the plain values the unmount stage needs.
/// Why:
///  Each stage runs in its own child,
///  so nothing the first opened can be handed to the second,
///  and the second proves its own preconditions again rather than trusting
///  the first's descriptors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ready = { route; mountId; parentId; directory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ready {
    /// The way the act takes here.
    pub route: Route,
    /// The copy's mount id.
    pub mount_id: u64,
    /// The mount the copy sits on,
    /// which the act requires to be the checkout's own.
    pub parent_id: u64,
    /// Way 7's directory name,
    /// which the unmount stage removes once the copy is gone.
    pub directory: Option<String>,
}

/// What:
///  What the unmount stage took out.
/// Why:
///  The journal names the mount id and,
///  for way 7,
///  the directory whose life the report records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WayOut = { mountId: number; directory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WayOut {
    /// The copy's mount id.
    pub mount_id: u64,
    /// Way 7's directory,
    /// already removed when the act finished.
    pub directory: Option<String>,
}

/// What:
///  Why the act stopped,
///  in the design's own refusal word and the call it named.
/// Why:
///  A busy unmount after an empty search is a different answer from a mount
///  under the copy,
///  and the word is what the receipt's failed step carries,
///  so the act commits to it here rather than leaving a caller to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRefusal = { word; calling: string; errno?: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyRefusal {
    /// The word the report and the receipt carry.
    pub word: RefusalWord,
    /// Which call refused, or which read.
    pub calling: &'static str,
    /// The errno it answered, where it answered one.
    pub errno: Option<i32>,
    /// What it said, in plain words.
    pub said: String,
}

impl std::fmt::Display for CopyRefusal {
    /// One sentence naming the word, the call, the errno and what it said.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self.errno {
            Some(errno) => {
                write!(formatter, "{}: {} answered errno {errno}: {}", self.word.word(), self.calling, self.said)
            }
            None => write!(formatter, "{}: {}: {}", self.word.word(), self.calling, self.said),
        };
    }
}

/// What:
///  One row of the kernel's table as the closure's own shape,
///  so the receivers are computed from what the act read.
/// Why:
///  The preflight and the act read one table and must judge it one way,
///  and the closure already takes rows with the tags the kernel prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = rowsOf(mountinfoText);
/// ```
pub fn rows_of(table: &str) -> Vec<MountRow> {
    let parsed: Vec<MountFacts> = parse_mount_table(table.as_bytes());
    let mut rows: Vec<MountRow> = Vec::with_capacity(parsed.len());
    for facts in parsed {
        let mut groups: Vec<u32> = Vec::new();
        let mut master: Option<u32> = None;
        for field in &facts.optional_fields {
            if let Some(number) = field.strip_prefix("shared:").and_then(|text| return text.parse::<u32>().ok()) {
                groups.push(number);
            }
            if let Some(number) = field.strip_prefix("master:").and_then(|text| return text.parse::<u32>().ok()) {
                master = Some(number);
            }
        }
        rows.push(MountRow {
            mount_id: u32::try_from(facts.mount_id).unwrap_or(u32::MAX),
            parent_id: u32::try_from(facts.parent_id).unwrap_or(u32::MAX),
            mount_point: facts.mount_point.to_string_lossy().into_owned(),
            root: facts.root.to_string_lossy().into_owned(),
            filesystem: facts.filesystem_type,
            source: facts.source,
            shared: groups,
            master,
        });
    }
    return rows;
}

/// What:
///  The row the kernel lists for one mount id.
/// Why:
///  Every stage's read-after compares the kernel's own number,
///  and a row found by number is the mount the act changed.
pub fn row_of(rows: &[MountRow], mount_id: u64) -> Option<MountRow> {
    return rows.iter().find(|row| return u64::from(row.mount_id) == mount_id).cloned();
}

/// What:
///  The topmost mount at one path,
///  which is the last row the kernel prints for it.
/// Why:
///  A mount stacked over another is printed after it,
///  and the unmount by the handle acts on the topmost one,
///  so the act reads the same row the kernel will.
pub fn topmost_at(rows: &[MountRow], at: &Path) -> Option<MountRow> {
    let wanted: String = at.to_string_lossy().into_owned();
    let mut found: Option<MountRow> = None;
    for row in rows {
        if row.mount_point == wanted {
            found = Some(row.clone());
        }
    }
    return found;
}

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
pub fn make_the_way_ready(route: Route, copy: &NamespaceCopy<'_>) -> Result<Ready, CopyRefusal> {
    let wanted: Route = route;
    let report: ActReport = run_in(copy.namespace, || return ready_in_here(wanted, copy))
        .map_err(|refusal| return CopyRefusal { word: RefusalWord::HolderUnendable, calling: refusal.calling, errno: refusal.errno, said: refusal.said })?;
    let mount_id: u64 = match report.outcome {
        crate::worktree_clone_forced_removal::mount_namespace_child::ActOutcome::Done => {
            report.detail.split('|').next().and_then(|text| return text.parse::<u64>().ok()).ok_or_else(|| {
                return CopyRefusal { word: RefusalWord::HolderUnendable, calling: "report", errno: None, said: String::from("the child's report names no copy mount") };
            })?
        }
        _ => return Err(refused_from(&report)),
    };
    let rows: Vec<MountRow> = rows_of(report.table.as_str());
    let Some(copy_row) = row_of(&rows, mount_id) else {
        return Err(CopyRefusal { word: RefusalWord::HolderUnendable, calling: "mountinfo", errno: None, said: format!("the copy's mount {mount_id} is not in the table the child read after the stage") });
    };
    let directory: Option<String> = match route {
        Route::InPlaceByTheHandle => None,
        Route::SlaveMoveAndUnmount => Some(String::from(copy.directory)),
    };
    return Ok(Ready { route, mount_id, parent_id: u64::from(copy_row.parent_id), directory });
}

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
pub fn take_the_copy_out(ready: &Ready, copy: &NamespaceCopy<'_>) -> Result<WayOut, CopyRefusal> {
    let planned: Ready = ready.clone();
    let report: ActReport = run_in(copy.namespace, || return unmount_in_here(&planned, copy))
        .map_err(|refusal| return CopyRefusal { word: RefusalWord::HolderUnendable, calling: refusal.calling, errno: refusal.errno, said: refusal.said })?;
    if report.outcome != crate::worktree_clone_forced_removal::mount_namespace_child::ActOutcome::Done {
        return Err(refused_from(&report));
    }
    let rows: Vec<MountRow> = rows_of(report.table.as_str());
    if row_of(&rows, ready.mount_id).is_some() {
        return Err(CopyRefusal { word: RefusalWord::HolderUnendable, calling: "mountinfo", errno: None, said: format!("the copy's mount {} is still in the table the child read after the unmount", ready.mount_id) });
    }
    return Ok(WayOut { mount_id: ready.mount_id, directory: ready.directory.clone() });
}

/// What:
///  The word and the sentence one failed child report becomes.
/// Why:
///  A locked copy answers `EINVAL` to both ways,
///  a busy unmount after the search found no blocker is the unseen case,
///  and a plain refusal is the act's own;
///  the word is what the report prints.
fn refused_from(report: &ActReport) -> CopyRefusal {
    let errno: Option<i32> = report.errno;
    let word: RefusalWord = match errno {
        Some(libc::EINVAL) => RefusalWord::HolderLocked,
        Some(libc::EBUSY) => RefusalWord::HolderUnseen,
        Some(libc::EPERM) => RefusalWord::HolderUnendable,
        _ => RefusalWord::HolderStructural,
    };
    return CopyRefusal { word, calling: "the child's act", errno, said: report.detail.clone() };
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
fn ready_in_here(route: Route, copy: &NamespaceCopy<'_>) -> ActReport {
    let step = |calling: &'static str, error: std::io::Error, said: &str| -> ActReport {
        return ActReport::failed(format!("{calling}: {said}"), error.raw_os_error().unwrap_or(libc::EINVAL), String::new());
    };
    let checkout: std::fs::File = match open_directory(copy.checkout) {
        Ok(folder) => folder,
        Err(error) => return step("open", error, "the checkout's own directory"),
    };
    let checkout_id: u64 = match mount_id_of(&checkout) {
        Ok(id) => id,
        Err(refusal) => return ActReport::failed(format!("fdinfo: {}", refusal.said), refusal.errno, String::new()),
    };
    let table: String = match read_the_table() {
        Ok(text) => text,
        Err(error) => return step("mountinfo", error, "the holder namespace's own table"),
    };
    let rows: Vec<MountRow> = rows_of(table.as_str());
    let Some(topmost) = topmost_at(&rows, copy.slot) else {
        return ActReport::refused(format!("no mount sits at {} in this namespace", copy.slot.display()), table);
    };
    if topmost.filesystem != "zfs" || topmost.source != copy.dataset {
        return ActReport::refused(
            format!("the topmost mount at {} is {} of `{}` and not a mount of `{}`", copy.slot.display(), topmost.mount_id, topmost.source, copy.dataset),
            table,
        );
    }
    if u64::from(topmost.parent_id) != checkout_id {
        return ActReport::refused(
            format!("the copy at {} sits on mount {} and the checkout's own mount is {checkout_id}", copy.slot.display(), topmost.parent_id),
            table,
        );
    }
    match route {
        Route::InPlaceByTheHandle => match open_from_the_handle(&checkout, copy.handle) {
            Ok(covered) => match mount_id_of(&covered) {
                Ok(id) if id == checkout_id => {}
                Ok(id) => {
                    return ActReport::refused(format!("the handle reopened a descriptor on mount {id} and the checkout's own mount is {checkout_id}"), table);
                }
                Err(refusal) => return ActReport::failed(format!("fdinfo: {}", refusal.said), refusal.errno, String::new()),
            },
            Err(refusal) => {
                let errno: i32 = refusal.errno;
                let said: String = format!("open_by_handle_at: {}", refusal.said);
                return if errno == libc::EINVAL { ActReport::refused(said, table) } else { ActReport::failed(said, errno, String::new()) };
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
                    return ActReport::refused(format!("the checkout's mount {checkout_id} still shares with group {:?}", row.shared), after);
                }
                None => return ActReport::refused(format!("the checkout's mount {checkout_id} is not in the table after the slave change"), after),
            }
            if let Err(error) = make_directory_at(&checkout, copy.directory, DESTINATION_DIRECTORY_MODE) {
                return step("mkdirat", error, "way 7's directory in the checkout");
            }
            return ActReport::done(format!("{}|slave-and-directory", topmost.mount_id), after);
        }
    }
    return ActReport::done(format!("{}|handle-proved", topmost.mount_id), table);
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
fn unmount_in_here(ready: &Ready, copy: &NamespaceCopy<'_>) -> ActReport {
    let step = |calling: &'static str, error: std::io::Error, said: &str| -> ActReport {
        return ActReport::failed(format!("{calling}: {said}"), error.raw_os_error().unwrap_or(libc::EINVAL), String::new());
    };
    match ready.route {
        Route::InPlaceByTheHandle => {
            let checkout: std::fs::File = match open_directory(copy.checkout) {
                Ok(folder) => folder,
                Err(error) => return step("open", error, "the checkout's own directory"),
            };
            let covered: std::fs::File = match open_from_the_handle(&checkout, copy.handle) {
                Ok(covered) => covered,
                Err(refusal) => return ActReport::failed(format!("open_by_handle_at: {}", refusal.said), refusal.errno, String::new()),
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
                return ActReport::refused(String::from("way 7 names no directory to move onto"), String::new());
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
                Ok(text) => ActReport::done(format!("{}|moved-and-unmounted", ready.mount_id), text),
                Err(error) => step("mountinfo", error, "the table after the unmount"),
            };
        }
    }
}

/// What:
///  Read `/proc/self/mountinfo` inside the namespace the child is in.
/// Why:
///  The table is the kernel's own and this child's, not the handler's,
///  which is what makes the read-after a fact about the namespace it changed.
fn read_the_table() -> Result<String, std::io::Error> {
    return std::fs::read_to_string(MOUNTINFO_PATH);
}

/// What:
///  Make one mount stop propagating,
///  naming it by its descriptor's `/proc` path.
/// Why:
///  `MS_SLAVE` on a descriptor's path acts on the mount the descriptor names
///  because a propagation change requires the path to be a mount root,
///  and way 7 needs the parent to stop propagating before the move.
fn make_the_parent_a_slave(mount: &std::fs::File) -> Result<(), std::io::Error> {
    let named: CString = CString::new(format!("/proc/self/fd/{}", mount.as_raw_fd())).expect("no NUL in a descriptor path");
    // SAFETY: no source, file system or data is named, the path outlives the call, and the flag asks only for the
    // slave change.
    let changed: libc::c_int = unsafe { libc::mount(std::ptr::null(), named.as_ptr(), std::ptr::null(), libc::MS_SLAVE, std::ptr::null()) };
    if changed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Move one mount onto a directory of this namespace.
/// Why:
///  The move names both ends by descriptor,
///  and a non-shared destination takes it without propagating the move.
fn move_the_copy(source: &std::fs::File, destination: &std::fs::File) -> Result<(), std::io::Error> {
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: both descriptors are this child's own, the empty paths outlive the call, and the flag word names both
    // ends as descriptors.
    let moved: libc::c_int = unsafe {
        libc::syscall(libc::SYS_move_mount, source.as_raw_fd(), empty.as_ptr(), destination.as_raw_fd(), empty.as_ptr(), libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH)
            as libc::c_int
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
    let named: CString = CString::new(format!("/proc/self/fd/{}", mount.as_raw_fd())).expect("no NUL in a descriptor path");
    // SAFETY: the path outlives the call and no flag asks for force or lazy detach.
    let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    if taken != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_copy_taken_out_of_a_namespace_tests.rs"]
mod tests;
