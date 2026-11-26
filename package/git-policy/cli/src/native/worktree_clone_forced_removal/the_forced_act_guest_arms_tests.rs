//! What:
//!  The forced act's guest arm:
//!  every kernel call the act makes, run by the route's own code in one
//!  throwaway guest,
//!  phase by phase,
//!  for every kind of holder the earlier arm measured.
//! Why:
//!  The design's measurements owed are the build's evidence
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Measurements owed",
//!  and the plan's step 17),
//!  and the calls are mounts,
//!  unmounts,
//!  a move and two handle calls,
//!  which no host test may make.
//!  Each phase runs the route's own code against a file-backed pool:
//!  `setns` into the holder's namespace,
//!  `name_to_handle_at` and `open_by_handle_at` on the add's handle,
//!  `mount(MS_SLAVE)` on the parent,
//!  `move_mount` onto the reserved directory,
//!  `umount2` at the copy and at the destination,
//!  the directory's removal,
//!  `NS_GET_USERNS` on the holder's namespace,
//!  and the pinned `SIGTERM`, wait and `SIGKILL` of the signal stage.
//!  The wait and the heartbeat are a harness's provisional values,
//!  recorded with the run and shipped by nothing: the release's constants have
//!  no value until step 22 measures them.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the forced act guest arms', () => { for (const phase of PHASES) expect(phase()).toMatch(/ok/); });
//! ```

use super::the_forced_act_guest_fixture::{
    SLOT, end_now, holds_the_copy_at, in_a_private_namespace, make_the_checkout_a_slave, make_the_checkout_shared, phase_line, the_table,
    the_table_of_the_process, the_versions, Fixture, a_row,
};
use crate::worktree_clone_forced_removal::blocker_end::{Blocker, EndResult, end_the_blockers};
use crate::worktree_clone_forced_removal::descriptor_calls::open_directory;
use crate::worktree_clone_forced_removal::forced_arm_numbers::TimeoutWindow;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::namespace_owner::{NamespaceOwner, read_the_owner};
use crate::worktree_clone_forced_removal::receiver_closure::{MountRow, NamespaceTable, ReceiverSet, plan_the_receivers};
use crate::worktree_clone_forced_removal::recorded_handle::{RecordedHandle, name_the_handle};
use crate::worktree_clone_forced_removal::the_copy_taken_out_of_a_namespace::{NamespaceCopy, Ready, make_the_way_ready, take_the_copy_out, topmost_at};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The wait between `SIGTERM` and `SIGKILL` a run of this arm uses,
/// which is the harness's provisional value and not a shipped one.
const A_HARNESSES_TERMINATION_WAIT: Duration = Duration::from_secs(10);
/// The heartbeat one run of this arm uses,
/// which is the harness's provisional value and not a shipped one.
const A_HARNESSES_HEARTBEAT: Duration = Duration::from_secs(1);
/// The reserved name way 7's directory carries here,
/// which the release derives from the clone's GUID.
const THE_RESERVED_NAME: &str = ".git-worktree-clone-removal-0";

/// What:
///  One holder the phase started,
///  which the phase ends on every path.
/// Why:
///  A holder that outlived its phase would hold a copy in a namespace the next
///  phase cannot see,
///  and the release's own rule is that a child is ended by what started it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AHolder = { pid: number; kind: string };
/// ```
struct AHolder {
    pid: libc::pid_t,
}

impl AHolder {
    /// Whether the kernel still answers for the holder's number.
    fn alive(&self) -> bool {
        // SAFETY: signal 0 asks for the process's existence and reaches nothing.
        return unsafe { libc::kill(self.pid, 0) } == 0;
    }

    /// End the holder and reap it,
    /// which every phase does whatever it proved.
    fn end(self) {
        // SAFETY: the pid is this phase's own child, and signal 9 ends it.
        let _sent: libc::c_int = unsafe { libc::kill(self.pid, libc::SIGKILL) };
        let mut status: libc::c_int = 0;
        // SAFETY: the status word is this call's own and the child is this phase's.
        let _waited: libc::pid_t = unsafe { libc::waitpid(self.pid, &mut status, 0) };
    }
}

/// What:
///  The shape of one holder's own namespaces.
/// Why:
///  The design's kinds are told apart by where the copy lives:
///  a working directory in a session of its own,
///  a copy in a mount namespace of its own,
///  and a copy in a namespace the kernel locked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AShape = 'own-session' | 'own-namespace' | 'own-namespace-and-shared' | 'own-user-namespace';
/// ```
#[derive(Clone, Copy)]
enum AShape {
    /// A task in a session of its own,
    /// with its working directory inside the clone.
    SessionOfItsOwn,
    /// A mount namespace of its own,
    /// copied after the clone was mounted.
    NamespaceOfItsOwn,
    /// A mount namespace of its own whose checkout is shared with a group of
    /// its own,
    /// which is the shape systemd leaves a unit's parent.
    NamespaceOfItsOwnAndShared,
    /// A mount namespace of its own whose checkout is a slave of the host's
    /// group,
    /// which is the shape a unit started while the checkout was shared keeps.
    NamespaceOfItsOwnAsSlave,
    /// A user namespace and a mount namespace of its own,
    /// the shape `unshare --user --map-root-user --mount` makes.
    UserNamespaceOfItsOwn,
}

