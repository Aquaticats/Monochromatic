//! What:
//!  The one shape a test's helper process is started with:
//!  a group of its own that dies with the test thread and with this process,
//!  and the group written down so the thread's fixtures can end it.
//! Why:
//!  `Child` does not kill on drop,
//!  so a child a failing assertion never reached the end of (a service whose idle exit is hours away,
//!  a shell loop) kept running after its test:
//!  a forced-removal fixture's child outlived its run by 45 minutes (issue 6463),
//!  and a test binary ended by `SIGKILL` or by the watchdog's `abort` runs no guard at all.
//!  The group kill is the guard,
//! and the kernel's parent-death signal is what still runs when a
//!  process is killed outright.
//!  Written once here,
//! every target that starts such a child answers the same way:
//!  the library's tests,
//! the binary controls' crate and the candidate consumer's crate include this file by path.
//!  The fixture folders a helper works in are `fixture_support_shared_by_both_test_targets.rs`'s,
//!  so the ownership stays here,
//!  where the candidate consumer's crate,
//! which keeps no fixtures,
//! can take it without the folder code it never runs.
//! Gotcha:
//!  A child that must lead a session of its own cannot be started here,
//! because a group leader cannot start a session;
//!  `child_test_process` starts that one with `own_group` alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = spawnOwned(command);   // its own group, killed with the test thread and with the process
//! ```

/// What:
///  `Command` is the standard library's description of a program to start (the equivalent of TS's
///  `{ program, args, options }` object),
/// and `Child` is the running program that starting it hands back.
/// Why:
///  The start below fills in one description and answers the running child it created,
///  so the caller can watch or end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const command = { program: '/bin/sleep', args: [], options: {} };
/// // const child: { pid: number; kill(): void; wait(): Promise<{ code: number }> } = spawn(command);
/// ```
use std::process::{
    Child,
    Command,
};

/// What:
///  The process groups the helper processes of one test thread lead,
///  killed when the thread ends however its test ended.
/// Why:
///   `Child` does not kill on drop,
///  so a helper a failing assertion never reached the end of (a service whose idle exit is hours away,
///  a shell loop) kept running after its test.
///  A panic unwinds the test thread,
/// which runs this type's destructor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ownedGroups: number[] = []; // a FinalizationRegistry or a try/finally kills them at the thread's end
/// ```
pub(crate) struct OwnedGroups {
    /// What:
    ///  The group numbers,
    /// which are the numbers of the group leaders;
    ///  `RefCell` allows the list to be changed through a shared reference (TS has no such distinction).
    /// Why:
    ///  Removing a helper's group from the list needs a mutable view of it while the thread-local holding it is
    ///  shared,
    /// and a `RefCell` checks at run time that only one such view is alive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// groups: number[];
    /// ```
    groups: std::cell::RefCell<Vec<libc::pid_t>>,
}

impl OwnedGroups {
    /// What:
    ///  Read back the group the child is in,
    ///  fail the control when the child does not lead a group of its own,
    ///  and remember the group so the thread's end kills it.
    /// Why:
    ///  A child that a control started right leads its own group,
    ///  and one it started wrong is in its parent's;
    ///  the number has to be read before the child is ended and reaped,
    ///  or the question cannot be answered at all.
    ///  A child that does not lead one is a defect of its control,
    /// not a helper:
    ///  the number registered here would name no group,
    ///  the kill would send its signal nowhere,
    ///  and what the helper started would outlive the test.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// own(child: Child): void; // const group = getpgid(child.pid); assert(group === child.pid); groups.push(group);
    /// ```
    fn own(
        &self,
        child: &Child,
    ) {
        let group: libc::pid_t = group_of(child);
        // What: `assert_eq!` ends the test unless the two values are equal, naming both and the message it is given.
        // Why: a helper in the wrong group is not owned, and this must fail here rather than leave a process running.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (group !== child.pid) throw new Error(`the helper ${child.pid} was registered as a process group but is in group ${group}`);
        // ```
        assert_eq!(
            group,
            libc::pid_t::try_from(child.id()).expect("a pid fits the pid type"),
            "the helper {} was registered as a process group but is in group {group}: it must be started with `process_group(0)` (spawn_owned does that), or nothing it starts dies with the test",
            child.id()
        );
        self.remember(group);
    }

