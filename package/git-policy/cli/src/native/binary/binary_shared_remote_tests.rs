//! What:
//!  Every command that changes the remote settings a repository's worktrees share,
//!  through the built
//!       executable:
//!  rejected in a linked worktree,
//!  allowed in the main checkout,
//!  and allowed for the
//!       forms that read or that write a file of their own.
//! Why:
//!  A worktree shares one `.git/config` with every other worktree of the repository,
//!  so a `remote set-url` run
//!       there once broke every push of the main checkout with no record of which command did it;
//!  the owner chose
//!       "Refuse in worktrees" on 2026-11-10 (`doc/handover/questions-for-the-owner.md`,
//!  "A push URL changed
//!       from a worktree").
//!  A linked worktree is made both inside and outside the repository folder,
//!  because the
//!       refusal depends on which worktree the Git directory belongs to,
//!  never on where the directory sits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runWrapped(linked, ['remote', 'set-url', 'origin', url]).status === 1
//! ```

use super::support::{Fixture, Observed, direct, executable, fixture, git, observe, remove, repository, run_direct, run_wrapped, stderr_of, wrapped};
use git_policy_cli::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The remote URL every fixture repository starts with.
const ORIGIN_URL: &str = "https://example.invalid/origin.git";

/// The URL a refused or allowed form tries to store.
const OTHER_URL: &str = "https://example.invalid/elsewhere.git";

/// The section an editor session adds to the file it was given,
///  so a run proves the editor reached that file.
const EDITOR_SECTION: &str = "[editor]\n\tran = true";

/// The words a linked-worktree refusal prints,
/// so a caller can tell it from Git's own failure.
const SHARED_REMOTE_CODE: &str = "\"code\":\"linked-worktree-only/shared-remote-settings\"";

/// The whole shape of the refusal sentence a person reads:
/// the shared file,
/// the main checkout,
/// and the copy that keeps its own settings.
const SHARED_SENTENCE: &str = "every worktree of this repository shares one Git configuration";

/// Every form that changes a remote setting,
/// as a person would write it.
const REFUSED_FORMS: &[&[&str]] = &[
    &["remote", "set-url", "origin", OTHER_URL],
    &["remote", "set-url", "--push", "origin", OTHER_URL],
    &["remote", "set-url", "--add", "origin", OTHER_URL],
    &["remote", "add", "beta", OTHER_URL],
    &["remote", "remove", "origin"],
    &["remote", "rm", "origin"],
    &["remote", "rename", "origin", "origin-renamed"],
    &["remote", "set-branches", "origin", "main"],
    &["remote", "set-head", "origin", "main"],
    // `-v`/`--verbose` and its abbreviations are the only options `git remote` reads before a subcommand.
    &["remote", "-v", "set-url", "origin", OTHER_URL],
    &["remote", "--verbose", "add", "beta", OTHER_URL],
    &["remote", "--verb", "add", "beta", OTHER_URL],
    // A config write names its key as the first token Git reads as an argument,
    // so every option after it is a value, the scope flags included.
    &["config", "remote.origin.url", OTHER_URL],
    &["config", "Remote.Origin.URL", OTHER_URL],
    &["config", "remote.origin.url", "--worktree"],
    &["config", "--add", "remote.origin.pushurl", OTHER_URL],
    &["config", "--unset", "remote.origin.url"],
    &["config", "--unset-all", "remote.origin.url"],
    &["config", "--replace-all", "remote.origin.url", OTHER_URL],
    &["config", "--remove-section", "remote.origin"],
    &["config", "--remove-section", "Remote.Origin"],
    &["config", "--rename-section", "remote.origin", "user.origin"],
    &["config", "--rename-section", "user.kept", "remote.created"],
    &["config", "set", "remote.origin.url", OTHER_URL],
    &["config", "unset", "remote.origin.url"],
    &["config", "remove-section", "remote.origin"],
    &["config", "rename-section", "remote.origin", "user.origin"],
];

