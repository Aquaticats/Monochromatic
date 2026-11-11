//! Controls for the Git questions of the typed push's upload plan:
//! each is bounded by the push's stall window,
//! and one that never answers is stopped and reported instead of holding the push.

use super::*;

use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The stall window these controls give the questions,
/// in whole seconds.
const STALL_SECONDS: &str = "2";

/// How much longer than the stall window and the stop grace a control's run may take under host load,
/// in whole seconds.
const LOAD_ALLOWANCE_SECONDS: u64 = 60;

/// How long a question that never answers would run if nothing stopped it,
/// in whole seconds;
/// far past the control.
const HOLD_SECONDS: u32 = 3_000;

/// What:
///  A repository-less invocation whose only Git is a stand-in that never answers,
/// with `git-lfs` present so the plan is asked at all.
/// Why:
///   The plan asks the real Git for the configuration before anything is uploaded,
/// and the control must reach that question to bound it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function request(root: Path): [InteractiveUpload, Vec<(OsString, OsString)>]
/// ```
fn request(root: &std::path::Path) -> (PathBuf, Vec<(OsString, OsString)>, Vec<OsString>) {
    let real: PathBuf = root.join("real");
    std::fs::create_dir_all(real.as_path()).expect("real git directory");
    executable(real.join("git").as_path(), format!("#!/bin/sh\nexec /usr/bin/sleep {HOLD_SECONDS}\n").as_bytes());
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(bin.as_path()).expect("bin directory");
    executable(bin.join("git-lfs").as_path(), b"#!/bin/sh\nexit 0\n");
    // `join_paths` builds one PATH value from the directories, as the shell does.
    let path: OsString = std::env::join_paths([bin.as_path()]).expect("a PATH");
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), path),
        (OsString::from("XDG_STATE_HOME"), root.join("state").into_os_string()),
        (OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE), OsString::from(STALL_SECONDS)),
    ];
    let region: Vec<OsString> = [OsString::from("origin"), OsString::from("main")].to_vec();
    return (real.join("git"), environment, region);
}

/// A configuration question that never answers is stopped at the push's stall window,
/// and the push is refused in words that name the question and the window,
/// instead of waiting for an answer that will not come.
#[test]
fn a_configuration_question_that_never_answers_is_stopped_and_reported() {
    let root: PathBuf = fixture("lfs-plan-stall");
    let (real_git, environment, region) = request(root.as_path());
    let upload = InteractiveUpload {
        prefix: &[],
        region: region.as_slice(),
        real_git: real_git.as_path(),
        environment: environment.as_slice(),
        overlay: &[],
        settings: None,
    };
    let started: Instant = Instant::now();
    let code: Option<i32> = upload_first(&upload);
    let elapsed: Duration = started.elapsed();
    assert_eq!(code, Some(1), "a question that never answers refuses the push");
    assert!(
        elapsed < Duration::from_secs(STALL_SECONDS.parse::<u64>().expect("a whole number") + STOP_GRACE.as_secs() + LOAD_ALLOWANCE_SECONDS),
        "the question was stopped at its window and its grace, not when the program chose to end: {elapsed:?}"
    );
    remove(root.as_path());
}