    /// What:
    ///  Add one group number to the list the thread's end kills.
    /// Why:
    ///   `own` and `own_group_of_number` each prove the number they are given,
    ///  and both then want this one thing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remember(group: number): void; // groups.push(group)
    /// ```
    fn remember(
        &self,
        group: libc::pid_t,
    ) {
        // What: `borrow_mut` asks the `RefCell` for a mutable view of the list, and `push` adds one group number.
        // Why: the thread's destructor kills every group the list holds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // groups.push(group);
        // ```
        self.groups.borrow_mut().push(group);
    }

    /// What:
    ///  Kill every process group in the list.
    /// Why:
    ///  The thread's end and the type's own destructor both end the helpers here,
    /// and what the helpers started is in
    ///  these groups,
    /// so one signal per group reaches all of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// killAll(): void; // endGroups(groups)
    /// ```
    fn kill_all(&self) {
        // What: `borrow_mut` holds a mutable view of the list while `end_groups` walks it.
        // Why: the signal is sent while the list is read, and nothing else may change it meanwhile.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // endGroups(groups);
        // ```
        let groups: std::cell::RefMut<Vec<libc::pid_t>> = self.groups.borrow_mut();
        end_groups(groups.as_slice());
    }
}

/// A panic unwinds the test thread,
/// which runs this.
impl Drop for OwnedGroups {
    fn drop(&mut self) {
        self.kill_all();
    }
}