/// What:
///  Start one holder in a session of its own,
///  with its working directory inside the clone where the shape says so.
/// Why:
///  A holder in the caller's own session is the protected set the act never
///  ends,
///  so every holder here is outside it by its own session,
///  which is the shape the earlier arm measured.
fn hold(shape: AShape, checkout: &Path, slot: &Path, ignores_the_ask: bool) -> AHolder {
    // SAFETY: `fork` takes no argument, and the child below returns through `_exit` only.
    let child: libc::pid_t = unsafe { libc::fork() };
    assert!(child >= 0, "the holder's fork: {}", std::io::Error::last_os_error());
    if child > 0 {
        return AHolder { pid: child };
    }
    // SAFETY: every one of these takes numbers and reads numbers; none touches memory of ours.
    let (child_pid, parent, group, session_before): (libc::pid_t, libc::pid_t, libc::pid_t, libc::pid_t) =
        unsafe { (libc::getpid(), libc::getppid(), libc::getpgid(0), libc::getsid(0)) };
    eprintln!("HOLDER-ENTER pid={child_pid} ppid={parent} pgid={group} sid={session_before}");
    // SAFETY: `prctl` reads its four words and touches no memory of ours.
    let _death: libc::c_int = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
    // SAFETY: `getppid` takes no argument and reads one number.
    if unsafe { libc::getppid() } == 1 {
        end_now(1);
    }
    if ignores_the_ask {
        // SAFETY: the disposition of this child's own process is its to set and nothing else in the phase is touched.
        let _ignored: libc::sighandler_t = unsafe { libc::signal(libc::SIGTERM, libc::SIG_IGN) };
    }
    // SAFETY: `setsid` takes no argument and makes this child a session leader of its own.
    let session: libc::pid_t = unsafe { libc::setsid() };
    // SAFETY: `getpid` takes no argument and reads one number; in this child it is the pid the parent knows.
    let mine: libc::pid_t = unsafe { libc::getpid() };
    // SAFETY: `getsid` reads one pid and writes nothing of ours.
    let now: libc::pid_t = unsafe { libc::getsid(0) };
    eprintln!("HOLDER-AFTER setsid={session} sid={now} pid={mine} errno={}", std::io::Error::last_os_error());
    assert_eq!(session, mine, "the holder's own session: {}", std::io::Error::last_os_error());
    match shape {
        AShape::SessionOfItsOwn => {}
        AShape::NamespaceOfItsOwn | AShape::NamespaceOfItsOwnAndShared | AShape::NamespaceOfItsOwnAsSlave => {
            // SAFETY: `unshare` reads one flag word and touches no memory of ours.
            let made: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWNS) };
            assert_eq!(made, 0, "the holder's own mount namespace: {}", std::io::Error::last_os_error());
        }
        AShape::UserNamespaceOfItsOwn => {
            // The user and mount namespaces are made in one call, so the mount namespace is owned by the new user
            // namespace and the kernel locks every mount it copied: the shape bubblewrap and a sandboxing unit make.
            // SAFETY: `unshare` reads one flag word and touches no memory of ours.
            let made: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNS) };
            assert_eq!(made, 0, "the holder's own user and mount namespace: {}", std::io::Error::last_os_error());
            // This process is root in the parent user namespace, which is what lets it write its own map after the
            // call; an unprivileged caller needs a helper for it.
            for (path, mapping) in [("/proc/self/setgroups", "deny"), ("/proc/self/uid_map", "0 0 1"), ("/proc/self/gid_map", "0 0 1")] {
                std::fs::write(path, mapping.as_bytes()).unwrap_or_else(|error| panic!("the holder's own {path}: {error}"));
            }
        }
    }
    match shape {
        AShape::NamespaceOfItsOwnAndShared => make_the_checkout_shared(checkout).expect("the holder makes its own checkout a group of its own"),
        AShape::NamespaceOfItsOwnAsSlave => make_the_checkout_a_slave(checkout).expect("the holder makes its own checkout a slave of the host's group"),
        AShape::SessionOfItsOwn | AShape::NamespaceOfItsOwn | AShape::UserNamespaceOfItsOwn => {}
    }
    // The working directory is this forked child's own and never the crate's test process's, so the call is the
    // kernel's and not the library call the crate forbids anywhere. A namespace holder keeps the copy in its mount
    // namespace only, so only a session holder works inside the clone.
    let working: &Path = match shape {
        AShape::SessionOfItsOwn => slot,
        AShape::NamespaceOfItsOwn | AShape::NamespaceOfItsOwnAndShared | AShape::NamespaceOfItsOwnAsSlave | AShape::UserNamespaceOfItsOwn => Path::new("/"),
    };
    let inside: std::ffi::CString = std::ffi::CString::new(working.as_os_str().as_encoded_bytes()).expect("no NUL in the path");
    // SAFETY: the path is this call's own and outlives it; the child becomes a holder whose working directory is the
    // clone, which is exactly what a path holder holds.
    let moved: libc::c_int = unsafe { libc::chdir(inside.as_ptr()) };
    assert_eq!(moved, 0, "the holder's working directory is inside the clone: {}", std::io::Error::last_os_error());
    loop {
        // SAFETY: `pause` waits for one signal and touches no memory of ours.
        let _paused: libc::c_int = unsafe { libc::pause() };
    }
}