/// Every form that reads the remote settings or writes somewhere else,
/// and must still run in a linked worktree.
const ALLOWED_FORMS: &[&[&str]] = &[
    &["remote"],
    &["remote", "-v"],
    &["remote", "get-url", "origin"],
    &["remote", "get-url", "--push", "origin"],
    &["config", "--get", "remote.origin.url"],
    &["config", "get", "remote.origin.url"],
    &["config", "list"],
    &["config", "--list"],
    &["config", "--get-regexp", "remote\\..*"],
    // One positional is Git's reading form.
    &["config", "remote.origin.url"],
    // `-c` is this one command's override and changes no file.
    &[
        "-c",
        "remote.origin.url=https://example.invalid/override.git",
        "config",
        "--get",
        "remote.origin.url",
    ],
    &[
        "-c",
        "remote.origin.url=https://example.invalid/override.git",
        "remote",
        "get-url",
        "origin",
    ],
];

/// The shared configuration file of the repository at `repo`.
fn config_file(repo: &Path) -> PathBuf {
    return repo.join(".git").join("config");
}

/// The shared configuration file byte for byte.
fn config_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(config_file(repo)).expect("shared configuration file");
}

/// A repository with `origin` and a file in its main checkout,
/// plus a linked worktree `name` made at `place`.
fn repository_with_worktree(fixture: &Fixture, name: &str, place: &Path) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("tracked.txt"), b"base\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    git(fixture, repo.as_path(), &["remote", "add", "origin", ORIGIN_URL]);
    git(fixture, repo.as_path(), &["config", "user.kept.name", "kept"]);
    git(
        fixture,
        repo.as_path(),
        &["worktree", "add", "--detach", place.to_str().expect("text path"), "HEAD"],
    );
    return (repo, place.to_path_buf());
}

/// What:
///  Run every refused form in a linked worktree and require the refusal,
///  with the shared file unchanged after each.
/// Why:
///  The refusal must change nothing,
///  so the file is compared byte for byte after every form,
/// and the
///  message must name what is shared,
/// where the setting belongs,
/// and how to get a copy with its own settings.
fn every_form_is_refused_in(fixture: &Fixture, linked: &Path, repo: &Path) {
    let fixture_wide: Vec<u8> = config_bytes(repo);
    let main_text: String = repo.to_string_lossy().into_owned();
    let config_text: String = config_file(repo).to_string_lossy().into_owned();
    for form in REFUSED_FORMS {
        let observed: Observed = run_wrapped(fixture, linked, form);
        let stderr: String = stderr_of(&observed);
        assert_eq!(
            observed.code,
            Some(1),
            "git {} was not refused in {}: {stderr}",
            form.join(" "),
            linked.display()
        );
        assert!(
            stderr.contains("\"policyId\":\"linked-worktree-only\""),
            "git {} printed no linked-worktree finding: {stderr}",
            form.join(" ")
        );
        assert!(
            stderr.contains(SHARED_REMOTE_CODE),
            "git {} printed no shared-remote finding: {stderr}",
            form.join(" ")
        );
        assert!(stderr.contains(SHARED_SENTENCE), "{}", form.join(" "));
        assert!(
            stderr.contains(config_text.as_str()),
            "the refusal does not name the shared configuration file: {stderr}"
        );
        assert!(
            stderr.contains(main_text.as_str()),
            "the refusal does not name the main checkout: {stderr}"
        );
        assert!(
            stderr.contains("git clone --no-hardlinks"),
            "the refusal does not name a real clone: {stderr}"
        );
        assert!(
            observed.stdout.is_empty(),
            "the refused command printed on standard output: {}",
            String::from_utf8_lossy(&observed.stdout)
        );
        assert_eq!(
            config_bytes(repo),
            fixture_wide,
            "git {} changed the shared configuration",
            form.join(" ")
        );
    }
}

/// Every form that changes a remote setting is refused in a linked worktree made inside the
/// repository folder.
#[test]
fn every_shared_remote_write_is_refused_in_a_linked_worktree_inside_the_repository() {
    let fixture: Fixture = fixture("shared-remote-inside");
    let place: PathBuf = fixture.root.join("repo").join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    every_form_is_refused_in(&fixture, linked.as_path(), repo.as_path());
    remove(&fixture);
}

/// Every form that changes a remote setting is refused in a linked worktree made outside the
/// repository folder,
/// because the Git directory it shares is what decides,
/// not where it sits.
#[test]
fn every_shared_remote_write_is_refused_in_a_linked_worktree_outside_the_repository() {
    let fixture: Fixture = fixture("shared-remote-outside");
    let place: PathBuf = fixture.root.join("linked-outside");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    every_form_is_refused_in(&fixture, linked.as_path(), repo.as_path());
    remove(&fixture);
}

