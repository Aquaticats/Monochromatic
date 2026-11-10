//! Controls for one look at the step and the programs below it.

use super::*;

/// One process a look saw,
/// as the kernel reports it:
/// its name,
/// start time,
/// state letter and processor time.
fn seen(pid: i32, state: char, cpu_millis: u64) -> (i32, ProcessUse) {
    return (pid, ProcessUse { name: String::from("git"), started: 1, state, cpu: Duration::from_millis(cpu_millis) });
}

/// A process inside a call a signal cannot end is the one a ceiling cannot end,
/// and a process that can act on a signal is not,
/// nor is a zombie,
/// which has acted on it already.
#[test]
fn a_process_in_uninterruptible_sleep_is_the_one_a_signal_cannot_end() {
    assert!(process_is_blocked(&seen(7, 'D', 0)));
    assert!(!process_is_blocked(&seen(7, 'S', 0)), "a process that can act on a signal is not held by the kernel");
    assert!(!process_is_blocked(&seen(7, 'Z', 0)), "a zombie has acted on its signal");
}

/// A look reports the processor time its process used.
#[test]
fn a_look_reports_the_processor_time_its_process_used() {
    assert_eq!(process_cpu(&seen(7, 'S', 17)), Duration::from_millis(17));
}
