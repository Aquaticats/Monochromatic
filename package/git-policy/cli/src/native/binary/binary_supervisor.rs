//! What:
//!  The hang guard of the binary tests:
//! a thread that watches the process tree of each wrapped command the test started
//!  and ends it when it makes no progress or burns too much processor time,
//! instead of ending every command after five
//!  seconds of wall time.
//! Why:
//!  A wall-clock bound cannot tell a command that waits on the disk,
//! or is starved of the processor by other processes,
//!  from one that loops or has deadlocked.
//! On this host other sessions' write-back stalls a single `fsync` of a trivial
//!  command for several seconds (4.74 s measured,
//! `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
//!  and a test that fails for that is flaky,
//! not a finding about the wrapper.
//! What a hang looks like instead is a tree that
//!  does nothing:
//! every task asleep on a lock,
//! a futex,
//! a pipe or a socket,
//! none runnable and none waiting for the disk,
//!  with its processor time,
//! the bytes it moved,
//! the work of the children it waited for and its set of processes all standing
//!  still for a window.
//! A tree waiting in the kernel's read of a Unix socket is not that:
//! it waits for an answer from a
//!  program outside it,
//! the wrapper for the commit service's,
//! and that answer's latency is the service's own disk work,
//!  so no look of this tree can see it as progress or as a hang of its own
//!  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`,
//! "A merge that waited for the commit service's answer").
//! A loop looks like processor time that keeps growing past a budget no real command comes near.
//! Both end
//!  the command with a message that names it,
//! the reason and the tasks of the tree;
//! a wall-clock backstop of minutes ends
//!  anything else.
//!  The idle window never ends a command the old five-second wall bound would have let finish:
//! it needs five seconds of wall
//!  time first.
//! The processor budget is a second,
//! independent limit and is not a function of wall time:
//! a tree that runs in
//!  parallel can use ten processor seconds in less than five wall seconds,
//! which the old bound would have let finish,
//! and the
//!  budget ends it.
//! No real command of these tests comes near it (they use well under a second),
//! and a control that needs more
//!  says so with limits of its own.
//!  One monitor thread per test process finds the shared-bound commands among the test's own children (a `timeout` with
//!  the shared backstop),
//! so every way a test runs a command is watched,
//! whether it goes through `observe_raw` or starts
//!  the command itself.
//! A command the monitor ended is recorded before the first signal goes out;
//! the test that ran the command
//!  claims the record right after it waited,
//! and a record nobody claims fails the test:
//! the process aborts three seconds after
//!  the kill when no thread of the test is occupied with that command,
//! and a process that exits first exits with status 101 from an exit handler,
//! so a kill is never silent.
//!  The abort asks whether a thread is occupied with the command,
//!  never how long a claim took:
//! a test thread this host does not schedule for seconds would otherwise lose a
//!  passing result to a signal no output of it survives
//!  (issue 6469,
//! a gate on 2026-11-25).
//! Limits:
//!  - Signals reach only pinned processes (`binary_supervisor_pin.rs`):
//! a pidfd where the kernel has one,
//! and the start time read
//!    again before each signal where it has not.
//!  - A process that double-forks and starts a session leaves the tree it was found in and is neither watched nor ended by the
//!    guard.
//! The commit service and other background programs of the wrapper outlive their commands on purpose,
//! so ending
//!    everything that was ever the command's descendant would break the tests;
//! a hang in such a process is ended by the per-test
//!    limit of the test runner.
//!  - The idle rule cannot tell an endless wait from a finite wait on something outside the tree.
//! A test that holds a command
//!    for its own reasons says so with `hold_idle_judgment`;
//! a task that is stopped or traced is held from outside and is not
//!    judged idle either;
//! a pipe that fills is drained from the spawn by `observe_raw`,
//! so a command that writes much is not idle.
//!  - A loop that polls without moving bytes,
//! forking or burning processor time stays below every sign the guard reads.
//! No
//!    predicate over `/proc` decides it,
//! so it is ended by the wall-clock backstop or the per-test limit,
//! whichever comes first.
//!    Process churn is treated as a sign of life.
//!
//! The guard's parts,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `binary_supervisor_limits.rs`:
//!   the numbers it judges by and the reason it writes when it ends a command.
//! - `binary_supervisor_judge.rs`:
//!   the judgement of one watched command.
//! - `binary_supervisor_registry.rs`:
//!   the one registry of this process and the holds a thread takes on a command.
//! - `binary_supervisor_kills.rs`:
//!   the record of a command the monitor ended,
//!   and the abort and exit status of a record nobody claims.
//! - `binary_supervisor_shared_bound_commands.rs`:
//!   which commands run under the shared bound.
//! - `binary_supervisor_what_the_monitor_saw.rs`:
//!   what a control may ask about one watched command.
//! - `binary_supervisor_monitor.rs`:
//!   the monitor thread and the call that puts a command under it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new Judge(limits);
//! setInterval(() => { const reason = judge.observe(elapsed(), lookAt(pid)); if (reason) { record(reason); endTree(pin); fail(reason); } }, 100);
//! ```

/// The judgement of one watched command:
/// what its last look showed,
/// since when nothing moved,
/// and the reason that ends it.
pub use super::supervisor_judge::Judge;
/// The record of a command the monitor ended,
/// and what a test does with it.
pub use super::supervisor_kills::{
    end_command,
    kill_recorded,
    kill_to_abort,
    plant_kill,
    take_kill,
    unclaimed_kill,
};
/// The numbers the guard judges by,
/// the shared limits,
/// and the reason it writes when it ends a command.
pub use super::supervisor_limits::{
    CPU_BUDGET,
    Limits,
    NO_PROGRESS_WINDOW,
    POLL,
    Reason,
    SHARED,
    SHARED_BACKSTOP_SECONDS,
    UNCLAIMED_EXIT_STATUS,
    UNCLAIMED_GRACE,
};
/// The monitor thread,
/// and the call that puts one command under it.
pub use super::supervisor_monitor::{
    ensure_monitor,
    watch,
};
/// The holds a thread takes on a command's entry in the registry.
pub use super::supervisor_registry::{
    IdleHold,
    Occupied,
    ending_the_command_of,
    hold_idle_judgment,
    waiting_for,
};
/// Which commands run under the shared bound every test shares.
pub use super::supervisor_shared_bound_commands::{
    is_shared,
    is_shared_command_line,
};
/// What a control may ask about one watched command.
pub use super::supervisor_what_the_monitor_saw::{
    disk_wait_looks,
    is_watching,
    looks_of,
};