/// The global options a person may write before the subcommand reach the same shared file,
/// so they are refused too,
/// and so is an alias that expands to a refused form.
#[test]
fn global_options_and_aliases_reach_the_same_refusal() {
    let fixture: Fixture = fixture("shared-remote-global-options");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    let before: Vec<u8> = config_bytes(repo.as_path());
    let linked_text: String = linked.to_string_lossy().into_owned();
    // Each form runs in the directory a person would run it in:
    //  `-C` moves a command started in the main checkout,
    //  and an
    // alias runs where it is written.
    let forms: [(PathBuf, Vec<&str>); 6] = [
        (
            repo.clone(),
            vec!["-C", linked_text.as_str(), "remote", "set-url", "origin", OTHER_URL],
        ),
        (
            repo.clone(),
            vec!["-C", linked_text.as_str(), "config", "remote.origin.url", OTHER_URL],
        ),
        (
            linked.clone(),
            vec!["-c", "alias.seturl=remote set-url", "seturl", "origin", OTHER_URL],
        ),
        (
            linked.clone(),
            vec!["-c", "alias.setconfig=config", "setconfig", "remote.origin.url", OTHER_URL],
        ),
        // A shell alias reaches the command the shell runs, so it is refused like the command itself.
        (
            linked.clone(),
            vec!["-c", "alias.shellsu=!git remote set-url", "shellsu", "origin", OTHER_URL],
        ),
        (
            linked.clone(),
            vec!["-c", "alias.shellcfg=!git config", "shellcfg", "remote.origin.url", OTHER_URL],
        ),
    ];
    for (directory, form) in forms {
        let observed: Observed = run_wrapped(&fixture, directory.as_path(), form.as_slice());
        let stderr: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(1), "git {} was not refused: {stderr}", form.join(" "));
        assert!(stderr.contains(SHARED_REMOTE_CODE), "git {} printed no shared-remote finding: {stderr}", form.join(" "));
        assert_eq!(
            config_bytes(repo.as_path()),
            before,
            "git {} changed the shared configuration",
            form.join(" ")
        );
    }
    remove(&fixture);
}

/// The forms that read or that write a file of their own still run in a linked worktree,
/// and a `--worktree` write lands only in this worktree's own configuration file.
#[test]
fn reads_and_this_worktrees_own_file_still_run() {
    let fixture: Fixture = fixture("shared-remote-allowed");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    for form in ALLOWED_FORMS {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), form);
        let stderr: String = stderr_of(&observed);
        assert_eq!(
            observed.code,
            Some(0),
            "git {} was refused in a linked worktree: {stderr}",
            form.join(" ")
        );
        assert!(!stderr.contains(SHARED_REMOTE_CODE), "git {} was refused: {stderr}", form.join(" "));
    }
    // A per-worktree configuration file needs the extension that enables it.
    git(&fixture, repo.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    let before: Vec<u8> = config_bytes(repo.as_path());
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["config", "--worktree", "remote.origin.url", OTHER_URL]);
    assert_eq!(observed.code, Some(0), "git config --worktree was refused: {}", stderr_of(&observed));
    assert_eq!(config_bytes(repo.as_path()), before, "a per-worktree write changed the shared configuration");
    let own: PathBuf = repo.join(".git").join("worktrees").join("linked").join("config.worktree");
    let written: String = String::from_utf8_lossy(&std::fs::read(own.as_path()).expect("per-worktree configuration")).into_owned();
    assert!(written.contains(OTHER_URL), "the per-worktree file does not hold the value: {written}");
    // A file the command names itself is not the shared one.
    let named: PathBuf = fixture.root.join("named.config");
    let named_run: Observed = run_wrapped(
        &fixture,
        linked.as_path(),
        &["config", "--file", named.to_str().expect("text path"), "remote.origin.url", OTHER_URL],
    );
    assert_eq!(named_run.code, Some(0), "a named file was refused: {}", stderr_of(&named_run));
    assert!(std::fs::read_to_string(named.as_path()).expect("named configuration").contains(OTHER_URL));
    assert_eq!(config_bytes(repo.as_path()), before, "a named file changed the shared configuration");
    remove(&fixture);
}

