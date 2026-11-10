//! Controls for the line a commit prints when the step it stopped is still held by the kernel.

use super::*;

/// The line names the step,
/// says the stop was sent,
/// says the kernel has not released the step and in which state it is held,
/// and says what the person's work is worth after it.
#[test]
fn the_line_names_the_step_the_stop_the_state_and_what_is_kept() {
    let said: String = held_by_the_kernel_line(HeldByTheKernel { step: 4242, waited: Duration::from_secs(17), held: HeldProgram { number: 4242, name: String::from("git") } });
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
    let said: String = held_by_the_kernel_line(HeldByTheKernel { step: 7, waited: Duration::from_secs(1), held: HeldProgram { number: 7, name: String::from("git") } });
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.ends_with('\n'), "{said}");
    assert!(!said.contains('\r'), "{said}");
}

/// What:
///  A program below the step that the kernel is holding is named as that program,
///  and the step is still named as the commit that was stopped.
/// Why:
///   A `git commit` that started a signing program is blocked through that program's call,
///  not by a call of its own,
///  and a line that named the step for a state only the program was seen in states a false fact about the step
///  (finding 4 of the 2026-11-10 review of the held-step notice).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(line({ step: 4242, waited: 17, held: { number: 5001, name: 'ssh-keygen' } })).toContain('`ssh-keygen` (process 5001)')
/// ```
#[test]
fn the_line_names_the_program_the_kernel_holds_when_it_is_not_the_step() {
    let said: String = held_by_the_kernel_line(HeldByTheKernel {
        step: 4242,
        waited: Duration::from_secs(17),
        held: HeldProgram { number: 5001, name: String::from("ssh-keygen") },
    });
    assert!(
        said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process 4242) was sent `SIGKILL`"),
        "{said}"
    );
    assert!(
        said.contains("the program it started, `ssh-keygen` (process 5001), is in uninterruptible sleep (state `D`)"),
        "the line names the program the kernel is holding: {said}"
    );
    assert!(
        said.contains("ends when the call that program is inside returns"),
        "the line says what the wait is: {said}"
    );
    assert!(!said.contains("(process 4242) was sent `SIGKILL` when its ceiling came after 17 seconds, and the kernel has not released it: it is in"),
        "the line still said the step itself is in uninterruptible sleep: {said}");
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.ends_with('\n'), "{said}");
}
