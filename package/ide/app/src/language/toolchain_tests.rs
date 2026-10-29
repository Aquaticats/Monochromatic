//! Resolution against a double that answers like the toolchain lookup,
//!  the notice text,
//!  and the retry rule.

use super::{
    ENTRY_POINT, Failed, Provided, RETRY_AFTER, Runner, TIME_LIMIT, ToolchainRequest,
    ToolchainState,
    command::{Completed, Invocation, RunFailure},
    due, entry_point, resolve_disabled, resolve_with,
};
use std::{cell::RefCell, fs, path::Path, path::PathBuf, time::Duration};

fn request(root: &Path) -> ToolchainRequest {
    return ToolchainRequest {
        project_root: root.to_path_buf(),
        project_spellings: Vec::new(),
        state_root: Some(PathBuf::from("/state")),
    };
}

fn done(stdout: &str, stderr: &str) -> Result<Completed, RunFailure> {
    return Ok(Completed {
        success: true,
        status: "exit status: 0".to_string(),
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    });
}

fn exited(stderr: &str) -> Result<Completed, RunFailure> {
    return Ok(Completed {
        success: false,
        status: "exit status: 1".to_string(),
        stdout: String::new(),
        stderr: stderr.to_string(),
    });
}

/// The double that answers like the lookup and records every run it was asked for.
struct Double {
    /// Every invocation with the limit it was given.
    seen: RefCell<Vec<(Invocation, Duration)>>,
    /// What it answers.
    answer: Result<Completed, RunFailure>,
}

impl Runner for Double {
    fn run(&self, invocation: &Invocation, limit: Duration) -> Result<Completed, RunFailure> {
        self.seen.borrow_mut().push((invocation.clone(), limit));
        return self.answer.clone();
    }
}

/// A disposable worktree:
///  `root` with the published entry point in it.
fn worktree(root: &Path) -> PathBuf {
    let script = root.join(ENTRY_POINT);
    fs::create_dir_all(script.parent().expect("parent")).expect("directory");
    fs::write(&script, "// fixture entry point\n").expect("entry point");
    return root.to_path_buf();
}

/// The pinned answer of `doc/planning/mpm-toolchain-migration.md`.
fn answer(dirs: &[&str], bins: &[(&str, &str)]) -> String {
    let listed: Vec<String> = dirs.iter().map(|dir| format!("{dir:?}")).collect();
    let links: Vec<String> = bins
        .iter()
        .map(|(name, path)| format!("{name:?}: {path:?}"))
        .collect();
    return format!(
        r#"{{"exposureDir": ".cache/toolchain-bin", "pathDirs": [{}], "bins": {{{}}}}}"#,
        listed.join(", "),
        links.join(", ")
    );
}

/// The path the lookup runs the entry point by:
///  its canonical path,
///  which is what the entry point compares its own to.
fn script_of(root: &Path) -> String {
    return std::fs::canonicalize(root.join(ENTRY_POINT))
        .expect("canonical entry point")
        .to_string_lossy()
        .into_owned();
}

/// Run resolution and return the state with every invocation and limit the double saw.
fn resolve(
    root: &Path,
    answer: Result<Completed, RunFailure>,
) -> (ToolchainState, Vec<(Invocation, Duration)>) {
    let runner = Double {
        seen: RefCell::new(Vec::new()),
        answer,
    };
    let state = resolve_with(&request(root), &runner);
    return (state, runner.seen.into_inner());
}

#[test]
fn a_project_in_a_toolchain_worktree_gets_the_directories_the_lookup_prints() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let tools = crate::test_scratch::scratch().expect("directory");
    let node = tools.path().join("node/bin");
    let taplo = tools.path().join("taplo");
    fs::create_dir_all(&node).expect("directory");
    fs::create_dir_all(&taplo).expect("directory");
    let (state, seen) = resolve(
        &root,
        done(
            &answer(
                &[
                    node.to_str().expect("Unicode"),
                    taplo.to_str().expect("Unicode"),
                ],
                &[("taplo", taplo.to_str().expect("Unicode"))],
            ),
            "a warning about the machine\n",
        ),
    );
    let ToolchainState::Provided(provided) = state else {
        panic!("not provided: {state:?}");
    };
    assert_eq!(provided.directories, [node, taplo]);
    assert_eq!(provided.warnings, "a warning about the machine");
    assert_eq!(seen.len(), 1);
    let (invocation, limit) = &seen[0];
    assert_eq!(invocation.arguments, [script_of(&root)]);
    assert_eq!(invocation.directory, root);
    assert!(
        *limit <= TIME_LIMIT && *limit > TIME_LIMIT - Duration::from_secs(2),
        "{limit:?}"
    );
}

#[test]
fn the_lookup_is_only_given_the_entry_point_path_and_never_a_tool_or_an_option() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let (state, seen) = resolve(&root, done(&answer(&[], &[]), ""));
    assert!(matches!(state, ToolchainState::Provided(_)), "{state:?}");
    let (invocation, _) = &seen[0];
    assert_eq!(
        invocation.arguments,
        [script_of(&root)],
        "the lookup must be asked only for the entry point"
    );
    assert!(invocation.arguments[0].starts_with('/'), "{invocation:?}");
}

#[test]
fn a_folder_below_the_worktree_root_is_served_by_the_worktrees_entry_point() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let folder = base.path().join("package/ide/app");
    let found = entry_point(&folder).expect("entry point");
    assert_eq!(found.worktree_root, root);
    assert_eq!(found.script, root.join(ENTRY_POINT));
    let (state, _) = resolve(&folder, done(&answer(&[], &[]), ""));
    assert!(matches!(state, ToolchainState::Provided(_)), "{state:?}");
}