/// A `git config --edit` editor that appends the file it was given to a log and adds
///  `EDITOR_SECTION` to it.
///  `PathBuf` is the script it wrote.
fn recording_editor(fixture: &Fixture) -> PathBuf {
    let script: PathBuf = fixture.root.join("editor.sh");
    let log: PathBuf = fixture.root.join("editor.log");
    let text: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >>'{}'\nprintf '{}\\n' >>\"$1\"\n",
        log.display(),
        EDITOR_SECTION.replace('\n', "\\n"),
    );
    executable(script.as_path(), text.as_bytes());
    return script;
}

/// The files the recording editor ran on,
///  one per line,
///  in the order it ran.
fn edited_files(fixture: &Fixture) -> String {
    let log: PathBuf = fixture.root.join("editor.log");
    return std::fs::read_to_string(log.as_path()).unwrap_or_default();
}

/// A write that names the shared configuration file with `--file`,
///  and an editor session on it,
///  change the bytes every worktree shares,
///  so both are refused however the file is spelled,
///  while a file of the command's own still runs.
#[test]
fn a_write_that_names_or_edits_the_shared_configuration_is_refused() {
    let fixture: Fixture = fixture("shared-remote-named-shared");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    let shared: PathBuf = config_file(repo.as_path());
    let spelled: String = shared.to_str().expect("text path").to_string();
    // A link to the shared file names the same bytes through another path.
    let link: PathBuf = fixture.root.join("config-link");
    std::os::unix::fs::symlink(shared.as_path(), link.as_path()).expect("link to the shared configuration");
    let linked_to: String = link.to_str().expect("text path").to_string();
    let editor: PathBuf = recording_editor(&fixture);
    let editor_text: String = editor.to_str().expect("text path").to_string();
    // The linked worktree sits beside the repository, so this relative path is the same file.
    let relative: String = String::from("../repo/.git/config");
    let before: Vec<u8> = config_bytes(repo.as_path());
    let forms: Vec<Vec<String>> = vec![
        vec![String::from("config"), String::from("--file"), spelled.clone(), String::from("remote.origin.url"), String::from(OTHER_URL)],
        vec![String::from("config"), format!("--file={spelled}"), String::from("remote.origin.url"), String::from(OTHER_URL)],
        vec![String::from("config"), String::from("--file"), relative, String::from("remote.origin.url"), String::from(OTHER_URL)],
        vec![String::from("config"), String::from("--file"), linked_to.clone(), String::from("remote.origin.url"), String::from(OTHER_URL)],
        vec![String::from("config"), String::from("--file"), spelled.clone(), String::from("--remove-section"), String::from("remote.origin")],
        vec![String::from("config"), String::from("--file"), spelled.clone(), String::from("--rename-section"), String::from("remote.origin"), String::from("remote.renamed")],
        // An editor session names no key and can change any of them.
        vec![String::from("config"), String::from("--edit")],
        vec![String::from("config"), String::from("edit")],
        vec![String::from("config"), String::from("--edit"), String::from("user.name")],
        vec![String::from("config"), String::from("--file"), spelled.clone(), String::from("--edit")],
    ];
    let config_text: String = spelled.clone();
    for form in &forms {
        let arguments: Vec<&str> = form.iter().map(String::as_str).collect();
        let observed: Observed = observe(
            wrapped(&fixture)
                .current_dir(linked.as_path())
                .env("GIT_EDITOR", editor_text.as_str())
                .args(arguments.as_slice()),
            b"",
        );
        let stderr: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(1), "git {} was not refused in {}", form.join(" "), linked.display());
        assert!(
            stderr.contains("\"code\":\"linked-worktree-only/shared-remote-settings\""),
            "git {} printed no shared-remote finding: {stderr}",
            form.join(" ")
        );
        assert!(
            stderr.contains(config_text.as_str()),
            "the refusal does not name the shared configuration file: {stderr}"
        );
        assert_eq!(
            config_bytes(repo.as_path()),
            before,
            "git {} changed the shared configuration",
            form.join(" ")
        );
        assert_eq!(
            edited_files(&fixture),
            String::new(),
            "git {} started the editor it refused",
            form.join(" ")
        );
    }
    // A file of the command's own is no reason to refuse, and an editor of it really runs.
    let named: PathBuf = fixture.root.join("named.config");
    let named_text: String = named.to_str().expect("text path").to_string();
    for form in [
        vec![String::from("config"), String::from("--file"), named_text.clone(), String::from("remote.origin.url"), String::from(OTHER_URL)],
        vec![String::from("config"), String::from("--file"), named_text.clone(), String::from("--edit")],
    ] {
        let arguments: Vec<&str> = form.iter().map(String::as_str).collect();
        let observed: Observed = observe(
            wrapped(&fixture)
                .current_dir(linked.as_path())
                .env("GIT_EDITOR", editor_text.as_str())
                .args(arguments.as_slice()),
            b"",
        );
        assert_eq!(observed.code, Some(0), "git {} was refused: {}", form.join(" "), stderr_of(&observed));
        let stored: String = std::fs::read_to_string(named.as_path()).expect("named configuration");
        assert!(stored.contains(OTHER_URL), "{} stored nothing: {stored}", form.join(" "));
        assert_eq!(config_bytes(repo.as_path()), before, "git {} changed the shared configuration", form.join(" "));
    }
    // The per-user file is the person's own, so an editor session on it runs.
    let global: PathBuf = fixture.root.join("global-config");
    let global_text: String = global.to_str().expect("text path").to_string();
    let global_run: Observed = observe(
        wrapped(&fixture)
            .current_dir(linked.as_path())
            .env("GIT_EDITOR", editor_text.as_str())
            .env("GIT_CONFIG_GLOBAL", global_text.as_str())
            .args(["config", "--global", "--edit"]),
        b"",
    );
    assert_eq!(global_run.code, Some(0), "git config --global --edit was refused: {}", stderr_of(&global_run));
    assert_eq!(config_bytes(repo.as_path()), before, "git config --global --edit changed the shared configuration");
    let global_edited: String = std::fs::read_to_string(global.as_path()).expect("per-user configuration");
    assert!(
        global_edited.contains(EDITOR_SECTION),
        "the editor of the per-user file did not run: {global_edited}"
    );
    // The directory a relative path is read from is the one the command runs in: a subdirectory of the
    //  worktree, and the worktree a `-C` option moves to.
    let below: PathBuf = linked.join("below");
    std::fs::create_dir_all(below.as_path()).expect("subdirectory of the linked worktree");
    let linked_text: String = linked.to_str().expect("text path").to_string();
    let places: [(PathBuf, Vec<&str>); 2] = [
        (
            below.clone(),
            vec![
                // Below the worktree root another policy refuses the command first, so its hatch is what lets
                //  the relative path reach this rule.
                "config",
                "--no-enforce-require-root",
                "--file",
                "../../repo/.git/config",
                "remote.origin.url",
                OTHER_URL,
            ],
        ),
        (
            fixture.root.clone(),
            vec![
                "-C",
                linked_text.as_str(),
                "config",
                "--file",
                "../repo/.git/config",
                "remote.origin.url",
                OTHER_URL,
            ],
        ),
    ];
    for (directory, form) in &places {
        let observed: Observed = observe(
            wrapped(&fixture)
                .current_dir(directory.as_path())
                .env("GIT_EDITOR", editor_text.as_str())
                .args(form.as_slice()),
            b"",
        );
        assert_eq!(
            observed.code,
            Some(1),
            "git {} run in {} was not refused: {}",
            form.join(" "),
            directory.display(),
            stderr_of(&observed)
        );
        assert_eq!(
            config_bytes(repo.as_path()),
            before,
            "git {} run in {} changed the shared configuration",
            form.join(" "),
            directory.display()
        );
    }
    // The positive control the refusals rest on: real Git's editor session in this same linked worktree
    //  opens the shared file and adds to the bytes every worktree reads.
    let direct_run: Observed = observe(
        direct(&fixture)
            .current_dir(linked.as_path())
            .env("GIT_EDITOR", editor_text.as_str())
            .args(["config", "--edit"]),
        b"",
    );
    assert_eq!(direct_run.code, Some(0), "real Git refused its own editor session: {}", stderr_of(&direct_run));
    assert_eq!(
        edited_files(&fixture).lines().next_back(),
        Some(spelled.as_str()),
        "real Git edited another file than the shared one"
    );
    assert!(
        String::from_utf8_lossy(&config_bytes(repo.as_path())).contains(EDITOR_SECTION),
        "real Git's editor session changed nothing shared, so the refusals above prove nothing"
    );
    // The hatch the refusal names runs the same editor session, and leaves the line that attributes it.
    let hatched: Observed = observe(
        wrapped(&fixture)
            .current_dir(linked.as_path())
            .env("GIT_EDITOR", editor_text.as_str())
            .args(["config", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "--edit"]),
        b"",
    );
    assert_eq!(hatched.code, Some(0), "the hatch did not run the editor session: {}", stderr_of(&hatched));
    assert_eq!(
        edited_files(&fixture).lines().next_back(),
        Some(spelled.as_str()),
        "the hatch ran the editor on another file than the shared one"
    );
    let audit: PathBuf = repo
        .join(".git")
        .join("monochromatic-git-policy-cli-audit.jsonl");
    let lines: String = std::fs::read_to_string(audit.as_path()).expect("audit file after the hatch");
    assert!(lines.contains("\"form\":\"SharedRemote\""), "{lines}");
    assert!(lines.contains("\"outcome\":\"hatch\""), "{lines}");
    remove(&fixture);
}