/// Wait for one period without holding a thread's clock or spinning.
fn pause(period: Duration) {
    let (_sender, receiver) = std::sync::mpsc::channel::<()>();
    let _timed_out: Result<(), std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(period);
}

/// The signal stage's own window,
/// with the harness's provisional values.
fn a_harness_window() -> TimeoutWindow {
    return TimeoutWindow::at(A_HARNESSES_TERMINATION_WAIT, A_HARNESSES_HEARTBEAT);
}

/// What:
///  Start one job in the caller's own session and keep it alive to the phase's
///  end,
///  which every phase checks.
/// Why:
///  The design's one promise to the person is that the caller's session's other
///  jobs live through the act,
///  and a phase that did not have one could not show it.
fn a_job_of_the_callers_session() -> AHolder {
    // SAFETY: `fork` takes no argument, and the child below returns through `_exit` only.
    let child: libc::pid_t = unsafe { libc::fork() };
    assert!(child >= 0, "the caller's other job: {}", std::io::Error::last_os_error());
    if child == 0 {
        // SAFETY: `prctl` reads its four words and touches no memory of ours; the job stays in the phase's session.
        let _death: libc::c_int = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
        loop {
            // SAFETY: `pause` waits for one signal and touches no memory of ours.
            let _paused: libc::c_int = unsafe { libc::pause() };
        }
    }
    return AHolder { pid: child };
}

/// Mount the checkout's own dataset at its path in this namespace.
fn mount_the_checkout(fixture: &Fixture) -> PathBuf {
    let checkout: PathBuf = fixture.checkout_path();
    std::fs::create_dir_all(checkout.as_path()).expect("the checkout's own path");
    let (worked, said) = fixture.ask("mount", &["-t", "zfs", fixture.checkout().as_str(), checkout.to_str().expect("the checkout path is text")]);
    assert!(worked, "the checkout is mounted: {said}");
    return checkout;
}

/// Mount the clone at the slot in this namespace,
/// over the underlay directory a removal would delete.
fn mount_the_clone(fixture: &Fixture, slot: &Path) {
    std::fs::create_dir_all(slot).expect("the slot's underlay");
    let (worked, said) = fixture.ask("mount", &["-t", "zfs", fixture.clone().as_str(), slot.to_str().expect("the slot path is text")]);
    assert!(worked, "the clone is mounted at its slot: {said}");
}

/// The mount table's row at the slot,
/// or a panic naming what the table held.
fn the_copy(fixture: &Fixture, rows: &[MountRow], slot: &Path) -> MountRow {
    let Some(row) = topmost_at(rows, slot) else {
        panic!("no mount sits at {}: the rows are {rows:?}", slot.display());
    };
    assert_eq!(row.source, fixture.clone(), "the topmost mount at the slot is the clone and not `{}`", row.source);
    return row;
}

/// What one mount's propagation tags say it is,
/// in the words the phases judge by.
fn propagation_of(one: Option<&MountRow>) -> &'static str {
    let Some(row) = one else {
        return "absent";
    };
    if row.master.is_some() {
        return "slave";
    }
    if !row.shared.is_empty() {
        return "shared";
    }
    return "private";
}

