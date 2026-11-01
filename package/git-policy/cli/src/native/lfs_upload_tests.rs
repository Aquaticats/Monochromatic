//! Controls for what the Git LFS upload sends and for the parent's run of it.

use super::*;
use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// A planner whose Git answers come from `answers` (a query's first word, then its answer) and which counts its queries.
fn plan_with(
    global_prefix: &[&str],
    region: &[&str],
    environment: &[(OsString, OsString)],
    answers: &[(&str, &str)],
) -> (Option<UploadPlan>, usize) {
    let mut asked: usize = 0;
    let mut query = |arguments: &[&str]| -> Option<String> {
        asked += 1;
        return answers
            .iter()
            .find(|(first, _)| {
                return *first == arguments[0];
            })
            .map(|(_, answer)| {
                return (*answer).to_owned();
            });
    };
    let plan = plan_upload(words(global_prefix).as_slice(), words(region).as_slice(), environment, &mut query);
    return (plan, asked);
}

/// A push that names its repository and refs needs no Git query, and uploads exactly those refs to that remote.
#[test]
fn a_push_that_names_its_remote_and_refs_uploads_those_refs_without_asking_git() {
    let (planned, asked) = plan_with(&[], &["origin", "main", "topic:refs/heads/other"], &[], &[]);
    let plan: UploadPlan = planned.expect("a plan");
    assert_eq!(asked, 0);
    assert_eq!(plan.remote, OsString::from("origin"));
    assert_eq!(plan.refs, words(&["main", "topic"]));
    assert!(!plan.everything);
    assert_eq!(plan.arguments(), words(&["push", "origin", "main", "topic"]));
}

/// A forced refspec uploads its source, a deletion uploads nothing, and `tag <name>` is the tag's ref.
#[test]
fn forced_deleting_and_tag_refspecs_read_as_git_reads_them() {
    let (forced, _) = plan_with(&[], &["origin", "+HEAD:refs/heads/x"], &[], &[]);
    assert_eq!(forced.expect("a plan").refs, words(&["HEAD"]));
    let (deleting, _) = plan_with(&[], &["origin", ":gone"], &[], &[("symbolic-ref", "work")]);
    assert_eq!(deleting, None, "a refspec that only deletes sends no objects");
    let (deleted, _) = plan_with(&[], &["--delete", "origin", "gone"], &[], &[]);
    assert_eq!(deleted, None);
    let (tagged, _) = plan_with(&[], &["origin", "tag", "v1"], &[], &[]);
    assert_eq!(tagged.expect("a plan").refs, words(&["refs/tags/v1"]));
}

/// Options and refspecs that may send any ref upload everything, with `--all` and no refs.
#[test]
fn a_push_that_may_send_any_ref_uploads_everything() {
    for region in [
        vec!["--all", "origin"],
        vec!["--tags", "origin"],
        vec!["--mirror", "origin"],
        vec!["origin", "refs/heads/*:refs/heads/*"],
    ] {
        let (planned, _) = plan_with(&[], region.as_slice(), &[], &[]);
        let plan: UploadPlan = planned.unwrap_or_else(|| panic!("a plan for {region:?}"));
        assert!(plan.everything, "{region:?}");
        assert_eq!(plan.arguments(), words(&["push", "--all", "origin"]), "{region:?}");
    }
}

/// A bare push takes the remote and the branch from Git: the push remote, then the default push remote, then the
///  branch's remote, then `origin`; a branch without a name is `HEAD`; `push.default=matching` uploads everything.
#[test]
fn a_bare_push_resolves_its_remote_and_branch_the_way_git_does() {
    let config = |lines: &str| -> Vec<(&str, String)> {
        return vec![("symbolic-ref", String::from("work")), ("config", lines.to_owned())];
    };
    let resolve = |lines: &str| -> Option<UploadPlan> {
        let answers = config(lines);
        let borrowed: Vec<(&str, &str)> = answers
            .iter()
            .map(|(first, answer)| {
                return (*first, answer.as_str());
            })
            .collect();
        return plan_with(&[], &[], &[], borrowed.as_slice()).0;
    };
    let all: &str = "branch.work.remote upstream\nremote.pushdefault shared\nbranch.work.pushremote mine";
    assert_eq!(resolve(all).expect("a plan").remote, OsString::from("mine"));
    let without_push_remote: &str = "branch.work.remote upstream\nremote.pushdefault shared";
    assert_eq!(resolve(without_push_remote).expect("a plan").remote, OsString::from("shared"));
    assert_eq!(resolve("branch.work.remote upstream").expect("a plan").remote, OsString::from("upstream"));
    assert_eq!(resolve("branch.other.remote elsewhere").expect("a plan").remote, OsString::from("origin"));
    let bare: UploadPlan = resolve("").expect("a plan");
    assert_eq!(bare.refs, words(&["work"]));
    let detached = plan_with(&[], &[], &[], &[("config", "")]).0.expect("a plan");
    assert_eq!(detached.refs, words(&["HEAD"]));
    let matching: UploadPlan = resolve("push.default matching").expect("a plan");
    assert!(matching.everything);
}

/// A push with a remote but no refs takes the branch Git would send, and `--repo` names the remote like a positional.
#[test]
fn a_push_with_a_remote_only_or_a_repo_option_still_resolves_its_branch() {
    let (named_remote, _) = plan_with(&[], &["origin"], &[], &[("symbolic-ref", "work")]);
    let remote_only: UploadPlan = named_remote.expect("a plan");
    assert_eq!((remote_only.remote, remote_only.refs), (OsString::from("origin"), words(&["work"])));
    let (named_by_option, _) = plan_with(&[], &["--repo=elsewhere", "main"], &[], &[]);
    let by_option: UploadPlan = named_by_option.expect("a plan");
    assert_eq!((by_option.remote, by_option.refs), (OsString::from("elsewhere"), words(&["main"])));
}