/// The escape hatch runs the same command and records that it did.
#[test]
fn the_hatch_runs_the_shared_remote_write() {
    let fixture: Fixture = fixture("shared-remote-hatch");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    let before: Vec<u8> = config_bytes(repo.as_path());
    let observed: Observed = run_wrapped(
        &fixture,
        linked.as_path(),
        &["remote", WORKTREE_ENFORCEMENT_ESCAPE_HATCH, "set-url", "origin", OTHER_URL],
    );
    assert_eq!(observed.code, Some(0), "the hatch did not run the command: {}", stderr_of(&observed));
    let after: String = String::from_utf8_lossy(&config_bytes(repo.as_path())).into_owned();
    assert_ne!(config_bytes(repo.as_path()), before, "the hatch ran nothing that changed the configuration");
    assert!(after.contains(OTHER_URL), "the hatch ran the command but stored nothing: {after}");
    // A change made under the hatch is still attributable: one `hatch` line names this form and these arguments.
    let audit: PathBuf = repo
        .join(".git")
        .join("monochromatic-git-policy-cli-audit.jsonl");
    let lines: String = std::fs::read_to_string(audit.as_path()).expect("audit file after the hatch");
    assert!(lines.contains("\"form\":\"SharedRemote\""), "{lines}");
    assert!(lines.contains("\"outcome\":\"hatch\""), "{lines}");
    assert!(lines.contains(OTHER_URL), "{lines}");
    remove(&fixture);
}

