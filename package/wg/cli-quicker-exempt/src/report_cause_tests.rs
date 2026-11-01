//! Verifies what makes two failure lines the same cause:
//! numbers and the names a user chose do not,
//! a different reason does,
//! and the fixed phrase every loud line starts with must not hide the reason.

/// The cause under test.
use crate::report_cause::{held_suffix, normalized_cause, quote_name, without_held_suffix};
/// The producer of lines that embed cgroup names.
use crate::routing_support::adoptable_cgroup;
/// The producer of lines that embed install roots.
use crate::install_roots::RefusedRoot;
/// The loud report for a refused root.
use crate::update_report::refused_root_failures;
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

/// A name between backticks ends where the closing backtick is,
/// so a cgroup name with spaces is one name:
/// two cgroups that differ only in their names,
/// spaces included,
/// are one cause.
#[test]
fn a_quoted_name_with_spaces_is_replaced_whole() {
    let first = normalized_cause("its cgroup `/user.slice/user-1000.slice/user@1000.service/app.slice/my app one.scope` is outside the subtree");
    let second = normalized_cause("its cgroup `/user.slice/user-1000.slice/user@1000.service/app.slice/another very different name 7` is outside the subtree");
    assert_eq!(first, second);
    assert_eq!(first, "its cgroup <name> is outside the subtree");
    assert_ne!(first, normalized_cause("its cgroup `/user.slice/x y` is inside the subtree"), "the reason after the name still counts");
}

/// The lines the producers really make:
/// the same reason for cgroups with different spaced names is one cause,
/// and a different reason is another.
#[test]
fn the_real_producer_lines_with_spaced_names_are_one_cause_per_reason() {
    let one = adoptable_cgroup("/system.slice/my odd name/x", 1000).expect_err("outside the subtree");
    let two = adoptable_cgroup("/system.slice/a completely different name 42/y z", 1000).expect_err("outside the subtree");
    assert_eq!(normalized_cause(&policy_failure(&one)), normalized_cause(&policy_failure(&two)));
    let service = adoptable_cgroup("/user.slice/user-1000.slice/user@1000.service/app.slice/some thing.service", 1000).expect_err("a service");
    let other_service = adoptable_cgroup("/user.slice/user-1000.slice/user@1000.service/app.slice/other thing.service", 1000).expect_err("a service");
    assert_eq!(normalized_cause(&policy_failure(&service)), normalized_cause(&policy_failure(&other_service)));
    assert_ne!(normalized_cause(&policy_failure(&one)), normalized_cause(&policy_failure(&service)));
}

/// A backtick inside a name is written as an escape,
/// so it cannot end the name early and leak the rest of the name into the cause.
#[test]
fn a_backtick_inside_a_name_cannot_end_the_name_early() {
    assert_eq!(quote_name("/a/b"), "`/a/b`");
    assert_eq!(quote_name("/a`b"), "`/a\\u{0060}b`");
    let first = normalized_cause(&format!("its cgroup {} is outside", quote_name("/x/a`b secret one")));
    let second = normalized_cause(&format!("its cgroup {} is outside", quote_name("/x/c`d other secret")));
    assert_eq!(first, second);
    assert_eq!(first, "its cgroup <name> is outside");
}

/// Backticks that do not quote a name are kept,
/// with the numbers made alike,
/// and a backtick without its partner is ordinary text,
/// read word by word.
#[test]
fn backticks_that_do_not_quote_a_name_are_kept() {
    assert_eq!(normalized_cause("look at `journalctl --unit=wg-1-2` now"), "look at `journalctl --unit=wg-#-#` now");
    assert_eq!(normalized_cause("a ` b 5"), "a ` b #");
    assert_eq!(normalized_cause("`/a b"), "`<name> b", "the rest of an unclosed backtick is read word by word");
    assert_eq!(normalized_cause("``"), "``");
}

/// The loud report for one refused install root.
fn refused_root_failure_for(root: &str, why: &str) -> String {
    let refused = RefusedRoot { root: std::path::PathBuf::from(root), why: why.to_string() };
    return refused_root_failures(&[refused]).remove(0);
}

/// The install roots in the loud reports are quoted too:
/// two refused roots whose names differ in spaces and further words are one cause,
/// with the link targets and the home inside the reason quoted by their producer.
#[test]
fn refused_install_roots_with_spaced_names_are_one_cause() {
    let one = refused_root_failure_for("/home/tester/my toolchain one", "it is a link; links are not followed (it points to `/home/tester/other dir`)");
    let two = refused_root_failure_for("/home/tester/a completely different root 9", "it is a link; links are not followed (it points to `/home/tester/yet another place`)");
    assert_eq!(normalized_cause(&one), normalized_cause(&two));
    assert!(one.contains("`/home/tester/my toolchain one`"), "{one}");
}