/// Unmount the slot in this namespace,
/// answering the errno the call gave.
fn unmount_the_slot(slot: &Path) -> Result<(), std::io::Error> {
    let named: std::ffi::CString = std::ffi::CString::new(slot.as_os_str().as_encoded_bytes()).expect("no NUL in the slot path");
    // SAFETY: the path outlives the call and no flag asks for force or lazy detach.
    let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    if taken != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// Destroy the clone's head and answer whether it went.
fn destroy_the_clone(fixture: &Fixture) -> bool {
    let (worked, _said) = fixture.ask("zfs", &["destroy", fixture.clone().as_str()]);
    return worked;
}

/// What:
///  One phase of the arm:
///  its line and the checks it made.
/// Why:
///  The recipe greps one line per phase,
///  and a phase that returned without its line would read as a passing arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = runAPhase('path-holder-asked', () => { … return phaseLine(…); });
/// ```

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_forced_act_guest_arm_tests() {
    let Some(fixture) = Fixture::from_environment("the_forced_act_guest_arm_tests") else {
        return;
    };
    println!("FORCEDACT {}", the_versions(&fixture));
    fixture.build_the_pool();

    // Phase 1: a path holder in a session of its own, asked out, and the host's unmount accepted.
    let asked: String = in_a_private_namespace("path-holder-asked", &|| return phase_path_holder(&fixture, false));
    println!("{asked}");
    assert!(asked.contains("end=exited-after-the-ask"), "the phase's line says how the holder ended: {asked}");
    assert!(asked.contains("unmount=accepted"), "the host's unmount is accepted once no task holds the clone: {asked}");
    assert!(asked.contains("destroy=gone"), "the head destroy answers gone: {asked}");
    assert!(asked.contains("caller-job=alive"), "the caller's session's other job lived through the act: {asked}");
    assert!(asked.contains("ms="), "the phase's own timing is in the line: {asked}");

    // Phase 2: the same, for a holder that ignores the ask and is killed after the wait.
    let killed: String = in_a_private_namespace("path-holder-killed", &|| return phase_path_holder(&fixture, true));
    println!("{killed}");
    assert!(killed.contains("end=killed"), "a holder that ignores the ask is killed at the wait's end: {killed}");
    assert!(killed.contains("destroy=gone"), "the head destroy answers gone after the kill: {killed}");
    assert!(killed.contains("caller-job=alive"), "the caller's session's other job lived through the kill: {killed}");

    // Phase 3: way 8, in place by the add's handle, in the shape systemd leaves a unit's parent.
    let way8: String = in_a_private_namespace("namespace-way8-systemd-shape", &|| return phase_way_8(&fixture));
    println!("{way8}");
    assert!(way8.contains("parent=shared"), "the holder's parent shares with a group of its own: {way8}");
    assert!(way8.contains("holders-copy=gone") && way8.contains("hosts-copy=stays"), "way 8 takes the holder's copy and leaves the host's: {way8}");
    assert!(way8.contains("holder=alive"), "the holder's own process is not ended by way 8: {way8}");
    assert!(way8.contains("unmount=accepted") && way8.contains("destroy=gone"), "the host's unmount and the head destroy follow: {way8}");

    // Phase 4: the peer shape, where the closure names the host as a receiver and the act falls to way 7.
    let peer: String = in_a_private_namespace("namespace-way8-peer-sent-to-way7", &|| return phase_peer(&fixture));
    println!("{peer}");
    assert!(peer.contains("way-eight=false") && peer.contains("outside=the-host"), "the closure names the host as an outside receiver: {peer}");
    assert!(peer.contains("route=slave-move-and-unmount"), "so the act takes way 7: {peer}");
    assert!(peer.contains("parent-after=slave"), "the parent is a slave of its former group: {peer}");
    assert!(peer.contains("holders-copy=gone") && peer.contains("hosts-copy=stays"), "way 7 takes the holder's copy and leaves the host's: {peer}");
    assert!(peer.contains("directory=removed"), "way 7's directory is removed: {peer}");

    // Phase 5: way 7 where the holder's parent shares with no group of its own.
    let repaired: String = in_a_private_namespace("namespace-way7-repaired", &|| return phase_way_7(&fixture, false));
    println!("{repaired}");
    assert!(repaired.contains("parent-before=private") && repaired.contains("way-eight=false"), "a private parent seeds no group, so the act takes way 7: {repaired}");
    assert!(repaired.contains("route=slave-move-and-unmount"), "and the route is way 7: {repaired}");
    assert!(repaired.contains("parent-after=private"), "way 7's slave change on a private parent reads as private, which is way 3 there: {repaired}");
    assert!(repaired.contains("holders-copy=gone") && repaired.contains("hosts-copy=stays"), "the copy goes and the host's stays: {repaired}");
    assert!(repaired.contains("directory=removed"), "way 7's directory is removed after the unmount: {repaired}");
    assert!(repaired.contains("destroy=gone"), "the head destroy answers gone: {repaired}");

    // Phase 6: way 7 with the checkout shared, so the parent is a peer of the host's.
    let shared: String = in_a_private_namespace("namespace-way7-shared", &|| return phase_way_7(&fixture, true));
    println!("{shared}");
    assert!(shared.contains("parent-before=slave"), "the holder's parent is a slave of the host's group: {shared}");
    assert!(shared.contains("parent-after=slave"), "it stays a slave of that group after the change: {shared}");
    assert!(shared.contains("hosts-copy=stays") && shared.contains("holders-copy=gone"), "the holder's copy goes and the host's stays: {shared}");
    assert!(shared.contains("directory=removed"), "way 7's directory is removed after the unmount: {shared}");
    assert!(shared.contains("destroy=gone"), "the head destroy answers gone: {shared}");

    // Phase 7: a holder in a user namespace of its own is refused before any signal.
    let locked: String = in_a_private_namespace("locked-holder-refused", &|| return phase_locked(&fixture));
    println!("{locked}");
    assert!(locked.contains("owner=another"), "the read names a user namespace of its own: {locked}");
    assert!(locked.contains("way8=holder-locked") && locked.contains("way7=holder-locked"), "both ways are refused with the design's word: {locked}");
    assert!(locked.contains("holders-copy=stays"), "nothing was unmounted: {locked}");
    assert!(locked.contains("hosts-copy=stays"), "the host's own copy is untouched: {locked}");
    assert!(locked.contains("unmount-after=accepted") && locked.contains("destroy-after=gone"), "once the holder ended, the host's unmount and the head destroy follow: {locked}");
    assert!(locked.contains("caller-job=alive"), "the caller's session's other job lived through it: {locked}");

    println!("{}", phase_line("arm-finished", &[("phases", "7"), ("result", "every-phase-held")]));
    fixture.teardown();
}

/// What:
///  Wait, bounded, until the holder has the namespaces and the session its shape promises.
/// Why:
///  `fork` returns in the parent before the child has run a line,
///  so a phase that read the holder's session at once would read the caller's:
///  the arm's own log shows the parent printing the caller's session before the
///  child had moved,
///  and the child then killed by its own recheck because the parent had gone.
///  A holder that never gets there is a failure with what it waited for.
fn wait_for_the_holder(shape: AShape, holder: &AHolder) -> bool {
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        // SAFETY: `getsid` reads one pid and writes nothing of ours.
        let session: libc::pid_t = unsafe { libc::getsid(holder.pid) };
        if session == holder.pid {
            let namespace_of = |path: String| -> u64 {
                use std::os::unix::fs::MetadataExt;
                return std::fs::metadata(path.as_str()).expect("the holder's own namespace link").ino();
            };
            let ours: u64 = namespace_of(String::from("/proc/self/ns/mnt"));
            let theirs: u64 = namespace_of(format!("/proc/{}/ns/mnt", holder.pid));
            let namespace_ready: bool = match shape {
                AShape::SessionOfItsOwn => true,
                AShape::NamespaceOfItsOwn | AShape::NamespaceOfItsOwnAndShared | AShape::NamespaceOfItsOwnAsSlave => theirs != ours,
                AShape::UserNamespaceOfItsOwn => theirs != ours && namespace_of(format!("/proc/{}/ns/user", holder.pid)) != namespace_of(String::from("/proc/self/ns/user")),
            };
            if namespace_ready {
                return true;
            }
        }
        pause(Duration::from_millis(20));
    }
    return false;
}