/// The main checkout keeps today's behaviour:
///  the same forms run there and change its own settings.
#[test]
fn the_main_checkout_still_changes_its_remote_settings() {
    let fixture: Fixture = fixture("shared-remote-main");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["remote", "add", "origin", ORIGIN_URL]);
    for form in [
        vec!["remote", "set-url", "origin", OTHER_URL],
        vec!["remote", "set-url", "--push", "origin", OTHER_URL],
        vec!["remote", "add", "beta", OTHER_URL],
        vec!["remote", "set-branches", "origin", "main"],
        vec!["config", "remote.origin.url", OTHER_URL],
        vec!["config", "--add", "remote.origin.pushurl", OTHER_URL],
        vec!["config", "set", "remote.origin.url", OTHER_URL],
        vec!["config", "--replace-all", "remote.origin.url", OTHER_URL],
        vec!["config", "--remove-section", "remote.beta"],
        vec!["config", "extensions.worktreeConfig", "true"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), form.as_slice());
        let stderr: String = stderr_of(&observed);
        assert_eq!(
            observed.code,
            Some(0),
            "git {} was refused in the main checkout: {stderr}",
            form.join(" ")
        );
        assert!(
            !stderr.contains(SHARED_REMOTE_CODE),
            "git {} earned a shared-remote refusal in the main checkout: {stderr}",
            form.join(" ")
        );
    }
    let stored: String = String::from_utf8_lossy(&config_bytes(repo.as_path())).into_owned();
    assert!(stored.contains(OTHER_URL), "the main checkout stored nothing: {stored}");
    // The shared file is this checkout's own, so naming it with `--file` is no refusal here either.
    let shared: PathBuf = config_file(repo.as_path());
    let named_run: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &[
            "config",
            "--file",
            shared.to_str().expect("text path"),
            "remote.origin.url",
            OTHER_URL,
        ],
    );
    assert_eq!(
        named_run.code,
        Some(0),
        "a `--file` write to the shared file was refused in the main checkout: {}",
        stderr_of(&named_run)
    );
    remove(&fixture);
}

