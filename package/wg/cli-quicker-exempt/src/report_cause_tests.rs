//! Verifies what makes two failure lines the same cause:
//! numbers and the names a user chose do not,
//! a different reason does,
//! and the fixed phrase every loud line starts with must not hide the reason.

/// The cause under test.
use crate::report_cause::{held_suffix, normalized_cause, without_held_suffix};
/// The fixed phrase every loud line starts with.
use crate::update_report::policy_failure;

/// Numbers become alike,
/// and everything else is kept.
#[test]
fn numbers_are_made_alike_and_text_is_kept() {
    assert_eq!(normalized_cause("pid 12 failed (attempt 3) in 30s"), "pid # failed (attempt #) in #s");
    assert_eq!(normalized_cause("pid 12 failed"), normalized_cause("pid 99999 failed"));
    assert_ne!(normalized_cause("pid 12 failed: boom"), normalized_cause("pid 12 failed: other"));
    assert_eq!(normalized_cause(""), "");
}

/// Absolute paths,
/// which carry cgroup names,
/// and unit names are the names a user chose:
/// they become a placeholder and keep their surrounding punctuation.
#[test]
fn names_a_user_chose_are_made_alike() {
    let first = normalized_cause("its cgroup /user.slice/user-1000.slice/user@1000.service/app.slice/app-a.scope is outside");
    let second = normalized_cause("its cgroup /user.slice/user-1000.slice/user@1000.service/app.slice/app-completely-other.scope is outside");
    assert_eq!(first, second);
    assert_eq!(first, "its cgroup <name> is outside");
    assert_eq!(
        normalized_cause("it is the main process of the service app-x@1.service, and moving it would empty that service"),
        normalized_cause("it is the main process of the service app-other@22.service, and moving it would empty that service")
    );
    assert_eq!(normalized_cause("(/a/b/c), then /d/e."), "(<name>), then <name>.");
    assert_ne!(normalized_cause("its cgroup /a is outside"), normalized_cause("its cgroup /a is inside"));
    assert_eq!(normalized_cause("a word that merely ends .service-like"), "a word that merely ends .service-like");
    assert_eq!(normalized_cause("the .service suffix alone"), "the .service suffix alone");
}

/// Every loud line starts with a fixed phrase of over a hundred characters,
/// and two different errors of one kind must not collapse because a short bound ended inside it.
/// This is the case of the reviewer:
/// two different `busctl` errors for a move that failed.
#[test]
fn two_different_errors_behind_the_fixed_phrase_are_two_causes() {
    assert!(policy_failure("").len() > 100, "the fixed phrase is long, which is why the bound must not end inside it");
    let denied = policy_failure("pid 5 matched but the move failed (attempt 1, next attempt in 1s; the limits of its old cgroup are never dropped to make it succeed): Access denied");
    let exists = policy_failure("pid 5 matched but the move failed (attempt 1, next attempt in 1s; the limits of its old cgroup are never dropped to make it succeed): Unit wg-quicker-update-5-9.scope already exists");
    assert_ne!(normalized_cause(&denied), normalized_cause(&exists));
    let other_pid = policy_failure("pid 77 matched but the move failed (attempt 2, next attempt in 2s; the limits of its old cgroup are never dropped to make it succeed): Access denied");
    assert_eq!(normalized_cause(&denied), normalized_cause(&other_pid), "the same error for another pid and attempt is one cause");
}

/// The line counts up to a bound,
/// so a line cannot make the cause unboundedly long.
#[test]
fn the_cause_is_bounded() {
    assert_eq!(normalized_cause(&"x".repeat(5000)).chars().count(), 1024);
    assert_eq!(normalized_cause(&"word ".repeat(5000)).chars().count(), 1024);
}

/// The note of held-back repeats is removed only when it has its exact shape.
#[test]
fn the_held_repeats_note_is_removed_only_in_its_exact_shape() {
    assert_eq!(without_held_suffix(&format!("x{}", held_suffix(12))), "x");
    assert_eq!(without_held_suffix("x (repeated many more time(s) since the last report)"), "x (repeated many more time(s) since the last report)");
    assert_eq!(without_held_suffix("x"), "x");
    assert_eq!(held_suffix(3), " (repeated 3 more time(s) since the last report)");
}
