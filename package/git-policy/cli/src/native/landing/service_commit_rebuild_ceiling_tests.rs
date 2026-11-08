//! Controls for the ceiling on rebuilding a commit whose branch moved.

use super::*;

/// A commit that moved under its build is built again while the time since its first build is under the ceiling,
/// however many builds that took,
/// and not once it is reached.
#[test]
fn a_commit_is_built_again_while_the_time_since_its_first_build_is_under_the_ceiling() {
    let ceiling: std::time::Duration = std::time::Duration::from_secs(1200);
    assert!(rebuild_may_continue(std::time::Duration::ZERO, ceiling));
    assert!(rebuild_may_continue(ceiling - std::time::Duration::from_millis(1), ceiling));
    assert!(!rebuild_may_continue(ceiling, ceiling));
    assert!(!rebuild_may_continue(ceiling + std::time::Duration::from_secs(1), ceiling));
}