#[test]
fn a_project_with_a_dashed_name_is_only_the_working_directory() {
    let base = crate::test_scratch::scratch().expect("directory");
    worktree(base.path());
    let folder = base.path().join("--cd=etc");
    let (state, seen) = resolve(&folder, done(&answer(&[], &[]), ""));
    assert!(matches!(state, ToolchainState::Provided(_)), "{state:?}");
    let (invocation, _) = &seen[0];
    assert_eq!(invocation.directory, folder);
    assert_eq!(invocation.arguments.len(), 1, "{invocation:?}");
    assert!(invocation.arguments[0].starts_with('/'), "{invocation:?}");
}

#[test]
fn a_project_without_the_entry_point_is_not_a_toolchain_project() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = base.path().to_path_buf();
    let (state, seen) = resolve(
        &root,
        exited("the lookup must not be run for a project without an entry point"),
    );
    assert!(matches!(state, ToolchainState::Unused(_)), "{state:?}");
    assert!(!state.serves_project());
    assert!(
        seen.is_empty(),
        "the lookup ran for a project it does not serve"
    );
}

#[test]
fn a_worktree_that_is_not_above_the_project_does_not_serve_it() {
    let base = crate::test_scratch::scratch().expect("directory");
    worktree(&base.path().join("one"));
    let other = base.path().join("two");
    assert!(entry_point(&other).is_none());
}

#[test]
fn an_unreadable_answer_fails_instead_of_guessing() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let (state, _) = resolve(&root, done("<html>", ""));
    let ToolchainState::Failed(not_json) = state else {
        panic!("not failed: {state:?}");
    };
    assert!(
        not_json.message.contains("not JSON"),
        "{}",
        not_json.message
    );
}

#[test]
fn an_answer_of_another_shape_fails_instead_of_guessing() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let (state, _) = resolve(&root, done(r#"{"exposureDir": "x", "bins": {}}"#, ""));
    let ToolchainState::Failed(no_dirs) = state else {
        panic!("not failed: {state:?}");
    };
    assert!(no_dirs.message.contains("pathDirs"), "{}", no_dirs.message);
}

#[test]
fn a_failed_run_quotes_the_lookup_and_drops_the_node_version_footer() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let stderr = "file:///work/project/package/file-enforcer/core/src/toolchain/bin-paths.ts:3
Error: toolchain.lock.json is unreadable
    at readLock (file:///work/project/package/file-enforcer/core/src/toolchain/bin-paths.ts:3:9)
Node.js v26.12.0
";
    let (state, _) = resolve(&root, exited(stderr));
    let ToolchainState::Failed(Failed { message }) = state else {
        panic!("not failed: {state:?}");
    };
    assert!(
        message.contains("toolchain.lock.json is unreadable"),
        "{message}"
    );
    assert!(!message.contains("Node.js v"), "{message}");
}

#[test]
fn a_failure_without_a_message_still_says_what_ended() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let (state, _) = resolve(&root, exited(""));
    let ToolchainState::Failed(failed) = state else {
        panic!("not failed: {state:?}");
    };
    assert!(
        failed
            .message
            .contains("`node package/file-enforcer/core/src/toolchain/bin-paths.ts` ended with exit status: 1"),
        "{}",
        failed.message
    );
}

#[test]
fn a_run_that_timed_out_or_could_not_start_fails_and_a_refused_sandbox_does_not() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    let timed_out = RunFailure::Failed("did not finish within 10 s and was stopped".to_string());
    let (state, _) = resolve(&root, Err(timed_out));
    let ToolchainState::Failed(failed) = state else {
        panic!("not failed: {state:?}");
    };
    assert_eq!(
        failed.message,
        "`node package/file-enforcer/core/src/toolchain/bin-paths.ts` did not finish within 10 s and was stopped"
    );
    let (refused, seen) = resolve(
        &root,
        Err(RunFailure::Refused(
            "bubblewrap is not installed".to_string(),
        )),
    );
    assert_eq!(
        refused,
        ToolchainState::Unused("bubblewrap is not installed".to_string())
    );
    assert_eq!(seen.len(), 1);
}

#[test]
fn the_disabled_policy_never_runs_the_lookup() {
    let base = crate::test_scratch::scratch().expect("directory");
    let root = worktree(base.path());
    assert_eq!(resolve_disabled(&request(&root)), ToolchainState::Disabled);
}

#[test]
fn a_failure_is_asked_again_after_the_retry_time_and_a_good_answer_only_for_a_missing_server() {
    let failed = ToolchainState::Failed(Failed {
        message: "x".to_string(),
    });
    let provided = ToolchainState::Provided(Provided {
        directories: Vec::new(),
        warnings: String::new(),
    });
    let young = RETRY_AFTER - Duration::from_millis(1);
    assert!(!due(&failed, young, true));
    assert!(due(&failed, RETRY_AFTER, false));
    assert!(!due(&provided, RETRY_AFTER * 10, false));
    assert!(!due(&provided, young, true));
    assert!(due(&provided, RETRY_AFTER, true));
    for idle in [
        ToolchainState::Disabled,
        ToolchainState::Unused("x".to_string()),
    ] {
        assert!(!due(&idle, RETRY_AFTER * 100, true));
    }
}