/// What:
///  The path holder's phase:
///  the control refuses,
///  the signal stage ends the holder,
///  the host's unmount is accepted and the head destroy answers `gone`.
/// Why:
///  This is the kind the table's path-holder row names,
///  and the arm's first two phases are its two endings.
fn phase_path_holder(fixture: &Fixture, ignores_the_ask: bool) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    mount_the_clone(fixture, slot.as_path());
    let holder: AHolder = hold(AShape::SessionOfItsOwn, checkout.as_path(), slot.as_path(), ignores_the_ask);
    // The holder's own session must be outside the caller's, or this phase would measure the protected set instead.
    assert!(wait_for_the_holder(AShape::SessionOfItsOwn, &holder), "the holder leads a session of its own");
    let control: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("control", if control == "accepted" { "unexpectedly-accepted" } else { "refused" }));
    let blocker: Blocker = Blocker::read(holder.pid).expect("the holder's own record is read");
    let stage = end_the_blockers(&[blocker], a_harness_window(), pause, || return holder.alive());
    let ended: &'static str = match stage.endings.first().map(|(_blocker, result)| return *result) {
        Some(EndResult::ExitedAfterTheAsk) => "exited-after-the-ask",
        Some(EndResult::Killed) => "killed",
        Some(_other) => "other",
        None => "nothing",
    };
    fields.push(("end", ended));
    let unmounted: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("unmount", unmounted.as_str()));
    fields.push(("destroy", if destroy_the_clone(fixture) { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    holder.end();
    caller_job.end();
    assert!(alive, "the caller's session's other job lived through the phase");
    return phase_line(if ignores_the_ask { "path-holder-killed" } else { "path-holder-asked" }, fields.as_slice());
}

/// What:
///  One holder namespace's copy, its own descriptor and the tables the arm reads.
/// Why:
///  The act takes the namespace's descriptor and the identity the report names,
///  and the descriptor must be dropped when the phase's act is over: it is a
///  reference to that namespace, and a namespace kept alive keeps its mounts and
///  the dataset their superblock holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = aCopyInTheHolder(...); const copy = held.named(...);
/// ```
struct AHeldCopy {
    namespace: std::fs::File,
    identity: String,
    deeper: NamespaceTable,
    receivers: ReceiverSet,
}

impl AHeldCopy {
    /// The act's own view of the copy,
    /// borrowed from the descriptor this value owns.
    fn named<'a>(&'a self, fixture: &'a Fixture, checkout: &'a Path, slot: &'a Path, handle: &'a RecordedHandle) -> NamespaceCopy<'a> {
        return NamespaceCopy {
            namespace: &self.namespace,
            identity: self.identity.as_str(),
            checkout,
            slot,
            dataset: fixture.pool.as_str(),
            handle,
            directory: THE_RESERVED_NAME,
        };
    }
}

