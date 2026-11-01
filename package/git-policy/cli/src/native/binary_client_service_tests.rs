//! What:
//!  Client-side behavior of a commit that lands through the resident service, seen from the person's terminal.
//! Why:
//!  The wrapper process prints what the person reads (warnings, the summary line, the wait notices), while the service
//!  lands the commit; each control starts a service of its own at a throwaway socket and runs the built wrapper against it.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::{Running, runtime_for, start};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A repository with `a.txt` committed and edited in the worktree, ready for one commit.
fn edited_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    return repo;
}

/// The wrapped command with the runtime directory of the service named `name`.
fn served(fixture: &Fixture, repo: &Path, name: &str, arguments: &[&str]) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, arguments);
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    return command;
}

/// The number of batches the service of `fixture` finished.
fn finished_batches(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .count();
}

/// A commit that lands through the service in a repository that configures an external hook prints the same
///  `external-hooks-unsupported` warning the transaction route prints, and the hook never runs.
#[test]
fn a_service_commit_warns_that_configured_hooks_never_run() {
    let fixture: Fixture = fixture("client-hooks-warning");
    let repo: PathBuf = edited_repository(&fixture);
    let marker: PathBuf = fixture.root.join("marker");
    let script: PathBuf = fixture.root.join("hook-program");
    super::support::executable(script.as_path(), format!("#!/bin/sh\necho ran >> '{}'\nexit 9\n", marker.display()).as_bytes());
    git(&fixture, repo.as_path(), &["config", "hook.on-commit.event", "pre-commit"]);
    git(&fixture, repo.as_path(), &["config", "hook.on-commit.command", script.to_str().expect("UTF-8")]);
    let name: String = format!("client-hooks-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let output: Output = served(&fixture, &repo, name.as_str(), &["commit", "--quiet", "-m", "second", "--", "a.txt"]).output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    assert_eq!(finished_batches(&fixture), 1, "the service landed it: {stderr}");
    assert!(!marker.exists(), "the hook ran");
    assert!(stderr.contains("external-hooks-unsupported") && stderr.contains("hook.on-commit"), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "second");
    remove_settled(&fixture);
}

/// The summary lines on standard output, each with the name it carries: the first word that ends the bracket.
fn summaries(output: &Output) -> Vec<(String, String)> {
    return String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| { return line.starts_with('['); })
        .map(|line| {
            let name: &str = line.split_whitespace().find(|word| { return word.ends_with(']'); }).unwrap_or("").trim_end_matches(']');
            return (line.to_string(), name.to_string());
        })
        .collect();
}

/// The name Git itself gives `HEAD`.
fn git_short_head(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["rev-parse", "--short", "HEAD"]).trim().to_string();
}

/// A commit the service builds from the command line prints the name Git abbreviates to: the length `core.abbrev` asks for,
///  and the automatic length when it is unset.
#[test]
fn the_summary_line_of_a_service_commit_has_the_name_length_git_gives() {
    for (label, abbrev) in [("unset", None), ("twelve", Some("12")), ("whole", Some("no"))] {
        let fixture: Fixture = fixture(format!("client-abbrev-{label}").as_str());
        let repo: PathBuf = edited_repository(&fixture);
        if let Some(value) = abbrev {
            git(&fixture, repo.as_path(), &["config", "core.abbrev", value]);
        }
        let name: String = format!("client-abbrev-{label}-{}", std::process::id());
        let _service: Running = start(&fixture, name.as_str());
        let output: Output = served(&fixture, &repo, name.as_str(), &["commit", "-m", "second", "--", "a.txt"]).output().expect("the wrapped commit runs");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(finished_batches(&fixture), 1, "the service landed it");
        let lines: Vec<(String, String)> = summaries(&output);
        assert_eq!(lines.len(), 1, "{label}: {:?}", String::from_utf8_lossy(&output.stdout));
        assert_eq!(lines[0].1, git_short_head(&fixture, &repo), "{label}: {}", lines[0].0);
        assert!(lines[0].0.starts_with("[main "), "{label}: {}", lines[0].0);
        remove_settled(&fixture);
    }
}

/// A commit that Git builds (an option the service does not read) prints one summary line with the same name, and `--quiet`
///  prints none, whoever builds the commit.
#[test]
fn a_commit_git_builds_prints_one_summary_line_and_quiet_prints_none() {
    let fixture: Fixture = fixture("client-built-summary");
    let repo: PathBuf = edited_repository(&fixture);
    git(&fixture, repo.as_path(), &["config", "core.abbrev", "11"]);
    let name: String = format!("client-built-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let built: Output = served(&fixture, &repo, name.as_str(), &["commit", "--cleanup=scissors", "-m", "built by git", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    assert_eq!(finished_batches(&fixture), 1, "the service landed it");
    let lines: Vec<(String, String)> = summaries(&built);
    assert_eq!(lines.len(), 1, "{:?}", String::from_utf8_lossy(&built.stdout));
    assert_eq!(lines[0].1, git_short_head(&fixture, &repo));
    assert!(lines[0].0.ends_with("built by git"), "{}", lines[0].0);
    for options in [&["--quiet"][..], &["-q"], &["-qn"]] {
        std::fs::write(repo.join("a.txt"), format!("quiet {options:?}\n")).expect("edit");
        let mut arguments: Vec<&str> = vec!["commit"];
        arguments.extend_from_slice(options);
        arguments.extend_from_slice(&["--cleanup=scissors", "-m", "silent", "--", "a.txt"]);
        let quiet: Output = served(&fixture, &repo, name.as_str(), arguments.as_slice()).output().expect("the wrapped commit runs");
        assert!(quiet.status.success(), "{options:?}: {}", String::from_utf8_lossy(&quiet.stderr));
        assert_eq!(String::from_utf8_lossy(&quiet.stdout), "", "{options:?}");
    }
    remove_settled(&fixture);
}

/// The first commit of a branch is worded `(root-commit)` and a detached head `detached HEAD`, as Git words them.
#[test]
fn a_root_commit_and_a_detached_head_are_worded_as_git_words_them() {
    let fixture: Fixture = fixture("client-summary-states");
    let unborn: PathBuf = fixture.root.join("unborn");
    std::fs::create_dir(&unborn).expect("directory");
    git(&fixture, unborn.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(unborn.join("a.txt"), b"first\n").expect("file");
    git(&fixture, unborn.as_path(), &["add", "a.txt"]);
    git(&fixture, unborn.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let name: String = format!("client-states-{}", std::process::id());
    let _service: Running = start(&fixture, name.as_str());
    let first: Output = served(&fixture, &unborn, name.as_str(), &["commit", "-m", "the root", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let lines: Vec<(String, String)> = summaries(&first);
    assert_eq!(lines.len(), 1, "{:?}", String::from_utf8_lossy(&first.stdout));
    assert!(lines[0].0.starts_with("[main (root-commit) "), "{}", lines[0].0);
    assert_eq!(lines[0].1, git_short_head(&fixture, &unborn));
    git(&fixture, unborn.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(unborn.join("a.txt"), b"first\nsecond\n").expect("edit");
    let detached: Output = served(&fixture, &unborn, name.as_str(), &["commit", "-m", "adrift", "--", "a.txt"]).output().expect("the wrapped commit runs");
    assert!(detached.status.success(), "{}", String::from_utf8_lossy(&detached.stderr));
    let lines: Vec<(String, String)> = summaries(&detached);
    assert_eq!(lines.len(), 1, "{:?}", String::from_utf8_lossy(&detached.stdout));
    assert!(lines[0].0.starts_with("[detached HEAD "), "{}", lines[0].0);
    remove_settled(&fixture);
}
