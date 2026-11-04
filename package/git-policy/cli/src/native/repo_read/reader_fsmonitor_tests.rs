//! What:
//!  The Git processes the layout questions of a commit start for a user whose own configuration file holds
//!  `core.fsmonitor = true`,
//! counted by a recording Git,
//! against the same questions for a user with no such setting.
//! Why:
//!  The in-process layout reader declined every repository of such a user,
//! so each question cost a Git process that a user
//!  without the setting did not pay.
//! A control that sees the processes,
//! with the same answers,
//! shows the cost is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(startsFor({ core: { fsmonitor: true } })).toEqual(startsFor({}));
//! ```

use crate::capture_invocation::{InvocationLayout, capture_invocation_layout};
use crate::landing_location::resolve_landing_location;
use crate::recording_git::{recorded_starts, recording_context};
use crate::repo_read_layout::{read_layout_after_success, read_layout_after_success_for};
use crate::service_route::worktree_git_dir;
use crate::test_support::{fixture, isolated_overlay, remove, repository};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The user configuration that turns the built-in monitor daemon on.
const MONITOR_ON: &str = "[core]\n\tfsmonitor = true\n";

/// A context whose Git is the recording one and `settings` the user's own file;
/// the in-process readers are on unless
///  `readers` is false.
fn context_for(root: &Path, settings: &str, readers: bool) -> (GitContext, PathBuf) {
    let (mut context, log): (GitContext, PathBuf) = recording_context(root);
    let file: PathBuf = root.join("user.gitconfig");
    std::fs::write(&file, settings).expect("user configuration");
    if readers {
        context.overlay = isolated_overlay().into_iter().filter(|(name, _)| { return name != "GIT_CONFIG_GLOBAL"; }).collect();
        context.overlay.push((OsString::from("GIT_CONFIG_GLOBAL"), file.into_os_string()));
    }
    return (context, log);
}

/// What the three questions of a commit's start answer and the Git starts they made,
/// for a user whose file holds `settings`.
fn asked(name: &str, settings: &str, readers: bool) -> (Vec<String>, Vec<String>) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (context, log): (GitContext, PathBuf) = context_for(root.as_path(), settings, readers);
    let routed: Option<PathBuf> = worktree_git_dir(&context, repo.as_path());
    let landing = resolve_landing_location(&context, &[OsString::from("-C"), repo.clone().into_os_string()]).expect("landing location").expect("inside a repository");
    let captured: InvocationLayout = capture_invocation_layout(&context, repo.as_path(), false).expect("capture");
    let answers: Vec<String> = vec![
        format!("{routed:?}"),
        format!("{landing:?}"),
        format!("{:?} {:?} {:?} {:?}", captured.repository_root, captured.git_dir, captured.common_dir, captured.real_index_path),
    ]
    .into_iter()
    .map(|text| { return text.replace(root.to_string_lossy().as_ref(), "<root>"); })
    .collect();
    let starts: Vec<String> = recorded_starts(log.as_path()).into_iter().map(|text| { return text.replace(root.to_string_lossy().as_ref(), "<root>"); }).collect();
    remove(root.as_path());
    return (answers, starts);
}

/// A user-level `core.fsmonitor = true` costs the layout questions no Git process over a user without it,
/// with the same
/// answers.
/// Before,
/// the reader declined for it and each of the three questions started a `rev-parse`.
#[test]
fn a_user_level_fsmonitor_true_costs_the_layout_questions_no_git_process() {
    let (plain_answers, plain_starts): (Vec<String>, Vec<String>) = asked("reader-monitor-plain", "[user]\n\tname = Someone\n", true);
    let (monitor_answers, monitor_starts): (Vec<String>, Vec<String>) = asked("reader-monitor-on", MONITOR_ON, true);
    assert_eq!(monitor_answers, plain_answers);
    assert_eq!(monitor_starts, plain_starts, "the layout reports are read from the files for both users");
    assert!(monitor_starts.is_empty(), "no question needed a Git process: {monitor_starts:?}");
    // The control:
    // with the readers off,
    // the same questions ask Git,
    // and the recorder sees the layout report.
    let (_, asked_of_git): (Vec<String>, Vec<String>) = asked("reader-monitor-off", MONITOR_ON, false);
    assert!(asked_of_git.iter().any(|start| { return start.contains("--show-toplevel"); }), "{asked_of_git:?}");
    assert!(asked_of_git.iter().any(|start| { return start.contains("--absolute-git-dir"); }), "{asked_of_git:?}");
}

/// The reader for a caller that follows a successful Git run takes the same `-C` options as the other reader,
/// and answers for a
/// repository whose configuration holds a key the typed proof refuses.
#[test]
fn the_after_success_reader_for_a_prefix_follows_the_dash_c_options() {
    let root: PathBuf = fixture("reader-after-success-prefix");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (context, _log): (GitContext, PathBuf) = context_for(root.as_path(), "[diff]\n\tdirstat = changes\n", true);
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.clone().into_os_string()];
    let by_prefix = read_layout_after_success_for(prefix.as_slice(), context.overlay.as_slice()).expect("answers through -C");
    let by_directory = read_layout_after_success(repo.as_path(), context.overlay.as_slice()).expect("answers in the directory");
    assert_eq!(by_prefix.git_dir, by_directory.git_dir);
    assert_eq!(by_prefix.worktree_root, by_directory.worktree_root);
    assert!(read_layout_after_success_for(&[OsString::from("--git-dir=x")], context.overlay.as_slice()).is_none(), "another global option is left to Git");
    remove(root.as_path());
}