/// What:
///  Read one holder namespace's copy through its own table,
///  with the host's table beside it for the closure.
/// Why:
///  The closure needs both namespaces' rows and each namespace's topmost mount at
///  the slot,
///  and the identity is the one the journal and the report name.
fn a_copy_in_the_holder(fixture: &Fixture, slot: &Path, holder: &AHolder) -> AHeldCopy {
    let mut deeper: NamespaceTable = the_table_of_the_process(holder.pid);
    deeper.at_the_slot = topmost_at(deeper.rows.as_slice(), slot);
    let host_rows: Vec<MountRow> = the_table();
    let ours: NamespaceTable = NamespaceTable { namespace: String::from("the host's"), at_the_slot: topmost_at(host_rows.as_slice(), slot), rows: host_rows };
    let copy_row: MountRow = the_copy(fixture, deeper.rows.as_slice(), slot);
    let tables: Vec<NamespaceTable> = vec![deeper.clone(), ours];
    // The act handles the holder's namespace and never the host's, so a host that receives the unmount is the
    // outside receiver the design's closure names.
    let handled: Vec<String> = vec![deeper.namespace.clone()];
    let receivers: ReceiverSet = plan_the_receivers(&deeper, &copy_row, tables.as_slice(), fixture.pool.as_str(), handled.as_slice());
    let namespace: std::fs::File = std::fs::File::open(format!("/proc/{}/ns/mnt", holder.pid)).expect("the holder's namespace is opened");
    return AHeldCopy { namespace, identity: deeper.namespace.clone(), deeper, receivers };
}