/// What:
///  Run one command in a linked worktree from `directory`,
///  under a `--work-tree` that names another directory,
///  and require the shared-remote refusal with the shared file unchanged.
/// Why:
///  Git reads a relative `--file` path from the directory the command runs in,
///  and `--work-tree` does not move
///  that directory for `git config`,
///  so a path written that way names the shared file whenever the same path from the
///  same directory does;
///  the refusal must say so and change nothing.
fn another_work_tree_still_leaves_a_relative_file_path_in_the_shared_refusal(
    fixture: &Fixture,
    directory: &Path,
    global_options: &[&str],
    named: &str,
) {
    let before: Vec<u8> = config_bytes(fixture.root.join("repo").as_path());
    let mut form: Vec<&str> = global_options.to_vec();
    form.extend_from_slice(&["config", "--file", named, "remote.origin.url", OTHER_URL]);
    let observed: Observed = run_wrapped(fixture, directory, form.as_slice());
    let stderr: String = stderr_of(&observed);
    assert_eq!(
        observed.code,
        Some(1),
        "git {} was not refused in {}: {stderr}",
        form.join(" "),
        directory.display()
    );
    assert!(
        stderr.contains(SHARED_REMOTE_CODE),
        "git {} printed no shared-remote finding: {stderr}",
        form.join(" ")
    );
    assert_eq!(
        config_bytes(fixture.root.join("repo").as_path()),
        before,
        "git {} changed the shared configuration",
        form.join(" ")
    );
}

/// A relative `--file` path names the shared file from the directory the command runs in,
///  which `--work-tree` does not change,
///  so the write is refused there too,
///  and a file of the command's own under the same options still runs.
///  The last step is the oracle:
///  real Git resolves the same spelling from the same directory and writes the shared file.
#[test]
fn a_relative_file_path_beside_another_work_tree_is_read_from_the_directory_the_command_runs_in() {
    let fixture: Fixture = fixture("shared-remote-work-tree-relative");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    let elsewhere: PathBuf = fixture.root.join("nested").join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("nested directory");
    let elsewhere_text: &str = elsewhere.to_str().expect("text path");
    // From the worktree's own root, the shared file is one level up beside the repository.
    another_work_tree_still_leaves_a_relative_file_path_in_the_shared_refusal(
        &fixture,
        linked.as_path(),
        &["--work-tree", elsewhere_text],
        "../repo/.git/config",
    );
    // `-C` moves the directory the command runs in before the path is read, so the same refusal holds
    //  when the caller starts somewhere else.
    let linked_text: &str = linked.to_str().expect("text path");
    another_work_tree_still_leaves_a_relative_file_path_in_the_shared_refusal(
        &fixture,
        repo.as_path(),
        &["-C", linked_text, "--work-tree", elsewhere_text],
        "../repo/.git/config",
    );
    // A file of the command's own is still written, and where the command really runs.
    let own: PathBuf = linked.join("own.config");
    let allowed: Observed = run_wrapped(
        &fixture,
        linked.as_path(),
        &[
            "--work-tree",
            elsewhere_text,
            "config",
            "--file",
            "own.config",
            "remote.origin.url",
            "https://example.invalid/own.git",
        ],
    );
    assert_eq!(
        allowed.code,
        Some(0),
        "a file of the command's own was refused: {}",
        stderr_of(&allowed)
    );
    let own_text: String = std::fs::read_to_string(own.as_path()).expect("named configuration");
    assert!(
        own_text.contains("https://example.invalid/own.git"),
        "the file of the command's own holds nothing: {own_text}"
    );
    assert!(
        !elsewhere.join("own.config").exists(),
        "the wrapper read the path from the work tree instead of the command's directory"
    );
    // Real Git is the oracle: the spelling above names the shared file from the same directory.
    let oracle: Observed = run_direct(
        &fixture,
        linked.as_path(),
        &[
            "--work-tree",
            elsewhere_text,
            "config",
            "--file",
            "../repo/.git/config",
            "remote.origin.url",
            OTHER_URL,
        ],
    );
    assert_eq!(oracle.code, Some(0), "real Git refused the spelling: {}", stderr_of(&oracle));
    assert!(
        String::from_utf8_lossy(&config_bytes(repo.as_path())).contains(OTHER_URL),
        "real Git did not write the shared file through the same relative path"
    );
    remove(&fixture);
}

