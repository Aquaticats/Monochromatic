//! What:
//! The rule at one tool name:
//! two copies of the shadow,
//! each of a different version,
//! each the first program the other one finds on `PATH`,
//! must end at once instead of replacing themselves with each other.
//! Why:
//! That pair is the loop the rule was written for (measured 2026-11-07:
//! about 105 starts per second under `strace`).
//! A copy is told apart by the identity text inside the program file,
//! so the version the copy carries changes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = copyOf(SHADOW, firstFolder);
//! const second = copyOfAnotherVersion(secondFolder);
//! assert(runBounded(grepOf(first, [first, second, real])).kind === 'finished');
//! ```

/// Imports the borrowed path type used to name the copied programs.
use std::path::Path;

/// Imports the owned path type one test keeps the shadow link in.
use std::path::PathBuf;

/// Imports the fixtures and the deadline this part of the rule runs on.
use super::{
    SHADOW_EXECUTABLE,
    command_for,
    copy_of_another_version,
    copy_program,
    finished,
    link_as,
    run_bounded,
    scratch,
    write_real_script,
};

#[test]
fn two_copies_of_different_versions_that_each_come_first_for_the_other_end_at_once_and_the_real_tool_runs()
 {
    let root = scratch();
    let (first, second, real) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("real"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");
    write_real_script(&real, "grep");

    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &second, &real],
        &["needle", "haystack"],
    )));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "REAL grep needle haystack\n"
    );
}

#[test]
fn two_copies_with_no_real_tool_behind_them_end_at_once_with_the_not_found_message() {
    let root = scratch();
    let (first, second) = (root.path().join("first"), root.path().join("second"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");

    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &second],
        &["needle", "haystack"],
    )));

    assert_eq!(
        output.status.code(),
        Some(127),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no real `grep`"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
