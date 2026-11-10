//! Controls for the line a commit prints when the step it stopped is still held by the kernel.

use super::*;

/// The line names the step,
/// says the stop was sent,
/// says the kernel has not released the step and in which state it is held,
/// and says what the person's work is worth after it.
#[test]
fn the_line_names_the_step_the_stop_the_state_and_what_is_kept() {
    let said: String = held_by_the_kernel_line(HeldByTheKernel { step: 4242, waited: Duration::from_secs(17) });
    assert!(
        said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process 4242) was sent `SIGKILL`"),
        "{said}"
    );
    assert!(said.contains("its ceiling came after 17 seconds"), "the line says how long the step lasted: {said}");
    assert!(said.contains("the kernel has not released it"), "{said}");
    assert!(said.contains("uninterruptible sleep (state `D`)"), "the line names the state the person sees in `ps`: {said}");
    assert!(said.contains("ends when the call it is inside returns"), "{said}");
    assert!(said.contains("nothing of yours is at risk") && said.contains("your index and files are as they were"), "{said}");
    assert!(said.contains("The terminal stays here until the kernel lets go"), "{said}");
}

/// The line is one line,
/// so a terminal that is unusable anyway is not buried in words,
/// and it ends where a line does.
#[test]
fn the_line_is_one_line_that_ends_where_a_line_does() {
    let said: String = held_by_the_kernel_line(HeldByTheKernel { step: 7, waited: Duration::from_secs(1) });
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.ends_with('\n'), "{said}");
    assert!(!said.contains('\r'), "{said}");
}