/// What:
///  A `!` shell alias runs the Git inside its shell from the top of the working tree,
///  so a relative `--file` path it names is read from there,
///  names the shared file from a subdirectory of a linked worktree,
///  and is refused.
/// Why:
///  Git changes to the top of the working tree before it starts the shell of a `!` alias
///  (`git.c:388` to `git.c:407`),
///  so the same words name different files in the caller's directory and in the shell's;
///  a guard that read the path from the caller's directory found no shared file,
///  returned clean,
///  and let the write change the configuration every worktree shares.
///  Real Git is the oracle:
///  the same words in the same directory write the shared file.
#[test]
fn a_shell_alias_reads_a_relative_file_path_where_git_runs_its_shell() {
    let fixture: Fixture = fixture("shared-remote-shell-alias-relative");
    let place: PathBuf = fixture.root.join("linked");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_worktree(&fixture, "repo", place.as_path());
    let below: PathBuf = linked.join("below");
    std::fs::create_dir_all(below.as_path()).expect("subdirectory of the linked worktree");
    // The command runs below the worktree root, where require-root refuses first, so this repository turns it off.
    std::fs::write(
        linked.join("monochromatic-git-policy-cli.config.jsonc"),
        b"{ \"policies\": { \"require-root\": \"off\" } }\n",
    )
    .expect("settings");
    let form: [&str; 7] = [
        "-c",
        "alias.shellcfg=!git config",
        "shellcfg",
        "--file",
        "../repo/.git/config",
        "remote.origin.url",
        OTHER_URL,
    ];
    let before: Vec<u8> = config_bytes(repo.as_path());
    let observed: Observed = run_wrapped(&fixture, below.as_path(), form.as_slice());
    let stderr: String = stderr_of(&observed);
    assert_eq!(
        observed.code,
        Some(1),
        "a shell alias below the worktree root was not refused: {stderr}"
    );
    assert!(
        stderr.contains(SHARED_REMOTE_CODE),
        "the refusal does not name the shared settings: {stderr}"
    );
    assert_eq!(
        config_bytes(repo.as_path()),
        before,
        "the shell alias changed the shared configuration"
    );
    // The oracle: real Git reads the same relative path from the directory it runs the shell in.
    let oracle: Observed = run_direct(&fixture, below.as_path(), form.as_slice());
    assert_eq!(oracle.code, Some(0), "real Git refused its own alias: {}", stderr_of(&oracle));
    assert!(
        String::from_utf8_lossy(&config_bytes(repo.as_path())).contains(OTHER_URL),
        "real Git did not write the shared file through the same spelling, so the refusal above proves nothing"
    );
    remove(&fixture);
}

/// A linked worktree of a repository that lives under an exempt tool cache keeps today's behaviour,
///  as every other worktree policy allows it there.
#[test]
fn a_linked_worktree_under_a_tool_cache_is_not_refused() {
    let fixture: Fixture = fixture("shared-remote-cache");
    let cache: PathBuf = fixture.root.join("uv-cache");
    std::fs::create_dir_all(cache.as_path()).expect("cache directory");
    let place: PathBuf = cache.join("repo").join("linked");
    let (repo, linked): (PathBuf, PathBuf) =
        repository_with_worktree(&fixture, "uv-cache/repo", place.as_path());
    let observed: Observed = observe(
        wrapped(&fixture)
            .current_dir(linked.as_path())
            .args(["remote", "set-url", "origin", OTHER_URL])
            .env("UV_CACHE_DIR", cache.as_os_str()),
        b"",
    );
    assert_eq!(observed.code, Some(0), "an exempt clone was refused: {}", stderr_of(&observed));
    assert!(String::from_utf8_lossy(&config_bytes(repo.as_path())).contains(OTHER_URL));
    remove(&fixture);
}
