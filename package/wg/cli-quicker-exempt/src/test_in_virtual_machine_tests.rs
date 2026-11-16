//! Whether the marker decides the two answers,
//! and whether the refusal names the task that runs the test in a machine.

/// The reading under test.
use crate::test_in_virtual_machine::require_marker;
/// Fixture paths.
use crate::test_scratch::scratch_root;

/// A marker path inside the test's own scratch directory.
fn marker(name: &str) -> std::path::PathBuf {
    return scratch_root().join(format!("wg-quicker-marker-{name}-{}", std::process::id()));
}

/// A test without the marker is refused,
/// and the refusal names the mise task.
#[test]
fn a_missing_marker_is_refused_with_the_task_that_provides_one() {
    let path = marker("absent");
    let _removed = std::fs::remove_dir_all(&path);
    let refusal = require_marker(&path, "bypass", "//package/wg/cli-quicker:test:integration:bypass").expect_err("a missing marker refuses");
    assert!(refusal.contains("bypass"), "{refusal}");
    assert!(refusal.contains("mise run //package/wg/cli-quicker:test:integration:bypass"), "{refusal}");
    assert!(refusal.contains(&path.display().to_string()), "{refusal}");
}

/// A test with the marker goes on.
#[test]
fn a_present_marker_lets_the_test_go_on() {
    let path = marker("present");
    std::fs::create_dir_all(&path).expect("make the marker");
    assert_eq!(require_marker(&path, "bypass", "//package/wg/cli-quicker:test:integration:bypass"), Ok(()));
    std::fs::remove_dir(&path).expect("remove the marker");
}
