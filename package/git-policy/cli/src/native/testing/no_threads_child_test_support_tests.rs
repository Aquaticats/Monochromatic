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
    println!("{}", if started { "thread-started" } else { "thread-refused" });
}