thread_local! {
    /// The helper groups of the running test thread.
    /// Why:
    ///  Each test thread owns the children it started,
    /// so the list lives with the thread and is killed when it ends.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // thread-local: const ownedGroups: number[] = [];
    /// ```
    static OWNED_GROUPS: OwnedGroups = const { OwnedGroups { groups: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  `killpg` every group in `groups`,
/// with `SIGKILL`,
/// and read no memory of theirs.
/// Why:
///  A helper leads a group of its own,
/// so this reaches it and everything it started;
///  a group with no member left answers `ESRCH`,
/// which is the state this wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const group of groups) process.kill(-group, 'SIGKILL');
/// ```
fn end_groups(groups: &[libc::pid_t]) {
    for group in groups {
        // SAFETY: `killpg` takes a group number and a signal and touches no memory; a group with no member is `ESRCH`.
        // What: `killpg` sends a signal to every process of a group; `unsafe` because the compiler cannot check the call.
        // Why: the helper and everything it started end together.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // process.kill(-group, 'SIGKILL');
        // ```
        unsafe {
            libc::killpg(*group, libc::SIGKILL);
        }
    }
}

/// What:
///  End every helper group this thread started,
/// now,
///  which is what the thread's fixtures do before they remove the directories those helpers work in.
/// Why:
///  A helper still running holds its fixture directory open,
/// and a directory removed under a live process is a
///  failure the next control reads as its own;
///  the fixtures call this first,
/// so the helpers die before their directories go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endOwnedGroups(): void { endGroups(ownedGroups.get().groups); ownedGroups.get().groups.length = 0; }
/// ```
#[allow(
    dead_code,
    reason = "the library's thread fixtures are what call this before their directories go; a target that keeps no fixtures ends its groups with the thread's own end"
)]
pub(crate) fn end_owned_groups() {
    // What: `try_with` asks the thread-local for this thread's list and answers an error instead of panicking when the
    //  list has already been destroyed, which happens when the fixtures go after the list does.
    // Why: a thread whose own list was destroyed before its fixtures is the case `try_with` answers: that list's `Drop`
    //  ended its groups already, so there is nothing here to end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const groups = tryTakeOwnedGroups() ?? [];
    // ```
    let groups: Vec<libc::pid_t> = OWNED_GROUPS
        // The list is taken first and emptied after, so a group ended twice is not an error and no group is missed.
        .try_with(|owned| {
            return std::mem::take(&mut *owned.groups.borrow_mut());
        })
        .unwrap_or_default();
    end_groups(groups.as_slice());
}

/// What:
///  The process group `child` is in,
/// asked while the child is still there.
/// Why:
///   A helper a control started right leads its own group,
///  and one it started wrong is in its parent's;
///  the number has to be read before the child is ended and reaped,
///  or the question cannot be answered at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupOf(child: Child): number; // getpgid(child.pid)
/// ```
pub(crate) fn group_of(child: &Child) -> libc::pid_t {
    // What: `pid_t` is the C signed integer type that holds a process number; `try_from` converts the child's
    //  unsigned number into it and answers a `Result` that a too-large number would fail.
    // Why: the kernel calls below take a `pid_t`, so the number has to be in that type.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pid = child.pid;
    // ```
    let pid: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    // What: `getpgid` asks the kernel which group that process is in.
    // Why: the answer is compared with the child's own number, which is the group a `process_group(0)` start leads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const group = getpgid(pid);
    // ```
    // SAFETY: `getpgid` answers for a child of this process and touches no memory.
    let group: libc::pid_t = unsafe { libc::getpgid(pid) };
    // What: `assert!` ends the test unless its condition holds, and names the process and the system's error.
    // Why: a group of zero or less is a failed call, and a caller that went on would register a number that names
    //  nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (group <= 0) throw new Error(`the group of the helper ${pid} is asked while it is there: ${errno}`);
    // ```
    assert!(
        group > 0,
        "the group of the helper {pid} is asked while it is there: {}",
        std::io::Error::last_os_error()
    );
    return group;
}

/// What:
///  Make the process group a child leads die with the test thread that started it.
/// Why:
///   A helper that must become a session leader (a terminal helper calls `setsid` between `fork` and `exec`) cannot
///  also be put in a group by `spawn_owned`,
///  because a group leader cannot start a session.
///  Such a child leads a group of its own already,
///  so only the thread's list of groups to end needs its number,
///  and the number is read back so a control that started the helper wrong fails here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownGroup(child: Child): void; // ownedGroups.push(assertOwnGroup(child))
/// ```
pub(crate) fn own_group(child: &Child) {
    OWNED_GROUPS.with(|owned| {
        owned.own(child);
    });
}

/// What:
///  Make the process group a child of a shipped call leads die with the test thread that started the call,
///  named by the number that call or the child itself answered rather than by a `Child` the test keeps.
/// Why:
///   The relay starts the child it reads itself and keeps no `Child` the test can reach,
///  so a control of what the child left holds a number and nothing `own_group` takes (issue 6501).
///  The child must be started with `process_group(0)`,
///  and a group's number cannot be given to another process while one member is left in it,
///  so the number names that child's group and nothing else.
/// Gotcha:
///  A number of `0`,
///  or one whose group holds no process,
///  is refused,
///  because nothing would die at the thread's end and one of them would end the run itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownGroupOfNumber(number: number): void; // assert(groupHasAMember(number)); ownedGroups.push(number)
/// ```
#[allow(
    dead_code,
    reason = "the library's own controls read a relayed child; the binary controls' crate and the candidate consumer's crate include this file by path and read no such child"
)]
pub(crate) fn own_group_of_number(number: u32) {
    let group: libc::pid_t = libc::pid_t::try_from(number).expect("a pid fits the pid type");
    // What: to `killpg` and every other signal call, `0` names the group of the process that sends it.
    // Why:  A read answers `0` as the child's number when no `SignalGuard` started that child
    //  (`read_to_the_child_s_end`), and a group registered as `0` would have the thread's end kill every process of
    //  the run, this test binary included (issue 6501).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(group > 0, 'the group of a helper is the process number of the child that leads it');
    // ```
    assert!(
        group > 0,
        "a group number names a helper, and {group} names the group of the process that sends the signal instead: the read answers 0 for a child no signal guard started"
    );
    // What: `killpg` with signal `0` asks whether the group has any member and sends no signal.
    // Why:  A number that names no group names nothing this thread must end: either the child was not started with
    //  `process_group(0)`, so the number is the child alone and no group, or what it left has ended already.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(killpg(group, 0) === 0, `the group ${group} holds no process`);
    // ```
    // SAFETY: `killpg` takes a group number and a signal number and touches no memory of this process.
    let present: libc::c_int = unsafe { libc::killpg(group, 0) };
    assert_eq!(
        present, 0,
        "the group {group} holds no process: start the child with `process_group(0)`, so what it leaves stays in a group the test can end, or end what it left before registering it"
    );
    OWNED_GROUPS.with(|owned| {
        owned.remember(group);
    });
}

