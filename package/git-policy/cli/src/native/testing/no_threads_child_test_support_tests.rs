//! The probe test of `no_threads_child_test_support.rs`.
//!
//! What:
//! one test,
//! `thread_probe`,
//! that tries to start one thread and prints whether it could.
//! Why:
//! `ran_in_child` starts it first in a child whose process limit is one;
//! if a thread still starts there,
//! this user may exceed the limit,
//! so the real test is skipped instead of proving nothing.
//! Run without the child's environment variable,
//! the probe returns at once and checks nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test("thread_probe", () => {
//!   if (!process.env.CLI_GIT_NO_THREADS_CHILD) return;
//!   console.log(canStartThread() ? "thread-started" : "thread-refused");
//! });
//! ```

/// Says whether one thread can start in this process,
/// in a child under the limit.
/// A test run without the variable does nothing.
#[test]
fn thread_probe() {
    if std::env::var_os(super::CHILD_VARIABLE).is_none() {
        return;
    }
    let started: bool = std::thread::Builder::new().spawn(|| {}).is_ok();
    println!(
        "{}",
        if started {
            "thread-started"
        } else {
            "thread-refused"
        }
    );
}

/// What:
///  Lowering this process's limit on the processes and threads it starts is refused
///  outside the child that `ran_in_child` started.
/// Why:
///  The limit belongs to the whole process,
///  and a control that lowers it where a threaded run keeps its tests stops every other test's starts with
///  `EAGAIN` (issue 6496);
///  the refusal is what makes such a control fail here, at its own first line, instead of in the rest of the run.
#[test]
#[should_panic(expected = "belongs to the whole process")]
fn refusing_the_thread_limit_outside_the_child_is_refused() {
    let _refused: std::io::Result<()> = super::refuse_threads_again();
}

/// What:
///  Giving the room for threads back is refused outside the child that `ran_in_child` started,
///  as the refusal itself is.
/// Why:
///  It is the same limit of the same process,
///  and a control that reaches it in the parent decides what every other test of the run may start.
#[test]
#[should_panic(expected = "belongs to the whole process")]
fn giving_the_room_for_threads_outside_the_child_is_refused() {
    let _allowed: std::io::Result<()> = super::let_threads_start_again();
}