/// What:
///  Way 8's phase:
///  the holder's parent shares with a group of its own,
///  the closure names no outside receiver,
///  the unmount in place takes the holder's copy and the host's stays.
fn phase_way_8(fixture: &Fixture) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    std::fs::create_dir_all(slot.as_path()).expect("the slot's underlay");
    let underlay: std::fs::File = open_directory(slot.as_path()).expect("the underlay is opened");
    let handle: RecordedHandle = name_the_handle(&underlay).expect("the underlay's handle is taken before the clone is mounted");
    mount_the_clone(fixture, slot.as_path());
    // The holder's copy is a private copy of the host's tree, and its own checkout is made a group of its own, which
    // is the shape systemd leaves a unit's parent.
    let holder: AHolder = hold(AShape::NamespaceOfItsOwnAndShared, checkout.as_path(), slot.as_path(), false);
    assert!(wait_for_the_holder(AShape::NamespaceOfItsOwnAndShared, &holder), "the holder's own namespace and session are there");
    let held: AHeldCopy = a_copy_in_the_holder(fixture, slot.as_path(), &holder);
    let copy: NamespaceCopy<'_> = held.named(fixture, checkout.as_path(), slot.as_path(), &handle);
    let receivers: ReceiverSet = held.receivers.clone();
    fields.push(("parent", if held.deeper.rows.iter().any(|row| return row.mount_point == checkout.to_string_lossy() && !row.shared.is_empty()) { "shared" } else { "not-shared" }));
    fields.push(("way-eight", if receivers.way_eight { "true" } else { "false" }));
    let ready: Ready = make_the_way_ready(Route::InPlaceByTheHandle, &copy).expect("the handle is proved and the copy is the topmost mount at the slot");
    let taken = take_the_copy_out(&ready, &copy).expect("the unmount by the handle is accepted");
    let after_theirs: Vec<MountRow> = the_table_of_the_process(holder.pid).rows;
    let after_ours: Vec<MountRow> = the_table();
    fields.push(("mount", Box::leak(taken.mount_id.to_string().into_boxed_str())));
    fields.push(("holders-copy", if holds_the_copy_at(after_theirs.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("hosts-copy", if holds_the_copy_at(after_ours.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("holder", if holder.alive() { "alive" } else { "gone" }));
    let unmounted: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("unmount", unmounted.as_str()));
    fields.push(("destroy", if destroy_the_clone(fixture) { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    holder.end();
    caller_job.end();
    assert!(taken.mount_id > 0);
    return phase_line("namespace-way8-systemd-shape", fields.as_slice());
}

/// What:
///  The peer shape:
///  the checkout is shared before the holder unshares,
///  so its copy is a peer of the host's,
///  the closure names the host and the act falls to way 7.
fn phase_peer(fixture: &Fixture) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    std::fs::create_dir_all(slot.as_path()).expect("the slot's underlay");
    let underlay: std::fs::File = open_directory(slot.as_path()).expect("the underlay is opened");
    let handle: RecordedHandle = name_the_handle(&underlay).expect("the underlay's handle is taken before the clone is mounted");
    mount_the_clone(fixture, slot.as_path());
    make_the_checkout_shared(checkout.as_path()).expect("the host makes the checkout a group of its own");
    let holder: AHolder = hold(AShape::NamespaceOfItsOwn, checkout.as_path(), slot.as_path(), false);
    pause(Duration::from_millis(200));
    let held: AHeldCopy = a_copy_in_the_holder(fixture, slot.as_path(), &holder);
    let copy: NamespaceCopy<'_> = held.named(fixture, checkout.as_path(), slot.as_path(), &handle);
    let receivers: ReceiverSet = held.receivers.clone();
    fields.push(("way-eight", if receivers.way_eight { "true" } else { "false" }));
    fields.push(("outside", if receivers.outside.is_empty() { "none" } else { "the-host" }));
    let route: Route = if receivers.way_eight { Route::InPlaceByTheHandle } else { Route::SlaveMoveAndUnmount };
    fields.push(("route", route.word()));
    let ready: Ready = make_the_way_ready(route, &copy).expect("the parent is made a slave and way 7's directory is made");
    let taken = take_the_copy_out(&ready, &copy).expect("the move and the unmount at the destination are accepted");
    let after_theirs: Vec<MountRow> = the_table_of_the_process(holder.pid).rows;
    let after_ours: Vec<MountRow> = the_table();
    let parent_now: Option<MountRow> = after_theirs.iter().find(|row| return row.mount_point == checkout.to_string_lossy()).cloned();
    fields.push(("parent-after", propagation_of(parent_now.as_ref())));
    fields.push(("holders-copy", if holds_the_copy_at(after_theirs.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("hosts-copy", if holds_the_copy_at(after_ours.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("directory", if checkout.join(THE_RESERVED_NAME).exists() { "left" } else { "removed" }));
    let unmounted: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("unmount", unmounted.as_str()));
    fields.push(("destroy", if destroy_the_clone(fixture) { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    holder.end();
    caller_job.end();
    assert!(taken.directory.is_some(), "way 7 is the route the closure sends this shape to");
    return phase_line("namespace-way8-peer-sent-to-way7", fields.as_slice());
}

/// What:
///  Way 7's phase,
///  in the repaired shape and with the checkout shared:
///  the parent's tags before and after,
///  the move,
///  the unmount at the destination and the directory's removal.
fn phase_way_7(fixture: &Fixture, shared_here: bool) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    std::fs::create_dir_all(slot.as_path()).expect("the slot's underlay");
    let underlay: std::fs::File = open_directory(slot.as_path()).expect("the underlay is opened");
    let handle: RecordedHandle = name_the_handle(&underlay).expect("the underlay's handle is taken before the clone is mounted");
    mount_the_clone(fixture, slot.as_path());
    let shape: AShape = if shared_here {
        make_the_checkout_shared(checkout.as_path()).expect("the host shares the checkout");
        AShape::NamespaceOfItsOwnAsSlave
    } else {
        // The repaired state: the checkout is private, so the holder's copy of it is private too, and its seed group
        // is empty.
        AShape::NamespaceOfItsOwn
    };
    let holder: AHolder = hold(shape, checkout.as_path(), slot.as_path(), false);
    assert!(wait_for_the_holder(shape, &holder), "the holder's own namespace and session are there");
    let held: AHeldCopy = a_copy_in_the_holder(fixture, slot.as_path(), &holder);
    let copy: NamespaceCopy<'_> = held.named(fixture, checkout.as_path(), slot.as_path(), &handle);
    let receivers: ReceiverSet = held.receivers.clone();
    let parent_before: Option<MountRow> = held.deeper.rows.iter().find(|row| return row.mount_point == checkout.to_string_lossy()).cloned();
    fields.push(("parent-before", propagation_of(parent_before.as_ref())));
    let route: Route = if receivers.way_eight { Route::InPlaceByTheHandle } else { Route::SlaveMoveAndUnmount };
    fields.push(("way-eight", if receivers.way_eight { "true" } else { "false" }));
    fields.push(("route", route.word()));
    let ready: Ready = make_the_way_ready(Route::SlaveMoveAndUnmount, &copy).expect("the parent is made a slave and the directory is made");
    let taken = take_the_copy_out(&ready, &copy).expect("the move and the unmount at the destination are accepted");
    let after_theirs: Vec<MountRow> = the_table_of_the_process(holder.pid).rows;
    let after_ours: Vec<MountRow> = the_table();
    let parent_after: Option<MountRow> = after_theirs.iter().find(|row| return row.mount_point == checkout.to_string_lossy()).cloned();
    fields.push(("parent-after", propagation_of(parent_after.as_ref())));
    fields.push(("holders-copy", if holds_the_copy_at(after_theirs.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("hosts-copy", if holds_the_copy_at(after_ours.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    fields.push(("directory", if checkout.join(THE_RESERVED_NAME).exists() { "left" } else { "removed" }));
    fields.push(("mount", Box::leak(taken.mount_id.to_string().into_boxed_str())));
    let unmounted: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("unmount", unmounted.as_str()));
    fields.push(("destroy", if destroy_the_clone(fixture) { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    holder.end();
    caller_job.end();
    assert!(a_row(after_theirs.as_slice(), taken.mount_id).is_none(), "the copy's mount id is gone from the holder's table");
    return phase_line(if shared_here { "namespace-way7-shared" } else { "namespace-way7-repaired" }, fields.as_slice());
}

/// What:
///  The locked holder's phase:
///  the owner read names a user namespace of its own,
///  and both ways are refused before any signal.
fn phase_locked(fixture: &Fixture) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    std::fs::create_dir_all(slot.as_path()).expect("the slot's underlay");
    let underlay: std::fs::File = open_directory(slot.as_path()).expect("the underlay is opened");
    let handle: RecordedHandle = name_the_handle(&underlay).expect("the underlay's handle is taken before the clone is mounted");
    mount_the_clone(fixture, slot.as_path());
    let holder: AHolder = hold(AShape::UserNamespaceOfItsOwn, checkout.as_path(), slot.as_path(), false);
    assert!(wait_for_the_holder(AShape::UserNamespaceOfItsOwn, &holder), "the holder's own user namespace and session are there");
    let owner: NamespaceOwner = read_the_owner(Path::new(format!("/proc/{}/ns/mnt", holder.pid).as_str())).expect("the holder's namespace owner is read");
    fields.push(("owner", if owner.locked() { "another" } else { "ours" }));
    let held: AHeldCopy = a_copy_in_the_holder(fixture, slot.as_path(), &holder);
    let copy: NamespaceCopy<'_> = held.named(fixture, checkout.as_path(), slot.as_path(), &handle);
    // Way 8's structural stage only opens the handle, which a locked namespace still allows; its unmount is where the
    // kernel refuses, so the phase takes the copy out and reports the word that stopped it.
    let way8: String = match make_the_way_ready(Route::InPlaceByTheHandle, &copy) {
        Ok(ready) => match take_the_copy_out(&ready, &copy) {
            Ok(_taken) => String::from("accepted"),
            Err(refusal) => String::from(refusal.word.word()),
        },
        Err(refusal) => String::from(refusal.word.word()),
    };
    let way7: String = match make_the_way_ready(Route::SlaveMoveAndUnmount, &copy) {
        Ok(ready) => match take_the_copy_out(&ready, &copy) {
            Ok(_taken) => String::from("accepted"),
            Err(refusal) => String::from(refusal.word.word()),
        },
        Err(refusal) => String::from(refusal.word.word()),
    };
    fields.push(("way8", way8.as_str()));
    fields.push(("way7", way7.as_str()));
    let after_theirs: Vec<MountRow> = the_table_of_the_process(holder.pid).rows;
    fields.push(("holders-copy", if holds_the_copy_at(after_theirs.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    // No destroy is tried while the copy stands: `zfs destroy` unmounts the dataset from the caller's own namespace on
    // its way, so it is no non-destructive control. The act's own descriptor is a reference to the holder's namespace,
    // so it is dropped before the holder ends, or the namespace and its copy would outlive the process.
    drop(held);
    holder.end();
    pause(Duration::from_millis(200));
    let after_ending: Vec<MountRow> = the_table();
    fields.push(("hosts-copy", if holds_the_copy_at(after_ending.as_slice(), slot.as_path(), fixture.clone().as_str()) { "stays" } else { "gone" }));
    let ended: String = match unmount_the_slot(slot.as_path()) {
        Ok(()) => String::from("accepted"),
        Err(error) => format!("errno {}", error.raw_os_error().unwrap_or(0)),
    };
    fields.push(("unmount-after", ended.as_str()));
    fields.push(("destroy-after", if destroy_the_clone(fixture) { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    caller_job.end();
    assert!(owner.locked(), "the holder's namespace is owned by a user namespace of its own");
    return phase_line("locked-holder-refused", fields.as_slice());
}
