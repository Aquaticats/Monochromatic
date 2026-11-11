//! Verifies which socket marks one reconciliation keeps:
//! a mark stays while the scan still selects its cgroup,
//! it comes off when the scan no longer does,
//! a cgroup whose application has gone is reported once,
//! and a cgroup that regains its application is reported again if it loses it later.

/// The decision under test.
use crate::application_marks::{decide_marks, reports_that_still_apply};
/// What one scan found.
use crate::application_targets::ApplicationScan;
/// Cgroup paths.
use std::path::PathBuf;

/// A cgroup directory of this test's scratch root,
/// which exists on disk,
/// so a rule that keeps a mark while the directory exists is testable beside one that reads the scan.
fn cgroup(name: &str) -> PathBuf {
    let directory = crate::test_scratch::scratch_root()
        .join(format!("wg-quicker-marks-{}", std::process::id()))
        .join("user.slice/user-1000.slice/user@1000.service/app.slice")
        .join(name);
    std::fs::create_dir_all(&directory).expect("make the cgroup directory");
    return directory;
}

/// A scan that selected `targets` and found `without_application` claiming a name alone.
fn scan(targets: &[PathBuf], without_application: &[PathBuf]) -> ApplicationScan {
    return ApplicationScan {
        targets: targets.to_vec(),
        without_application: without_application.to_vec(),
    };
}

/// A mark on a name-selected cgroup whose application has gone comes off.
///
/// The cgroup directory still exists on disk,
/// so a rule that kept the mark while the directory existed
/// left the application's exemption attached after the application exited,
/// and every process that later ran in that cgroup left the tunnel.
#[test]
fn a_mark_comes_off_when_the_scan_no_longer_selects_its_cgroup() {
    let firefox = cgroup("app-firefox@fixture.service");
    let decision = decide_marks(std::slice::from_ref(&firefox), &scan(&[], std::slice::from_ref(&firefox)), &[]);
    assert_eq!(decision.keep, Vec::<PathBuf>::new(), "the mark stayed on a cgroup with no application in it");
    assert_eq!(decision.release, vec![firefox.clone()]);
    assert_eq!(decision.report, vec![firefox], "the person is told the mark is not attached");
}

/// A mark on a cgroup the scan still selects stays,
/// and is not reported as a name without its program.
#[test]
fn a_mark_stays_while_the_scan_selects_its_cgroup() {
    let firefox = cgroup("app-firefox@fixture.service");
    let decision = decide_marks(std::slice::from_ref(&firefox), &scan(std::slice::from_ref(&firefox), &[]), &[]);
    assert_eq!(decision.keep, vec![firefox]);
    assert_eq!(decision.release, Vec::<PathBuf>::new());
    assert_eq!(decision.report, Vec::<PathBuf>::new());
}

/// A cgroup reported once is not reported again on the next scan,
/// and a cgroup that regains its application is dropped from the reports that apply,
/// so losing it again is said again instead of staying silent.
#[test]
fn a_cgroup_is_reported_once_and_reported_again_after_it_regains_its_application() {
    let firefox = cgroup("app-firefox@fixture.service");
    let found = scan(&[], std::slice::from_ref(&firefox));
    assert_eq!(decide_marks(&[], &found, &[]).report, vec![firefox.clone()]);
    assert_eq!(decide_marks(&[], &found, std::slice::from_ref(&firefox)).report, Vec::<PathBuf>::new(), "the same line is repeated for ever");
    let selected_again = scan(std::slice::from_ref(&firefox), &[]);
    assert_eq!(reports_that_still_apply(std::slice::from_ref(&firefox), &selected_again), Vec::<PathBuf>::new());
    assert_eq!(reports_that_still_apply(std::slice::from_ref(&firefox), &found), vec![firefox]);
}

/// A mark on a cgroup the scan never mentions comes off too,
/// which is what an executable-owned cgroup does when its program exits and the cgroup is removed.
#[test]
fn a_mark_on_a_cgroup_the_scan_does_not_mention_comes_off() {
    let gone = cgroup("app-org.chromium.Chromium-42.scope");
    let decision = decide_marks(std::slice::from_ref(&gone), &scan(&[], &[]), &[]);
    assert_eq!(decision.release, vec![gone]);
    assert_eq!(decision.report, Vec::<PathBuf>::new(), "a cgroup the scan did not mention claims no application name");
}