/// What:
///  Start a helper process that dies with the test thread that started it,
///  whether the test passes,
///  fails,
///  panics,
///  or the whole test process is killed.
/// Why:
///   A shell loop that a failing push-progress test never reached the end of ran for 87 minutes in a deleted fixture.
///  The helper leads a process group of its own,
///  so what it starts (a `sleep` of the loop) is killed with it,
///  and the group is
///  killed when the thread's fixtures are dropped,
///  the same moment the fixture directories go.
///   The group kill is a guard this process runs,
///  and a process ended by `SIGKILL` or by `run_progress`'s `abort` runs no guard,
///  so the helper also asks the kernel for a parent-death signal,
///  which ends it when the thread that started it is gone however the process ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnOwned(command): Child; // command.options.detached = true; preExec(() => prctl(PR_SET_PDEATHSIG, SIGKILL)); ownGroup(spawn(command))
/// ```
pub(crate) fn spawn_owned(command: &mut Command) -> std::io::Result<Child> {
    // What: `CommandExt` adds the Unix-only methods `process_group` and `pre_exec` to the standard library's command
    //  type; `as _` imports the trait without naming it.
    // Why: the group and the parent-death signal are set on the command before the child starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // spawn(command, { detached: true })
    // ```
    use std::os::unix::process::CommandExt as _;
    // What: `process_group(0)` asks the child to lead a new process group whose number is its own process number.
    // Why: a signal to that group then reaches the child and everything it starts, and nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // command.detached = true;
    // ```
    command.process_group(0);
    // What: `pre_exec` registers a function that runs in the child between `fork` and `exec`, where only
    //  async-signal-safe calls are allowed; `prctl(PR_SET_PDEATHSIG, SIGKILL)` asks the kernel to kill the child when
    //  the thread that started it ends.
    // Why: a test binary ended by `SIGKILL`, or by the `abort`s of `run_progress`, runs no guard at all, so the group
    //  kill at the thread's end never happens; the kernel's parent-death signal is what ends the child then.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // command.preExec = () => { prctl(PR_SET_PDEATHSIG, 9); };
    // ```
    // SAFETY: between `fork` and `exec` only `prctl`, an async-signal-safe call, runs, and it touches no memory of this process.
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            return Ok(());
        });
    }
    // What: the `?` after `spawn()` hands a failed start to the caller as the function's own failure, and otherwise
    //  takes the running child out of the `Result`.
    // Why: a program a control needs that could not start must fail the control, not be panicked over here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = spawnOrThrow(command);
    // ```
    let child: Child = command.spawn()?;
    own_group(&child);
    // What: the tail expression hands the running child to the caller.
    // Why: the caller reads its output or ends it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return child;
    // ```
    return Ok(child);
}