/// Every way of bypassing the upload plans nothing.
#[test]
fn bypasses_and_dry_runs_plan_no_upload() {
    for region in [vec!["--no-verify", "origin", "main"], vec!["--no-ver", "origin", "main"], vec!["--dry-run", "origin", "main"]] {
        assert_eq!(plan_with(&[], region.as_slice(), &[], &[]).0, None, "{region:?}");
    }
    let off: Vec<&str> = vec!["-c", "hook.cli-git-lfs-pre-push.enabled=false"];
    assert_eq!(plan_with(off.as_slice(), &["origin", "main"], &[], &[]).0, None);
    let skip: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_LFS_SKIP_PUSH"), OsString::from("1"))];
    assert_eq!(plan_with(&[], &["origin", "main"], skip.as_slice(), &[]).0, None);
    assert_eq!(plan_with(&[], &["--no-such-option"], &[], &[]).0, None, "a region Git refuses is left to Git");
}

/// A stand-in `git-lfs` in a fixture directory that records its arguments and the directory it ran in, then exits.
fn stand_in(root: &std::path::Path, status: i32) -> PathBuf {
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir(&bin).expect("bin directory");
    let source: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{root}/arguments.txt'\npwd > '{root}/directory.txt'\n\
         command -v git > '{root}/git-seen.txt'\nexit {status}\n",
        root = root.display()
    );
    executable(bin.join("git-lfs").as_path(), source.as_bytes());
    return bin;
}

/// Run the interactive upload against a stand-in with the given status, returning its result and the diagnostics.
fn run_interactive(root: &std::path::Path, bin: &std::path::Path) -> (Option<i32>, String) {
    let real_git_directory: PathBuf = root.join("real");
    std::fs::create_dir(&real_git_directory).expect("real git directory");
    executable(real_git_directory.join("git").as_path(), b"#!/bin/sh\n");
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), bin.as_os_str().to_owned())];
    let plan: UploadPlan = UploadPlan { remote: OsString::from("origin"), refs: words(&["main"]), everything: false };
    let mut diagnostics: Vec<u8> = Vec::new();
    let code = run_upload_interactive(
        &plan,
        root,
        real_git_directory.join("git").as_path(),
        environment.as_slice(),
        &[],
        &mut diagnostics,
    );
    return (code, String::from_utf8_lossy(diagnostics.as_slice()).into_owned());
}

/// A succeeding upload is silent, gets the plan's arguments in the chosen directory, and sees the real Git first.
#[test]
fn a_succeeding_upload_is_silent_and_sees_the_real_git_first() {
    let root = fixture("lfs-upload-ok");
    let bin: PathBuf = stand_in(root.as_path(), 0);
    let (code, diagnostics) = run_interactive(root.as_path(), bin.as_path());
    assert_eq!(code, None);
    assert_eq!(diagnostics, "");
    assert_eq!(std::fs::read_to_string(root.join("arguments.txt")).expect("arguments"), "push\norigin\nmain\n");
    assert_eq!(std::fs::read_to_string(root.join("directory.txt")).expect("directory").trim(), root.to_string_lossy());
    assert_eq!(
        std::fs::read_to_string(root.join("git-seen.txt")).expect("git seen").trim(),
        root.join("real/git").to_string_lossy()
    );
    remove(root.as_path());
}

/// A failing upload stops with its own status and the explanation; a missing program stops with status 1.
#[test]
fn a_failing_or_missing_upload_stops_with_the_explanation() {
    let root = fixture("lfs-upload-fail");
    let failing_status: i32 = 7;
    let bin: PathBuf = stand_in(root.as_path(), failing_status);
    let (code, diagnostics) = run_interactive(root.as_path(), bin.as_path());
    assert_eq!(code, Some(failing_status));
    assert!(diagnostics.contains(&failed_message(&format!("exited with status {failing_status}"))), "{diagnostics}");
    let empty: PathBuf = root.join("empty");
    std::fs::create_dir(&empty).expect("empty directory");
    let (missing_code, missing) = run_interactive_in(root.as_path(), empty.as_path());
    assert_eq!(missing_code, Some(1));
    assert_eq!(missing, missing_program_message());
    remove(root.as_path());
}

/// Same as `run_interactive`, for a directory where the real Git directory already exists.
fn run_interactive_in(root: &std::path::Path, bin: &std::path::Path) -> (Option<i32>, String) {
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), bin.as_os_str().to_owned())];
    let plan: UploadPlan = UploadPlan { remote: OsString::from("origin"), refs: words(&["main"]), everything: false };
    let mut diagnostics: Vec<u8> = Vec::new();
    let code = run_upload_interactive(
        &plan,
        root,
        root.join("real/git").as_path(),
        environment.as_slice(),
        &[],
        &mut diagnostics,
    );
    return (code, String::from_utf8_lossy(diagnostics.as_slice()).into_owned());
}

/// An upload ended by a signal says so by number.
#[test]
fn an_upload_ended_by_a_signal_names_the_signal() {
    let root = fixture("lfs-upload-signal");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir(&bin).expect("bin directory");
    executable(bin.join("git-lfs").as_path(), b"#!/bin/sh\nkill -TERM $$\nsleep 5\n");
    let (code, diagnostics) = run_interactive(root.as_path(), bin.as_path());
    assert!(code.is_some_and(|status| {
        return status != 0;
    }));
    assert!(diagnostics.contains(&format!("was ended by signal {}", libc::SIGTERM)), "{diagnostics}");
    remove(root.as_path());
}
